import { useEffect } from "react";
import type { QueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { listen, type Event } from "@tauri-apps/api/event";

import type {
  ActivityHintPayload,
  ProjectGroup,
  SessionActivityStatus,
  SessionInfo,
  SessionTargetedPayload,
} from "../types";

function normalizePath(path: string): string {
  // Windows 路徑大小寫不敏感，正規化為小寫用於分組比對
  return path.replace(/\//g, "\\").toLowerCase();
}

function getRealtimeSyncLabel(): string {
  return new Date().toLocaleTimeString("zh-TW", { hour12: false });
}

/** 統一三個 `*-activity-hint` 事件的 `activity_statuses` 快取更新邏輯（design.md D1） */
function applyActivityStatusUpdate(
  queryClient: QueryClient,
  sessionId: string,
  update: {
    status: SessionActivityStatus["status"];
    detail?: SessionActivityStatus["detail"];
    lastActivityAt?: string | null;
  },
): void {
  queryClient.setQueriesData<SessionActivityStatus[]>(
    { queryKey: ["activity_statuses"], exact: false },
    (old) => {
      if (!old) return old;
      const idx = old.findIndex((s) => s.sessionId === sessionId);
      const updated: SessionActivityStatus = {
        ...(old[idx] ?? { sessionId }),
        status: update.status,
        detail: update.detail,
        lastActivityAt: update.lastActivityAt ?? old[idx]?.lastActivityAt ?? null,
      };
      if (idx === -1) return [...old, updated];
      const next = [...old];
      next[idx] = updated;
      return next;
    },
  );
}

/** 統一 `copilot-session-targeted` / `claude-session-targeted` 的處理邏輯（design.md D2） */
function createSessionTargetedHandler(
  queryClient: QueryClient,
  copilotRoot: string | undefined,
  isMounted: () => boolean,
  onSynced: () => void,
): (event: Event<SessionTargetedPayload>) => Promise<void> {
  return async (event) => {
    const { cwd } = event.payload;
    const updated = await invoke<SessionInfo | null>("get_session_by_cwd", {
      cwd,
      rootDir: copilotRoot,
    }).catch(() => null);

    if (!isMounted()) return;

    if (updated) {
      queryClient.setQueriesData<SessionInfo[]>(
        { queryKey: ["sessions"], exact: false },
        (old) => {
          if (!old) return old;
          const idx = old.findIndex((s) => s.id === updated.id);
          if (idx === -1) return [...old, updated];
          const next = [...old];
          next[idx] = updated;
          return next;
        },
      );
    } else {
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
    }

    onSynced();
  };
}

type UseSessionRealtimeEventsParams = {
  activePlanSession: SessionInfo | null;
  activeProject: ProjectGroup | null;
  copilotRoot: string | undefined;
  queryClient: QueryClient;
  sessionsDataRef: React.RefObject<SessionInfo[]>;
  refreshProjectPlansSpecs: (dir: string) => Promise<void>;
  setRealtimeStatus: (status: "active") => void;
  setLastRealtimeSyncAt: (label: string) => void;
  setActiveView: (view: string) => void;
  showToast: (message: string) => void;
  planReloadedToast: string;
};

export function useSessionRealtimeEvents(params: UseSessionRealtimeEventsParams): void {
  const {
    activePlanSession,
    activeProject,
    copilotRoot,
    queryClient,
    sessionsDataRef,
    refreshProjectPlansSpecs,
    setRealtimeStatus,
    setLastRealtimeSyncAt,
    setActiveView,
    showToast,
    planReloadedToast,
  } = params;

  useEffect(() => {
    let mounted = true;

    const markActive = () => {
      setRealtimeStatus("active");
      setLastRealtimeSyncAt(getRealtimeSyncLabel());
    };

    const onSessionsRefresh = async () => {
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
      if (mounted) markActive();
    };

    const setup = async () => {
      const unlistenCopilot = await listen("copilot-sessions-updated", onSessionsRefresh);
      const unlistenOpencode = await listen("opencode-sessions-updated", onSessionsRefresh);
      const unlistenCodex = await listen("codex-sessions-updated", onSessionsRefresh);
      const unlistenClaude = await listen("claude-sessions-updated", onSessionsRefresh);

      const sessionTargetedHandler = createSessionTargetedHandler(
        queryClient,
        copilotRoot,
        () => mounted,
        markActive,
      );

      const unlistenCopilotTargeted = await listen<SessionTargetedPayload>(
        "copilot-session-targeted",
        sessionTargetedHandler,
      );

      const unlistenClaudeTargeted = await listen<SessionTargetedPayload>(
        "claude-session-targeted",
        sessionTargetedHandler,
      );

      // copilot-activity-hint：輕量活動通知，不做任何 IPC 或 session 掃描。
      // 只更新 activityStatusQuery 快取中對應 session 的狀態，並刷新 status bar。
      const unlistenActivityHint = await listen<ActivityHintPayload>(
        "copilot-activity-hint",
        (event) => {
          if (!mounted) return;
          const { cwd, eventType, title } = event.payload;
          const normalizedCwd = normalizePath(cwd);
          const session = sessionsDataRef.current.find(
            (s) => normalizePath(s.cwd ?? "") === normalizedCwd,
          );
          if (!session) return;

          // 依 eventType 計算 activity detail
          let detail: SessionActivityStatus["detail"] = "tool_call";
          if (eventType === "prompt.submitted") {
            detail = "thinking";
          } else if (eventType === "tool.pre" && title) {
            const lowerTitle = title.toLowerCase();
            if (/edit|write|patch|create/.test(lowerTitle)) {
              detail = "file_op";
            } else if (/task|subtask|agent/.test(lowerTitle)) {
              detail = "sub_agent";
            }
          }

          applyActivityStatusUpdate(queryClient, session.id, { status: "active", detail });
          markActive();
        },
      );

      // claude-activity-hint：後端已計算 status/detail，直接 patch activityStatusMap
      const unlistenClaudeActivityHint = await listen<ActivityHintPayload>(
        "claude-activity-hint",
        (event) => {
          if (!mounted) return;
          const { cwd, eventType, title, sessionId: hintSessionId, status: hintStatus, detail: hintDetail, lastActivityAt } = event.payload;

          // 優先使用後端傳來的 sessionId，否則從 cwd 查找
          const normalizedCwd = normalizePath(cwd);
          const session = hintSessionId
            ? sessionsDataRef.current.find((s) => s.id === hintSessionId)
            : sessionsDataRef.current.find((s) => normalizePath(s.cwd ?? "") === normalizedCwd);
          if (!session) return;

          // 後端已提供 status 時直接使用，否則 fallback 到前端推算（向後相容）
          let status: SessionActivityStatus["status"];
          let detail: SessionActivityStatus["detail"];
          if (hintStatus) {
            status = hintStatus;
            detail = hintDetail ?? undefined;
          } else {
            status = "active";
            detail = "tool_call";
            if (eventType === "prompt.submitted") {
              detail = "thinking";
            } else if (eventType === "tool.pre" && title) {
              const lowerTitle = title.toLowerCase();
              if (/edit|write|patch|create/.test(lowerTitle)) {
                detail = "file_op";
              } else if (/task|subtask|agent/.test(lowerTitle)) {
                detail = "sub_agent";
              }
            }
          }

          applyActivityStatusUpdate(queryClient, session.id, { status, detail, lastActivityAt });
          markActive();
        },
      );

      const unlistenOpenCodeActivityHint = await listen<ActivityHintPayload>(
        "opencode-activity-hint",
        (event) => {
          if (!mounted) return;
          const { cwd, sessionId: hintSessionId, status: hintStatus, detail: hintDetail, lastActivityAt } = event.payload;
          const normalizedCwd = normalizePath(cwd);
          const session = hintSessionId
            ? sessionsDataRef.current.find((s) => s.id === hintSessionId)
            : sessionsDataRef.current.find((s) => normalizePath(s.cwd ?? "") === normalizedCwd);
          if (!session || !hintStatus) return;

          applyActivityStatusUpdate(queryClient, session.id, {
            status: hintStatus,
            detail: hintDetail ?? undefined,
            lastActivityAt,
          });
          markActive();
        },
      );

      const unlistenPlan = await listen<string>("plan-file-changed", async (event) => {
        if (!activePlanSession || event.payload !== activePlanSession.sessionDir) return;
        await queryClient.invalidateQueries({ queryKey: ["plan", activePlanSession.sessionDir] });
        if (mounted) {
          setRealtimeStatus("active");
          showToast(planReloadedToast);
        }
      });

      const unlistenProjectFiles = await listen<string>("project-files-changed", async (event) => {
        if (!activeProject || normalizePath(event.payload) !== normalizePath(activeProject.pathLabel)) return;
        await refreshProjectPlansSpecs(activeProject.pathLabel);
        if (mounted) markActive();
      });

      const unlistenQuotaSnapshots = await listen("quota-snapshots-updated", () => {
        if (!mounted) return;
        void queryClient.invalidateQueries({ queryKey: ["quota_snapshots"] });
      });

      const unlistenNavigateMainView = await listen<string>("navigate-main-view", (event) => {
        if (!mounted) return;
        setActiveView(event.payload);
      });

      return () => {
        unlistenCopilot();
        unlistenOpencode();
        unlistenCodex();
        unlistenClaude();
        unlistenCopilotTargeted();
        unlistenClaudeTargeted();
        unlistenActivityHint();
        unlistenClaudeActivityHint();
        unlistenOpenCodeActivityHint();
        unlistenPlan();
        unlistenProjectFiles();
        unlistenQuotaSnapshots();
        unlistenNavigateMainView();
      };
    };

    let cleanup: (() => void) | undefined;
    void setup().then((dispose) => { cleanup = dispose; });
    return () => { mounted = false; cleanup?.(); };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activePlanSession, activeProject, queryClient, refreshProjectPlansSpecs, copilotRoot, planReloadedToast]);
}
