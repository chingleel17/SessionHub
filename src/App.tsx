import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useMutation, useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
import DOMPurify from "dompurify";
import { marked } from "marked";

import { useI18n } from "./i18n/I18nProvider";
import type {
  AgentsMdScanResult,
  AgentsRootLinkStatus,
  AnalyticsDataPoint,
  AnalyticsGroupBy,
  AppSettings,
  BridgeEventLogEntry,
  CommandsScanResult,
  ConfirmDialogState,
  EditDialogState,
  IdeLauncherType,
  McpProviderConfig,
  OpenSpecData,
  ProjectAgentsPrefs,
  ProjectGroup,
  ProjectSubTabState,
  ProviderIntegrationStatus,
  ProviderQuota,
  QuotaSnapshot,
  SaveProjectAgentsPrefsResult,
  SessionActivityStatus,
  SessionInfo,
  SessionStats,
  SessionTodo,
  SkillsScanResult,
  SisyphusData,
  SyncActionResult,
  SyncReport,
  SyncRequest,
  ToolAvailability,
} from "./types";
import { formatDateTime } from "./utils/formatDate";
import { parseTaskProgress } from "./utils/parseTaskProgress";
import { resolveErrorMessage } from "./utils/resolveErrorMessage";
import { useSessionRealtimeEvents } from "./hooks/useSessionRealtimeEvents";
import { useAppSettingsForm, type ProviderIntegrationAction } from "./hooks/useAppSettingsForm";

import { AgentsConfigView, type AgentsScopeDataBundle } from "./components/AgentsConfigView";
import { ConfirmDialog } from "./components/ConfirmDialog";
import { BridgeEventMonitorDialog } from "./components/BridgeEventMonitorDialog";
import { DashboardView } from "./components/DashboardView";
import { EditDialog } from "./components/EditDialog";
import { ProjectView } from "./components/ProjectView";
import { SettingsView } from "./components/SettingsView";
import { Sidebar } from "./components/Sidebar";
import { SyncConflictDialog } from "./components/SyncConflictDialog";
import { StatusBar } from "./components/StatusBar";

// ─── helpers ─────────────────────────────────────────────────────────────────

