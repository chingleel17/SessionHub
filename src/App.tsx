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
  AnalyticsQuery,
  AnalyticsReport,
  AnalyticsSessionPage,
  AnalyticsSessionSort,
  AppSettings,
  BridgeEventLogEntry,
  CommandsScanResult,
  ConfirmDialogState,
  EditDialogState,
  IdeLauncherType,
  McpProviderConfig,
  ManualModelPricingInput,
  ModelPricingEntry,
  OpenSpecData,
  ProjectAgentsPrefs,
  ProjectGroup,
  ProjectSubTabState,
  ProviderIntegrationStatus,
  ProviderQuota,
  QuotaSnapshot,
  ResetCreditAutomationStatus,
  SaveProjectAgentsPrefsResult,
  SessionActivityStatus,
  SessionInfo,
  SessionStats,
  SessionSearchTarget,
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
import { rebuildPinnedProjectOrder } from "./utils/reorderPinnedProjects";
import { resolveErrorMessage } from "./utils/resolveErrorMessage";
import { createCodexResetCreditConfirmation } from "./utils/codexResetConfirmation";
import { newerResetCreditStatus, resetCreditMessage, validResetCreditSettings } from "./utils/resetCreditAutomation";
import { updateSessionMetadataCache } from "./utils/updateSessionMetadataCache";
import { RESOURCE_QUERY_OPTIONS, resourceCanLoad, resourceKeyId, resourceQueryKey, refreshResource, updateResourceAfterWrite, writtenSyncKinds } from "./utils/agentsResourceCache";
import { canLoadGlobalAgents, mergeAgentsMdLayers, mergeSkillsLayers, scanAgentsInLayers } from "./utils/agentsLayeredScan";
import {
  createDashboardAnalyticsQuery,
  createDefaultAnalyticsQuery,
  resolveAnalyticsTimeZone,
} from "./utils/analyticsTimeZone";
import { useSessionRealtimeEvents } from "./hooks/useSessionRealtimeEvents";
import { useAppSettingsForm, type ProviderIntegrationAction } from "./hooks/useAppSettingsForm";

import { AgentsConfigView, type AgentsScopeDataBundle } from "./components/AgentsConfigView";
import type { McpConnectionTestResult } from "./components/McpConfigView";
import { ConfirmDialog } from "./components/ConfirmDialog";
import { GlobalContextMenu } from "./components/GlobalContextMenu";
import { BridgeEventMonitorDialog } from "./components/BridgeEventMonitorDialog";
import { DashboardView } from "./components/DashboardView";
import { EditDialog } from "./components/EditDialog";
import { ProjectView } from "./components/ProjectView";
import { ProjectPickerDialog } from "./components/ProjectPickerDialog";
import { SettingsView } from "./components/SettingsView";
import { Sidebar } from "./components/Sidebar";
import { SyncConflictDialog } from "./components/SyncConflictDialog";
import { StatusBar } from "./components/StatusBar";
import { TagEditDialog } from "./components/TagEditDialog";
import { UsageAnalyticsView } from "./components/UsageAnalyticsView";

// ─── helpers ─────────────────────────────────────────────────────────────────

