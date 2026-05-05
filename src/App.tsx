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
  ActivityHintPayload,
  AnalyticsDataPoint,
  AnalyticsGroupBy,
  AppSettings,
  BridgeEventLogEntry,
  ConfirmDialogState,
  EditDialogState,
  IdeLauncherType,
  OpenSpecData,
  ProjectGroup,
  ProviderIntegrationStatus,
  SessionActivityStatus,
  SessionInfo,
  SessionStats,
  SessionTargetedPayload,
  SisyphusData,
  ToolAvailability,
} from "./types";
import { formatDateTime } from "./utils/formatDate";

import { ConfirmDialog } from "./components/ConfirmDialog";
import { BridgeEventMonitorDialog } from "./components/BridgeEventMonitorDialog";
import { DashboardView } from "./components/DashboardView";
import { EditDialog } from "./components/EditDialog";
import { ProjectView } from "./components/ProjectView";
import { SettingsView } from "./components/SettingsView";
import { Sidebar } from "./components/Sidebar";
import { StatusBar } from "./components/StatusBar";

// ─── helpers ─────────────────────────────────────────────────────────────────

function normalizePath(path: string): string {
  // Windows 路徑大小寫不敏感，正規化為小寫用於分組比對
  return path.toLowerCase();
}

function getProjectKey(session: SessionInfo, uncategorizedLabel: string): string {
  const raw = session.cwd?.trim();
  if (!raw) return uncategorizedLabel;
  return normalizePath(raw);
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

function isSessionInUpdatedRange(session: SessionInfo, periodStartTime: number): boolean {
  if (!session.updatedAt) return false;
  const updatedAtTime = Date.parse(session.updatedAt);
  return !Number.isNaN(updatedAtTime) && updatedAtTime >= periodStartTime;
}

function buildProjectGroups(sessions: SessionInfo[], uncategorizedLabel: string, locale: string): ProjectGroup[] {
  const groupMap = new Map<string, { displayPath: string; sessions: SessionInfo[] }>();

  for (const session of sessions) {
    const key = getProjectKey(session, uncategorizedLabel);
    const displayPath = session.cwd?.trim() || uncategorizedLabel;
    if (!groupMap.has(key)) {
      groupMap.set(key, { displayPath, sessions: [] });
    }
    groupMap.get(key)!.sessions.push(session);
  }

  const getTitle = (pathLabel: string) => {
    if (pathLabel === uncategorizedLabel) return pathLabel;
    const parts = pathLabel.split("\\").filter(Boolean);
    return parts[parts.length - 1] ?? pathLabel;
  };

  return Array.from(groupMap.entries())
    .map(([key, { displayPath, sessions: groupedSessions }]) => ({
      key,
      title: getTitle(displayPath),
      pathLabel: displayPath,
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

type ProviderIntegrationAction = "install" | "update" | "recheck";

function getProviderLabel(
  provider: string,
  copilotLabel: string,
  opencodeLabel: string,
): string {
  switch (provider) {
    case "copilot":
      return copilotLabel;
    case "opencode":
      return opencodeLabel;
    default:
      return provider;
  }
}

function resolveProviderTargetPath(integration: ProviderIntegrationStatus): string | null {
  const configPath = integration.configPath?.trim();
  if (configPath) return configPath;
  const bridgePath = integration.bridgePath?.trim();
  return bridgePath || null;
}

function upsertProviderIntegrationStatus(
  integrations: ProviderIntegrationStatus[] | undefined,
  nextStatus: ProviderIntegrationStatus,
): ProviderIntegrationStatus[] {
  const nextIntegrations = [...(integrations ?? [])];
  const existingIndex = nextIntegrations.findIndex(
    (integration) => integration.provider === nextStatus.provider,
  );

  if (existingIndex === -1) {
    nextIntegrations.push(nextStatus);
  } else {
    nextIntegrations[existingIndex] = nextStatus;
  }

  const providerOrder = ["copilot", "opencode"];
  nextIntegrations.sort((left, right) => {
    const leftIndex = providerOrder.indexOf(left.provider);
    const rightIndex = providerOrder.indexOf(right.provider);
    const normalizedLeft = leftIndex === -1 ? Number.MAX_SAFE_INTEGER : leftIndex;
    const normalizedRight = rightIndex === -1 ? Number.MAX_SAFE_INTEGER : rightIndex;
    return normalizedLeft - normalizedRight || left.provider.localeCompare(right.provider);
  });

  return nextIntegrations;
}

function resolveErrorMessage(error: unknown, fallback: string): string {
  if (typeof error === "string" && error.trim()) return error;
  if (error instanceof Error && error.message.trim()) return error.message;
  if (
    typeof error === "object" &&
    error &&
    "message" in error &&
    typeof error.message === "string" &&
    error.message.trim()
  ) {
    return error.message;
  }
  return fallback;
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
    Map<string, { openPlanKeys: string[]; activeSubTab: string }>
  >(new Map());

  const getProjectSubTabState = (projectKey: string) =>
    projectSubTabStates.get(projectKey) ?? { openPlanKeys: [], activeSubTab: "sessions" };

  const handleSubTabStateChange = (
    projectKey: string,
    state: { openPlanKeys: string[]; activeSubTab: string },
  ) => {
    setProjectSubTabStates((prev) => new Map(prev).set(projectKey, state));
  };

  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [confirmDialog, setConfirmDialog] = useState<ConfirmDialogState | null>(null);
  const [editDialog, setEditDialog] = useState<EditDialogState | null>(null);

  const [realtimeStatus, setRealtimeStatus] = useState<"connecting" | "active" | "error">(
    "connecting",
  );
  const [lastRealtimeSyncAt, setLastRealtimeSyncAt] = useState<string | null>(null);
  const [forceFull, setForceFull] = useState(false);
  const [pendingProviderAction, setPendingProviderAction] = useState<string | null>(null);

  const [bridgeEventLog, setBridgeEventLog] = useState<BridgeEventLogEntry[]>([]);
  const [lastBridgeEvent, setLastBridgeEvent] = useState<{ entry: BridgeEventLogEntry; receivedAt: Date } | null>(null);
  const [showEventMonitor, setShowEventMonitor] = useState(false);
  const lastBridgeEventTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const hasShownOutdatedToast = useRef(false);
  const prevActivityStatusRef = useRef<Map<string, string>>(new Map());
  const lastInterventionSessionRef = useRef<string | null>(null);
  const lastNotificationSessionRef = useRef<{ id: string; type: string } | null>(null);
  // sessionsDataRef 讓事件 listener 不因 stale closure 而讀到舊的 sessionsQuery.data
  const sessionsDataRef = useRef<SessionInfo[]>([]);
  const [settingsForm, setSettingsForm] = useState<AppSettings>({
    copilotRoot: "",
    opencodeRoot: "",
    terminalPath: "",
    externalEditorPath: "",
    showArchived: false,
    enabledProviders: ["copilot", "opencode"],
    providerIntegrations: [],
    defaultLauncher: "terminal",
    enableInterventionNotification: true,
    enableSessionEndNotification: false,
    showStatusBar: true,
    analyticsRefreshInterval: 30,
    analyticsPanelCollapsed: false,
  });

  const settingsQuery = useQuery({
    queryKey: ["settings"],
    queryFn: () => invoke<AppSettings>("get_settings"),
  });

  const sessionsQuery = useQuery({
    queryKey: [
      "sessions",
      settingsQuery.data?.copilotRoot ?? "",
      settingsQuery.data?.opencodeRoot ?? "",
      settingsQuery.data?.showArchived ?? false,
      settingsQuery.data?.enabledProviders ?? [],
      forceFull,
    ],
    enabled: Boolean(settingsQuery.data),
    queryFn: () =>
      invoke<SessionInfo[]>("get_sessions", {
        rootDir: settingsQuery.data?.copilotRoot,
        opencodeRoot: settingsQuery.data?.opencodeRoot,
        showArchived: settingsQuery.data?.showArchived,
        enabledProviders: settingsQuery.data?.enabledProviders,
        forceFull,
      }).then((result) => {
        // 全掃完成後重置 forceFull flag
        if (forceFull) setForceFull(false);
        return result;
      }),
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
      setSettingsForm({
        copilotRoot: settingsQuery.data.copilotRoot,
        opencodeRoot: settingsQuery.data.opencodeRoot ?? "",
        terminalPath: settingsQuery.data.terminalPath ?? "",
        externalEditorPath: settingsQuery.data.externalEditorPath ?? "",
        showArchived: settingsQuery.data.showArchived,
        pinnedProjects: settingsQuery.data.pinnedProjects ?? [],
        enabledProviders: settingsQuery.data.enabledProviders ?? ["copilot", "opencode"],
        providerIntegrations: settingsQuery.data.providerIntegrations ?? [],
        defaultLauncher: settingsQuery.data.defaultLauncher ?? "terminal",
        enableInterventionNotification: settingsQuery.data.enableInterventionNotification ?? true,
        enableSessionEndNotification: settingsQuery.data.enableSessionEndNotification ?? false,
        showStatusBar: settingsQuery.data.showStatusBar ?? true,
        analyticsRefreshInterval: settingsQuery.data.analyticsRefreshInterval ?? 30,
        analyticsPanelCollapsed: settingsQuery.data.analyticsPanelCollapsed ?? false,
      });
      setPinnedProjects((settingsQuery.data.pinnedProjects ?? []).map(normalizePath));
    }
  }, [settingsQuery.data]);

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

      setBridgeEventLog((prev) => {
        const next = [...prev, entry];
        return next.length > MAX_LOG ? next.slice(next.length - MAX_LOG) : next;
      });

      setLastBridgeEvent({ entry, receivedAt: new Date() });

      if (lastBridgeEventTimerRef.current) clearTimeout(lastBridgeEventTimerRef.current);
      lastBridgeEventTimerRef.current = setTimeout(() => {
        setLastBridgeEvent(null);
      }, LAST_EVENT_TTL_MS);
    });

    return () => {
      void unlisten.then((fn) => fn());
      if (lastBridgeEventTimerRef.current) clearTimeout(lastBridgeEventTimerRef.current);
    };
  }, []);

  useEffect(() => {
    if (!toastMessage) return undefined;
    const timer = window.setTimeout(() => setToastMessage(null), 2600);
    return () => window.clearTimeout(timer);
  }, [toastMessage]);

  const showToast = (message: string) => setToastMessage(message);

  const settingsMutation = useMutation({
    mutationFn: (next: AppSettings) => invoke("save_settings", { settings: next }),
    onSuccess: async () => {
      showToast(t("toast.settingsSaved"));
      await queryClient.invalidateQueries({ queryKey: ["settings"] });
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
      try {
        await invoke("restart_session_watcher", {
          copilotRoot: settingsForm.copilotRoot.trim(),
          opencodeRoot: settingsForm.opencodeRoot.trim(),
          enabledProviders: settingsForm.enabledProviders,
        });
        setRealtimeStatus("active");
      } catch {
        setRealtimeStatus("error");
      }
    },
  });

  const archiveMutation = useMutation({
    mutationFn: (sessionId: string) =>
      invoke("archive_session", { rootDir: settingsQuery.data?.copilotRoot, sessionId }),
    onSuccess: async () => {
      showToast(t("toast.sessionArchived"));
      setForceFull(true);
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
    },
  });

  const unarchiveMutation = useMutation({
    mutationFn: (sessionId: string) =>
      invoke("unarchive_session", { rootDir: settingsQuery.data?.copilotRoot, sessionId }),
    onSuccess: async () => {
      showToast(t("toast.sessionUnarchived"));
      setForceFull(true);
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (sessionId: string) =>
      invoke("delete_session", { rootDir: settingsQuery.data?.copilotRoot, sessionId }),
    onSuccess: async () => {
      showToast(t("toast.sessionDeleted"));
      setForceFull(true);
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

  const detectTerminalMutation = useMutation({
    mutationFn: () => invoke<string | null>("detect_terminal"),
    onSuccess: (terminalPath) => {
      if (terminalPath) {
        setSettingsForm((v) => ({ ...v, terminalPath }));
        showToast(t("toast.terminalDetected"));
      } else {
        showToast(t("toast.terminalMissing"));
      }
    },
  });

  const detectVscodeMutation = useMutation({
    mutationFn: () => invoke<string | null>("detect_vscode"),
    onSuccess: (editorPath) => {
      if (editorPath) {
        setSettingsForm((v) => ({ ...v, externalEditorPath: editorPath }));
        showToast(t("toast.editorDetected"));
      } else {
        showToast(t("toast.editorMissing"));
      }
    },
  });

  const providerIntegrationMutation = useMutation({
    mutationFn: ({ provider, action }: { provider: string; action: ProviderIntegrationAction }) => {
      const command =
        action === "install"
          ? "install_provider_integration"
          : action === "update"
            ? "update_provider_integration"
            : "recheck_provider_integration";

      return invoke<ProviderIntegrationStatus>(command, {
        provider,
        copilotRoot: settingsForm.copilotRoot.trim() || null,
      });
    },
    onSuccess: (status, variables) => {
      const providerLabel = getProviderLabel(
        status.provider,
        t("settings.fields.providerCopilot"),
        t("settings.fields.providerOpencode"),
      );
      setSettingsForm((current) => ({
        ...current,
        providerIntegrations: upsertProviderIntegrationStatus(
          current.providerIntegrations,
          status,
        ),
      }));

      if (
        (variables.action === "install" || variables.action === "update") &&
        status.status !== "installed"
      ) {
        showToast(
          status.lastError ||
            t("toast.providerActionIncomplete").replace("{provider}", providerLabel),
        );
        return;
      }

      const toastMessage =
        variables.action === "install"
          ? t("toast.providerInstalled")
          : variables.action === "update"
            ? t("toast.providerUpdated")
            : t("toast.providerRechecked");
      showToast(toastMessage.replace("{provider}", providerLabel));
    },
    onError: (error, variables) => {
      const providerLabel = getProviderLabel(
        variables.provider,
        t("settings.fields.providerCopilot"),
        t("settings.fields.providerOpencode"),
      );
      showToast(
        resolveErrorMessage(
          error,
          t("toast.providerActionFailed").replace("{provider}", providerLabel),
        ),
      );
    },
    onSettled: () => setPendingProviderAction(null),
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
      setForceFull(true);
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
    },
  });

  const buildSettingsPayload = (overrides: Partial<AppSettings> = {}): AppSettings => ({
    copilotRoot: (overrides.copilotRoot ?? settingsForm.copilotRoot).trim(),
    opencodeRoot: (overrides.opencodeRoot ?? settingsForm.opencodeRoot).trim(),
    terminalPath: (overrides.terminalPath ?? settingsForm.terminalPath)?.trim() || null,
    externalEditorPath:
      (overrides.externalEditorPath ?? settingsForm.externalEditorPath)?.trim() || null,
    showArchived: overrides.showArchived ?? settingsForm.showArchived,
    pinnedProjects: overrides.pinnedProjects ?? pinnedProjects,
    enabledProviders: overrides.enabledProviders ?? settingsForm.enabledProviders,
    providerIntegrations: overrides.providerIntegrations ?? settingsForm.providerIntegrations ?? [],
    defaultLauncher: overrides.defaultLauncher ?? settingsForm.defaultLauncher ?? null,
    enableInterventionNotification:
      overrides.enableInterventionNotification ?? settingsForm.enableInterventionNotification ?? true,
    enableSessionEndNotification:
      overrides.enableSessionEndNotification ?? settingsForm.enableSessionEndNotification ?? false,
    showStatusBar: overrides.showStatusBar ?? settingsForm.showStatusBar ?? true,
    analyticsRefreshInterval:
      overrides.analyticsRefreshInterval ?? settingsForm.analyticsRefreshInterval ?? 30,
    analyticsPanelCollapsed:
      overrides.analyticsPanelCollapsed ?? settingsForm.analyticsPanelCollapsed ?? false,
  });

  const persistSettingsSilently = async (next: AppSettings) => {
    await invoke("save_settings", { settings: next });
    await queryClient.invalidateQueries({ queryKey: ["settings"] });
  };

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

  const refreshProjectPlansSpecs = useCallback(
    async (projectDir: string) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["project_plans", projectDir] }),
        queryClient.invalidateQueries({ queryKey: ["project_specs", projectDir] }),
      ]);
    },
    [queryClient],
  );

  useEffect(() => {
    if (!activeProject?.pathLabel) {
      void invoke("stop_project_watch");
      return undefined;
    }
    void invoke("watch_project_files", { projectDir: activeProject.pathLabel });
    return () => { void invoke("stop_project_watch"); };
  }, [activeProject?.pathLabel]);

  useEffect(() => {
    let mounted = true;

    const onSessionsRefresh = async () => {
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
      if (mounted) {
        setRealtimeStatus("active");
        setLastRealtimeSyncAt(getRealtimeSyncLabel());
      }
    };

    const setup = async () => {
      const unlistenCopilot = await listen("copilot-sessions-updated", onSessionsRefresh);
      const unlistenOpencode = await listen("opencode-sessions-updated", onSessionsRefresh);

      const unlistenCopilotTargeted = await listen<SessionTargetedPayload>(
        "copilot-session-targeted",
        async (event) => {
          const { cwd } = event.payload;
          const updated = await invoke<SessionInfo | null>("get_session_by_cwd", {
            cwd,
            rootDir: settingsQuery.data?.copilotRoot,
          }).catch(() => null);

          if (!mounted) return;

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
              }
            );
          } else {
            await queryClient.invalidateQueries({ queryKey: ["sessions"] });
          }

          setRealtimeStatus("active");
          setLastRealtimeSyncAt(getRealtimeSyncLabel());
        }
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

          queryClient.setQueriesData<SessionActivityStatus[]>(
            { queryKey: ["activity_statuses"], exact: false },
            (old) => {
              if (!old) return old;
              const idx = old.findIndex((s) => s.sessionId === session.id);
              const updated: SessionActivityStatus = {
                ...(old[idx] ?? { sessionId: session.id, provider: session.provider }),
                status: "active",
                detail,
              };
              if (idx === -1) return [...old, updated];
              const next = [...old];
              next[idx] = updated;
              return next;
            },
          );

          setRealtimeStatus("active");
          setLastRealtimeSyncAt(getRealtimeSyncLabel());
        }
      );

      const unlistenPlan = await listen<string>("plan-file-changed", async (event) => {
        if (!activePlanSession || event.payload !== activePlanSession.sessionDir) return;
        await queryClient.invalidateQueries({ queryKey: ["plan", activePlanSession.sessionDir] });
        if (mounted) {
          setRealtimeStatus("active");
          showToast(t("toast.planReloaded"));
        }
      });

      const unlistenProjectFiles = await listen<string>("project-files-changed", async (event) => {
        if (!activeProject || event.payload !== activeProject.pathLabel) return;
        await refreshProjectPlansSpecs(activeProject.pathLabel);
        if (mounted) {
          setRealtimeStatus("active");
          setLastRealtimeSyncAt(getRealtimeSyncLabel());
        }
      });

      return () => {
        unlistenCopilot();
        unlistenOpencode();
        unlistenCopilotTargeted();
        unlistenActivityHint();
        unlistenPlan();
        unlistenProjectFiles();
      };
    };

    let cleanup: (() => void) | undefined;
    void setup().then((dispose) => { cleanup = dispose; });
    return () => { mounted = false; cleanup?.(); };
  }, [activePlanSession, activeProject, queryClient, refreshProjectPlansSpecs, settingsQuery.data, t]);

  const deletableEmptySessionCount = useMemo(
    () =>
      (sessionsQuery.data ?? []).filter(
        (session) => session.provider === "copilot" && !session.hasEvents,
      ).length,
    [sessionsQuery.data],
  );

  const sessionStatsQueries = useQueries({
    queries: (sessionsQuery.data ?? []).map((session) => ({
      queryKey: ["session_stats", session.sessionDir],
      queryFn: () => invoke<SessionStats>("get_session_stats", { sessionDir: session.sessionDir }),
      staleTime: 60_000,
      enabled: Boolean(session.sessionDir),
    })),
  });

  const sessionStatsMap = useMemo(
    () => Object.fromEntries(
      (sessionsQuery.data ?? []).map((session, index) => [session.id, sessionStatsQueries[index]?.data]),
    ) as Record<string, SessionStats | undefined>,
    [sessionStatsQueries, sessionsQuery.data],
  );

  const sessionStatsLoadingMap = useMemo(
    () => Object.fromEntries(
      (sessionsQuery.data ?? []).map((session, index) => [session.id, sessionStatsQueries[index]?.isLoading]),
    ) as Record<string, boolean | undefined>,
    [sessionStatsQueries, sessionsQuery.data],
  );

  const { activeSessions, waitingSessions } = useMemo(() => {
    const sessions = sessionsQuery.data ?? [];
    const now = Date.now();
    const THIRTY_MIN = 30 * 60 * 1000;
    let active = 0;
    let waiting = 0;
    for (const s of sessions) {
      if (s.isArchived) continue;
      const stats = sessionStatsMap[s.id];
      if (stats?.isLive) {
        active++;
        continue;
      }
      // 只有 stats 已載入且明確非 live，才用時間判斷「最近活動」
      if (stats !== undefined) {
        const updatedAt = s.updatedAt ? new Date(s.updatedAt).getTime() : 0;
        if (updatedAt > 0 && now - updatedAt <= THIRTY_MIN) waiting++;
      }
    }
    return { activeSessions: active, waitingSessions: waiting };
  }, [sessionsQuery.data, sessionStatsMap]);

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

  const activityStatusQuery = useQuery({
    queryKey: ["activity_statuses", sessionsQuery.data?.map((s) => s.id)],
    enabled: Boolean(sessionsQuery.data?.length),
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
   refetchInterval: 30_000,
    staleTime: 25_000,
  });

  const toolAvailabilityQuery = useQuery({
    queryKey: ["tool_availability"],
    queryFn: () => invoke<ToolAvailability>("check_tool_availability"),
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

  const handleReadOpenspecFile = async (projectCwd: string, relativePath: string): Promise<string> => {
    return invoke<string>("read_openspec_file", { projectCwd, relativePath });
  };

  const handleRefreshPlansSpecs = async (): Promise<void> => {
    if (!activeProject?.pathLabel) return;
    await Promise.all([sisyphusQuery.refetch(), openspecQuery.refetch()]);
    setRealtimeStatus("active");
    setLastRealtimeSyncAt(getRealtimeSyncLabel());
  };

  const handleOpenInTool = async (session: SessionInfo, toolType: IdeLauncherType) => {
    if (!session.cwd) { showToast(t("toast.cwdMissing")); return; }
    const exists = await invoke<boolean>("check_directory_exists", { path: session.cwd });
    if (!exists) { showToast(t("toast.cwdMissing")); return; }
    try {
      await invoke("open_in_tool", {
        toolType,
        cwd: session.cwd,
        terminalPath: settingsQuery.data?.terminalPath || null,
        sessionId: session.id,
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

    if (requiresCopilotRoot && !next.copilotRoot) {
      showToast(t("toast.settingsRootRequired"));
      return;
    }

    if (requiresOpencodeRoot && !next.opencodeRoot) {
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

  const handleBrowseDirectory = async (field: "copilotRoot" | "opencodeRoot") => {
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

  const handleOpenTerminal = async (session: SessionInfo) => {
    if (!session.cwd) { showToast(t("toast.cwdMissing")); return; }
    const exists = await invoke<boolean>("check_directory_exists", { path: session.cwd });
    if (!exists) { showToast(t("toast.cwdMissing")); return; }
    if (!settingsQuery.data?.terminalPath) { showToast(t("toast.terminalInvalid")); return; }
    try {
      await invoke("open_terminal", {
        terminalPath: settingsQuery.data.terminalPath,
        cwd: session.cwd,
        sessionId: session.id,
      });
      showToast(t("toast.terminalOpened"));
    } catch (error) {
      showToast(error instanceof Error ? error.message : t("toast.terminalOpenFailed"));
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
    const command =
      session.provider === "opencode"
        ? `opencode session ${session.id}`
        : `copilot --resume=${session.id}`;
    await navigator.clipboard.writeText(command);
    showToast(t("toast.commandCopied"));
  };

  const handleEditNotes = (session: SessionInfo) => {
    setEditDialog({
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
      title: t("session.actions.editTags"),
      message: t("session.prompt.tags"),
      actionLabel: t("session.actions.editTags"),
      initialValue: session.tags.join(", "),
      onConfirm: (nextValue) => {
        const tags = nextValue.split(",").map((v) => v.trim()).filter(Boolean);
        saveMetaMutation.mutate({ sessionId: session.id, notes: session.notes ?? null, tags });
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
    providerIntegrationMutation.mutate({ provider, action });
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
              <h2 className="workspace-title">
                {activeView === "dashboard"
                  ? t("tabs.dashboard")
                  : activeView === "settings"
                    ? t("settings.title")
                    : activeProject?.title ?? ""}
              </h2>
              <p className="workspace-subtitle">
                {activeView === "dashboard"
                  ? t("dashboard.subtitle")
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
              onOpenInTool={(session, tool) => void handleOpenInTool(session, tool)}
              onFocusTerminal={(session) => void handleFocusTerminal(session)}
              defaultLauncher={settingsQuery.data?.defaultLauncher ?? null}
              toolAvailability={toolAvailabilityQuery.data ?? null}
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
            />
          ) : null}

          {activeView === "settings" ? (
            <SettingsView
              settingsForm={settingsForm}
              onFormChange={setSettingsForm}
              onSave={() => void handleSaveSettings()}
              onBrowseDirectory={(field) => void handleBrowseDirectory(field)}
              onBrowseFile={(field) => void handleBrowseFile(field)}
              onDetectTerminal={() => detectTerminalMutation.mutate()}
              onDetectVscode={() => detectVscodeMutation.mutate()}
              onProviderAction={handleProviderAction}
              onOpenProviderPath={(integration) => void handleOpenProviderPath(integration)}
              onEditProviderPath={(integration) => void handleEditProviderPath(integration)}
              pendingProviderAction={pendingProviderAction}
              onOpenEventMonitor={() => setShowEventMonitor(true)}
            />
          ) : null}

          {activeProject ? (
            <ProjectView
              project={activeProject}
              showArchived={settingsForm.showArchived}
              hideEmptySessions={hideEmptySessions}
              onHideEmptySessionsChange={setHideEmptySessions}
              totalEmptySessions={deletableEmptySessionCount}
              onToggleArchived={(v) => void handleToggleArchived(v)}
              onOpenTerminal={(s) => void handleOpenTerminal(s)}
              onCopyCommand={(s) => void handleCopyCommand(s)}
              onEditNotes={handleEditNotes}
              onEditTags={handleEditTags}
              onOpenPlan={handleOpenPlan}
              onArchive={handleArchiveSession}
              onUnarchive={handleUnarchiveSession}
              onDelete={handleDeleteSession}
              onDeleteEmptySessions={() => handleDeleteEmptySessions(deletableEmptySessionCount)}
              isPinned={pinnedProjects.includes(activeProject.key)}
              onTogglePin={() => void togglePinProject(activeProject.key)}
              sessionStats={sessionStatsMap}
              sessionStatsLoading={sessionStatsLoadingMap}
              sessionsLoading={sessionsQuery.isLoading}
              sisyphusData={sisyphusQuery.data}
              openspecData={openspecQuery.data}
              plansSpecsLoading={sisyphusQuery.isLoading || openspecQuery.isLoading}
              plansSpecsRefreshing={sisyphusQuery.isFetching || openspecQuery.isFetching}
              onReadFileContent={handleReadFileContent}
              onReadOpenspecFile={handleReadOpenspecFile}
              onRefreshPlansSpecs={handleRefreshPlansSpecs}
              plansSpecsRefreshToken={`${sisyphusQuery.dataUpdatedAt}:${openspecQuery.dataUpdatedAt}`}
              activePlanSessionId={activePlanSessionId}
              onActivePlanChange={setActivePlanSessionId}
              planDraft={planDraft}
              planPreviewHtml={planPreviewHtml}
              onPlanDraftChange={setPlanDraft}
              onSavePlan={handleSavePlan}
              onOpenPlanExternal={(s) => void handleOpenPlanExternal(s)}
              openPlanKeys={getProjectSubTabState(activeProject.key).openPlanKeys}
              activeSubTab={getProjectSubTabState(activeProject.key).activeSubTab}
              onSubTabStateChange={(state) => handleSubTabStateChange(activeProject.key, state)}
              onFetchAnalytics={(cwd, startDate, endDate, groupBy) =>
                fetchAnalyticsData(cwd, startDate, endDate, groupBy)
              }
              activityStatusMap={activityStatusMap}
              onOpenInTool={(session, tool) => void handleOpenInTool(session, tool)}
              onFocusTerminal={(session) => void handleFocusTerminal(session)}
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
            isLoadingSessions={sessionsQuery.isLoading}
          />
        ) : null}
      </section>

      {confirmDialog ? (
        <ConfirmDialog dialog={confirmDialog} onCancel={() => setConfirmDialog(null)} />
      ) : null}

      {editDialog ? (
        <EditDialog
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

      {toastMessage ? <div className="toast-banner">{toastMessage}</div> : null}
    </main>
  );
}

export default App;