function normalizePath(path: string): string {
  // Windows 路徑大小寫不敏感，正規化為小寫用於分組比對
  return path.replace(/\//g, "\\").toLowerCase();
}

function normalizePinnedProjectKey(projectKey: string): string {
  const branchSeparatorIndex = projectKey.lastIndexOf(":");
  if (branchSeparatorIndex <= 1) {
    return normalizePath(projectKey);
  }

  const projectPath = projectKey.slice(0, branchSeparatorIndex);
  const branch = projectKey.slice(branchSeparatorIndex + 1);
  return `${normalizePath(projectPath)}:${branch}`;
}

function getProjectKey(session: SessionInfo, uncategorizedLabel: string): string {
  const raw = session.repoRoot?.trim() || session.cwd?.trim();
  if (!raw) return uncategorizedLabel;
  const branch = session.gitBranch?.trim() ?? "";
  return `${normalizePath(raw)}:${branch}`;
}

function getProjectDisplayPath(session: SessionInfo, uncategorizedLabel: string): string {
  return session.repoRoot?.trim() || session.cwd?.trim() || uncategorizedLabel;
}

function getProjectTitle(session: SessionInfo, displayPath: string, uncategorizedLabel: string): string {
  if (displayPath === uncategorizedLabel) return uncategorizedLabel;
  const repoName = session.repoName?.trim();
  if (repoName) return repoName;
  const parts = displayPath.replace(/\//g, "\\").split("\\").filter(Boolean);
  return parts[parts.length - 1] ?? displayPath;
}

function getProjectBranchLabel(sessions: SessionInfo[]): string | null {
  return sessions.map((session) => session.gitBranch?.trim()).find((branch): branch is string => Boolean(branch)) ?? null;
}

function getDashboardPeriodStart(period: "week" | "month"): number {
  const now = new Date();
  if (period === "week") {
    const start = new Date(now);
    start.setDate(now.getDate() - ((now.getDay() + 6) % 7));
    start.setHours(0, 0, 0, 0);
    return start.getTime();
  }

  return new Date(now.getFullYear(), now.getMonth(), 1).getTime();
}

function formatDateInput(value: Date): string {
  return value.toISOString().slice(0, 10);
}

function getDirectoryPath(filePath: string): string {
  const normalized = filePath.replace(/\\/g, "/");
  const index = normalized.lastIndexOf("/");
  if (index <= 0) {
    return filePath;
  }
  const separator = filePath.includes("\\") ? "\\" : "/";
  return normalized.slice(0, index).replace(/\//g, separator);
}

function readSessionTodos(sessionDir: string): Promise<SessionTodo[]> {
  return invoke<SessionTodo[]>("read_session_todos", { sessionDir });
}

function normalizeTags(tags: string[]): string[] {
  const seen = new Set<string>();
  const normalized: string[] = [];
  for (const tag of tags) {
    const trimmed = tag.trim();
    if (!trimmed) continue;
    const dedupeKey = trimmed.toLowerCase();
    if (seen.has(dedupeKey)) continue;
    seen.add(dedupeKey);
    normalized.push(trimmed);
  }
  return normalized;
}

function triggerStatsBackfill(rootDir: string | null | undefined): Promise<number> {
  return invoke<number>("trigger_stats_backfill", { rootDir: rootDir ?? null });
}

function isSessionInUpdatedRange(session: SessionInfo, periodStartTime: number): boolean {
  if (!session.updatedAt) return false;
  const updatedAtTime = Date.parse(session.updatedAt);
  return !Number.isNaN(updatedAtTime) && updatedAtTime >= periodStartTime;
}

function buildProjectGroups(sessions: SessionInfo[], uncategorizedLabel: string, locale: string): ProjectGroup[] {
  const groupMap = new Map<string, { displayPath: string; title: string; sessions: SessionInfo[] }>();

  for (const session of sessions) {
    const key = getProjectKey(session, uncategorizedLabel);
    const displayPath = getProjectDisplayPath(session, uncategorizedLabel);
    if (!groupMap.has(key)) {
      groupMap.set(key, {
        displayPath,
        title: getProjectTitle(session, displayPath, uncategorizedLabel),
        sessions: [],
      });
    }
    groupMap.get(key)!.sessions.push(session);
  }

  return Array.from(groupMap.entries())
    .map(([key, { displayPath, title, sessions: groupedSessions }]) => ({
      key,
      title,
      pathLabel: displayPath,
      branchLabel: getProjectBranchLabel(groupedSessions),
      sessions: groupedSessions.sort((a, b) =>
        (b.updatedAt ?? "").localeCompare(a.updatedAt ?? ""),
      ),
      updatedAtLabel:
        formatDateTime(
          groupedSessions.map((s) => s.updatedAt).find((v): v is string => Boolean(v)),
          locale,
        ),
    }))
    .sort((a, b) => b.sessions.length - a.sessions.length);
}

function resolveProviderTargetPath(integration: ProviderIntegrationStatus): string | null {
  const configPath = integration.configPath?.trim();
  if (configPath) return configPath;
  const bridgePath = integration.bridgePath?.trim();
  return bridgePath || null;
}

// Provider → resume 指令對照。與後端 `src-tauri/src/commands/tools.rs` 的 `resume_session_command` 保持同步。
function getSessionOpenCommand(provider: string, sessionId: string): string {
  switch (provider) {
    case "copilot":
      return `copilot --resume=${sessionId}`;
    case "opencode":
      return `opencode --session ${sessionId}`;
    case "codex":
      return `codex resume ${sessionId}`;
    case "claude":
      return `claude --resume=${sessionId}`;
    default:
      return "";
  }
}

const DEFAULT_PROJECT_AGENTS_PREFS: ProjectAgentsPrefs = {
  conflictChoice: null,
  ignoredPaths: [],
  enabledTargets: ["claude", "codex", "opencode", "copilot"],
};

function formatAgentsReportToast(report: SyncReport, template: string): string {
  const summary = report.actions.reduce<Record<string, number>>((acc, action) => {
    acc[action.action] = (acc[action.action] ?? 0) + 1;
    return acc;
  }, {});

  return template
    .replace("{create}", String(summary.create ?? 0))
    .replace("{overwrite}", String(summary.overwrite ?? 0))
    .replace("{skip}", String(summary["skip-in-sync"] ?? 0))
    .replace("{error}", String(summary.error ?? 0));
}

function formatProjectConfigCreatedToast(template: string, storedPath: string): string {
  return template.replace("{path}", storedPath);
}

function getRealtimeSyncLabel(): string {
  return new Date().toLocaleTimeString("zh-TW", { hour12: false });
}

// ─── App ─────────────────────────────────────────────────────────────────────

function App() {
  const { t, locale } = useI18n();
  const queryClient = useQueryClient();

  const [openProjectKeys, setOpenProjectKeys] = useState<string[]>([]);
  const [activeView, setActiveView] = useState<string>("dashboard");
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);
  const [hideEmptySessions, setHideEmptySessions] = useState(false);
  const [pinnedProjects, setPinnedProjects] = useState<string[]>([]);

  const [activePlanSessionId, setActivePlanSessionId] = useState<string | null>(null);
  const [planDraft, setPlanDraft] = useState("");

  // Plan sub-tab state per project — preserved across project switches
  const [projectSubTabStates, setProjectSubTabStates] = useState<
    Map<string, ProjectSubTabState>
  >(new Map());

  const getProjectSubTabState = (projectKey: string) =>
    projectSubTabStates.get(projectKey) ?? { openDetailKeys: [], activeSubTab: "sessions" };

  const handleSubTabStateChange = (
    projectKey: string,
    state: ProjectSubTabState,
  ) => {
    setProjectSubTabStates((prev) => new Map(prev).set(projectKey, state));
  };

  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [confirmDialog, setConfirmDialog] = useState<ConfirmDialogState | null>(null);
  const [editDialog, setEditDialog] = useState<EditDialogState | null>(null);
  const [globalAgentsPrefs, setGlobalAgentsPrefs] = useState<ProjectAgentsPrefs>(DEFAULT_PROJECT_AGENTS_PREFS);
  const [syncConflictDialog, setSyncConflictDialog] = useState<{
    conflicts: SyncActionResult[];
    canRememberChoice: boolean;
    onResolve: (result: { items: SyncRequest["items"]; rememberChoice: boolean; rememberedChoice: "source-wins" | "target-wins" | null }) => void;
  } | null>(null);

  const [realtimeStatus, setRealtimeStatus] = useState<"connecting" | "active" | "error">(
    "connecting",
  );
  const [lastRealtimeSyncAt, setLastRealtimeSyncAt] = useState<string | null>(null);
  // 下一次 sessions 掃描是否強制全掃。用 ref 而非 state，避免它進入 queryKey 造成 fetch 過程中
  // queryKey 變動而連續觸發兩次掃描（async 化後會讓 isFetching 永遠為 true，狀態列卡在「掃描中」）。
  const forceFullRef = useRef(false);
  const [pendingProviderAction, setPendingProviderAction] = useState<string | null>(null);

  const [bridgeEventLog, setBridgeEventLog] = useState<BridgeEventLogEntry[]>([]);
  const [lastBridgeEvent, setLastBridgeEvent] = useState<{ entry: BridgeEventLogEntry; receivedAt: Date } | null>(null);
  const [showEventMonitor, setShowEventMonitor] = useState(false);
  const lastBridgeEventTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const bridgeEventBufferRef = useRef<BridgeEventLogEntry[]>([]);
  const bridgeEventFlushTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const hasShownOutdatedToast = useRef(false);
  const prevActivityStatusRef = useRef<Map<string, string>>(new Map());
  const lastInterventionSessionRef = useRef<string | null>(null);
  const lastNotificationSessionRef = useRef<{ id: string; type: string } | null>(null);
  const lastBackfillRequestRef = useRef<string | null>(null);
  // sessionsDataRef 讓事件 listener 不因 stale closure 而讀到舊的 sessionsQuery.data
  const sessionsDataRef = useRef<SessionInfo[]>([]);

  const settingsQuery = useQuery({
    queryKey: ["settings"],
    queryFn: () => invoke<AppSettings>("get_settings"),
  });

  const sessionsCachedQuery = useQuery({
    queryKey: [
      "sessions_cached",
      settingsQuery.data?.showArchived ?? false,
      settingsQuery.data?.enabledProviders ?? [],
    ],
    enabled: Boolean(settingsQuery.data),
    staleTime: Infinity,
    queryFn: () =>
      invoke<SessionInfo[]>("get_sessions_cached", {
        showArchived: settingsQuery.data?.showArchived,
        enabledProviders: settingsQuery.data?.enabledProviders,
      }),
  });

  const sessionsQuery = useQuery({
    queryKey: [
      "sessions",
      settingsQuery.data?.copilotRoot ?? "",
      settingsQuery.data?.opencodeRoot ?? "",
      settingsQuery.data?.codexRoot ?? "",
      settingsQuery.data?.claudeRoot ?? "",
      settingsQuery.data?.antigravityRoot ?? "",
      settingsQuery.data?.showArchived ?? false,
      settingsQuery.data?.enabledProviders ?? [],
    ],
    enabled: Boolean(settingsQuery.data),
    placeholderData: sessionsCachedQuery.data,
    queryFn: () => {
      // 讀取並立即清除全掃旗標：本次 fetch 用完即重置，不影響 queryKey。
      const forceFull = forceFullRef.current;
      forceFullRef.current = false;
      return invoke<SessionInfo[]>("get_sessions", {
        rootDir: settingsQuery.data?.copilotRoot,
        opencodeRoot: settingsQuery.data?.opencodeRoot,
        codexRoot: settingsQuery.data?.codexRoot,
        claudeRoot: settingsQuery.data?.claudeRoot,
        antigravityRoot: settingsQuery.data?.antigravityRoot,
        showArchived: settingsQuery.data?.showArchived,
        enabledProviders: settingsQuery.data?.enabledProviders,
        forceFull,
      });
    },
  });

  const providerQuotaQuery = useQuery({
    queryKey: ["provider_quota"],
    queryFn: () => invoke<ProviderQuota[]>("get_provider_quota"),
    staleTime: 5 * 60_000,
    refetchInterval: 5 * 60_000,
  });

  const quotaSnapshotQuery = useQuery({
    queryKey: ["quota_snapshots"],
    queryFn: () => invoke<QuotaSnapshot[]>("get_quota_snapshots"),
    staleTime: 60_000,
  });

  const activePlanSession = useMemo(
    () => sessionsQuery.data?.find((s) => s.id === activePlanSessionId) ?? null,
    [activePlanSessionId, sessionsQuery.data],
  );

  // 保持 sessionsDataRef 與最新 sessions 同步，供事件 listener 讀取（避免 stale closure）
  useEffect(() => {
    sessionsDataRef.current = sessionsQuery.data ?? [];
  }, [sessionsQuery.data]);

  const planQuery = useQuery({
    queryKey: ["plan", activePlanSession?.sessionDir ?? ""],
    enabled: Boolean(activePlanSession),
    queryFn: () =>
      invoke<string | null>("read_plan", { sessionDir: activePlanSession?.sessionDir }),
  });

  useEffect(() => {
    if (settingsQuery.data) {
      setPinnedProjects((settingsQuery.data.pinnedProjects ?? []).map(normalizePinnedProjectKey));
    }
  }, [settingsQuery.data]);

  useEffect(() => {
    try {
      const raw = window.localStorage.getItem("agents-global-prefs");
      if (!raw) return;
      const parsed = JSON.parse(raw) as ProjectAgentsPrefs;
      setGlobalAgentsPrefs({ ...DEFAULT_PROJECT_AGENTS_PREFS, ...parsed });
    } catch {
      setGlobalAgentsPrefs(DEFAULT_PROJECT_AGENTS_PREFS);
    }
  }, []);

  useEffect(() => {
    window.localStorage.setItem("agents-global-prefs", JSON.stringify(globalAgentsPrefs));
  }, [globalAgentsPrefs]);

  useEffect(() => {
    if (!sessionsQuery.isSuccess || !settingsQuery.data?.copilotRoot) return;

    const requestKey = `${settingsQuery.data.copilotRoot}:${sessionsQuery.dataUpdatedAt}`;
    if (lastBackfillRequestRef.current === requestKey) return;
    lastBackfillRequestRef.current = requestKey;

    void triggerStatsBackfill(settingsQuery.data.copilotRoot)
      .then((count) => {
        if (count > 0) {
          void queryClient.invalidateQueries({ queryKey: ["session_stats_all"] });
        }
      })
      .catch((error) => console.warn("[stats-backfill] trigger failed:", error));
  }, [
    queryClient,
    sessionsQuery.dataUpdatedAt,
    sessionsQuery.isSuccess,
    settingsQuery.data?.copilotRoot,
  ]);

  // 啟動時偵測 provider integration 版本是否過期，提示使用者前往設定更新
  useEffect(() => {
    if (!settingsQuery.data || hasShownOutdatedToast.current) return;
    const integrations = settingsQuery.data.providerIntegrations ?? [];
    const hasOutdated = integrations.some(
      (integration) => integration.status === "outdated" || integration.status === "missing",
    );
    if (hasOutdated) {
      hasShownOutdatedToast.current = true;
      showToast(t("toast.providerOutdatedOnStartup"));
    }
  }, [settingsQuery.data, t]);


  useEffect(() => {
    if (activePlanSession) {
      setPlanDraft(planQuery.data ?? "");
    }
  }, [activePlanSession, planQuery.data]);

  useEffect(() => {
    if (!settingsQuery.data) return undefined;
    void invoke("restart_session_watcher", {
      copilotRoot: settingsQuery.data.copilotRoot,
      opencodeRoot: settingsQuery.data.opencodeRoot,
      codexRoot: settingsQuery.data.codexRoot,
      hookScriptsPath: settingsQuery.data.hookScriptsPath,
      enabledProviders: settingsQuery.data.enabledProviders,
    })
      .then(() => setRealtimeStatus("active"))
      .catch(() => setRealtimeStatus("error"));
    return undefined;
  }, [settingsQuery.data]);

  useEffect(() => {
    if (!activePlanSession?.hasPlan) {
      void invoke("stop_plan_watch");
      return undefined;
    }
    void invoke("watch_plan_file", { sessionDir: activePlanSession.sessionDir });
    return () => { void invoke("stop_plan_watch"); };
  }, [activePlanSession]);

  useEffect(() => {
    const MAX_LOG = 100;
    const LAST_EVENT_TTL_MS = 5 * 60 * 1000;

    const unlisten = listen<BridgeEventLogEntry>("provider-bridge-event-logged", (event) => {
      const entry = event.payload;

      bridgeEventBufferRef.current.push(entry);
      if (bridgeEventFlushTimerRef.current) return;

      bridgeEventFlushTimerRef.current = setTimeout(() => {
        const bufferedEntries = bridgeEventBufferRef.current;
        bridgeEventBufferRef.current = [];
        bridgeEventFlushTimerRef.current = null;
        if (bufferedEntries.length === 0) return;

        setBridgeEventLog((prev) => {
          const next = [...prev, ...bufferedEntries];
          return next.length > MAX_LOG ? next.slice(next.length - MAX_LOG) : next;
        });

        const latestEntry = bufferedEntries[bufferedEntries.length - 1];
        setLastBridgeEvent({ entry: latestEntry, receivedAt: new Date() });

        if (lastBridgeEventTimerRef.current) clearTimeout(lastBridgeEventTimerRef.current);
        lastBridgeEventTimerRef.current = setTimeout(() => {
          setLastBridgeEvent(null);
        }, LAST_EVENT_TTL_MS);
      }, 200);
    });

    return () => {
      void unlisten.then((fn) => fn());
      if (lastBridgeEventTimerRef.current) clearTimeout(lastBridgeEventTimerRef.current);
      if (bridgeEventFlushTimerRef.current) clearTimeout(bridgeEventFlushTimerRef.current);
      bridgeEventBufferRef.current = [];
    };
  }, []);

  useEffect(() => {
    if (!toastMessage) return undefined;
    const timer = window.setTimeout(() => setToastMessage(null), 2600);
    return () => window.clearTimeout(timer);
  }, [toastMessage]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "F5" || (e.ctrlKey && e.key === "r")) {
        e.preventDefault();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  const showToast = (message: string) => setToastMessage(message);

  const {
    settingsForm,
    setSettingsForm,
    buildSettingsPayload,
    persistSettingsSilently,
    settingsMutation,
    detectTerminalMutation,
    detectVscodeMutation,
    providerIntegrationMutation,
  } = useAppSettingsForm({
    settingsQuery,
    pinnedProjects,
    showToast,
    t,
    onSettingsSaved: () => {
      void (async () => {
        try {
          await invoke("restart_session_watcher", {
            copilotRoot: settingsForm.copilotRoot.trim(),
            opencodeRoot: settingsForm.opencodeRoot.trim(),
            codexRoot: settingsForm.codexRoot.trim(),
            hookScriptsPath: settingsForm.hookScriptsPath?.trim() ?? "",
            enabledProviders: settingsForm.enabledProviders,
          });
          setRealtimeStatus("active");
        } catch {
          setRealtimeStatus("error");
        }
      })();
    },
  });

  const archiveMutation = useMutation({
    mutationFn: (sessionId: string) =>
      invoke("archive_session", { rootDir: settingsQuery.data?.copilotRoot, sessionId }),
    onSuccess: async () => {
      showToast(t("toast.sessionArchived"));
      forceFullRef.current = true;
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
    },
  });

  const unarchiveMutation = useMutation({
    mutationFn: (sessionId: string) =>
      invoke("unarchive_session", { rootDir: settingsQuery.data?.copilotRoot, sessionId }),
    onSuccess: async () => {
      showToast(t("toast.sessionUnarchived"));
      forceFullRef.current = true;
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (sessionId: string) =>
      invoke("delete_session", { rootDir: settingsQuery.data?.copilotRoot, sessionId }),
    onSuccess: async () => {
      showToast(t("toast.sessionDeleted"));
      forceFullRef.current = true;
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
    },
  });

  const saveMetaMutation = useMutation({
    mutationFn: ({ sessionId, notes, tags }: { sessionId: string; notes?: string | null; tags: string[] }) =>
      invoke("upsert_session_meta", { sessionId, notes, tags }),
    onSuccess: async () => {
      showToast(t("toast.metaSaved"));
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
    },
  });

  const savePlanMutation = useMutation({
    mutationFn: ({ sessionDir, content }: { sessionDir: string; content: string }) =>
      invoke("write_plan", { sessionDir, content }),
    onSuccess: async () => {
      showToast(t("toast.planSaved"));
      await queryClient.invalidateQueries({ queryKey: ["plan"] });
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
    },
  });

  const deleteEmptySessionsMutation = useMutation({
    mutationFn: () =>
      invoke<number>("delete_empty_sessions", { rootDir: settingsQuery.data?.copilotRoot }),
    onSuccess: async (count) => {
      showToast(t("toast.emptySessionsDeleted").replace("{count}", String(count)));
      forceFullRef.current = true;
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
    },
  });

  const togglePinProject = async (projectKey: string) => {
    const next = pinnedProjects.includes(projectKey)
      ? pinnedProjects.filter((k) => k !== projectKey)
      : [...pinnedProjects, projectKey];
    setPinnedProjects(next);
    const settings = buildSettingsPayload({ pinnedProjects: next });
    await invoke("save_settings", { settings });
    await queryClient.invalidateQueries({ queryKey: ["settings"] });
  };

  const clearOpenProjects = () => {
    const nonPinned = openProjectKeys.filter((k) => !pinnedProjects.includes(k));
    setOpenProjectKeys((v) => v.filter((k) => pinnedProjects.includes(k)));
    if (nonPinned.includes(activeView)) setActiveView("dashboard");
  };

  const reorderOpenProjects = (newNonPinnedKeys: string[]) => {
    setOpenProjectKeys((v) => {
      const pinnedKeys = v.filter((k) => pinnedProjects.includes(k));
      return [...pinnedKeys, ...newNonPinnedKeys];
    });
  };

  const pinProjectViaDrag = async (key: string) => {
    if (!pinnedProjects.includes(key)) {
      await togglePinProject(key);
    }
  };


  const uncategorizedLabel = t("session.uncategorized");

  const groupedProjects = useMemo(
    () => buildProjectGroups(sessionsQuery.data ?? [], uncategorizedLabel, locale),
    [sessionsQuery.data, uncategorizedLabel, locale],
  );

  useEffect(() => {
    const availableProjectKeys = new Set(groupedProjects.map((project) => project.key));
    const normalizedPinnedKeys = pinnedProjects.filter((projectKey) => availableProjectKeys.has(projectKey));

    setOpenProjectKeys((current) => {
      const normalizedCurrent = current.filter((projectKey) => availableProjectKeys.has(projectKey));
      const next = [...new Set([...normalizedPinnedKeys, ...normalizedCurrent])];

      if (
        next.length === current.length &&
        next.every((projectKey, index) => projectKey === current[index])
      ) {
        return current;
      }

      return next;
    });

    if (
      activeView !== "dashboard" &&
      activeView !== "agents-global" &&
      activeView !== "settings" &&
      !availableProjectKeys.has(activeView)
    ) {
      setActiveView("dashboard");
    }
  }, [activeView, groupedProjects, pinnedProjects]);

  const activeProject = useMemo(
    () => groupedProjects.find((p) => p.key === activeView) ?? null,
    [activeView, groupedProjects],
  );

  const planSpecsRefreshTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  // 樂觀更新期間壓制 openspec 重整，防止 watcher 觸發的 refetch 引發 UI 閃爍。
  // handleWriteOpenspecFile 寫入 tasks.md 後設定此時間戳，期間 watcher 事件只
  // invalidate sisyphus 快取，不碰 openspec 快取。超過時間後下一次 watcher 事件
  // 才會正常重整 openspec，以真實掃描結果覆蓋樂觀值。
  const openspecOptimisticUntilRef = useRef<number>(0);
  const refreshProjectPlansSpecs = useCallback(
    async (projectDir: string) => {
      // 清除之前的計時器，實現去抖動
      if (planSpecsRefreshTimerRef.current) {
        clearTimeout(planSpecsRefreshTimerRef.current);
      }

      // 延遲 100ms 再執行掃描，避免同一批次多個檔案變更時重複掃描
      planSpecsRefreshTimerRef.current = setTimeout(async () => {
        const suppressOpenspec = Date.now() < openspecOptimisticUntilRef.current;
        await Promise.all([
          queryClient.invalidateQueries({ queryKey: ["project_plans", projectDir] }),
          ...(!suppressOpenspec
            ? [queryClient.invalidateQueries({ queryKey: ["project_specs", projectDir] })]
            : []),
        ]);
        planSpecsRefreshTimerRef.current = null;
      }, 100);
    },
    [queryClient],
  );

  useEffect(() => {
    if (!activeProject?.pathLabel) {
      if (planSpecsRefreshTimerRef.current) clearTimeout(planSpecsRefreshTimerRef.current);
      void invoke("stop_project_watch");
      return undefined;
    }
    void invoke("watch_project_files", { projectDir: activeProject.pathLabel });
    return () => {
      void invoke("stop_project_watch");
      if (planSpecsRefreshTimerRef.current) clearTimeout(planSpecsRefreshTimerRef.current);
    };
  }, [activeProject?.pathLabel]);

  useSessionRealtimeEvents({
    activePlanSession,
    activeProject,
    copilotRoot: settingsQuery.data?.copilotRoot,
    queryClient,
    sessionsDataRef,
    refreshProjectPlansSpecs,
    setRealtimeStatus,
    setLastRealtimeSyncAt,
    setActiveView,
    showToast,
    planReloadedToast: t("toast.planReloaded"),
  });

  const deletableEmptySessionCount = useMemo(
    () =>
      (sessionsQuery.data ?? []).filter(
        (session) => session.provider === "copilot" && !session.hasEvents,
      ).length,
    [sessionsQuery.data],
  );

  const allSessionDirs = useMemo(
    () => (sessionsQuery.data ?? [])
      .filter((session) => session.provider !== "codex" && Boolean(session.sessionDir))
      .map((session) => session.sessionDir)
      .sort((left, right) => left.localeCompare(right)),
    [sessionsQuery.data],
  );

  const sessionStatsQuery = useQuery({
    queryKey: ["session_stats_all", allSessionDirs],
    enabled: allSessionDirs.length > 0,
    staleTime: 60_000,
    queryFn: () => invoke<Record<string, SessionStats>>("get_all_session_stats", { sessionDirs: allSessionDirs }),
    refetchInterval: (query: { state: { data?: Record<string, SessionStats> } }) =>
      Object.values(query.state.data ?? {}).some((stats) => stats.isLive) ? 30_000 : false,
  });

  const sessionStatsMap = useMemo(
    () => Object.fromEntries(
      (sessionsQuery.data ?? []).map((session) => [session.id, sessionStatsQuery.data?.[session.sessionDir]]),
    ) as Record<string, SessionStats | undefined>,
    [sessionStatsQuery.data, sessionsQuery.data],
  );

  const sessionStatsLoadingMap = useMemo(
    () => Object.fromEntries(
      (sessionsQuery.data ?? []).map((session) => [session.id, session.provider !== "codex" ? sessionStatsQuery.isLoading : false]),
    ) as Record<string, boolean | undefined>,
    [sessionStatsQuery.isLoading, sessionsQuery.data],
  );

  const sessionTodoQueries = useQueries({
    queries: (sessionsQuery.data ?? []).map((session) => ({
      queryKey: ["session_todos", session.sessionDir],
      queryFn: () => readSessionTodos(session.sessionDir),
      staleTime: 60_000,
      enabled: session.provider === "copilot" && Boolean(session.sessionDir),
      refetchInterval: () => (sessionStatsMap[session.id]?.isLive ? 30_000 : false),
    })),
  });

  const sessionTodosMap = useMemo(
    () => Object.fromEntries(
      (sessionsQuery.data ?? []).map((session, index) => [session.id, sessionTodoQueries[index]?.data ?? []]),
    ) as Record<string, SessionTodo[]>,
    [sessionTodoQueries, sessionsQuery.data],
  );

  const sessionTodosLoadingMap = useMemo(
    () => Object.fromEntries(
      (sessionsQuery.data ?? []).map((session, index) => [session.id, sessionTodoQueries[index]?.isLoading]),
    ) as Record<string, boolean | undefined>,
    [sessionTodoQueries, sessionsQuery.data],
  );

  const sisyphusQuery = useQuery({
    queryKey: ["project_plans", activeProject?.pathLabel ?? ""],
    enabled: Boolean(activeProject?.pathLabel),
    queryFn: () => invoke<SisyphusData>("get_project_plans", { projectDir: activeProject?.pathLabel }),
    staleTime: 30_000,
  });

  const openspecQuery = useQuery({
    queryKey: ["project_specs", activeProject?.pathLabel ?? ""],
    enabled: Boolean(activeProject?.pathLabel),
    queryFn: () => invoke<OpenSpecData>("get_project_specs", { projectDir: activeProject?.pathLabel }),
    staleTime: 30_000,
  });

  // 專案 agents sub-tab 啟用時，除專案 scope queries 外也啟用 global queries，
  // 供專案分頁的「全域」分區使用（雙分區，見 agents-config-view-ux spec）。
  const projectAgentsSubTabActive =
    Boolean(activeProject?.pathLabel) &&
    (activeProject ? getProjectSubTabState(activeProject.key).activeSubTab === "agents" : false);

  const projectAgentsPrefsQuery = useQuery({
    queryKey: ["agents-prefs", activeProject?.pathLabel ?? ""],
    enabled: projectAgentsSubTabActive,
    staleTime: 5 * 60_000,
    gcTime: 30 * 60_000,
    placeholderData: (previous) => previous,
    queryFn: () => invoke<ProjectAgentsPrefs>("load_project_agents_prefs", { projectCwd: activeProject?.pathLabel }),
  });

  const projectAgentsMdQuery = useQuery({
    queryKey: ["agents-md", activeProject?.pathLabel ?? ""],
    enabled: projectAgentsSubTabActive,
    staleTime: 5 * 60_000,
    gcTime: 30 * 60_000,
    placeholderData: (previous) => previous,
    queryFn: () => invoke<AgentsMdScanResult>("scan_agents_md", { projectCwd: activeProject?.pathLabel }),
  });

  const projectAgentsSkillsQuery = useQuery({
    queryKey: ["agents-skills", activeProject?.pathLabel ?? ""],
    enabled: projectAgentsSubTabActive,
    staleTime: 5 * 60_000,
    gcTime: 30 * 60_000,
    placeholderData: (previous) => previous,
    queryFn: () => invoke<SkillsScanResult>("scan_agents_skills", { scope: { kind: "project", projectCwd: activeProject?.pathLabel } }),
  });

  const projectAgentsCommandsQuery = useQuery({
    queryKey: ["agents-commands", activeProject?.pathLabel ?? ""],
    enabled: projectAgentsSubTabActive,
    staleTime: 5 * 60_000,
    gcTime: 30 * 60_000,
    placeholderData: (previous) => previous,
    queryFn: () => invoke<CommandsScanResult>("scan_agents_commands", { scope: { kind: "project", projectCwd: activeProject?.pathLabel } }),
  });

  const globalAgentsMdQuery = useQuery({
    queryKey: ["agents-md", "global"],
    enabled: activeView === "agents-global" || projectAgentsSubTabActive,
    staleTime: 5 * 60_000,
    gcTime: 30 * 60_000,
    placeholderData: (previous) => previous,
    queryFn: () => invoke<AgentsMdScanResult>("scan_global_agents_md"),
  });

  const globalAgentsSkillsQuery = useQuery({
    queryKey: ["agents-skills", "global"],
    enabled: activeView === "agents-global" || projectAgentsSubTabActive,
    staleTime: 5 * 60_000,
    gcTime: 30 * 60_000,
    placeholderData: (previous) => previous,
    queryFn: () => invoke<SkillsScanResult>("scan_agents_skills", { scope: { kind: "global" } }),
  });

  const globalAgentsCommandsQuery = useQuery({
    queryKey: ["agents-commands", "global"],
    enabled: activeView === "agents-global" || projectAgentsSubTabActive,
    staleTime: 5 * 60_000,
    gcTime: 30 * 60_000,
    placeholderData: (previous) => previous,
    queryFn: () => invoke<CommandsScanResult>("scan_agents_commands", { scope: { kind: "global" } }),
  });

  const agentsSourceRootConfigured = Boolean((settingsForm.agentsSourceRoot ?? "").trim());

  const agentsRootLinkQuery = useQuery({
    queryKey: ["agents-root-link"],
    enabled: (activeView === "agents-global" || projectAgentsSubTabActive) && agentsSourceRootConfigured,
    staleTime: 60_000,
    gcTime: 5 * 60_000,
    placeholderData: (previous) => previous,
    queryFn: () => invoke<AgentsRootLinkStatus>("check_agents_root_link"),
  });

  const linkAgentsRootMutation = useMutation({
    mutationFn: () => invoke<AgentsRootLinkStatus>("link_agents_root"),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["agents-root-link"] });
      void queryClient.invalidateQueries({ queryKey: ["agents-skills", "global"] });
    },
    onError: (error) => {
      showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
    },
  });

  const globalMcpConfigsQuery = useQuery({
    queryKey: ["mcp-configs", "global"],
    enabled: activeView === "agents-global" || projectAgentsSubTabActive,
    staleTime: 30_000,
    gcTime: 5 * 60_000,
    placeholderData: (previous) => previous,
    queryFn: () => invoke<McpProviderConfig[]>("list_mcp_configs", { scope: { kind: "global" } }),
  });

  const projectMcpConfigsQuery = useQuery({
    queryKey: ["mcp-configs", activeProject?.pathLabel ?? ""],
    enabled: projectAgentsSubTabActive,
    staleTime: 30_000,
    gcTime: 5 * 60_000,
    placeholderData: (previous) => previous,
    queryFn: () => invoke<McpProviderConfig[]>("list_mcp_configs", { scope: { kind: "project", projectCwd: activeProject?.pathLabel } }),
  });

  const codexProjectTrustQuery = useQuery({
    queryKey: ["codex-project-trust", activeProject?.pathLabel ?? ""],
    enabled: projectAgentsSubTabActive,
    staleTime: 30_000,
    gcTime: 5 * 60_000,
    placeholderData: (previous) => previous,
    queryFn: () => invoke<boolean>("check_codex_project_trust", { projectCwd: activeProject?.pathLabel }),
  });

  const invalidateMcpConfigs = async (scopeKeyForQuery: string) => {
    await queryClient.invalidateQueries({ queryKey: ["mcp-configs", scopeKeyForQuery] });
  };

  const upsertMcpServerMutation = useMutation({
    mutationFn: (params: {
      scope: { kind: "global" } | { kind: "project"; projectCwd: string };
      provider: string;
      name: string;
      originalName?: string | null;
      configJson: string;
    }) => invoke("upsert_mcp_server", params),
    onSuccess: (_data, variables) => {
      void invalidateMcpConfigs(variables.scope.kind === "global" ? "global" : variables.scope.projectCwd);
    },
    onError: (error) => {
      showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
    },
  });

  const deleteMcpServerMutation = useMutation({
    mutationFn: (params: {
      scope: { kind: "global" } | { kind: "project"; projectCwd: string };
      provider: string;
      name: string;
    }) => invoke("delete_mcp_server", params),
    onSuccess: (_data, variables) => {
      void invalidateMcpConfigs(variables.scope.kind === "global" ? "global" : variables.scope.projectCwd);
    },
    onError: (error) => {
      showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
    },
  });

  const setMcpServerEnabledMutation = useMutation({
    mutationFn: (params: {
      scope: { kind: "global" } | { kind: "project"; projectCwd: string };
      provider: string;
      name: string;
      enabled: boolean;
    }) => invoke("set_mcp_server_enabled", params),
    onSuccess: (_data, variables) => {
      void invalidateMcpConfigs(variables.scope.kind === "global" ? "global" : variables.scope.projectCwd);
    },
    onError: (error) => {
      showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
    },
  });

  const claudeHookInstalled = (settingsQuery.data?.providerIntegrations ?? [])
    .some((i) => i.provider === "claude" && i.status === "installed");

  const activityStatusQuery = useQuery({
    queryKey: ["activity_statuses", sessionsQuery.data?.map((s) => s.id)],
    enabled: Boolean(sessionsQuery.data?.length),
    refetchOnMount: true,
    queryFn: async () => {
      const sessions = sessionsQuery.data ?? [];
      return invoke<SessionActivityStatus[]>("get_session_activity_statuses", {
        sessions: sessions.map((s) => ({
          id: s.id,
          provider: s.provider,
          sessionDir: s.sessionDir,
        })),
        opencodeRoot: settingsQuery.data?.opencodeRoot || null,
      });
    },
    refetchInterval: claudeHookInstalled ? false : 30_000,
    staleTime: 25_000,
  });

  const toolAvailabilityQuery = useQuery({
    queryKey: ["tool_availability"],
    queryFn: () => invoke<ToolAvailability>("check_tool_availability"),
    staleTime: Infinity,
    gcTime: Infinity,
  });

  const jqAvailableQuery = useQuery({
    queryKey: ["jq_available"],
    enabled: activeView === "settings",
    queryFn: () => invoke<boolean>("check_jq_available"),
    staleTime: Infinity,
    gcTime: Infinity,
  });

  const activityStatusMap = useMemo<Map<string, SessionActivityStatus>>(() => {
    const m = new Map<string, SessionActivityStatus>();
    for (const status of activityStatusQuery.data ?? []) {
      m.set(status.sessionId, status);
    }
    return m;
  }, [activityStatusQuery.data]);

  // 偵測 session 狀態轉換，發送 Windows 通知
  useEffect(() => {
    const currentStatuses = activityStatusQuery.data ?? [];
    const enableWaiting = settingsQuery.data?.enableInterventionNotification ?? true;
    const enableSessionEnd = settingsQuery.data?.enableSessionEndNotification ?? false;

    for (const status of currentStatuses) {
      const prev = prevActivityStatusRef.current.get(status.sessionId);
      const session = sessionsQuery.data?.find((s) => s.id === status.sessionId);
      const projectName = session?.cwd?.split("\\").pop() ?? session?.cwd ?? status.sessionId;
      const summary = session?.summary ?? "";

      if (status.status === "waiting" && prev !== "waiting" && enableWaiting) {
        lastInterventionSessionRef.current = status.sessionId;
        lastNotificationSessionRef.current = { id: status.sessionId, type: "waiting" };
        invoke("send_intervention_notification", {
          sessionId: status.sessionId,
          projectName,
          summary,
          notificationType: "waiting",
        }).catch((e) => console.warn("[notification] send failed:", e));
      } else if (status.status === "done" && prev !== "done" && prev !== undefined && enableSessionEnd) {
        lastNotificationSessionRef.current = { id: status.sessionId, type: "session_end" };
        invoke("send_intervention_notification", {
          sessionId: status.sessionId,
          projectName,
          summary,
          notificationType: "session_end",
        }).catch((e) => console.warn("[notification] send failed:", e));
      }

      prevActivityStatusRef.current.set(status.sessionId, status.status);
    }
  }, [activityStatusQuery.data, settingsQuery.data?.enableInterventionNotification, settingsQuery.data?.enableSessionEndNotification, sessionsQuery.data]);

  // 通知點擊後聚焦視窗並導航至對應 session
  useEffect(() => {
    const unlistenPromise = listen("notification://action-performed", () => {
      const notif = lastNotificationSessionRef.current;
      if (!notif) return;
      const session = sessionsQuery.data?.find((s) => s.id === notif.id);
      if (!session) return;
      const status = activityStatusMap.get(notif.id);
      if (notif.type === "waiting" && status?.status !== "waiting") return;
      const projectKey = getProjectKey(session, uncategorizedLabel);
      setActiveView(projectKey);
      lastNotificationSessionRef.current = null;
      lastInterventionSessionRef.current = null;
    });
    return () => { unlistenPromise.then((fn) => fn()); };
  }, [sessionsQuery.data, activityStatusMap, uncategorizedLabel]);

  const handleReadFileContent = async (filePath: string): Promise<string> => {
    return invoke<string>("read_plan_content", { filePath });
  };

  const handleReadAgentsFile = async (filePath: string): Promise<string> => {
    return invoke<string>("read_agents_file", { filePath });
  };

  const handleReadOpenspecFile = async (projectCwd: string, relativePath: string): Promise<string> => {
    return invoke<string>("read_openspec_file", { projectCwd, relativePath });
  };

  const handleWriteOpenspecFile = async (
    projectCwd: string,
    relativePath: string,
    content: string,
  ): Promise<void> => {
    // 樂觀更新：若寫入的是某個 change 的 tasks.md，立即以前端解析的進度更新
    // openspecQuery 快取，避免等待後端 watcher 掃描造成的延遲。後端掃描完成後會以
    // 真實數字覆蓋此處的樂觀值。
    const projectKey = activeProject?.pathLabel ?? "";
    const normalizedPath = relativePath.replace(/\\/g, "/");
    const tasksMatch = /^changes\/(?:archive\/)?([^/]+)\/tasks\.md$/.exec(normalizedPath);
    let previousData: OpenSpecData | undefined;
    if (projectKey && tasksMatch) {
      const changeName = tasksMatch[1];
      const nextProgress = parseTaskProgress(content);
      const snapshot = queryClient.getQueryData<OpenSpecData>(["project_specs", projectKey]);
      if (snapshot) {
        previousData = snapshot;
        const applyProgress = (changes: OpenSpecData["activeChanges"]) =>
          changes.map((change) =>
            change.name === changeName ? { ...change, taskProgress: nextProgress } : change,
          );
        queryClient.setQueryData<OpenSpecData>(["project_specs", projectKey], {
          ...snapshot,
          activeChanges: applyProgress(snapshot.activeChanges),
          archivedChanges: applyProgress(snapshot.archivedChanges),
        });
        // 壓制接下來 2 秒內 watcher 觸發的 openspec invalidate，避免 refetch 閃爍。
        // 2 秒後下一次 watcher 事件才會以真實掃描結果覆蓋樂觀值。
        openspecOptimisticUntilRef.current = Date.now() + 2000;
      }
    }

    try {
      await invoke("write_openspec_file", { projectCwd, relativePath, content });
    } catch (error) {
      // 寫入失敗：回滾樂觀更新
      if (projectKey && previousData) {
        queryClient.setQueryData<OpenSpecData>(["project_specs", projectKey], previousData);
      }
      showToast(resolveErrorMessage(error, t("toast.openspecWriteFailed")));
      throw error;
    }
  };

  const handleRefreshPlansSpecs = async (): Promise<void> => {
    if (!activeProject?.pathLabel) return;
    await Promise.all([sisyphusQuery.refetch(), openspecQuery.refetch()]);
    setRealtimeStatus("active");
    setLastRealtimeSyncAt(getRealtimeSyncLabel());
  };

  const refreshAgentsQueries = async (scopeKey: string) => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: ["agents-md", scopeKey] }),
      queryClient.invalidateQueries({ queryKey: ["agents-skills", scopeKey] }),
      queryClient.invalidateQueries({ queryKey: ["agents-commands", scopeKey] }),
    ]);
  };

  const handleWriteProjectAgentsFile = async (filePath: string, content: string): Promise<void> => {
    if (!activeProject?.pathLabel) return;
    await invoke("write_agents_file", {
      scopeRoot: activeProject.pathLabel,
      filePath,
      content,
    });
    await queryClient.invalidateQueries({ queryKey: ["agents-md", activeProject.pathLabel] });
  };

  const handleWriteGlobalAgentsFile = async (filePath: string, content: string): Promise<void> => {
    await invoke("write_agents_file", {
      scopeRoot: getDirectoryPath(filePath),
      filePath,
      content,
    });
    await queryClient.invalidateQueries({ queryKey: ["agents-md", "global"] });
  };

  const handleRefreshProjectAgents = async (): Promise<void> => {
    if (!activeProject?.pathLabel) return;
    await refreshAgentsQueries(activeProject.pathLabel);
  };

  const handleRefreshGlobalAgents = async (): Promise<void> => {
    await refreshAgentsQueries("global");
  };

  const handlePreviewAgentsSync = async (request: SyncRequest): Promise<SyncReport> => {
    return invoke<SyncReport>("sync_agents_items", { request });
  };

  const handleApplyAgentsSync = async (request: SyncRequest): Promise<SyncReport> => {
    const report = await invoke<SyncReport>("sync_agents_items", { request });
    const conflicts = report.actions.filter((action) => action.action === "conflict");
    if (conflicts.length > 0) {
      setSyncConflictDialog({
        conflicts,
        canRememberChoice: Boolean(request.projectCwd),
        onResolve: ({ items, rememberChoice, rememberedChoice }) => {
          setSyncConflictDialog(null);
          void (async () => {
            if (rememberChoice && rememberedChoice && request.projectCwd) {
              const nextPrefs = {
                ...(projectAgentsPrefsQuery.data ?? DEFAULT_PROJECT_AGENTS_PREFS),
                conflictChoice: rememberedChoice,
              };
              const saveResult = await invoke<SaveProjectAgentsPrefsResult>("save_project_agents_prefs", {
                projectCwd: request.projectCwd,
                prefs: nextPrefs,
              });
              if (saveResult.createdProjectConfigDir) {
                showToast(formatProjectConfigCreatedToast(t("agents.report.projectConfigCreated"), saveResult.storedPath));
              }
              await queryClient.invalidateQueries({ queryKey: ["agents-prefs", request.projectCwd] });
            }

            const nextReport = await invoke<SyncReport>("sync_agents_items", {
              request: {
                ...request,
                dryRun: false,
                items,
              },
            });
            showToast(formatAgentsReportToast(nextReport, t("agents.report.toast")));
            await refreshAgentsQueries(request.projectCwd ?? "global");
          })().catch((error) => {
            showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
          });
        },
      });
      return report;
    }

    showToast(formatAgentsReportToast(report, t("agents.report.toast")));
    await refreshAgentsQueries(request.projectCwd ?? "global");
    return report;
  };

  const handleUpdateProjectAgentsPrefs = async (prefs: ProjectAgentsPrefs): Promise<void> => {
    if (!activeProject?.pathLabel) return;
    const saveResult = await invoke<SaveProjectAgentsPrefsResult>("save_project_agents_prefs", {
      projectCwd: activeProject.pathLabel,
      prefs,
    });
    if (saveResult.createdProjectConfigDir) {
      showToast(formatProjectConfigCreatedToast(t("agents.report.projectConfigCreated"), saveResult.storedPath));
    }
    await queryClient.invalidateQueries({ queryKey: ["agents-prefs", activeProject.pathLabel] });
  };

  const handleResumeSession = async (session: SessionInfo) => {
    if (!session.cwd) { showToast(t("toast.cwdMissing")); return; }
    const exists = await invoke<boolean>("check_directory_exists", { path: session.cwd });
    if (!exists) { showToast(t("toast.cwdMissing")); return; }
    try {
      await invoke("resume_session_in_terminal", {
        provider: session.provider,
        sessionId: session.id,
        cwd: session.cwd,
        terminalPath: settingsQuery.data?.terminalPath || null,
      });
      showToast(t("toast.toolOpened"));
    } catch (error) {
      showToast(error instanceof Error ? error.message : t("toast.toolOpenFailed"));
    }
  };

  const handleOpenProjectInTool = async (project: ProjectGroup, toolType: IdeLauncherType) => {
    if (!project.pathLabel) { showToast(t("toast.cwdMissing")); return; }
    const exists = await invoke<boolean>("check_directory_exists", { path: project.pathLabel });
    if (!exists) { showToast(t("toast.cwdMissing")); return; }
    try {
      await invoke("open_in_tool", {
        toolType,
        cwd: project.pathLabel,
        terminalPath: settingsQuery.data?.terminalPath || null,
        sessionId: null,
      });
      showToast(t("toast.toolOpened"));
    } catch (error) {
      showToast(error instanceof Error ? error.message : t("toast.toolOpenFailed"));
    }
  };

  const handleFocusTerminal = async (session: SessionInfo) => {
    const hint = session.cwd?.split("\\").pop() ?? session.id;
    try {
      await invoke("focus_terminal_window", { titleHint: hint });
    } catch {
      showToast(t("toast.terminalFocusFailed"));
    }
  };

  const [dashboardPeriod, setDashboardPeriod] = useState<"week" | "month">("week");
  const [dashboardViewMode, setDashboardViewMode] = useState<"list" | "kanban">("kanban");
  const [dashboardAnalyticsData, setDashboardAnalyticsData] = useState<AnalyticsDataPoint[]>([]);
  const [dashboardAnalyticsLoading, setDashboardAnalyticsLoading] = useState(false);
  const [dashboardAnalyticsRefreshing, setDashboardAnalyticsRefreshing] = useState(false);
  const [dashboardAnalyticsError, setDashboardAnalyticsError] = useState<string | null>(null);
  const [dashboardAnalyticsFetchedAt, setDashboardAnalyticsFetchedAt] = useState<number | null>(null);

  const dashboardPeriodStart = useMemo(
    () => getDashboardPeriodStart(dashboardPeriod),
    [dashboardPeriod],
  );

  const filteredDashboardSessions = useMemo(
    () => (sessionsQuery.data ?? []).filter((session) => isSessionInUpdatedRange(session, dashboardPeriodStart)),
    [dashboardPeriodStart, sessionsQuery.data],
  );

  // 狀態列統計範圍與看板目前選定的週期(本周/本月)一致，避免顯示歷史全部 session 造成數字落差。
  const { activeSessions, waitingSessions, idleSessions, doneSessions } = useMemo(() => {
    let active = 0;
    let waiting = 0;
    let idle = 0;
    let done = 0;
    for (const s of filteredDashboardSessions) {
      if (s.isArchived) {
        done++;
        continue;
      }
      const activityStatus = activityStatusMap.get(s.id)?.status;
      if (activityStatus === "active") {
        active++;
      } else if (activityStatus === "waiting") {
        waiting++;
      } else if (activityStatus === "idle") {
        idle++;
      }
    }
    return { activeSessions: active, waitingSessions: waiting, idleSessions: idle, doneSessions: done };
  }, [filteredDashboardSessions, activityStatusMap]);

  const filteredDashboardProjects = useMemo(
    () => buildProjectGroups(filteredDashboardSessions, uncategorizedLabel, locale),
    [filteredDashboardSessions, locale, uncategorizedLabel],
  );

  const filteredRecentSessions = useMemo(
    () =>
      [...filteredDashboardSessions]
        .sort((a, b) => (b.updatedAt ?? "").localeCompare(a.updatedAt ?? ""))
        .slice(0, 10),
    [filteredDashboardSessions],
  );

  const filteredDashboardTotals = useMemo(() => {
    return filteredDashboardSessions.reduce(
      (acc, session) => {
        const stats = sessionStatsMap[session.id];
        if (!stats) return acc;
        acc.totalOutputTokens += stats.outputTokens;
        acc.totalInteractions += stats.interactionCount;
        acc.totalCost += Object.values(stats.modelMetrics ?? {}).reduce(
          (sum, metric) => sum + metric.requestsCost,
          0,
        );
        return acc;
      },
      { totalOutputTokens: 0, totalInteractions: 0, totalCost: 0 },
    );
  }, [filteredDashboardSessions, sessionStatsMap]);

  const dashboardProjectSlices = useMemo(() => {
    const palette = ["#6366f1", "#14b8a6", "#f59e0b", "#ef4444", "#8b5cf6", "#06b6d4"];
    return filteredDashboardProjects
      .map((project, index) => {
        const total = project.sessions.reduce((sum, session) => {
          const stats = sessionStatsMap[session.id];
          return sum + (stats?.outputTokens ?? 0) + (stats?.inputTokens ?? 0);
        }, 0);
        return {
          label: project.title,
          value: total,
          color: palette[index % palette.length],
        };
      })
      .filter((slice) => slice.value > 0);
  }, [filteredDashboardProjects, sessionStatsMap]);

  const fetchAnalyticsData = useCallback(
    async (
      cwd: string | null,
      startDate: string,
      endDate: string,
      groupBy: AnalyticsGroupBy,
    ): Promise<AnalyticsDataPoint[] | null> => {
      try {
        return await invoke<AnalyticsDataPoint[]>("get_analytics_data", {
          cwd,
          startDate,
          endDate,
          groupBy,
        });
      } catch (error) {
        showToast(resolveErrorMessage(error, t("analytics.error.loadFailed")));
        return null;
      }
    },
    [t],
  );

  const fetchDashboardAnalytics = useCallback(async () => {
    const startDate = formatDateInput(new Date(dashboardPeriodStart));
    const endDate = formatDateInput(new Date());
    const hasExistingData = dashboardAnalyticsData.length > 0;

    setDashboardAnalyticsError(null);
    if (hasExistingData) {
      setDashboardAnalyticsRefreshing(true);
    } else {
      setDashboardAnalyticsLoading(true);
    }

    try {
      const result = await invoke<AnalyticsDataPoint[]>("get_analytics_data", {
        cwd: null,
        startDate,
        endDate,
        groupBy: "day",
      });
      setDashboardAnalyticsData(result);
      setDashboardAnalyticsFetchedAt(Date.now());
    } catch (error) {
      setDashboardAnalyticsError(resolveErrorMessage(error, t("analytics.error.loadFailed")));
    } finally {
      setDashboardAnalyticsLoading(false);
      setDashboardAnalyticsRefreshing(false);
    }
  }, [dashboardAnalyticsData.length, dashboardPeriodStart, t]);

  const planPreviewHtml = useMemo(
    () =>
      DOMPurify.sanitize(
        marked.parse(planDraft || "_Empty plan_", { async: false }),
      ),
    [planDraft],
  );

  const openProjectTab = (projectKey: string) => {
    setOpenProjectKeys((v) => (v.includes(projectKey) ? v : [...v, projectKey]));
    setActiveView(projectKey);
  };

  const closeProjectTab = (projectKey: string) => {
    setOpenProjectKeys((v) => v.filter((k) => k !== projectKey));
    setActiveView((v) => (v === projectKey ? "dashboard" : v));
  };

  const handleSaveSettings = async () => {
    const next = buildSettingsPayload();

    const requiresCopilotRoot = next.enabledProviders.includes("copilot");
    const requiresOpencodeRoot = next.enabledProviders.includes("opencode");
    const requiresCodexRoot = next.enabledProviders.includes("codex");

    if (requiresCopilotRoot && !next.copilotRoot) {
      showToast(t("toast.settingsRootRequired"));
      return;
    }

    if (requiresOpencodeRoot && !next.opencodeRoot) {
      showToast(t("toast.settingsRootRequired"));
      return;
    }

    if (requiresCodexRoot && !next.codexRoot) {
      showToast(t("toast.settingsRootRequired"));
      return;
    }

    if (next.terminalPath) {
      const isValid = await invoke<boolean>("validate_terminal_path", { path: next.terminalPath });
      if (!isValid) {
        showToast(t("toast.terminalInvalid"));
        return;
      }
    }

    settingsMutation.mutate(next);
  };

  const handleBrowseDirectory = async (field: "copilotRoot" | "opencodeRoot" | "codexRoot" | "claudeRoot" | "antigravityRoot" | "agentsSourceRoot") => {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") setSettingsForm((v) => ({ ...v, [field]: selected }));
  };

  const handleBrowseFile = async (field: "terminalPath" | "externalEditorPath") => {
    const selected = await open({ directory: false, multiple: false });
    if (typeof selected === "string") setSettingsForm((v) => ({ ...v, [field]: selected }));
  };

  const handleToggleArchived = async (nextValue: boolean) => {
    const next = buildSettingsPayload({ showArchived: nextValue });
    setSettingsForm((v) => ({ ...v, showArchived: nextValue }));
    settingsMutation.mutate(next);
  };

  const handleToggleAnalyticsPanel = async () => {
    const nextCollapsed = !(settingsForm.analyticsPanelCollapsed ?? false);
    setSettingsForm((current) => ({ ...current, analyticsPanelCollapsed: nextCollapsed }));
    await persistSettingsSilently(buildSettingsPayload({ analyticsPanelCollapsed: nextCollapsed }));
    const refreshIntervalMs = (settingsForm.analyticsRefreshInterval ?? 30) * 60_000;
    if (!nextCollapsed && (!dashboardAnalyticsFetchedAt || Date.now() - dashboardAnalyticsFetchedAt >= refreshIntervalMs)) {
      await fetchDashboardAnalytics();
    }
  };

  const handleArchiveSession = (session: SessionInfo) => {
    setConfirmDialog({
      title: t("dialog.archiveTitle"),
      message: `${t("session.confirm.archive")} ${session.summary?.trim() || session.id}?`,
      actionLabel: t("session.actions.archive"),
      tone: "primary",
      onConfirm: () => archiveMutation.mutate(session.id),
    });
  };

  const handleUnarchiveSession = (session: SessionInfo) => {
    setConfirmDialog({
      title: t("dialog.archiveTitle"),
      message: `${t("session.confirm.archive")} ${session.summary?.trim() || session.id}?`,
      actionLabel: t("session.actions.unarchive"),
      tone: "primary",
      onConfirm: () => unarchiveMutation.mutate(session.id),
    });
  };

  const handleDeleteSession = (session: SessionInfo) => {
    setConfirmDialog({
      title: t("dialog.deleteTitle"),
      message: `${t("session.confirm.delete")} ${session.summary?.trim() || session.id}?`,
      actionLabel: t("session.actions.delete"),
      tone: "danger",
      onConfirm: () => deleteMutation.mutate(session.id),
    });
  };

  const handleDeleteEmptySessions = (emptyCount: number) => {
    if (emptyCount === 0) return;
    setConfirmDialog({
      title: t("dialog.deleteEmptyTitle"),
      message: t("session.confirm.deleteEmpty").replace("{count}", String(emptyCount)),
      actionLabel: t("session.actions.deleteEmpty"),
      tone: "danger",
      onConfirm: () => deleteEmptySessionsMutation.mutate(),
    });
  };

  const handleCopyCommand = async (session: SessionInfo) => {
    const command = getSessionOpenCommand(session.provider, session.id);
    if (!command) {
      showToast(t("toast.commandUnavailable"));
      return;
    }
    await navigator.clipboard.writeText(command);
    showToast(t("toast.commandCopied"));
  };

  const handleEditNotes = (session: SessionInfo) => {
    setEditDialog({
      key: `notes:${session.id}:${session.updatedAt ?? ""}`,
      title: t("session.actions.editNotes"),
      message: t("session.prompt.notes"),
      actionLabel: t("session.actions.editNotes"),
      initialValue: session.notes ?? "",
      multiline: true,
      onConfirm: (nextNotes) =>
        saveMetaMutation.mutate({
          sessionId: session.id,
          notes: nextNotes.trim() || null,
          tags: session.tags,
        }),
    });
  };

  const handleEditTags = (session: SessionInfo) => {
    setEditDialog({
      key: `tags:${session.id}:${session.updatedAt ?? ""}`,
      title: t("session.actions.editTags"),
      message: t("session.prompt.tags"),
      actionLabel: t("session.actions.editTags"),
      initialValue: session.tags.join(", "),
      onConfirm: (nextValue) => {
        const tags = normalizeTags(nextValue.split(","));
        saveMetaMutation.mutate({ sessionId: session.id, notes: session.notes ?? null, tags });
      },
    });
  };

  const handleEditSingleTag = (session: SessionInfo, tag: string, tagIndex: number) => {
    setEditDialog({
      key: `tag:${session.id}:${tagIndex}:${tag}:${session.updatedAt ?? ""}`,
      title: t("session.actions.editTags"),
      message: t("session.prompt.singleTag"),
      actionLabel: t("session.actions.editTags"),
      secondaryActionLabel: t("session.actions.deleteTag"),
      secondaryActionTone: "danger",
      initialValue: tag,
      onConfirm: (nextValue) => {
        const normalized = nextValue.trim();
        const nextTags = session.tags.flatMap((currentTag, currentIndex) => {
          if (currentIndex !== tagIndex) return [currentTag];
          return normalized ? [normalized] : [];
        });
        saveMetaMutation.mutate({
          sessionId: session.id,
          notes: session.notes ?? null,
          tags: normalizeTags(nextTags),
        });
      },
      onSecondaryAction: () => {
        const nextTags = session.tags.filter((_, currentIndex) => currentIndex !== tagIndex);
        saveMetaMutation.mutate({
          sessionId: session.id,
          notes: session.notes ?? null,
          tags: normalizeTags(nextTags),
        });
      },
    });
  };

  const handleOpenPlan = (session: SessionInfo) => {
    setActivePlanSessionId(session.id);
  };

  const handleSavePlan = () => {
    if (!activePlanSession) return;
    savePlanMutation.mutate({ sessionDir: activePlanSession.sessionDir, content: planDraft });
  };

  const handleProviderAction = (provider: string, action: ProviderIntegrationAction) => {
    setPendingProviderAction(`${provider}:${action}`);
    providerIntegrationMutation.mutate(
      { provider, action },
      { onSettled: () => setPendingProviderAction(null) },
    );
  };

  const handleRefreshQuota = useCallback((provider?: string) => {
    void invoke<QuotaSnapshot[]>("refresh_quota", { provider: provider ?? null })
      .then((snapshots) => {
        const rateLimited = snapshots.find((s) => s.status === "rate_limited");
        if (rateLimited) {
          showToast(rateLimited.errorMessage ?? t("quota.rateLimited"));
        }
        return queryClient.invalidateQueries({ queryKey: ["quota_snapshots"] });
      })
      .catch(() => null);
  }, [queryClient]);

  const handleOpenProviderPath = async (integration: ProviderIntegrationStatus) => {
    const targetPath = resolveProviderTargetPath(integration);
    if (!targetPath) {
      showToast(t("toast.providerPathUnavailable"));
      return;
    }

    try {
      await revealItemInDir(targetPath);
      showToast(t("toast.providerPathOpened"));
    } catch (error) {
      showToast(resolveErrorMessage(error, t("toast.providerPathOpenFailed")));
    }
  };

  const handleEditProviderPath = async (integration: ProviderIntegrationStatus) => {
    const targetPath = resolveProviderTargetPath(integration);
    if (!targetPath) {
      showToast(t("toast.providerPathUnavailable"));
      return;
    }

    const editorPath = settingsForm.externalEditorPath?.trim();

    try {
      if (editorPath) {
        await openPath(targetPath, editorPath);
      } else {
        await openPath(targetPath);
      }
      showToast(t("toast.providerPathEdited"));
    } catch (error) {
      if (editorPath) {
        try {
          await openPath(targetPath);
          showToast(t("toast.providerPathEdited"));
          return;
        } catch (fallbackError) {
          showToast(resolveErrorMessage(fallbackError, t("toast.providerPathEditFailed")));
          return;
        }
      }

      showToast(resolveErrorMessage(error, t("toast.providerPathEditFailed")));
    }
  };

  const handleOpenPlanExternal = async (session: SessionInfo) => {
    try {
      await invoke("open_plan_external", {
        sessionDir: session.sessionDir,
        editorCmd: settingsQuery.data?.externalEditorPath ?? null,
      });
      showToast(t("toast.planOpenedExternal"));
    } catch (error) {
      showToast(error instanceof Error ? error.message : t("toast.planOpenFailed"));
    }
  };

  useEffect(() => {
    if (activeView !== "dashboard" || settingsForm.analyticsPanelCollapsed) return;
    void fetchDashboardAnalytics();
  }, [activeView, dashboardPeriod, settingsForm.analyticsPanelCollapsed, fetchDashboardAnalytics]);

  useEffect(() => {
    if (activeView !== "dashboard" || settingsForm.analyticsPanelCollapsed) return undefined;
    const intervalMs = (settingsForm.analyticsRefreshInterval ?? 30) * 60_000;
    const timer = window.setInterval(() => {
      void fetchDashboardAnalytics();
    }, intervalMs);
    return () => window.clearInterval(timer);
  }, [
    activeView,
    settingsForm.analyticsPanelCollapsed,
    settingsForm.analyticsRefreshInterval,
    fetchDashboardAnalytics,
  ]);

  const handleOpenPathExternal = (path: string) => {
    openPath(path).catch((error) => {
      showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
    });
  };

  const handleRevealPathInDir = (path: string) => {
    revealItemInDir(path).catch((error) => {
      showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
    });
  };

  // 全域 Agents 資料集合：供全域 Agents 頁（獨立元件）與專案分頁的 globalData prop 共用。
  const globalAgentsData: AgentsScopeDataBundle = {
    scope: { kind: "global" },
    agentsMdData: globalAgentsMdQuery.data,
    skillsData: globalAgentsSkillsQuery.data,
    commandsData: globalAgentsCommandsQuery.data,
    prefs: globalAgentsPrefs,
    isAgentsMdLoading: globalAgentsMdQuery.isLoading,
    isSkillsLoading: globalAgentsSkillsQuery.isLoading,
    isCommandsLoading: globalAgentsCommandsQuery.isLoading,
    isPrefsLoading: false,
    onRefreshAgentsMd: handleRefreshGlobalAgents,
    onRefreshSkills: handleRefreshGlobalAgents,
    onRefreshCommands: handleRefreshGlobalAgents,
    onPreviewSync: handlePreviewAgentsSync,
    onApplySync: handleApplyAgentsSync,
    onUpdatePrefs: async (prefs) => {
      setGlobalAgentsPrefs(prefs);
    },
    agentsRootLinkStatus: agentsSourceRootConfigured ? (agentsRootLinkQuery.data ?? null) : null,
    onCreateAgentsRootLink: async () => {
      await linkAgentsRootMutation.mutateAsync();
    },
    mcpProviders: globalMcpConfigsQuery.data ?? [],
    mcpLoading: globalMcpConfigsQuery.isLoading,
    onRefreshMcp: async () => { await globalMcpConfigsQuery.refetch(); },
    onUpsertMcpServer: (provider, name, originalName, configJson) =>
      upsertMcpServerMutation.mutateAsync({ scope: { kind: "global" }, provider, name, originalName, configJson }),
    onDeleteMcpServer: (provider, name) =>
      deleteMcpServerMutation.mutateAsync({ scope: { kind: "global" }, provider, name }),
    onSetMcpServerEnabled: (provider, name, enabled) =>
      setMcpServerEnabledMutation.mutateAsync({ scope: { kind: "global" }, provider, name, enabled }),
  };

  // 全域 Agents 檢視（含 MCP 頁籤）：獨立元件，供全域 Agents 頁（agents-global）渲染。
  const globalAgentsView = (
    <AgentsConfigView
      {...globalAgentsData}
      onReadFile={handleReadAgentsFile}
      onWriteFile={handleWriteGlobalAgentsFile}
      onOpenExternal={handleOpenPathExternal}
      onRevealPath={handleRevealPathInDir}
    />
  );

  return (
    <main className={`app-shell ${isSidebarCollapsed ? "sidebar-collapsed" : ""}`}>
      <Sidebar
        activeView={activeView}
        isSidebarCollapsed={isSidebarCollapsed}
        realtimeStatus={realtimeStatus}
        lastRealtimeSyncAt={lastRealtimeSyncAt}
        sessionsIsFetching={sessionsQuery.isFetching}
        pinnedProjects={pinnedProjects}
        projectGroups={groupedProjects}
        openProjectKeys={openProjectKeys}
        onNavigate={(view) => setActiveView(view)}
        onOpenProject={openProjectTab}
        onCloseProject={closeProjectTab}
        onClearOpenProjects={clearOpenProjects}
        onReorderOpenProjects={reorderOpenProjects}
        onPinProject={(key) => void pinProjectViaDrag(key)}
        onCollapseToggle={() => setIsSidebarCollapsed((v) => !v)}
        onRefresh={() => sessionsQuery.refetch()}
        onConfigurePath={() => setActiveView("settings")}
      />

      <section className="workspace">
        <header className="workspace-header">
            <div>
              <div className="workspace-title-row">
                <h2 className="workspace-title">
                  {activeView === "dashboard"
                    ? t("tabs.dashboard")
                    : activeView === "agents-global"
                      ? t("agents.nav")
                    : activeView === "settings"
                      ? t("settings.title")
                      : activeProject?.title ?? ""}
                </h2>
                {activeView !== "dashboard" && activeView !== "agents-global" && activeView !== "settings" && activeProject?.branchLabel ? (
                  <span className="project-branch-badge">{activeProject.branchLabel}</span>
                ) : null}
              </div>
              <p className="workspace-subtitle">
                {activeView === "dashboard"
                  ? t("dashboard.subtitle")
                  : activeView === "agents-global"
                    ? t("agents.globalSubtitle")
                  : activeView === "settings"
                    ? t("settings.subtitle")
                    : activeProject?.pathLabel ?? ""}
              </p>
            </div>
          </header>


        <div className="workspace-content">
          {activeView === "dashboard" ? (
            <DashboardView
              sessionsIsLoading={sessionsQuery.isLoading}
              sessionsIsFetching={sessionsQuery.isFetching}
              sessionsIsError={sessionsQuery.isError}
              sessionsError={sessionsQuery.error}
              groupedProjects={filteredDashboardProjects}
              recentSessions={filteredRecentSessions}
              dashboardPeriod={dashboardPeriod}
              onPeriodChange={setDashboardPeriod}
              filteredTotalOutputTokens={filteredDashboardTotals.totalOutputTokens}
              filteredTotalInteractions={filteredDashboardTotals.totalInteractions}
              filteredTotalCost={filteredDashboardTotals.totalCost}
              onOpenProject={openProjectTab}
              onOpenRecentSession={(session) =>
                openProjectTab(getProjectKey(session, uncategorizedLabel))
              }
              activityStatusMap={activityStatusMap}
              onResumeSession={(session) => void handleResumeSession(session)}
              onFocusTerminal={(session) => void handleFocusTerminal(session)}
              viewMode={dashboardViewMode}
              onViewModeChange={setDashboardViewMode}
              analyticsData={dashboardAnalyticsData}
              analyticsError={dashboardAnalyticsError}
              analyticsLoading={dashboardAnalyticsLoading}
              analyticsRefreshing={dashboardAnalyticsRefreshing}
              analyticsProjectSlices={dashboardProjectSlices}
              analyticsCollapsed={settingsForm.analyticsPanelCollapsed ?? false}
              onAnalyticsRetry={() => void fetchDashboardAnalytics()}
              onAnalyticsToggleCollapsed={() => void handleToggleAnalyticsPanel()}
              quotaSnapshots={quotaSnapshotQuery.data ?? []}
              enableQuotaMonitoring={settingsForm.enableQuotaMonitoring ?? true}
              quotaEnabledProviders={settingsForm.quotaEnabledProviders ?? []}
              onRefreshQuota={handleRefreshQuota}
            />
          ) : null}

          {activeView === "settings" ? (
            <SettingsView
              settingsForm={settingsForm}
              onFormChange={setSettingsForm}
              onSave={() => void handleSaveSettings()}
              onBrowseDirectory={handleBrowseDirectory}
              onBrowseFile={handleBrowseFile}
              onDetectTerminal={() => detectTerminalMutation.mutate()}
              onDetectVscode={() => detectVscodeMutation.mutate()}
              onProviderAction={handleProviderAction}
              onOpenProviderPath={(integration) => void handleOpenProviderPath(integration)}
              onEditProviderPath={(integration) => void handleEditProviderPath(integration)}
              pendingProviderAction={pendingProviderAction}
              onOpenEventMonitor={() => setShowEventMonitor(true)}
              jqAvailable={jqAvailableQuery.data ?? null}
              onRefreshQuota={handleRefreshQuota}
            />
          ) : null}

          {activeView === "agents-global" ? globalAgentsView : null}

          {activeProject ? (
            <ProjectView
              project={activeProject}
              showArchived={settingsForm.showArchived}
              hideEmptySessions={hideEmptySessions}
              onHideEmptySessionsChange={setHideEmptySessions}
              totalEmptySessions={deletableEmptySessionCount}
              onToggleArchived={(v) => void handleToggleArchived(v)}
              onCopyCommand={(s) => void handleCopyCommand(s)}
              onEditNotes={handleEditNotes}
              onEditTags={handleEditTags}
              onEditTag={handleEditSingleTag}
              onOpenPlan={handleOpenPlan}
              onArchive={handleArchiveSession}
              onUnarchive={handleUnarchiveSession}
              onDelete={handleDeleteSession}
              onDeleteEmptySessions={() => handleDeleteEmptySessions(deletableEmptySessionCount)}
              isPinned={pinnedProjects.includes(activeProject.key)}
              onTogglePin={() => void togglePinProject(activeProject.key)}
              sessionStats={sessionStatsMap}
              sessionStatsLoading={sessionStatsLoadingMap}
              sessionTodos={sessionTodosMap}
              sessionTodosLoading={sessionTodosLoadingMap}
              sessionsLoading={sessionsQuery.isLoading}
              sisyphusData={sisyphusQuery.data}
              openspecData={openspecQuery.data}
              agentsMdData={projectAgentsMdQuery.data}
              skillsData={projectAgentsSkillsQuery.data}
              commandsData={projectAgentsCommandsQuery.data}
              projectAgentsPrefs={projectAgentsPrefsQuery.data ?? DEFAULT_PROJECT_AGENTS_PREFS}
              plansSpecsLoading={sisyphusQuery.isLoading || openspecQuery.isLoading}
              plansSpecsRefreshing={sisyphusQuery.isFetching || openspecQuery.isFetching}
              agentsMdLoading={projectAgentsMdQuery.isLoading}
              skillsLoading={projectAgentsSkillsQuery.isLoading}
              commandsLoading={projectAgentsCommandsQuery.isLoading}
              agentsPrefsLoading={projectAgentsPrefsQuery.isLoading}
              onReadFileContent={handleReadFileContent}
              onReadOpenspecFile={handleReadOpenspecFile}
              onWriteOpenspecFile={handleWriteOpenspecFile}
              onWriteAgentsFile={handleWriteProjectAgentsFile}
              onRefreshPlansSpecs={handleRefreshPlansSpecs}
              onRefreshAgentsMd={handleRefreshProjectAgents}
              onRefreshAgentsSkills={handleRefreshProjectAgents}
              onRefreshAgentsCommands={handleRefreshProjectAgents}
              plansSpecsRefreshToken={`${sisyphusQuery.dataUpdatedAt}:${openspecQuery.dataUpdatedAt}`}
              onOpenAgentsExternal={(path) => {
                openPath(path).catch((error) => {
                  showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
                });
              }}
              onRevealAgentsPath={(path) => {
                revealItemInDir(path).catch((error) => {
                  showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
                });
              }}
              onPreviewAgentsSync={handlePreviewAgentsSync}
              onApplyAgentsSync={handleApplyAgentsSync}
              onUpdateProjectAgentsPrefs={handleUpdateProjectAgentsPrefs}
              mcpProviders={projectMcpConfigsQuery.data ?? []}
              mcpLoading={projectMcpConfigsQuery.isLoading}
              onRefreshMcp={async () => { await projectMcpConfigsQuery.refetch(); }}
              onUpsertMcpServer={(provider, name, originalName, configJson) =>
                upsertMcpServerMutation.mutateAsync({
                  scope: { kind: "project", projectCwd: activeProject.pathLabel },
                  provider,
                  name,
                  originalName,
                  configJson,
                })
              }
              onDeleteMcpServer={(provider, name) =>
                deleteMcpServerMutation.mutateAsync({
                  scope: { kind: "project", projectCwd: activeProject.pathLabel },
                  provider,
                  name,
                })
              }
              onSetMcpServerEnabled={(provider, name, enabled) =>
                setMcpServerEnabledMutation.mutateAsync({
                  scope: { kind: "project", projectCwd: activeProject.pathLabel },
                  provider,
                  name,
                  enabled,
                })
              }
              codexTrusted={codexProjectTrustQuery.data ?? false}
              globalAgentsData={globalAgentsData}
              activePlanSessionId={activePlanSessionId}
              onActivePlanChange={setActivePlanSessionId}
              planDraft={planDraft}
              planPreviewHtml={planPreviewHtml}
              onPlanDraftChange={setPlanDraft}
              onSavePlan={handleSavePlan}
              onOpenPlanExternal={(s) => void handleOpenPlanExternal(s)}
              openDetailKeys={getProjectSubTabState(activeProject.key).openDetailKeys}
              activeSubTab={getProjectSubTabState(activeProject.key).activeSubTab}
              onSubTabStateChange={(state) => handleSubTabStateChange(activeProject.key, state)}
              onFetchAnalytics={(cwd, startDate, endDate, groupBy) =>
                fetchAnalyticsData(cwd, startDate, endDate, groupBy)
              }
              activityStatusMap={activityStatusMap}
              onResumeSession={(session) => void handleResumeSession(session)}
              onFocusTerminal={(session) => void handleFocusTerminal(session)}
              onOpenProjectInTool={(project, tool) => void handleOpenProjectInTool(project, tool)}
              defaultLauncher={settingsQuery.data?.defaultLauncher ?? null}
              toolAvailability={toolAvailabilityQuery.data ?? null}
            />
          ) : null}
        </div>

        {(settingsForm.showStatusBar ?? true) ? (
          <StatusBar
            lastBridgeEvent={lastBridgeEvent}
            onOpenEventMonitor={() => setShowEventMonitor(true)}
            activeSessions={activeSessions}
            waitingSessions={waitingSessions}
            idleSessions={idleSessions}
            doneSessions={doneSessions}
            isLoadingSessions={sessionsQuery.isLoading}
            providerQuotas={providerQuotaQuery.data ?? []}
            quotaSnapshots={quotaSnapshotQuery.data ?? []}
            quotaEnabledProviders={settingsForm.quotaEnabledProviders ?? []}
            onRefreshQuota={handleRefreshQuota}
          />
        ) : null}
      </section>

      {confirmDialog ? (
        <ConfirmDialog dialog={confirmDialog} onCancel={() => setConfirmDialog(null)} />
      ) : null}

      {editDialog ? (
        <EditDialog
          key={editDialog.key ?? editDialog.title}
          dialog={editDialog}
          onCancel={() => setEditDialog(null)}
          onConfirm={(value) => {
            editDialog.onConfirm(value);
            setEditDialog(null);
          }}
        />
      ) : null}

      {showEventMonitor ? (
        <BridgeEventMonitorDialog
          events={bridgeEventLog}
          onClose={() => setShowEventMonitor(false)}
          onClear={() => setBridgeEventLog([])}
        />
      ) : null}

      {syncConflictDialog ? (
        <SyncConflictDialog
          conflicts={syncConflictDialog.conflicts}
          canRememberChoice={syncConflictDialog.canRememberChoice}
          onResolve={syncConflictDialog.onResolve}
          onCancel={() => setSyncConflictDialog(null)}
        />
      ) : null}

      {toastMessage ? <div className="toast-banner">{toastMessage}</div> : null}
    </main>
  );
}

export default App;