function normalizePath(path: string): string {
  // Windows 路徑大小寫不敏感，正規化為小寫用於分組比對
  return path.replace(/\//g, "\\").toLowerCase();
}

function formatDisplayPath(path: string): string {
  return path.replace(/^[a-z]:/, (drive) => drive.toUpperCase());
}

// 舊格式 key 為 `path:branch`，切換分支會使釘選失效；新格式僅以工作目錄為準。
// git worktree 的 `rev-parse --show-toplevel` 會回傳 worktree 自身路徑，因此不同 worktree 天然分開。
function normalizePinnedProjectKey(projectKey: string): string {
  const branchSeparatorIndex = projectKey.lastIndexOf(":");
  const projectPath =
    branchSeparatorIndex <= 1 ? projectKey : projectKey.slice(0, branchSeparatorIndex);
  return normalizePath(projectPath);
}

function getProjectKey(session: SessionInfo, uncategorizedLabel: string): string {
  const raw = session.repoRoot?.trim() || session.cwd?.trim();
  if (!raw) return uncategorizedLabel;
  return normalizePath(raw);
}

function getProjectDisplayPath(session: SessionInfo, uncategorizedLabel: string): string {
  return formatDisplayPath(session.repoRoot?.trim() || session.cwd?.trim() || uncategorizedLabel);
}

function getProjectTitle(session: SessionInfo, displayPath: string, uncategorizedLabel: string): string {
  if (displayPath === uncategorizedLabel) return uncategorizedLabel;
  const repoName = session.repoName?.trim();
  if (repoName) return repoName;
  const parts = displayPath.replace(/\//g, "\\").split("\\").filter(Boolean);
  return parts[parts.length - 1] ?? displayPath;
}

// 同一目錄可能累積多個分支的 session，顯示最近更新者的分支代表目前工作中的分支
function getProjectBranchLabel(sessions: SessionInfo[]): string | null {
  return (
    [...sessions]
      .sort((a, b) => (b.updatedAt ?? "").localeCompare(a.updatedAt ?? ""))
      .map((session) => session.gitBranch?.trim())
      .find((branch): branch is string => Boolean(branch)) ?? null
  );
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
  enabledTargets: ["claude", "opencode", "codex", "copilot"],
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
  const [analyticsQueryByScope, setAnalyticsQueryByScope] = useState<Record<string, AnalyticsQuery>>({});
  const [analyticsPageByScope, setAnalyticsPageByScope] = useState<Record<string, number>>({});
  const [analyticsPageSize, setAnalyticsPageSize] = useState(10);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);
  const [showEmptySessions, setShowEmptySessions] = useState(false);
  const [pinnedProjects, setPinnedProjects] = useState<string[]>([]);
  const pinnedProjectsRef = useRef<string[]>([]);
  const pinnedPersistedProjectsRef = useRef<string[]>([]);
  const pinnedSaveQueueRef = useRef<Promise<boolean>>(Promise.resolve(true));
  const pinnedSaveRequestRef = useRef(0);
  const pinnedSavePendingCountRef = useRef(0);

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
  const showToast = useCallback((message: string) => setToastMessage(message), []);
  const [confirmDialog, setConfirmDialog] = useState<ConfirmDialogState | null>(null);
  const [resetCreditBusy, setResetCreditBusy] = useState(false);
  const [editDialog, setEditDialog] = useState<EditDialogState | null>(null);
  const [tagEditDialog, setTagEditDialog] = useState<{
    session: SessionInfo;
    tags: string[];
  } | null>(null);
  const [showProjectPicker, setShowProjectPicker] = useState(false);
  const [projectPickerBusyKey, setProjectPickerBusyKey] = useState<string | null>(null);
  const [globalAgentsPrefs, setGlobalAgentsPrefs] = useState<ProjectAgentsPrefs>(DEFAULT_PROJECT_AGENTS_PREFS);
  const [globalAgentsTab, setGlobalAgentsTab] = useState<"agents-md" | "skills" | "commands" | "mcp">("agents-md");
  const [projectAgentsTabs, setProjectAgentsTabs] = useState<Record<string, "agents-md" | "skills" | "commands" | "mcp">>({});
  const [syncConflictDialog, setSyncConflictDialog] = useState<{
    conflicts: SyncActionResult[];
    canRememberChoice: boolean;
    onResolve: (result: { items: SyncRequest["items"]; rememberChoice: boolean; rememberedChoice: "source-wins" | "target-wins" | null }) => void;
  } | null>(null);
  const dirtyAgentsResourcesRef = useRef(new Set<string>());
  const activeAgentsResourcesRef = useRef(new Set<string>());

  const [realtimeStatus, setRealtimeStatus] = useState<"connecting" | "active" | "error">(
    "connecting",
  );
  const [lastRealtimeSyncAt, setLastRealtimeSyncAt] = useState<string | null>(null);
  // 下一次 sessions 掃描是否強制全掃。用 ref 而非 state，避免它進入 queryKey 造成 fetch 過程中
  // queryKey 變動而連續觸發兩次掃描（async 化後會讓 isFetching 永遠為 true，狀態列卡在「掃描中」）。
  const forceFullRef = useRef(false);
  const [pendingProviderAction, setPendingProviderAction] = useState<string | null>(null);
  const launchInProgressRef = useRef(false);
  const [launchingTarget, setLaunchingTarget] = useState<string | null>(null);

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
      settingsQuery.data?.copilotRoot ?? "",
      settingsQuery.data?.opencodeRoot ?? "",
      settingsQuery.data?.codexRoot ?? "",
      settingsQuery.data?.claudeRoot ?? "",
      settingsQuery.data?.antigravityRoot ?? "",
      settingsQuery.data?.showArchived ?? false,
      settingsQuery.data?.enabledProviders ?? [],
    ],
    enabled: Boolean(settingsQuery.data),
    staleTime: Infinity,
    queryFn: async () => {
      const sessions = await invoke<SessionInfo[]>("get_sessions_cached", {
        showArchived: settingsQuery.data?.showArchived,
        enabledProviders: settingsQuery.data?.enabledProviders,
      });
      const roots: Record<string, string | undefined> = {
        copilot: settingsQuery.data?.copilotRoot,
        opencode: settingsQuery.data?.opencodeRoot,
        codex: settingsQuery.data?.codexRoot,
        claude: settingsQuery.data?.claudeRoot,
        antigravity: settingsQuery.data?.antigravityRoot,
      };
      return sessions.filter((session) => {
        const root = roots[session.provider];
        return !root || normalizePath(session.sessionDir).startsWith(`${normalizePath(root).replace(/\\+$/, "")}\\`);
      });
    },
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

  const resetCreditStatusQuery = useQuery({
    queryKey: ["reset_credit_automation"],
    queryFn: async () => {
      const incoming = await invoke<ResetCreditAutomationStatus>("get_reset_credit_automation_status");
      return newerResetCreditStatus(queryClient.getQueryData<ResetCreditAutomationStatus>(["reset_credit_automation"]), incoming);
    },
    refetchInterval: 10_000,
    staleTime: 2_000,
  });
  const currentCodexScope = quotaSnapshotQuery.data?.find((snapshot) => snapshot.provider === "codex")?.accountScope;
  const statusAccountScope = resetCreditStatusQuery.data?.accountScope;
  const scopedResetCreditStatus = currentCodexScope && statusAccountScope && currentCodexScope !== statusAccountScope ? undefined : resetCreditStatusQuery.data;
  const resetCreditOperationBusy = resetCreditBusy || (scopedResetCreditStatus?.accountScope === currentCodexScope && scopedResetCreditStatus?.autoUseState === "running");

  useEffect(() => {
    const listener = listen<ResetCreditAutomationStatus>("reset-credit-automation-updated", ({ payload }) => {
      queryClient.setQueryData<ResetCreditAutomationStatus>(["reset_credit_automation"], (current) => newerResetCreditStatus(current, payload));
      void queryClient.invalidateQueries({ queryKey: ["quota_snapshots"] });
    }).catch(() => null);
    return () => { void listener.then((unlisten) => unlisten?.()); };
  }, [queryClient]);

  useEffect(() => {
    void invoke("set_reset_credit_notification_locale", { locale }).catch(() => null);
  }, [locale]);

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
    if (settingsQuery.data && pinnedSavePendingCountRef.current === 0) {
      // 舊格式多分支 key 正規化後會塌成同一個 path，需去重避免釘選區重複項目
      const nextPinnedProjects = [
        ...new Set((settingsQuery.data.pinnedProjects ?? []).map(normalizePinnedProjectKey)),
      ];
      pinnedProjectsRef.current = nextPinnedProjects;
      pinnedPersistedProjectsRef.current = nextPinnedProjects;
      setPinnedProjects(nextPinnedProjects);
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
  }, [settingsQuery.data, showToast, t]);


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

  const searchSessionContent = (query: string, sessions: SessionSearchTarget[]) =>
    invoke<string[]>("search_session_content", { query, sessions });

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
    onSettingsSaved: (next, effects) => {
      void queryClient.invalidateQueries({ queryKey: ["reset_credit_automation"] });
      if (!effects.restartWatchers) return;
      void (async () => {
        try {
          await invoke("restart_session_watcher", {
            copilotRoot: next.copilotRoot.trim(),
            opencodeRoot: next.opencodeRoot.trim(),
            codexRoot: next.codexRoot.trim(),
            claudeRoot: next.claudeRoot?.trim() ?? "",
            hookScriptsPath: next.hookScriptsPath?.trim() ?? "",
            enabledProviders: next.enabledProviders,
          });
          setRealtimeStatus("active");
        } catch {
          setRealtimeStatus("error");
        }
      })();
    },
  });

  const enqueuePinnedProjectsSave = (next: string[]): Promise<boolean> => {
    const requestId = ++pinnedSaveRequestRef.current;
    const settings = buildSettingsPayload({ pinnedProjects: next });
    pinnedSavePendingCountRef.current += 1;

    const saveTask = async (): Promise<boolean> => {
      try {
        await invoke("save_settings", { settings });
      } catch (error) {
        if (requestId === pinnedSaveRequestRef.current) {
          const rollback = pinnedPersistedProjectsRef.current;
          pinnedProjectsRef.current = rollback;
          setPinnedProjects(rollback);
          showToast(resolveErrorMessage(error, t("toast.pinnedProjectsSaveFailed")));
        }
        pinnedSavePendingCountRef.current -= 1;
        return false;
      }

      pinnedPersistedProjectsRef.current = next;
      pinnedSavePendingCountRef.current -= 1;
      if (requestId === pinnedSaveRequestRef.current) {
        try {
          await queryClient.invalidateQueries({ queryKey: ["settings"] });
        } catch (error) {
          console.warn("[pinned-projects] settings refresh failed:", error);
        }
      }
      return true;
    };

    const queuedSave = pinnedSaveQueueRef.current.then(saveTask, saveTask);
    pinnedSaveQueueRef.current = queuedSave;
    return queuedSave;
  };

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
    mutationFn: ({ sessionId, notes, tags }: { sessionId: string; notes: string | null; tags: string[] }) =>
      invoke("upsert_session_meta", { sessionId, notes, tags }),
    onSuccess: async (_data, variables) => {
      const updateCache = (oldData: SessionInfo[] | undefined) =>
        updateSessionMetadataCache(oldData, variables.sessionId, variables.notes, variables.tags);
      queryClient.setQueriesData<SessionInfo[]>({ queryKey: ["sessions"] }, updateCache);
      queryClient.setQueriesData<SessionInfo[]>({ queryKey: ["sessions_cached"] }, updateCache);
      showToast(t("toast.metaSaved"));
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
    },
    onError: (error) => showToast(resolveErrorMessage(error, t("toast.metaSaveFailed"))),
  });

  const invalidatePathRemapQueries = async () => {
    forceFullRef.current = true;
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: ["project_path_remaps"] }),
      queryClient.invalidateQueries({ queryKey: ["sessions_cached"] }),
      queryClient.invalidateQueries({ queryKey: ["sessions"] }),
    ]);
  };

  const upsertPathRemapMutation = useMutation({
    mutationFn: ({ oldPath, newPath }: { oldPath: string; newPath: string }) =>
      invoke("upsert_project_path_remap", { oldPath, newPath }),
    onSuccess: invalidatePathRemapQueries,
    onError: (error) => showToast(resolveErrorMessage(error, t("toast.pathRemapSaveFailed"))),
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
    const previous = pinnedProjectsRef.current;
    const next = previous.includes(projectKey)
      ? previous.filter((key) => key !== projectKey)
      : [...previous, projectKey];
    pinnedProjectsRef.current = next;
    setPinnedProjects(next);
    return enqueuePinnedProjectsSave(next);
  };

  const pinProject = async (projectKey: string) => {
    const previous = pinnedProjectsRef.current;
    if (previous.includes(projectKey)) return true;
    const next = [...previous, projectKey];
    pinnedProjectsRef.current = next;
    setPinnedProjects(next);
    return enqueuePinnedProjectsSave(next);
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
    await pinProject(key);
  };


  const uncategorizedLabel = t("session.uncategorized");

  const groupedProjects = useMemo(
    () => buildProjectGroups(sessionsQuery.data ?? [], uncategorizedLabel, locale),
    [sessionsQuery.data, uncategorizedLabel, locale],
  );

  const reorderPinnedProjects = (nextVisibleKeys: string[]) => {
    const current = pinnedProjectsRef.current;
    const visibleKeys = current.filter((projectKey) =>
      groupedProjects.some((project) => project.key === projectKey),
    );
    const next = rebuildPinnedProjectOrder(current, visibleKeys, nextVisibleKeys);
    const hasChanged = next.length !== current.length || next.some((key, index) => key !== current[index]);
    if (!hasChanged) return;

    pinnedProjectsRef.current = next;
    setPinnedProjects(next);
    void enqueuePinnedProjectsSave(next);
  };

  const projectPickerCandidates = useMemo(() => {
    const excludedKeys = new Set([...pinnedProjects, ...openProjectKeys]);
    return groupedProjects.filter((project) => !excludedKeys.has(project.key));
  }, [groupedProjects, openProjectKeys, pinnedProjects]);

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
      activeView !== "analytics" &&
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
  const isProjectAnalyticsActive = activeProject !== null
    && getProjectSubTabState(activeProject.key).activeSubTab === "analytics";
  const isAnalyticsWorkspaceActive = activeView === "analytics" || isProjectAnalyticsActive;
  const analyticsScopeKey = activeProject ? `project:${activeProject.pathLabel}` : "global";
  const activeAnalyticsQuery = analyticsQueryByScope[analyticsScopeKey] ?? null;
  const activeAnalyticsPage = analyticsPageByScope[analyticsScopeKey] ?? 1;
  const updateActiveAnalyticsQuery = useCallback((query: AnalyticsQuery) => {
    const scopedQuery = activeProject ? { ...query, cwd: activeProject.pathLabel } : query;
    setAnalyticsQueryByScope((previous) => ({ ...previous, [analyticsScopeKey]: scopedQuery }));
    setAnalyticsPageByScope((previous) => ({ ...previous, [analyticsScopeKey]: 1 }));
  }, [activeProject, analyticsScopeKey]);

  useEffect(() => {
    if (!isAnalyticsWorkspaceActive || activeAnalyticsQuery) return;
    const timeZone = resolveAnalyticsTimeZone(false);
    if (!timeZone) return;
    const cwd = activeProject?.pathLabel ?? null;
    setAnalyticsQueryByScope((previous) => {
      if (previous[analyticsScopeKey]) return previous;
      return {
        ...previous,
        [analyticsScopeKey]: {
          ...createDefaultAnalyticsQuery(
          timeZone,
          settingsForm.enabledProviders,
          ),
          cwd,
        },
      };
    });
  }, [activeAnalyticsQuery, activeProject, analyticsScopeKey, isAnalyticsWorkspaceActive, settingsForm.enabledProviders]);

  const analyticsSourceSettingsKey = {
    copilotRoot: settingsForm.copilotRoot,
    opencodeRoot: settingsForm.opencodeRoot,
    codexRoot: settingsForm.codexRoot,
    claudeRoot: settingsForm.claudeRoot,
    antigravityRoot: settingsForm.antigravityRoot,
    enabledProviders: [...settingsForm.enabledProviders].sort(),
  };
  const analyticsRevisionQuery = useQuery({
    queryKey: ["analyticsRevision"],
    queryFn: () => invoke<number>("get_analytics_revision"),
    enabled: isAnalyticsWorkspaceActive
      || (activeView === "dashboard" && !settingsForm.analyticsPanelCollapsed),
    staleTime: Number.POSITIVE_INFINITY,
  });
  const analyticsRevision = analyticsRevisionQuery.data ?? 0;
  const analyticsReportQuery = useQuery({
    queryKey: [
      "analyticsReport",
      analyticsScopeKey,
      analyticsSourceSettingsKey,
      activeAnalyticsQuery,
      analyticsRevision,
    ],
    queryFn: () => {
      if (!activeAnalyticsQuery) throw new Error(t("analytics.error.timeZoneUnavailable"));
      return invoke<AnalyticsReport>("get_analytics_report", { query: activeAnalyticsQuery });
    },
    placeholderData: (previousData) => previousData,
    retry: false,
    enabled: isAnalyticsWorkspaceActive && activeAnalyticsQuery !== null && analyticsRevisionQuery.isSuccess,
    staleTime: (settingsForm.analyticsRefreshInterval ?? 30) * 60_000,
    refetchInterval: isAnalyticsWorkspaceActive
      ? (settingsForm.analyticsRefreshInterval ?? 30) * 60_000
      : false,
  });
  const analyticsSessionPageQuery = useQuery({
    queryKey: [
      "analyticsSessionPage",
      analyticsScopeKey,
      analyticsSourceSettingsKey,
      activeAnalyticsQuery,
      analyticsReportQuery.data?.revision ?? analyticsRevision,
      activeAnalyticsPage,
      analyticsPageSize,
      "event_time_desc",
    ],
    queryFn: () => {
      if (!activeAnalyticsQuery) throw new Error(t("analytics.error.timeZoneUnavailable"));
      return invoke<AnalyticsSessionPage>("get_analytics_session_page", {
        query: activeAnalyticsQuery,
        requestedRevision: analyticsReportQuery.data?.revision ?? analyticsRevision,
        page: activeAnalyticsPage,
        pageSize: analyticsPageSize,
        sort: "event_time_desc" satisfies AnalyticsSessionSort,
      });
    },
    enabled: isAnalyticsWorkspaceActive
      && activeAnalyticsQuery !== null
      && Boolean(analyticsReportQuery.data)
      && !analyticsReportQuery.isPlaceholderData,
    staleTime: (settingsForm.analyticsRefreshInterval ?? 30) * 60_000,
    retry: false,
  });
  const modelPricingQuery = useQuery({
    queryKey: ["modelPricing"],
    queryFn: () => invoke<ModelPricingEntry[]>("list_model_pricing"),
    enabled: isAnalyticsWorkspaceActive,
  });
  const saveModelPricingMutation = useMutation({
    mutationFn: (input: ManualModelPricingInput) =>
      invoke<ModelPricingEntry[]>("save_manual_model_pricing", { input }),
    onSuccess: (entries) => queryClient.setQueryData(["modelPricing"], entries),
    onError: (error) => showToast(resolveErrorMessage(error, t("pricing.error.save"))),
  });
  const syncModelPricingMutation = useMutation({
    mutationFn: () => invoke<ModelPricingEntry[]>("sync_model_pricing"),
    onSuccess: (entries) => {
      queryClient.setQueryData(["modelPricing"], entries);
      void queryClient.invalidateQueries({ queryKey: ["analyticsRevision"] });
      void queryClient.invalidateQueries({ queryKey: ["analyticsReport"] });
      void queryClient.invalidateQueries({ queryKey: ["analyticsSessionPage"] });
      showToast(t("pricing.syncSuccess"));
    },
    onError: (error) => showToast(resolveErrorMessage(error, t("pricing.error.sync"))),
  });
  const deleteModelPricingMutation = useMutation({
    mutationFn: ({ provider, model }: { provider: string; model: string }) =>
      invoke<ModelPricingEntry[]>("delete_manual_model_pricing", { provider, model }),
    onSuccess: (entries) => queryClient.setQueryData(["modelPricing"], entries),
    onError: (error) => showToast(resolveErrorMessage(error, t("pricing.error.delete"))),
  });
  const modelPricingVisibilityMutation = useMutation({
    mutationFn: ({ provider, model, hidden }: { provider: string; model: string; hidden: boolean }) =>
      invoke<ModelPricingEntry[]>("set_model_pricing_visibility", { provider, model, hidden }),
    onSuccess: (entries) => queryClient.setQueryData(["modelPricing"], entries),
    onError: (error) => showToast(resolveErrorMessage(error, t("pricing.error.visibility"))),
  });

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<number>("analytics-revision-updated", (event) => {
      queryClient.setQueryData(["analyticsRevision"], event.payload);
      void queryClient.invalidateQueries({ queryKey: ["modelPricing"] });
      void queryClient.invalidateQueries({ queryKey: ["analyticsReport"] });
      void queryClient.invalidateQueries({ queryKey: ["analyticsSessionPage"] });
    }).then((stopListening) => {
      unlisten = stopListening;
    }).catch(() => undefined);
    return () => unlisten?.();
  }, [queryClient]);

  useEffect(() => {
    const result = analyticsSessionPageQuery.data;
    if (result?.status !== "stale_revision") return;
    queryClient.setQueryData(["analyticsRevision"], result.revision);
    setAnalyticsPageByScope((previous) => ({ ...previous, [analyticsScopeKey]: 1 }));
    void queryClient.invalidateQueries({ queryKey: ["analyticsReport"] });
  }, [analyticsScopeKey, analyticsSessionPageQuery.data, queryClient]);

  const handleUseUtcFallback = () => {
    updateActiveAnalyticsQuery(createDefaultAnalyticsQuery("UTC", settingsForm.enabledProviders));
  };

  const projectPathExistsQuery = useQuery({
    queryKey: ["project_path_exists", activeProject?.pathLabel ?? ""],
    enabled: Boolean(activeProject?.pathLabel),
    queryFn: () => invoke<boolean>("check_path_remap_directory_exists", { path: activeProject?.pathLabel }),
  });

  const planSpecsRefreshTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  // 樂觀更新期間壓制 openspec 重整，防止 watcher 觸發的 refetch 引發 UI 閃爍。
  // handleWriteOpenspecFile 寫入 tasks.md 後設定此時間戳，期間 watcher 事件只
  // invalidate sisyphus 快取，不碰 openspec 快取。超過時間後下一次 watcher 事件
  // 才會正常重整 openspec，以真實掃描結果覆蓋樂觀值。
  const openspecOptimisticUntilRef = useRef<number>(0);
  const refreshProjectSpecLayers = useCallback(async (projectDir: string) => {
    // 先標記所有層過期，僅立即重掃 active；其餘層由相依查詢依序接手。
    await queryClient.invalidateQueries({ queryKey: ["project_specs", projectDir], refetchType: "none" });
    await queryClient.invalidateQueries({ queryKey: ["project_specs", projectDir, "active"], exact: true });
  }, [queryClient]);
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
            ? [refreshProjectSpecLayers(projectDir)]
            : []),
        ]);
        planSpecsRefreshTimerRef.current = null;
      }, 100);
    },
    [queryClient, refreshProjectSpecLayers],
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

  const analyticsModelOptions = useMemo(
    () => [...new Set((modelPricingQuery.data ?? [])
      .filter((entry) => !entry.hidden)
      .map((entry) => entry.model))].sort(),
    [modelPricingQuery.data],
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

  const openspecActiveQuery = useQuery({
    queryKey: ["project_specs", activeProject?.pathLabel ?? "", "active"],
    enabled: Boolean(activeProject?.pathLabel),
    queryFn: () => invoke<OpenSpecData>("get_project_specs", { projectDir: activeProject?.pathLabel, section: "active" }),
    staleTime: 30_000,
  });

  const openspecSpecsQuery = useQuery({
    queryKey: ["project_specs", activeProject?.pathLabel ?? "", "specs"],
    enabled: Boolean(activeProject?.pathLabel) && openspecActiveQuery.isSuccess && !openspecActiveQuery.isFetching,
    queryFn: () => invoke<OpenSpecData>("get_project_specs", { projectDir: activeProject?.pathLabel, section: "specs" }),
    staleTime: 30_000,
  });

  const openspecArchivedQuery = useQuery({
    queryKey: ["project_specs", activeProject?.pathLabel ?? "", "archived"],
    enabled: Boolean(activeProject?.pathLabel) && openspecActiveQuery.isSuccess && !openspecActiveQuery.isFetching
      && openspecSpecsQuery.isSuccess && !openspecSpecsQuery.isFetching,
    queryFn: () => invoke<OpenSpecData>("get_project_specs", { projectDir: activeProject?.pathLabel, section: "archived" }),
    staleTime: 30_000,
  });

  const openspecData = useMemo<OpenSpecData | undefined>(() => {
    if (!openspecActiveQuery.data && !openspecSpecsQuery.data && !openspecArchivedQuery.data) return undefined;
    return {
      schema: openspecActiveQuery.data?.schema ?? null,
      activeChanges: openspecActiveQuery.data?.activeChanges ?? [],
      specs: openspecSpecsQuery.data?.specs ?? [],
      archivedChanges: openspecArchivedQuery.data?.archivedChanges ?? [],
    };
  }, [openspecActiveQuery.data, openspecSpecsQuery.data, openspecArchivedQuery.data]);
  const openspecLoadingSection = openspecActiveQuery.isPending || openspecActiveQuery.isFetching
    ? "active" as const
    : openspecActiveQuery.isSuccess && (openspecSpecsQuery.isPending || openspecSpecsQuery.isFetching)
      ? "specs" as const
      : openspecSpecsQuery.isSuccess && (openspecArchivedQuery.isPending || openspecArchivedQuery.isFetching)
        ? "archived" as const
        : undefined;
  const openspecLoadingMessage = openspecLoadingSection === "active" ? t("plansSpecs.loadingActive")
    : openspecLoadingSection === "specs" ? t("plansSpecs.loadingSpecs")
      : openspecLoadingSection === "archived" ? t("plansSpecs.loadingArchived") : undefined;
  const openspecError = openspecActiveQuery.error ?? openspecSpecsQuery.error ?? openspecArchivedQuery.error;

  // 專案先完成分層掃描，再載入附帶全域資料；已存在的全域快取仍可直接顯示。
  const projectAgentsSubTabActive =
    Boolean(activeProject?.pathLabel) &&
    (activeProject ? getProjectSubTabState(activeProject.key).activeSubTab === "agents" : false);
  const projectAgentsTab = activeProject ? projectAgentsTabs[activeProject.key] ?? "agents-md" : "agents-md";
  const projectAgentsSkillsActive = projectAgentsTab === "skills";
  const projectAgentsCommandsActive = projectAgentsTab === "commands";
  const projectAgentsMcpActive = projectAgentsTab === "mcp";

  const scanProjectAgentsMd = (projectCwd: string, signal: AbortSignal) => scanAgentsInLayers(
    queryClient, ["agents-md", projectCwd],
    (layer) => invoke<AgentsMdScanResult>("scan_agents_md", { projectCwd, layer }),
    mergeAgentsMdLayers, signal,
  );

  const scanProjectSkills = (projectCwd: string, key: readonly unknown[], signal: AbortSignal) => scanAgentsInLayers(
    queryClient, key,
    (layer) => invoke<SkillsScanResult>("scan_agents_skills", {
      scope: { kind: "project", projectCwd }, enabledProviders: settingsQuery.data?.enabledProviders ?? [], layer,
    }),
    mergeSkillsLayers, signal,
  );

  const projectAgentsPrefsQuery = useQuery({
    queryKey: ["agents-prefs", normalizePath(activeProject?.pathLabel ?? "")],
    enabled: projectAgentsSubTabActive && resourceCanLoad(queryClient, ["agents-prefs", normalizePath(activeProject?.pathLabel ?? "")]),
    ...RESOURCE_QUERY_OPTIONS,
    queryFn: () => invoke<ProjectAgentsPrefs>("load_project_agents_prefs", { projectCwd: activeProject?.pathLabel }),
  });

  const projectAgentsMdQuery = useQuery({
    queryKey: ["agents-md", activeProject?.pathLabel ?? ""],
    enabled: projectAgentsSubTabActive && projectAgentsTab === "agents-md" && resourceCanLoad(queryClient, ["agents-md", activeProject?.pathLabel ?? ""]),
    ...RESOURCE_QUERY_OPTIONS,
    staleTime: 5 * 60_000,
    gcTime: 30 * 60_000,
    queryFn: ({ signal }) => scanProjectAgentsMd(activeProject?.pathLabel ?? "", signal),
  });

  const projectScope = activeProject?.pathLabel ?? "";
  const projectReady = Boolean(settingsQuery.data && projectAgentsPrefsQuery.data && projectAgentsPrefsQuery.isSuccess);
  const projectSkillsKey = resourceQueryKey("skills", projectScope, settingsQuery.data ?? settingsForm, projectAgentsPrefsQuery.data);
  const projectCommandsKey = resourceQueryKey("commands", projectScope, settingsQuery.data ?? settingsForm);
  const projectMcpKey = resourceQueryKey("mcp", projectScope, settingsQuery.data ?? settingsForm);
  const globalSkillsKey = resourceQueryKey("skills", "global", settingsQuery.data ?? settingsForm);
  const globalCommandsKey = resourceQueryKey("commands", "global", settingsQuery.data ?? settingsForm);
  const globalMcpKey = resourceQueryKey("mcp", "global", settingsQuery.data ?? settingsForm);
  const globalSkillsActive = (activeView === "agents-global" && globalAgentsTab === "skills") || (projectAgentsSubTabActive && projectAgentsSkillsActive);
  const globalCommandsActive = (activeView === "agents-global" && globalAgentsTab === "commands") || (projectAgentsSubTabActive && projectAgentsCommandsActive);
  const globalMcpActive = (activeView === "agents-global" && globalAgentsTab === "mcp") || (projectAgentsSubTabActive && projectAgentsMcpActive);

  const projectAgentsSkillsQuery = useQuery({
    queryKey: projectSkillsKey,
    enabled: projectReady && projectAgentsSubTabActive && projectAgentsSkillsActive && resourceCanLoad(queryClient, projectSkillsKey),
    ...RESOURCE_QUERY_OPTIONS,
    queryFn: ({ signal }) => scanProjectSkills(projectScope, projectSkillsKey, signal),
   });

  const projectAgentsCommandsQuery = useQuery({
    queryKey: projectCommandsKey,
    enabled: projectReady && projectAgentsSubTabActive && projectAgentsCommandsActive && resourceCanLoad(queryClient, projectCommandsKey),
    ...RESOURCE_QUERY_OPTIONS,
     queryFn: () => invoke<CommandsScanResult>("scan_agents_commands", { scope: { kind: "project", projectCwd: activeProject?.pathLabel }, enabledProviders: settingsQuery.data?.enabledProviders ?? [] }),
  });

  const globalAgentsMdQuery = useQuery({
    queryKey: ["agents-md", "global"],
    enabled: canLoadGlobalAgents(activeView === "agents-global" && globalAgentsTab === "agents-md", projectAgentsSubTabActive && projectAgentsTab === "agents-md", projectAgentsMdQuery)
      && resourceCanLoad(queryClient, ["agents-md", "global"]),
    ...RESOURCE_QUERY_OPTIONS,
    staleTime: 5 * 60_000,
    gcTime: 30 * 60_000,
    queryFn: () => invoke<AgentsMdScanResult>("scan_global_agents_md"),
  });

  const projectSkillsSettled = {
    isFetched: projectAgentsSkillsQuery.isFetched || projectAgentsPrefsQuery.isError,
    isFetching: projectAgentsSkillsQuery.isFetching,
  };
  const globalAgentsSkillsQuery = useQuery({
    queryKey: globalSkillsKey,
    enabled: Boolean(settingsQuery.data) && canLoadGlobalAgents(activeView === "agents-global" && globalAgentsTab === "skills", projectAgentsSubTabActive && projectAgentsSkillsActive, projectSkillsSettled)
      && resourceCanLoad(queryClient, globalSkillsKey),
    ...RESOURCE_QUERY_OPTIONS,
     queryFn: () => invoke<SkillsScanResult>("scan_agents_skills", { scope: { kind: "global" }, enabledProviders: settingsQuery.data?.enabledProviders ?? [] }),
  });

  const globalAgentsCommandsQuery = useQuery({
    queryKey: globalCommandsKey,
    enabled: Boolean(settingsQuery.data) && globalCommandsActive && resourceCanLoad(queryClient, globalCommandsKey),
    ...RESOURCE_QUERY_OPTIONS,
     queryFn: () => invoke<CommandsScanResult>("scan_agents_commands", { scope: { kind: "global" }, enabledProviders: settingsQuery.data?.enabledProviders ?? [] }),
  });

  const agentsSourceRootConfigured = Boolean((settingsForm.agentsSourceRoot ?? "").trim());

  const agentsRootLinkQuery = useQuery({
    queryKey: ["agents-root-link", settingsQuery.data?.agentsSourceRoot ?? ""],
    enabled: Boolean(settingsQuery.data) && canLoadGlobalAgents(activeView === "agents-global" && globalAgentsTab === "skills", projectAgentsSubTabActive && projectAgentsSkillsActive, projectSkillsSettled)
      && agentsSourceRootConfigured && resourceCanLoad(queryClient, ["agents-root-link", settingsQuery.data?.agentsSourceRoot ?? ""]),
    ...RESOURCE_QUERY_OPTIONS,
    queryFn: () => invoke<AgentsRootLinkStatus>("check_agents_root_link"),
  });

  const linkAgentsRootMutation = useMutation({
    mutationFn: () => invoke<AgentsRootLinkStatus>("link_agents_root"),
    onSuccess: async (status) => {
      queryClient.setQueryData(["agents-root-link", settingsQuery.data?.agentsSourceRoot ?? ""], status);
      await updateWrittenResource("skills", "global", globalSkillsKey);
      if (queryClient.getQueryState(globalCommandsKey)) dirtyAgentsResourcesRef.current.add(resourceKeyId(globalCommandsKey));
    },
    onError: (error) => {
      showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
    },
  });

  const globalMcpConfigsQuery = useQuery({
    queryKey: globalMcpKey,
    enabled: Boolean(settingsQuery.data) && globalMcpActive && resourceCanLoad(queryClient, globalMcpKey),
    ...RESOURCE_QUERY_OPTIONS,
     queryFn: () => invoke<McpProviderConfig[]>("list_mcp_configs", { scope: { kind: "global" } }),
  });

  const projectMcpConfigsQuery = useQuery({
    queryKey: projectMcpKey,
    enabled: projectReady && projectAgentsSubTabActive && projectAgentsMcpActive && resourceCanLoad(queryClient, projectMcpKey),
    ...RESOURCE_QUERY_OPTIONS,
    queryFn: () => invoke<McpProviderConfig[]>("list_mcp_configs", { scope: { kind: "project", projectCwd: activeProject?.pathLabel } }),
  });

  const codexProjectTrustQuery = useQuery({
    queryKey: ["codex-project-trust", normalizePath(projectScope), settingsQuery.data?.codexRoot ?? ""],
    enabled: Boolean(settingsQuery.data) && projectAgentsSubTabActive && projectAgentsMcpActive && resourceCanLoad(queryClient, ["codex-project-trust", normalizePath(projectScope), settingsQuery.data?.codexRoot ?? ""]),
    ...RESOURCE_QUERY_OPTIONS,
    queryFn: () => invoke<boolean>("check_codex_project_trust", { projectCwd: activeProject?.pathLabel }),
  });

  activeAgentsResourcesRef.current = new Set([
    ...(projectAgentsSubTabActive && projectAgentsSkillsActive ? [resourceKeyId(projectSkillsKey), resourceKeyId(globalSkillsKey)] : []),
    ...(projectAgentsSubTabActive && projectAgentsCommandsActive ? [resourceKeyId(projectCommandsKey), resourceKeyId(globalCommandsKey)] : []),
    ...(projectAgentsSubTabActive && projectAgentsMcpActive ? [resourceKeyId(projectMcpKey), resourceKeyId(globalMcpKey)] : []),
    ...(globalSkillsActive ? [resourceKeyId(globalSkillsKey)] : []),
    ...(globalCommandsActive ? [resourceKeyId(globalCommandsKey)] : []),
    ...(globalMcpActive ? [resourceKeyId(globalMcpKey)] : []),
  ]);

  const scanResource = (kind: "skills" | "commands" | "mcp", scope: string, key: readonly unknown[], signal: AbortSignal) => {
    const agentsScope = scope === "global" ? { kind: "global" as const } : { kind: "project" as const, projectCwd: scope };
    if (kind === "mcp") return invoke<McpProviderConfig[]>("list_mcp_configs", { scope: agentsScope });
    if (kind === "skills") return scope === "global"
      ? invoke<SkillsScanResult>("scan_agents_skills", { scope: agentsScope, enabledProviders: settingsQuery.data?.enabledProviders ?? [] })
      : scanProjectSkills(scope, key, signal);
    return invoke<CommandsScanResult>("scan_agents_commands", { scope: agentsScope, enabledProviders: settingsQuery.data?.enabledProviders ?? [] });
  };

  const refreshAgentsResource = async (kind: "skills" | "commands" | "mcp", scope: string, key: readonly unknown[], afterWrite = false) => {
    try {
      await refreshResource<SkillsScanResult | CommandsScanResult | McpProviderConfig[]>(queryClient, key, ({ signal }) => scanResource(kind, scope, key, signal), afterWrite);
    } catch (error) {
      showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
    }
  };

  const updateWrittenResource = async (kind: "skills" | "commands" | "mcp", scope: string, key: readonly unknown[]) => {
    try {
      await updateResourceAfterWrite<SkillsScanResult | CommandsScanResult | McpProviderConfig[]>(queryClient, key, activeAgentsResourcesRef.current,
        dirtyAgentsResourcesRef.current, ({ signal }) => scanResource(kind, scope, key, signal));
    } catch (error) {
      showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
    }
  };

  useEffect(() => {
    const candidates = [
      ["skills", projectScope, projectSkillsKey, projectAgentsSubTabActive && projectAgentsSkillsActive],
      ["commands", projectScope, projectCommandsKey, projectAgentsSubTabActive && projectAgentsCommandsActive],
      ["mcp", projectScope, projectMcpKey, projectAgentsSubTabActive && projectAgentsMcpActive],
      ["skills", "global", globalSkillsKey, globalSkillsActive],
      ["commands", "global", globalCommandsKey, globalCommandsActive],
      ["mcp", "global", globalMcpKey, globalMcpActive],
    ] as const;
    for (const [kind, scope, key, active] of candidates) {
      const id = resourceKeyId(key);
      if (!active || !dirtyAgentsResourcesRef.current.has(id) || queryClient.getQueryState(key)?.fetchStatus === "fetching") continue;
      dirtyAgentsResourcesRef.current.delete(id);
      void refreshAgentsResource(kind, scope, key, true);
    }
  });

  const mcpMutationContext = (scope: { kind: "global" } | { kind: "project"; projectCwd: string }) => {
    const scopeKey = scope.kind === "global" ? "global" : scope.projectCwd;
    return { scopeKey, key: resourceQueryKey("mcp", scopeKey, settingsQuery.data ?? settingsForm) };
  };

  const upsertMcpServerMutation = useMutation({
    mutationFn: (params: {
      scope: { kind: "global" } | { kind: "project"; projectCwd: string };
      provider: string;
      name: string;
      originalName?: string | null;
      configJson: string;
    }) => {
      const context = mcpMutationContext(params.scope);
      return invoke("upsert_mcp_server", params).then(() => context);
    },
    onSuccess: ({ scopeKey, key }) => updateWrittenResource("mcp", scopeKey, key),
    onError: (error) => {
      showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
    },
  });

  const deleteMcpServerMutation = useMutation({
    mutationFn: (params: {
      scope: { kind: "global" } | { kind: "project"; projectCwd: string };
      provider: string;
      name: string;
    }) => {
      const context = mcpMutationContext(params.scope);
      return invoke("delete_mcp_server", params).then(() => context);
    },
    onSuccess: ({ scopeKey, key }) => updateWrittenResource("mcp", scopeKey, key),
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
    }) => {
      const context = mcpMutationContext(params.scope);
      return invoke("set_mcp_server_enabled", params).then(() => context);
    },
    onSuccess: ({ scopeKey, key }) => updateWrittenResource("mcp", scopeKey, key),
    onError: (error) => {
      showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
    },
  });

  // 測試連線不寫入任何設定，也不做 query invalidation；成功/失敗皆回傳分類結果供編輯視窗內顯示，
  // 不透過 toast（唯有連線層級以外的例外，如 IPC 本身失敗，才會 reject）。
  const testMcpHttpServerMutation = useMutation({
    mutationFn: (params: { url: string; headers: Record<string, string> }) =>
      invoke<McpConnectionTestResult>("test_mcp_http_server", params),
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
    staleTime: 10_000,
    gcTime: Infinity,
    refetchOnWindowFocus: true,
    refetchInterval: activeView === "settings" ? 10_000 : false,
  });

  const providerDirectoryQueries = useQueries({
    queries: [
      ["copilot", settingsForm.copilotRoot],
      ["opencode", settingsForm.opencodeRoot],
      ["codex", settingsForm.codexRoot],
      ["claude", settingsForm.claudeRoot],
      ["antigravity", settingsForm.antigravityRoot],
    ].map(([provider, path]) => ({
      queryKey: ["directory_exists", provider, path],
      enabled: Boolean(path),
      queryFn: () => invoke<boolean>("check_directory_exists", { path }),
      staleTime: 10_000,
    })),
  });

  const providerDirectoryExists = useMemo(
    () => Object.fromEntries(
      providerDirectoryQueries.map((query, index) => [
        ["copilot", "opencode", "codex", "claude", "antigravity"][index],
        query.data,
      ]),
    ) as Record<string, boolean | undefined>,
    [providerDirectoryQueries],
  );

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

  // overlay 提醒卡片點擊後聚焦主視窗並導航至對應 session（intervention-registry 常駐提醒）
  useEffect(() => {
    const unlistenPromise = listen<string>("intervention-focus-session", (event) => {
      const session = sessionsQuery.data?.find((s) => s.id === event.payload);
      if (!session) return;
      const projectKey = getProjectKey(session, uncategorizedLabel);
      void invoke("show_main_window", { view: projectKey });
    });
    return () => { unlistenPromise.then((fn) => fn()); };
  }, [sessionsQuery.data, uncategorizedLabel]);

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
    // 對應層快取，避免等待後端 watcher 掃描造成的延遲。後端掃描完成後會以
    // 真實數字覆蓋此處的樂觀值。
    const projectKey = projectCwd;
    const normalizedPath = relativePath.replace(/\\/g, "/");
    const tasksMatch = /^changes\/(?:archive\/)?([^/]+)\/tasks\.md$/.exec(normalizedPath);
    const specsQueryKey = ["project_specs", projectKey, normalizedPath.startsWith("changes/archive/") ? "archived" : "active"];
    let previousData: OpenSpecData | undefined;
    if (projectKey && tasksMatch) {
      const changeName = tasksMatch[1];
      const nextProgress = parseTaskProgress(content);
      await queryClient.cancelQueries({ queryKey: specsQueryKey, exact: true });
      const snapshot = queryClient.getQueryData<OpenSpecData>(specsQueryKey);
      if (snapshot) {
        previousData = snapshot;
        const applyProgress = (changes: OpenSpecData["activeChanges"]) =>
          changes.map((change) =>
            change.name === changeName ? { ...change, taskProgress: nextProgress } : change,
          );
        queryClient.setQueryData<OpenSpecData>(specsQueryKey, {
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
        queryClient.setQueryData<OpenSpecData>(specsQueryKey, previousData);
      }
      showToast(resolveErrorMessage(error, t("toast.openspecWriteFailed")));
      throw error;
    }
  };

  const handleRefreshPlansSpecs = async (): Promise<void> => {
    if (!activeProject?.pathLabel) return;
    await Promise.all([sisyphusQuery.refetch(), refreshProjectSpecLayers(activeProject.pathLabel)]);
    setRealtimeStatus("active");
    setLastRealtimeSyncAt(getRealtimeSyncLabel());
  };

  const handleRefreshResource = async (kind: "skills" | "commands" | "mcp", scope: string) => {
    let prefs = projectAgentsPrefsQuery.data;
    if (scope !== "global" && (kind === "skills" || kind === "commands")) {
      try {
        prefs = await queryClient.fetchQuery({
          queryKey: ["agents-prefs", normalizePath(scope)],
          queryFn: () => invoke<ProjectAgentsPrefs>("load_project_agents_prefs", { projectCwd: scope }),
          staleTime: 0, gcTime: Infinity, retry: false,
        });
      } catch (error) {
        showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
        return;
      }
    }
    const key = resourceQueryKey(kind, scope, settingsQuery.data ?? settingsForm, prefs);
    await refreshAgentsResource(kind, scope, key);
    if (scope === "global" && kind === "skills" && agentsSourceRootConfigured) {
      try {
        await refreshResource(queryClient, ["agents-root-link", settingsQuery.data?.agentsSourceRoot ?? ""], () => invoke<AgentsRootLinkStatus>("check_agents_root_link"));
      } catch (error) { showToast(resolveErrorMessage(error, t("toast.toolOpenFailed"))); }
    }
    if (scope !== "global" && kind === "mcp") {
      try {
        await refreshResource(queryClient, ["codex-project-trust", normalizePath(scope), settingsQuery.data?.codexRoot ?? ""], () => invoke<boolean>("check_codex_project_trust", { projectCwd: scope }));
      } catch (error) { showToast(resolveErrorMessage(error, t("toast.toolOpenFailed"))); }
    }
  };

  const handleWriteProjectAgentsFile = async (filePath: string, content: string): Promise<void> => {
    if (!activeProject?.pathLabel) return;
    await invoke("write_agents_file", {
      scopeRoot: activeProject.pathLabel,
      filePath,
      content,
    });
    await refreshResource(queryClient, ["agents-md", activeProject.pathLabel], ({ signal }) => scanProjectAgentsMd(activeProject.pathLabel, signal), true);
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
    const scope = activeProject.pathLabel;
    try {
      await Promise.all([
        refreshResource(queryClient, ["agents-md", scope], ({ signal }) => scanProjectAgentsMd(scope, signal)),
        refreshResource(queryClient, ["agents-prefs", normalizePath(scope)], () => invoke<ProjectAgentsPrefs>("load_project_agents_prefs", { projectCwd: scope })),
      ]);
    } catch (error) { showToast(resolveErrorMessage(error, t("toast.toolOpenFailed"))); }
  };

  const handleRefreshGlobalAgents = async (): Promise<void> => {
    try {
      await refreshResource(queryClient, ["agents-md", "global"], () => invoke<AgentsMdScanResult>("scan_global_agents_md"));
    } catch (error) { showToast(resolveErrorMessage(error, t("toast.toolOpenFailed"))); }
  };

  const updateSyncedResources = async (request: SyncRequest, report: SyncReport, keys: { skills: readonly unknown[]; commands: readonly unknown[] }) => {
    const scope = request.projectCwd ?? "global";
    for (const kind of writtenSyncKinds(request, report)) {
      await updateWrittenResource(kind, scope, keys[kind]);
    }
  };

  const handlePreviewAgentsSync = async (request: SyncRequest): Promise<SyncReport> => {
    return invoke<SyncReport>("sync_agents_items", { request });
  };

  const handleApplyAgentsSync = async (request: SyncRequest): Promise<SyncReport> => {
    const syncScope = request.projectCwd ?? "global";
    const syncKeys = {
      skills: resourceQueryKey("skills", syncScope, settingsQuery.data ?? settingsForm, projectAgentsPrefsQuery.data),
      commands: resourceQueryKey("commands", syncScope, settingsQuery.data ?? settingsForm),
    };
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
              queryClient.setQueryData(["agents-prefs", normalizePath(request.projectCwd)], nextPrefs);
            }

            const nextReport = await invoke<SyncReport>("sync_agents_items", {
              request: {
                ...request,
                dryRun: false,
                items,
              },
            });
            showToast(formatAgentsReportToast(nextReport, t("agents.report.toast")));
            await updateSyncedResources({ ...request, items }, nextReport, syncKeys);
          })().catch((error) => {
            showToast(resolveErrorMessage(error, t("toast.toolOpenFailed")));
          });
        },
      });
      return report;
    }

    showToast(formatAgentsReportToast(report, t("agents.report.toast")));
    await updateSyncedResources(request, report, syncKeys);
    return report;
  };

  const handleUpdateProjectAgentsPrefs = async (prefs: ProjectAgentsPrefs): Promise<void> => {
    if (!activeProject?.pathLabel) return;
    const scope = activeProject.pathLabel;
    const saveResult = await invoke<SaveProjectAgentsPrefsResult>("save_project_agents_prefs", {
      projectCwd: scope,
      prefs,
    });
    if (saveResult.createdProjectConfigDir) {
      showToast(formatProjectConfigCreatedToast(t("agents.report.projectConfigCreated"), saveResult.storedPath));
    }
    queryClient.setQueryData(["agents-prefs", normalizePath(scope)], prefs);
  };

  const handleResumeSession = async (session: SessionInfo) => {
    if (!session.cwd) { showToast(t("toast.cwdMissing")); return; }
    if (launchInProgressRef.current) return;
    launchInProgressRef.current = true;
    setLaunchingTarget(`session:${session.id}`);
    try {
      const exists = await invoke<boolean>("check_directory_exists", { path: session.cwd });
      if (!exists) { showToast(t("toast.cwdMissing")); return; }
      await invoke("resume_session_in_terminal", {
        provider: session.provider,
        sessionId: session.id,
        cwd: session.cwd,
        terminalPath: settingsQuery.data?.terminalPath || null,
      });
      showToast(t("toast.toolOpened"));
    } catch (error) {
      showToast(error instanceof Error ? error.message : t("toast.toolOpenFailed"));
    } finally {
      launchInProgressRef.current = false;
      setLaunchingTarget(null);
    }
  };

  const handleOpenProjectInTool = async (project: ProjectGroup, toolType: IdeLauncherType) => {
    if (!project.pathLabel) { showToast(t("toast.cwdMissing")); return; }
    if (launchInProgressRef.current) return;
    launchInProgressRef.current = true;
    setLaunchingTarget(`project:${project.key}`);
    try {
      const exists = await invoke<boolean>("check_directory_exists", { path: project.pathLabel });
      if (!exists) { showToast(t("toast.cwdMissing")); return; }
      await invoke("open_in_tool", {
        toolType,
        cwd: project.pathLabel,
        terminalPath: settingsQuery.data?.terminalPath || null,
        sessionId: null,
      });
      showToast(t("toast.toolOpened"));
    } catch (error) {
      showToast(error instanceof Error ? error.message : t("toast.toolOpenFailed"));
    } finally {
      launchInProgressRef.current = false;
      setLaunchingTarget(null);
    }
  };

  const handleFocusTerminal = async (session: SessionInfo) => {
    const hint = session.cwd?.split("\\").pop() ?? session.id;
    try {
      await invoke("focus_terminal_window", { titleHint: `${session.id}\n${hint}` });
    } catch {
      showToast(t("toast.terminalFocusFailed"));
    }
  };

  const [dashboardPeriod, setDashboardPeriod] = useState<"week" | "month">("week");
  const [dashboardViewMode, setDashboardViewMode] = useState<"list" | "kanban">("kanban");
  const [dashboardUtcFallbackConfirmed, setDashboardUtcFallbackConfirmed] = useState(false);

  const dashboardTimeZone = resolveAnalyticsTimeZone(dashboardUtcFallbackConfirmed);
  const dashboardAnalyticsQuery = useMemo(
    () => dashboardTimeZone
      ? createDashboardAnalyticsQuery(
        dashboardTimeZone,
        settingsForm.enabledProviders,
        dashboardPeriod,
      )
      : null,
    [dashboardPeriod, dashboardTimeZone, settingsForm.enabledProviders],
  );
  const dashboardReportQuery = useQuery({
    queryKey: [
      "analyticsReport",
      "dashboard",
      analyticsSourceSettingsKey,
      dashboardAnalyticsQuery,
      analyticsRevision,
    ],
    queryFn: () => {
      if (!dashboardAnalyticsQuery) throw new Error(t("analytics.error.timeZoneUnavailable"));
      return invoke<AnalyticsReport>("get_analytics_report", { query: dashboardAnalyticsQuery });
    },
    placeholderData: (previousData) => previousData,
    retry: false,
    enabled: activeView === "dashboard"
      && !settingsForm.analyticsPanelCollapsed
      && dashboardAnalyticsQuery !== null
      && analyticsRevisionQuery.isSuccess,
    staleTime: (settingsForm.analyticsRefreshInterval ?? 30) * 60_000,
    refetchInterval: activeView === "dashboard" && !settingsForm.analyticsPanelCollapsed
      ? (settingsForm.analyticsRefreshInterval ?? 30) * 60_000
      : false,
  });
  const [dashboardDisplayedPeriod, setDashboardDisplayedPeriod] = useState(dashboardPeriod);
  useEffect(() => {
    if (dashboardReportQuery.data && !dashboardReportQuery.isPlaceholderData) {
      setDashboardDisplayedPeriod(dashboardPeriod);
    }
  }, [dashboardPeriod, dashboardReportQuery.data, dashboardReportQuery.isPlaceholderData]);
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

  const handleOpenFullAnalytics = () => {
    if (dashboardAnalyticsQuery) {
      setAnalyticsQueryByScope((previous) => ({
        ...previous,
        global: dashboardAnalyticsQuery,
      }));
    }
    setActiveView("analytics");
  };

  const handleOpenAnalyticsSession = (provider: string, sessionId: string) => {
    const session = sessionsQuery.data?.find(
      (item) => item.provider === provider && item.id === sessionId,
    );
    if (!session) {
      showToast(t("analytics.sessionDetail.sessionUnavailable"));
      return;
    }
    openProjectTab(getProjectKey(session, uncategorizedLabel));
  };

  const closeProjectPicker = useCallback(() => {
    setShowProjectPicker(false);
    setProjectPickerBusyKey(null);
  }, []);

  const handleOpenProjectFromPicker = (projectKey: string) => {
    openProjectTab(projectKey);
    closeProjectPicker();
  };

  const handlePinProjectFromPicker = (projectKey: string) => {
    if (projectPickerBusyKey) return;
    setProjectPickerBusyKey(projectKey);
    void pinProject(projectKey)
      .then((saved) => {
        if (saved) closeProjectPicker();
      })
      .finally(() => setProjectPickerBusyKey(null));
  };

  const closeProjectTab = (projectKey: string) => {
    setOpenProjectKeys((v) => v.filter((k) => k !== projectKey));
    setActiveView((v) => (v === projectKey ? "dashboard" : v));
  };

  const handleSaveSettings = async () => {
    const next = buildSettingsPayload({}, true);
    if (!validResetCreditSettings(next)) { showToast(t("resetCredit.invalidSettings")); return; }

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

    if (next.terminalPath && (next.terminalPath !== settingsQuery.data?.terminalPath || next.terminalLauncher !== settingsQuery.data?.terminalLauncher)) {
      const isValid = await invoke<boolean>("validate_terminal_path", {
        path: next.terminalPath,
        launcher: next.terminalLauncher ?? "shell",
      });
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

  const handleSavePathRemap = async (oldPath: string, newPath: string) => {
    const exists = await invoke<boolean>("check_path_remap_directory_exists", { path: newPath });
    if (!exists) {
      showToast(t("toast.pathRemapDirectoryMissing"));
      return;
    }
    await upsertPathRemapMutation.mutateAsync({ oldPath, newPath });
    showToast(t("toast.pathRemapSaved"));
  };

  const handleRemapProjectPath = async (oldPath: string) => {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") await handleSavePathRemap(oldPath, selected);
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
    setTagEditDialog({ session, tags: session.tags });
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
  }, [queryClient, showToast, t]);

  const handleConsumeResetCredit = () => {
    if (resetCreditOperationBusy) return;
    const expectedAccountScope = quotaSnapshotQuery.data?.find((snapshot) => snapshot.provider === "codex")?.accountScope;
    if (!expectedAccountScope) { showToast(t("resetCredit.reason.accountChanged")); return; }
    setConfirmDialog(createCodexResetCreditConfirmation(t, () => {
        if (resetCreditOperationBusy) return;
        setResetCreditBusy(true);
        void invoke<string>("consume_codex_reset_credit", { requestId: crypto.randomUUID(), expectedAccountScope })
          .then((outcome) => {
            const messages = {
              reset: "quota.resetCredits.outcome.reset",
              nothing_to_reset: "quota.resetCredits.outcome.nothing_to_reset",
              no_credit: "quota.resetCredits.outcome.no_credit",
              already_redeemed: "quota.resetCredits.outcome.already_redeemed",
            } as const;
            showToast(t(messages[outcome as keyof typeof messages] ?? "quota.resetCredits.failed"));
          })
          .catch((error: unknown) => showToast(typeof error === "string" ? resetCreditMessage(t, error) : t("quota.resetCredits.failed")))
          .finally(() => {
            setResetCreditBusy(false);
            handleRefreshQuota("codex");
          });
      }));
  };

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

  const handleCopyProjectPath = (path: string) => {
    navigator.clipboard.writeText(path)
      .then(() => showToast(t("toast.projectPathCopied")))
      .catch((error) => showToast(resolveErrorMessage(error, t("toast.copyFailed"))));
  };

  // 全域 Agents 資料集合：供全域 Agents 頁（獨立元件）與專案分頁的 globalData prop 共用。
  const globalAgentsData: AgentsScopeDataBundle = {
    scope: { kind: "global" },
    agentsMdData: globalAgentsMdQuery.data,
    agentsMdError: globalAgentsMdQuery.error ? resolveErrorMessage(globalAgentsMdQuery.error, t("toast.toolOpenFailed")) : undefined,
    skillsData: globalAgentsSkillsQuery.data,
    commandsData: globalAgentsCommandsQuery.data,
    skillsError: globalAgentsSkillsQuery.error ? resolveErrorMessage(globalAgentsSkillsQuery.error, t("toast.toolOpenFailed")) : undefined,
    commandsError: globalAgentsCommandsQuery.error ? resolveErrorMessage(globalAgentsCommandsQuery.error, t("toast.toolOpenFailed")) : undefined,
    prefs: globalAgentsPrefs,
    isAgentsMdLoading: globalAgentsMdQuery.isFetching || (globalAgentsMdQuery.isPending && ((activeView === "agents-global" && globalAgentsTab === "agents-md") || (projectAgentsSubTabActive && projectAgentsTab === "agents-md"))),
    isSkillsLoading: globalAgentsSkillsQuery.isFetching || (globalSkillsActive && globalAgentsSkillsQuery.isPending),
    isCommandsLoading: globalAgentsCommandsQuery.isFetching,
    isPrefsLoading: false,
    onRefreshAgentsMd: handleRefreshGlobalAgents,
    onRefreshSkills: () => handleRefreshResource("skills", "global"),
    onRefreshCommands: () => handleRefreshResource("commands", "global"),
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
    mcpFetching: globalMcpConfigsQuery.isFetching,
    mcpError: globalMcpConfigsQuery.error ? resolveErrorMessage(globalMcpConfigsQuery.error, t("toast.toolOpenFailed")) : undefined,
    onRefreshMcp: () => handleRefreshResource("mcp", "global"),
    onActiveTabChange: setGlobalAgentsTab,
    onUpsertMcpServer: (provider, name, originalName, configJson) =>
      upsertMcpServerMutation.mutateAsync({ scope: { kind: "global" }, provider, name, originalName, configJson }),
    onDeleteMcpServer: (provider, name) =>
      deleteMcpServerMutation.mutateAsync({ scope: { kind: "global" }, provider, name }),
    onSetMcpServerEnabled: (provider, name, enabled) =>
      setMcpServerEnabledMutation.mutateAsync({ scope: { kind: "global" }, provider, name, enabled }),
    onTestMcpConnection: (url, headers) => testMcpHttpServerMutation.mutateAsync({ url, headers }),
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
      <GlobalContextMenu />
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
         onReorderPinnedProjects={reorderPinnedProjects}
         onPinProject={(key) => void pinProjectViaDrag(key)}
         onTogglePinProject={(key) => void togglePinProject(key)}
         onOpenProjectFolder={handleOpenPathExternal}
         onCopyProjectPath={handleCopyProjectPath}
         onCollapseToggle={() => setIsSidebarCollapsed((v) => !v)}
         onRefresh={() => sessionsQuery.refetch()}
         onConfigurePath={() => setActiveView("settings")}
         onRequestProjectPicker={() => setShowProjectPicker(true)}
       />

      <section className="workspace">
        <header className="workspace-header">
            <div>
              <div className="workspace-title-row">
                <h2 className="workspace-title">
                  {activeView === "dashboard"
                    ? t("tabs.dashboard")
                    : activeView === "analytics"
                      ? t("analytics.workspace.title")
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
                  : activeView === "analytics"
                    ? t("analytics.workspace.subtitle")
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
              onOpenProject={openProjectTab}
              onOpenRecentSession={(session) =>
                openProjectTab(getProjectKey(session, uncategorizedLabel))
              }
              activityStatusMap={activityStatusMap}
              onResumeSession={(session) => void handleResumeSession(session)}
              launchingTarget={launchingTarget}
              onFocusTerminal={(session) => void handleFocusTerminal(session)}
              viewMode={dashboardViewMode}
              onViewModeChange={setDashboardViewMode}
              analyticsReport={dashboardReportQuery.data}
              analyticsError={dashboardReportQuery.error
                ? resolveErrorMessage(dashboardReportQuery.error, t("analytics.error.loadFailed"))
                : null}
              analyticsLoading={dashboardReportQuery.isLoading}
              analyticsRefreshing={dashboardReportQuery.isFetching || dashboardReportQuery.isPlaceholderData}
              analyticsStale={dashboardReportQuery.isPlaceholderData}
              analyticsTimeZoneUnavailable={dashboardTimeZone === null}
              analyticsDisplayedPeriod={dashboardDisplayedPeriod}
              analyticsCollapsed={settingsForm.analyticsPanelCollapsed ?? false}
              onAnalyticsRetry={() => void dashboardReportQuery.refetch()}
              onAnalyticsToggleCollapsed={() => void handleToggleAnalyticsPanel()}
              onOpenAnalytics={handleOpenFullAnalytics}
              onUseUtcAnalyticsFallback={() => setDashboardUtcFallbackConfirmed(true)}
              quotaSnapshots={quotaSnapshotQuery.data ?? []}
              enableQuotaMonitoring={settingsForm.enableQuotaMonitoring ?? true}
              quotaEnabledProviders={settingsForm.quotaEnabledProviders ?? []}
              onRefreshQuota={handleRefreshQuota}
              onConsumeResetCredit={handleConsumeResetCredit}
              resetCreditBusy={resetCreditOperationBusy}
            />
          ) : null}

          {activeView === "analytics" ? (
            <UsageAnalyticsView
              query={activeAnalyticsQuery}
              report={analyticsReportQuery.data}
              sessionPage={analyticsSessionPageQuery.data}
              pageNumber={activeAnalyticsPage}
              pageSize={analyticsPageSize}
              isStaleResult={analyticsReportQuery.isPlaceholderData}
              availableProviders={settingsForm.enabledProviders}
              projects={groupedProjects}
              models={analyticsModelOptions}
              isLoading={analyticsReportQuery.isLoading || analyticsSessionPageQuery.isLoading}
              isRefreshing={analyticsReportQuery.isFetching || analyticsSessionPageQuery.isFetching || syncModelPricingMutation.isPending}
              errorMessage={analyticsReportQuery.error
                ? resolveErrorMessage(analyticsReportQuery.error, t("analytics.error.loadFailed"))
                : analyticsSessionPageQuery.error
                  ? resolveErrorMessage(analyticsSessionPageQuery.error, t("analytics.error.loadFailed"))
                  : null}
              onQueryChange={updateActiveAnalyticsQuery}
              onRetry={() => void analyticsReportQuery.refetch()}
              onPageChange={(page) => setAnalyticsPageByScope((previous) => ({
                ...previous,
                [analyticsScopeKey]: page,
              }))}
              onPageSizeChange={(pageSize) => {
                setAnalyticsPageSize(pageSize);
                setAnalyticsPageByScope((previous) => ({ ...previous, [analyticsScopeKey]: 1 }));
              }}
              onRefresh={async () => {
                await sessionsQuery.refetch();
                await syncModelPricingMutation.mutateAsync();
                await analyticsReportQuery.refetch();
              }}
              onUseUtcFallback={handleUseUtcFallback}
              onOpenSession={handleOpenAnalyticsSession}
              quotaSnapshots={quotaSnapshotQuery.data ?? []}
              quotaEnabled={settingsForm.enableQuotaMonitoring ?? true}
              quotaError={quotaSnapshotQuery.error
                ? resolveErrorMessage(quotaSnapshotQuery.error, t("analytics.error.loadFailed"))
                : null}
              onRefreshQuota={handleRefreshQuota}
              onConsumeResetCredit={handleConsumeResetCredit}
              resetCreditBusy={resetCreditOperationBusy}
              pricingEntries={modelPricingQuery.data ?? []}
              pricingLoading={modelPricingQuery.isLoading}
              pricingSyncing={syncModelPricingMutation.isPending}
              onSyncPricing={() => syncModelPricingMutation.mutate()}
              pricingSaving={saveModelPricingMutation.isPending}
              pricingError={modelPricingQuery.error
                ? resolveErrorMessage(modelPricingQuery.error, t("pricing.error.load"))
                : null}
              onSavePricing={async (input) => {
                await saveModelPricingMutation.mutateAsync(input);
              }}
              onDeletePricing={(provider, model) => deleteModelPricingMutation.mutate({ provider, model })}
              onPricingVisibilityChange={(provider, model, hidden) =>
                modelPricingVisibilityMutation.mutate({ provider, model, hidden })}
            />
          ) : null}

          {activeView === "settings" ? (
            <SettingsView
              isSaving={settingsMutation.isPending}
              settingsForm={settingsForm}
              onFormChange={setSettingsForm}
              onSave={() => void handleSaveSettings()}
              onBrowseDirectory={handleBrowseDirectory}
              onBrowseFile={handleBrowseFile}
              onDetectTerminal={() => detectTerminalMutation.mutate()}
              onDetectVscode={() => detectVscodeMutation.mutate()}
              onDetectTools={() => void toolAvailabilityQuery.refetch()}
              toolAvailability={toolAvailabilityQuery.data ?? null}
              providerDirectoryExists={providerDirectoryExists}
              onProviderAction={handleProviderAction}
              onOpenProviderPath={(integration) => void handleOpenProviderPath(integration)}
              onEditProviderPath={(integration) => void handleEditProviderPath(integration)}
              pendingProviderAction={pendingProviderAction}
              onOpenEventMonitor={() => setShowEventMonitor(true)}
              jqAvailable={jqAvailableQuery.data ?? null}
              onRefreshQuota={handleRefreshQuota}
              resetCreditStatus={scopedResetCreditStatus}
              onRefreshResetCredits={() => {
                void invoke("request_refresh_reset_credit_automation").then(() => resetCreditStatusQuery.refetch()).catch(() => showToast(t("resetCredit.reason.failed")));
                handleRefreshQuota("codex");
              }}
            />
          ) : null}

          {activeView === "agents-global" ? globalAgentsView : null}

          {activeProject ? (
            <ProjectView
              project={activeProject}
              showArchived={settingsForm.showArchived}
              showEmptySessions={showEmptySessions}
              onShowEmptySessionsChange={setShowEmptySessions}
              onSearchSessionContent={searchSessionContent}
              onContentSearchError={(error) => showToast(resolveErrorMessage(error, t("session.filter.searchError")))}
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
              openspecData={openspecData}
              agentsMdData={projectAgentsMdQuery.data}
              agentsMdError={projectAgentsMdQuery.error ? resolveErrorMessage(projectAgentsMdQuery.error, t("toast.toolOpenFailed")) : undefined}
              skillsData={projectAgentsSkillsQuery.data}
               commandsData={projectAgentsCommandsQuery.data}
               skillsError={(projectAgentsSkillsQuery.error ?? projectAgentsPrefsQuery.error) ? resolveErrorMessage(projectAgentsSkillsQuery.error ?? projectAgentsPrefsQuery.error, t("toast.toolOpenFailed")) : undefined}
               commandsError={projectAgentsCommandsQuery.error ? resolveErrorMessage(projectAgentsCommandsQuery.error, t("toast.toolOpenFailed")) : undefined}
              projectAgentsPrefs={projectAgentsPrefsQuery.data ?? DEFAULT_PROJECT_AGENTS_PREFS}
              plansSpecsLoading={sisyphusQuery.isLoading || openspecActiveQuery.isLoading}
              plansSpecsRefreshing={sisyphusQuery.isFetching || Boolean(openspecLoadingMessage)}
              plansSpecsLoadingMessage={openspecLoadingMessage}
              plansSpecsLoadingSection={openspecLoadingSection}
              plansSpecsError={openspecError ? resolveErrorMessage(openspecError, t("plansSpecs.loadFailed")) : undefined}
              agentsMdLoading={projectAgentsMdQuery.isFetching}
              skillsLoading={projectAgentsSkillsQuery.isFetching || (projectAgentsSubTabActive && projectAgentsSkillsActive && projectAgentsSkillsQuery.isPending && !projectAgentsPrefsQuery.isError)}
              commandsLoading={projectAgentsCommandsQuery.isFetching}
              agentsPrefsLoading={projectAgentsPrefsQuery.isLoading}
              onReadFileContent={handleReadFileContent}
              onReadOpenspecFile={handleReadOpenspecFile}
              onWriteOpenspecFile={handleWriteOpenspecFile}
              onWriteAgentsFile={handleWriteProjectAgentsFile}
              onRefreshPlansSpecs={handleRefreshPlansSpecs}
              onRefreshAgentsMd={handleRefreshProjectAgents}
               onRefreshAgentsSkills={() => handleRefreshResource("skills", activeProject.pathLabel)}
               onRefreshAgentsCommands={() => handleRefreshResource("commands", activeProject.pathLabel)}
              plansSpecsRefreshToken={`${sisyphusQuery.dataUpdatedAt}:${openspecActiveQuery.dataUpdatedAt}:${openspecSpecsQuery.dataUpdatedAt}:${openspecArchivedQuery.dataUpdatedAt}`}
              onOpenPlansSpecsFileExternal={handleOpenPathExternal}
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
               mcpFetching={projectMcpConfigsQuery.isFetching}
               mcpError={projectMcpConfigsQuery.error ? resolveErrorMessage(projectMcpConfigsQuery.error, t("toast.toolOpenFailed")) : undefined}
               onRefreshMcp={() => handleRefreshResource("mcp", activeProject.pathLabel)}
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
              onTestMcpConnection={(url, headers) => testMcpHttpServerMutation.mutateAsync({ url, headers })}
              codexTrusted={codexProjectTrustQuery.data ?? false}
              onAgentsTabChange={(tab) => {
                if (activeProject) {
                  setProjectAgentsTabs((current) => ({ ...current, [activeProject.key]: tab }));
                }
              }}
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
              analyticsWorkspace={{
                query: activeAnalyticsQuery,
                report: analyticsReportQuery.data,
                sessionPage: analyticsSessionPageQuery.data,
                pageNumber: activeAnalyticsPage,
                pageSize: analyticsPageSize,
                isStaleResult: analyticsReportQuery.isPlaceholderData,
                availableProviders: settingsForm.enabledProviders,
                projects: groupedProjects,
                models: analyticsModelOptions,
                isLoading: analyticsReportQuery.isLoading || analyticsSessionPageQuery.isLoading,
                isRefreshing: analyticsReportQuery.isFetching || analyticsSessionPageQuery.isFetching || syncModelPricingMutation.isPending,
                errorMessage: analyticsReportQuery.error
                  ? resolveErrorMessage(analyticsReportQuery.error, t("analytics.error.loadFailed"))
                  : analyticsSessionPageQuery.error
                    ? resolveErrorMessage(analyticsSessionPageQuery.error, t("analytics.error.loadFailed"))
                    : null,
                onQueryChange: updateActiveAnalyticsQuery,
                onRetry: () => void analyticsReportQuery.refetch(),
                onRefresh: async () => {
                  await sessionsQuery.refetch();
                  await syncModelPricingMutation.mutateAsync();
                  await analyticsReportQuery.refetch();
                },
                onUseUtcFallback: handleUseUtcFallback,
                onPageChange: (page) => setAnalyticsPageByScope((previous) => ({
                  ...previous,
                  [analyticsScopeKey]: page,
                })),
                onPageSizeChange: (pageSize) => {
                  setAnalyticsPageSize(pageSize);
                  setAnalyticsPageByScope((previous) => ({ ...previous, [analyticsScopeKey]: 1 }));
                },
                onOpenSession: handleOpenAnalyticsSession,
                quotaSnapshots: quotaSnapshotQuery.data ?? [],
                quotaEnabled: settingsForm.enableQuotaMonitoring ?? true,
                quotaError: quotaSnapshotQuery.error
                  ? resolveErrorMessage(quotaSnapshotQuery.error, t("analytics.error.loadFailed"))
                  : null,
                onRefreshQuota: handleRefreshQuota,
                onConsumeResetCredit: handleConsumeResetCredit,
                resetCreditBusy: resetCreditOperationBusy,
                pricingEntries: modelPricingQuery.data ?? [],
                pricingLoading: modelPricingQuery.isLoading,
                pricingSyncing: syncModelPricingMutation.isPending,
                onSyncPricing: () => syncModelPricingMutation.mutate(),
                pricingSaving: saveModelPricingMutation.isPending,
                pricingError: modelPricingQuery.error
                  ? resolveErrorMessage(modelPricingQuery.error, t("pricing.error.load"))
                  : null,
                onSavePricing: async (input) => {
                  await saveModelPricingMutation.mutateAsync(input);
                },
                onDeletePricing: (provider, model) => deleteModelPricingMutation.mutate({ provider, model }),
                onPricingVisibilityChange: (provider, model, hidden) =>
                  modelPricingVisibilityMutation.mutate({ provider, model, hidden }),
              }}
              activityStatusMap={activityStatusMap}
              onResumeSession={(session) => void handleResumeSession(session)}
              launchingTarget={launchingTarget}
              onFocusTerminal={(session) => void handleFocusTerminal(session)}
              onOpenProjectInTool={(project, tool) => void handleOpenProjectInTool(project, tool)}
              defaultLauncher={settingsQuery.data?.defaultLauncher ?? null}
              toolAvailability={toolAvailabilityQuery.data ?? null}
              projectPathExists={projectPathExistsQuery.data ?? true}
              onRemapProjectPath={(oldPath) => void handleRemapProjectPath(oldPath)}
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
            onConsumeResetCredit={handleConsumeResetCredit}
            resetCreditBusy={resetCreditOperationBusy}
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

      {tagEditDialog ? (
        <TagEditDialog
          title={t("session.actions.editTags")}
          message={t("session.prompt.tagInput")}
          actionLabel={t("session.actions.editTags")}
          initialTags={tagEditDialog.tags}
          onCancel={() => setTagEditDialog(null)}
          onConfirm={(tags) => {
            saveMetaMutation.mutate({
              sessionId: tagEditDialog.session.id,
              notes: tagEditDialog.session.notes ?? null,
              tags: normalizeTags(tags),
            });
            setTagEditDialog(null);
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

      {showProjectPicker ? (
        <ProjectPickerDialog
          projects={projectPickerCandidates}
          busyProjectKey={projectPickerBusyKey}
          onClose={closeProjectPicker}
          onOpenProject={handleOpenProjectFromPicker}
          onPinProject={handlePinProjectFromPicker}
        />
      ) : null}

      {toastMessage ? <div className="toast-banner">{toastMessage}</div> : null}
    </main>
  );
}

export default App;
