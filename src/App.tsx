import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import DOMPurify from "dompurify";
import { marked } from "marked";
import packageJson from "../package.json";

import { useI18n } from "./i18n/I18nProvider";

type SessionInfo = {
  id: string;
  cwd?: string | null;
  summary?: string | null;
  summaryCount?: number | null;
  createdAt?: string | null;
  updatedAt?: string | null;
  sessionDir: string;
  parseError: boolean;
  isArchived: boolean;
  notes?: string | null;
  tags: string[];
  hasPlan: boolean;
};

type AppSettings = {
  copilotRoot: string;
  terminalPath?: string | null;
  externalEditorPath?: string | null;
  showArchived: boolean;
};

type SettingsSection = "general" | "language" | "icon-style";

type ProjectGroup = {
  key: string;
  title: string;
  pathLabel: string;
  sessions: SessionInfo[];
  updatedAtLabel: string;
};

type SortKey = "updatedAt" | "createdAt" | "summaryCount" | "summary";
type RealtimeStatus = "connecting" | "active" | "error";

type ConfirmDialogState = {
  title: string;
  message: string;
  actionLabel: string;
  tone: "danger" | "primary";
  onConfirm: () => void;
};

type EditDialogState = {
  title: string;
  message: string;
  actionLabel: string;
  initialValue: string;
  multiline?: boolean;
  onConfirm: (value: string) => void;
};

function getSessionTitle(session: SessionInfo) {
  return session.summary?.trim() || session.id;
}

function getProjectKey(session: SessionInfo, uncategorizedLabel: string) {
  return session.cwd?.trim() || uncategorizedLabel;
}

function getProjectTitle(pathLabel: string, uncategorizedLabel: string) {
  if (pathLabel === uncategorizedLabel) {
    return pathLabel;
  }

  const segments = pathLabel.split("\\").filter(Boolean);
  return segments[segments.length - 1] ?? pathLabel;
}

function buildProjectGroups(sessions: SessionInfo[], uncategorizedLabel: string): ProjectGroup[] {
  const groupMap = new Map<string, SessionInfo[]>();

  for (const session of sessions) {
    const key = getProjectKey(session, uncategorizedLabel);
    const bucket = groupMap.get(key) ?? [];
    bucket.push(session);
    groupMap.set(key, bucket);
  }

  return Array.from(groupMap.entries())
    .map(([pathLabel, groupedSessions]) => ({
      key: pathLabel,
      title: getProjectTitle(pathLabel, uncategorizedLabel),
      pathLabel,
      sessions: groupedSessions.sort((left, right) =>
        (right.updatedAt ?? "").localeCompare(left.updatedAt ?? ""),
      ),
      updatedAtLabel:
        groupedSessions
          .map((session) => session.updatedAt)
          .find((value): value is string => Boolean(value)) ?? "-",
    }))
    .sort((left, right) => right.sessions.length - left.sessions.length);
}

function filterAndSortSessions(
  sessions: SessionInfo[],
  searchTerm: string,
  sortKey: SortKey,
  selectedTags: string[],
) {
  const normalizedSearchTerm = searchTerm.trim().toLowerCase();

  const filteredSessions = sessions.filter((session) => {
    const matchesTags =
      selectedTags.length === 0 || selectedTags.every((tag) => session.tags.includes(tag));

    if (!matchesTags) {
      return false;
    }

    if (!normalizedSearchTerm) {
      return true;
    }

    const haystacks = [
      session.id,
      session.cwd ?? "",
      session.summary ?? "",
      session.notes ?? "",
      session.tags.join(" "),
    ];

    return haystacks.some((value) => value.toLowerCase().includes(normalizedSearchTerm));
  });

  return filteredSessions.sort((left, right) => {
    switch (sortKey) {
      case "createdAt":
        return (right.createdAt ?? "").localeCompare(left.createdAt ?? "");
      case "summaryCount":
        return (right.summaryCount ?? 0) - (left.summaryCount ?? 0);
      case "summary":
        return getSessionTitle(left).localeCompare(getSessionTitle(right));
      case "updatedAt":
      default:
        return (right.updatedAt ?? "").localeCompare(left.updatedAt ?? "");
    }
  });
}

function App() {
  const { t } = useI18n();
  const queryClient = useQueryClient();
  const [openProjectKeys, setOpenProjectKeys] = useState<string[]>([]);
  const [activeView, setActiveView] = useState<string>("dashboard");
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState<boolean>(false);
  const [settingsSection, setSettingsSection] = useState<SettingsSection>("general");
  const [searchTerm, setSearchTerm] = useState("");
  const [sortKey, setSortKey] = useState<SortKey>("updatedAt");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [activePlanSession, setActivePlanSession] = useState<SessionInfo | null>(null);
  const [confirmDialog, setConfirmDialog] = useState<ConfirmDialogState | null>(null);
  const [editDialog, setEditDialog] = useState<EditDialogState | null>(null);
  const [realtimeStatus, setRealtimeStatus] = useState<RealtimeStatus>("connecting");
  const [lastRealtimeSyncAt, setLastRealtimeSyncAt] = useState<string | null>(null);
  const [planDraft, setPlanDraft] = useState("");
  const [settingsForm, setSettingsForm] = useState<AppSettings>({
    copilotRoot: "",
    terminalPath: "",
    externalEditorPath: "",
    showArchived: false,
  });

  const settingsQuery = useQuery({
    queryKey: ["settings"],
    queryFn: () => invoke<AppSettings>("get_settings"),
  });

  useEffect(() => {
    if (settingsQuery.data) {
      setSettingsForm({
        copilotRoot: settingsQuery.data.copilotRoot,
        terminalPath: settingsQuery.data.terminalPath ?? "",
        externalEditorPath: settingsQuery.data.externalEditorPath ?? "",
        showArchived: settingsQuery.data.showArchived,
      });
    }
  }, [settingsQuery.data]);

  const sessionsQuery = useQuery({
    queryKey: [
      "sessions",
      settingsQuery.data?.copilotRoot ?? "",
      settingsQuery.data?.showArchived ?? false,
    ],
    enabled: Boolean(settingsQuery.data),
    queryFn: () =>
      invoke<SessionInfo[]>("get_sessions", {
        rootDir: settingsQuery.data?.copilotRoot,
        showArchived: settingsQuery.data?.showArchived,
      }),
  });
  const planQuery = useQuery({
    queryKey: ["plan", activePlanSession?.sessionDir ?? ""],
    enabled: Boolean(activePlanSession),
    queryFn: () =>
      invoke<string | null>("read_plan", {
        sessionDir: activePlanSession?.sessionDir,
      }),
  });

  useEffect(() => {
    if (activePlanSession) {
      setPlanDraft(planQuery.data ?? "");
    }
  }, [activePlanSession, planQuery.data]);

  useEffect(() => {
    if (!settingsQuery.data) {
      return undefined;
    }

    void invoke("restart_session_watcher", {
      copilotRoot: settingsQuery.data.copilotRoot,
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

    void invoke("watch_plan_file", {
      sessionDir: activePlanSession.sessionDir,
    });

    return () => {
      void invoke("stop_plan_watch");
    };
  }, [activePlanSession]);

  useEffect(() => {
    let mounted = true;

    const setupListeners = async () => {
      const unlistenSessions = await listen("sessions-updated", async () => {
        await queryClient.invalidateQueries({ queryKey: ["sessions"] });
        if (mounted) {
          setRealtimeStatus("active");
          setLastRealtimeSyncAt(new Date().toLocaleTimeString("zh-TW", { hour12: false }));
          showToast(t("toast.sessionsUpdated"));
        }
      });

      const unlistenPlan = await listen<string>("plan-file-changed", async (event) => {
        if (!activePlanSession || event.payload !== activePlanSession.sessionDir) {
          return;
        }

        await queryClient.invalidateQueries({ queryKey: ["plan", activePlanSession.sessionDir] });
        if (mounted) {
          setRealtimeStatus("active");
          showToast(t("toast.planReloaded"));
        }
      });

      return () => {
        unlistenSessions();
        unlistenPlan();
      };
    };

    let cleanup: (() => void) | undefined;
    void setupListeners().then((dispose) => {
      cleanup = dispose;
    });

    return () => {
      mounted = false;
      cleanup?.();
    };
  }, [activePlanSession, queryClient, t]);

  useEffect(() => {
    if (!toastMessage) {
      return undefined;
    }

    const timer = window.setTimeout(() => setToastMessage(null), 2600);
    return () => window.clearTimeout(timer);
  }, [toastMessage]);

  const showToast = (message: string) => setToastMessage(message);

  const settingsMutation = useMutation({
    mutationFn: (nextSettings: AppSettings) => invoke("save_settings", { settings: nextSettings }),
    onSuccess: async () => {
      showToast(t("toast.settingsSaved"));
      await queryClient.invalidateQueries({ queryKey: ["settings"] });
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
      try {
        await invoke("restart_session_watcher", {
          copilotRoot: settingsForm.copilotRoot.trim(),
        });
        setRealtimeStatus("active");
      } catch {
        setRealtimeStatus("error");
      }
    },
  });

  const archiveMutation = useMutation({
    mutationFn: (sessionId: string) =>
      invoke("archive_session", {
        rootDir: settingsQuery.data?.copilotRoot,
        sessionId,
      }),
    onSuccess: async () => {
      showToast(t("toast.sessionArchived"));
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (sessionId: string) =>
      invoke("delete_session", {
        rootDir: settingsQuery.data?.copilotRoot,
        sessionId,
      }),
    onSuccess: async () => {
      showToast(t("toast.sessionDeleted"));
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
    },
  });

  const saveMetaMutation = useMutation({
    mutationFn: ({
      sessionId,
      notes,
      tags,
    }: {
      sessionId: string;
      notes?: string | null;
      tags: string[];
    }) =>
      invoke("upsert_session_meta", {
        sessionId,
        notes,
        tags,
      }),
    onSuccess: async () => {
      showToast(t("toast.metaSaved"));
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
    },
  });

  const detectTerminalMutation = useMutation({
    mutationFn: () => invoke<string | null>("detect_terminal"),
    onSuccess: (terminalPath) => {
      if (terminalPath) {
        setSettingsForm((currentValue) => ({ ...currentValue, terminalPath }));
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
        setSettingsForm((currentValue) => ({ ...currentValue, externalEditorPath: editorPath }));
        showToast(t("toast.editorDetected"));
      } else {
        showToast(t("toast.editorMissing"));
      }
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

  const uncategorizedLabel = t("session.uncategorized");
  const groupedProjects = useMemo(
    () => buildProjectGroups(sessionsQuery.data ?? [], uncategorizedLabel),
    [sessionsQuery.data, uncategorizedLabel],
  );

  const activeProject = useMemo(
    () => groupedProjects.find((project) => project.key === activeView) ?? null,
    [activeView, groupedProjects],
  );

  useEffect(() => {
    setSelectedTags([]);
  }, [activeProject?.key]);

  const filteredSessions = useMemo(
    () =>
      activeProject
        ? filterAndSortSessions(activeProject.sessions, searchTerm, sortKey, selectedTags)
        : [],
    [activeProject, searchTerm, sortKey, selectedTags],
  );

  const availableTags = useMemo(
    () =>
      [...new Set((activeProject?.sessions ?? []).flatMap((session) => session.tags))]
        .filter(Boolean)
        .sort((left, right) => left.localeCompare(right)),
    [activeProject],
  );

  const recentSessions = useMemo(
    () =>
      [...(sessionsQuery.data ?? [])]
        .sort((left, right) => (right.updatedAt ?? "").localeCompare(left.updatedAt ?? ""))
        .slice(0, 10),
    [sessionsQuery.data],
  );

  const planPreviewHtml = useMemo(
    () =>
      DOMPurify.sanitize(
        marked.parse(planDraft || "_Empty plan_", {
          async: false,
        }),
      ),
    [planDraft],
  );

  const openProjectTab = (projectKey: string) => {
    setOpenProjectKeys((currentValue) =>
      currentValue.includes(projectKey) ? currentValue : [...currentValue, projectKey],
    );
    setActiveView(projectKey);
  };

  const closeProjectTab = (projectKey: string) => {
    setOpenProjectKeys((currentValue) => currentValue.filter((item) => item !== projectKey));
    setActiveView((currentValue) => (currentValue === projectKey ? "dashboard" : currentValue));
  };

  const handleSaveSettings = async () => {
    const nextSettings: AppSettings = {
      copilotRoot: settingsForm.copilotRoot.trim(),
      terminalPath: settingsForm.terminalPath?.trim() || null,
      externalEditorPath: settingsForm.externalEditorPath?.trim() || null,
      showArchived: settingsForm.showArchived,
    };

    if (!nextSettings.copilotRoot) {
      showToast(t("toast.settingsRootRequired"));
      return;
    }

    if (nextSettings.terminalPath) {
      const isValid = await invoke<boolean>("validate_terminal_path", {
        path: nextSettings.terminalPath,
      });

      if (!isValid) {
        showToast(t("toast.terminalInvalid"));
        return;
      }
    }

    settingsMutation.mutate(nextSettings);
  };

  const handleOpenTerminal = async (session: SessionInfo) => {
    if (!session.cwd) {
      showToast(t("toast.cwdMissing"));
      return;
    }

    const exists = await invoke<boolean>("check_directory_exists", { path: session.cwd });

    if (!exists) {
      showToast(t("toast.cwdMissing"));
      return;
    }

    if (!settingsQuery.data?.terminalPath) {
      showToast(t("toast.terminalInvalid"));
      return;
    }

    try {
      await invoke("open_terminal", {
        terminalPath: settingsQuery.data.terminalPath,
        cwd: session.cwd,
      });
      showToast(t("toast.terminalOpened"));
    } catch (error) {
      showToast(error instanceof Error ? error.message : t("toast.terminalOpenFailed"));
    }
  };

  const handleArchiveSession = async (session: SessionInfo) => {
    setConfirmDialog({
      title: t("dialog.archiveTitle"),
      message: `${t("session.confirm.archive")} ${getSessionTitle(session)}?`,
      actionLabel: t("session.actions.archive"),
      tone: "primary",
      onConfirm: () => archiveMutation.mutate(session.id),
    });
  };

  const handleDeleteSession = async (session: SessionInfo) => {
    setConfirmDialog({
      title: t("dialog.deleteTitle"),
      message: `${t("session.confirm.delete")} ${getSessionTitle(session)}?`,
      actionLabel: t("session.actions.delete"),
      tone: "danger",
      onConfirm: () => deleteMutation.mutate(session.id),
    });
  };

  const handleCopyCommand = async (sessionId: string) => {
    await navigator.clipboard.writeText(`gh copilot session resume ${sessionId}`);
    showToast(t("toast.commandCopied"));
  };

  const handleEditNotes = async (session: SessionInfo) => {
    setEditDialog({
      title: t("session.actions.editNotes"),
      message: t("session.prompt.notes"),
      actionLabel: t("session.actions.editNotes"),
      initialValue: session.notes ?? "",
      multiline: true,
      onConfirm: (nextNotes) => {
        saveMetaMutation.mutate({
          sessionId: session.id,
          notes: nextNotes.trim() ? nextNotes.trim() : null,
          tags: session.tags,
        });
      },
    });
  };

  const handleEditTags = async (session: SessionInfo) => {
    setEditDialog({
      title: t("session.actions.editTags"),
      message: t("session.prompt.tags"),
      actionLabel: t("session.actions.editTags"),
      initialValue: session.tags.join(", "),
      onConfirm: (nextTagsValue) => {
        const tags = nextTagsValue
          .split(",")
          .map((value) => value.trim())
          .filter(Boolean);

        saveMetaMutation.mutate({
          sessionId: session.id,
          notes: session.notes ?? null,
          tags,
        });
      },
    });
  };

  const handleOpenPlan = (session: SessionInfo) => {
    setActivePlanSession(session);
  };

  const handleSavePlan = () => {
    if (!activePlanSession) {
      return;
    }

    savePlanMutation.mutate({
      sessionDir: activePlanSession.sessionDir,
      content: planDraft,
    });
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

  const handleBrowseDirectory = async (field: "copilotRoot") => {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      setSettingsForm((currentValue) => ({ ...currentValue, [field]: selected }));
    }
  };

  const handleBrowseFile = async (field: "terminalPath" | "externalEditorPath") => {
    const selected = await open({ directory: false, multiple: false });
    if (typeof selected === "string") {
      setSettingsForm((currentValue) => ({ ...currentValue, [field]: selected }));
    }
  };

  const handleToggleArchived = async (nextValue: boolean) => {
    const nextSettings: AppSettings = {
      copilotRoot: settingsForm.copilotRoot.trim(),
      terminalPath: settingsForm.terminalPath?.trim() || null,
      externalEditorPath: settingsForm.externalEditorPath?.trim() || null,
      showArchived: nextValue,
    };

    setSettingsForm((currentValue) => ({ ...currentValue, showArchived: nextValue }));
    settingsMutation.mutate(nextSettings);
  };

  const realtimeLabel =
    realtimeStatus === "error"
      ? t("dashboard.status.realtimeError")
      : realtimeStatus === "active"
        ? t("dashboard.status.realtimeActive")
        : t("dashboard.status.realtimeConnecting");

  const loadingStatsValue = sessionsQuery.isLoading ? "..." : sessionsQuery.data?.length ?? 0;
  const activeProjectCount = new Set(groupedProjects.map((project) => project.key)).size;
  const archivedCount = (sessionsQuery.data ?? []).filter((session) => session.isArchived).length;
  const parseErrorCount = (sessionsQuery.data ?? []).filter((session) => session.parseError).length;

  return (
    <main className={`app-shell ${isSidebarCollapsed ? "sidebar-collapsed" : ""}`}>
      <aside className="sidebar">
        <div className="sidebar-brand">
          <div className="sidebar-brand-icon">CS</div>
          <div className="sidebar-brand-copy">
            <span className="topbar-badge">{t("app.badge")}</span>
            <h1 className="topbar-title">{t("app.title")}</h1>
          </div>
        </div>

        <button
          type="button"
          className="sidebar-collapse-button"
          onClick={() => setIsSidebarCollapsed((currentValue) => !currentValue)}
          aria-label={isSidebarCollapsed ? t("sidebar.expand") : t("sidebar.collapse")}
          title={isSidebarCollapsed ? t("sidebar.expand") : t("sidebar.collapse")}
        >
          {isSidebarCollapsed ? "»" : "«"}
        </button>

        <nav className="sidebar-menu">
          <button
            type="button"
            className={`sidebar-link ${activeView === "dashboard" ? "active" : ""}`}
            onClick={() => setActiveView("dashboard")}
          >
            <span className="sidebar-link-icon">◫</span>
            <span>{t("sidebar.menu.dashboard")}</span>
          </button>

          <button
            type="button"
            className={`sidebar-link ${activeView === "settings" && settingsSection === "icon-style" ? "active" : ""}`}
            onClick={() => {
              setSettingsSection("icon-style");
              setActiveView("settings");
            }}
          >
            <span className="sidebar-link-icon">◌</span>
            <span>{t("sidebar.menu.iconStyle")}</span>
          </button>

          <button
            type="button"
            className={`sidebar-link ${activeView === "settings" && settingsSection === "language" ? "active" : ""}`}
            onClick={() => {
              setSettingsSection("language");
              setActiveView("settings");
            }}
          >
            <span className="sidebar-link-icon">文</span>
            <span>{t("sidebar.menu.language")}</span>
          </button>

          <button
            type="button"
            className={`sidebar-link ${activeView === "settings" && settingsSection === "general" ? "active" : ""}`}
            onClick={() => {
              setSettingsSection("general");
              setActiveView("settings");
            }}
          >
            <span className="sidebar-link-icon">⚙</span>
            <span>{t("sidebar.menu.settings")}</span>
          </button>
        </nav>

        <section className="sidebar-panel">
          <span className="sidebar-panel-label">{t("sidebar.language.label")}</span>
          <strong>{t("sidebar.language.current")}</strong>
          <p>{t("sidebar.language.description")}</p>
        </section>

        <section className="sidebar-panel">
          <span className="sidebar-panel-label">{t("sidebar.iconStyle.label")}</span>
          <strong>{t("sidebar.iconStyle.current")}</strong>
          <p>{t("sidebar.iconStyle.description")}</p>
        </section>

        <footer className="sidebar-footer">
          <button
            type="button"
            className="ghost-button sidebar-footer-button"
            onClick={() => {
              setSettingsSection("general");
              setActiveView("settings");
            }}
          >
            {t("app.actions.configureCopilotPath")}
          </button>
          <div className="sidebar-version">
            <span>{t("sidebar.version")}</span>
            <strong>v{packageJson.version}</strong>
          </div>
        </footer>
      </aside>

      <section className="workspace">
        <header className="workspace-header">
          <div>
            <h2 className="workspace-title">
              {activeView === "dashboard"
                ? t("tabs.dashboard")
                : activeView === "settings"
                  ? t("settings.title")
                  : activeProject?.title}
            </h2>
            <p className="workspace-subtitle">
              {activeView === "settings"
                ? t("settings.subtitle")
                : activeView === "dashboard"
                  ? t("dashboard.description")
                  : activeProject?.pathLabel}
            </p>
          </div>

          <div className="topbar-actions">
            <div className={`realtime-indicator realtime-${realtimeStatus}`}>
              <span className="realtime-dot" />
              <span>
                {realtimeLabel}
                {lastRealtimeSyncAt ? ` · ${lastRealtimeSyncAt}` : ""}
              </span>
            </div>
            <button type="button" onClick={() => sessionsQuery.refetch()}>
              {t("app.actions.refresh")}
            </button>
          </div>
        </header>

        {activeView !== "settings" ? (
          <section className="tabbar">
            <button
              type="button"
              className={`tab-item ${activeView === "dashboard" ? "active" : ""}`}
              onClick={() => setActiveView("dashboard")}
            >
              {t("tabs.dashboard")}
            </button>

            {openProjectKeys.map((projectKey) => {
              const project = groupedProjects.find((item) => item.key === projectKey);

              if (!project) {
                return null;
              }

              return (
                <div
                  key={project.key}
                  className={`tab-item tab-item-project ${activeView === project.key ? "active" : ""}`}
                >
                  <button
                    type="button"
                    className="tab-label"
                    onClick={() => setActiveView(project.key)}
                  >
                    {project.title}
                  </button>
                  <button
                    type="button"
                    className="tab-close"
                    onClick={() => closeProjectTab(project.key)}
                    aria-label={`${t("tabs.close")} ${project.title}`}
                  >
                    ×
                  </button>
                </div>
              );
            })}
          </section>
        ) : null}

        {activeView === "dashboard" ? (
          <section className="dashboard-layout">
            <article className="hero-card">
              <span className="hero-badge">{t("dashboard.badge")}</span>
              <h2>{t("dashboard.title")}</h2>
              <p className="hero-copy">{t("dashboard.description")}</p>
            </article>

            {sessionsQuery.isError ? (
              <article className="info-card status-card status-card-error">
                <h3>{t("dashboard.status.errorTitle")}</h3>
                <p className="placeholder-copy">
                  {sessionsQuery.error instanceof Error
                    ? sessionsQuery.error.message
                    : t("dashboard.status.errorDescription")}
                </p>
              </article>
            ) : null}

            <section className="stats-grid">
              <article className="stat-card">
                <span className="stat-label">{t("dashboard.stats.totalSessions")}</span>
                <strong>{loadingStatsValue}</strong>
              </article>
              <article className="stat-card">
                <span className="stat-label">{t("dashboard.stats.activeProjects")}</span>
                <strong>{sessionsQuery.isLoading ? "..." : activeProjectCount}</strong>
              </article>
              <article className="stat-card">
                <span className="stat-label">{t("dashboard.stats.archivedSessions")}</span>
                <strong>{sessionsQuery.isLoading ? "..." : archivedCount}</strong>
              </article>
              <article className="stat-card">
                <span className="stat-label">{t("dashboard.stats.parseErrors")}</span>
                <strong>{sessionsQuery.isLoading ? "..." : parseErrorCount}</strong>
              </article>
              <article className="stat-card">
                <span className="stat-label">{t("dashboard.stats.loadingState")}</span>
                <strong>{realtimeLabel}</strong>
              </article>
            </section>

            <section className="content-grid">
              <article className="info-card">
                <div className="section-heading">
                  <h3>{t("dashboard.projects.title")}</h3>
                  <span>{t("dashboard.projects.subtitle")}</span>
                </div>

                <div className="project-list">
                  {groupedProjects.map((project) => (
                    <button
                      key={project.key}
                      type="button"
                      className="project-item"
                      onClick={() => openProjectTab(project.key)}
                    >
                      <div>
                        <strong>{project.title}</strong>
                        <p>{project.pathLabel}</p>
                      </div>

                      <div className="project-meta">
                        <span>
                          {project.sessions.length} {t("dashboard.projects.sessionCountSuffix")}
                        </span>
                        <span>{project.updatedAtLabel}</span>
                      </div>
                    </button>
                  ))}
                </div>
              </article>

              <article className="info-card">
                <div className="section-heading">
                  <h3>{t("dashboard.recent.title")}</h3>
                  <span>{t("dashboard.recent.subtitle")}</span>
                </div>

                <ul className="feature-list feature-list-tight">
                  {recentSessions.map((session) => (
                    <li key={session.id}>
                      <button
                        type="button"
                        className="inline-link"
                        onClick={() => {
                          const projectKey = getProjectKey(session, uncategorizedLabel);
                          openProjectTab(projectKey);
                        }}
                      >
                        {getSessionTitle(session)}
                      </button>
                    </li>
                  ))}
                </ul>
              </article>
            </section>
          </section>
        ) : null}

        {activeView === "settings" ? (
          <section className="settings-layout">
            <article className="info-card">
              <div className="section-heading">
                <h3>{t("settings.general.title")}</h3>
                <span>{t("settings.general.subtitle")}</span>
              </div>

              <div className="settings-form">
                <label className="field-group">
                  <span>{t("settings.fields.copilotRoot")}</span>
                  <div className="field-with-action">
                    <input
                      value={settingsForm.copilotRoot}
                      onChange={(event) =>
                        setSettingsForm((currentValue) => ({
                          ...currentValue,
                          copilotRoot: event.currentTarget.value,
                        }))
                      }
                    />
                    <button type="button" className="ghost-button" onClick={() => void handleBrowseDirectory("copilotRoot")}>
                      {t("settings.actions.browseDirectory")}
                    </button>
                  </div>
                </label>

                <label className="field-group">
                  <span>{t("settings.fields.terminalPath")}</span>
                  <div className="field-with-action">
                    <input
                      value={settingsForm.terminalPath ?? ""}
                      onChange={(event) =>
                        setSettingsForm((currentValue) => ({
                          ...currentValue,
                          terminalPath: event.currentTarget.value,
                        }))
                      }
                    />
                    <button type="button" className="ghost-button" onClick={() => void handleBrowseFile("terminalPath")}>
                      {t("settings.actions.browseFile")}
                    </button>
                  </div>
                </label>

                <label className="field-group">
                  <span>{t("settings.fields.externalEditorPath")}</span>
                  <div className="field-with-action">
                    <input
                      value={settingsForm.externalEditorPath ?? ""}
                      onChange={(event) =>
                        setSettingsForm((currentValue) => ({
                          ...currentValue,
                          externalEditorPath: event.currentTarget.value,
                        }))
                      }
                    />
                    <button
                      type="button"
                      className="ghost-button"
                      onClick={() => void handleBrowseFile("externalEditorPath")}
                    >
                      {t("settings.actions.browseFile")}
                    </button>
                  </div>
                </label>

                <label className="checkbox-group">
                  <input
                    type="checkbox"
                    checked={settingsForm.showArchived}
                    onChange={(event) =>
                      setSettingsForm((currentValue) => ({
                        ...currentValue,
                        showArchived: event.currentTarget.checked,
                      }))
                    }
                  />
                  <span>{t("settings.fields.showArchived")}</span>
                </label>

                <div className="settings-actions">
                  <button type="button" onClick={handleSaveSettings}>
                    {t("settings.actions.save")}
                  </button>
                  <button
                    type="button"
                    className="ghost-button"
                    onClick={() => detectTerminalMutation.mutate()}
                  >
                    {t("settings.actions.detectTerminal")}
                  </button>
                  <button
                    type="button"
                    className="ghost-button"
                    onClick={() => detectVscodeMutation.mutate()}
                  >
                    {t("settings.actions.detectEditor")}
                  </button>
                </div>
              </div>
            </article>

            <article className="info-card">
              <div className="section-heading">
                <h3>{t("settings.language.title")}</h3>
                <span>{t("settings.language.subtitle")}</span>
              </div>
              <p className="placeholder-copy">{t("sidebar.language.description")}</p>
            </article>

            <article className="info-card">
              <div className="section-heading">
                <h3>{t("settings.iconStyle.title")}</h3>
                <span>{t("settings.iconStyle.subtitle")}</span>
              </div>
              <p className="placeholder-copy">{t("sidebar.iconStyle.description")}</p>
            </article>
          </section>
        ) : null}

        {activeProject ? (
          <section className="project-page">
            <article className="hero-card">
              <span className="hero-badge">{t("project.badge")}</span>
              <h2>{activeProject.title}</h2>
              <p className="hero-copy">{activeProject.pathLabel}</p>
            </article>

            <section className="toolbar-card">
              <label className="field-group compact-field">
                <span>{t("session.search")}</span>
                <input
                  value={searchTerm}
                  onChange={(event) => setSearchTerm(event.currentTarget.value)}
                  placeholder={t("session.searchPlaceholder")}
                />
              </label>

              <label className="field-group compact-field">
                <span>{t("session.sort")}</span>
                <select
                  value={sortKey}
                  onChange={(event) => setSortKey(event.currentTarget.value as SortKey)}
                >
                  <option value="updatedAt">{t("session.sortUpdatedAt")}</option>
                  <option value="createdAt">{t("session.sortCreatedAt")}</option>
                  <option value="summaryCount">{t("session.sortSummaryCount")}</option>
                  <option value="summary">{t("session.sortSummary")}</option>
                </select>
              </label>

              <label className="checkbox-group compact-checkbox">
                <input
                  type="checkbox"
                  checked={settingsForm.showArchived}
                  onChange={(event) => void handleToggleArchived(event.currentTarget.checked)}
                />
                <span>{t("project.showArchivedToggle")}</span>
              </label>
            </section>

            {availableTags.length > 0 ? (
              <section className="tag-filter-bar">
                <span className="session-meta-label">{t("session.tagFilter")}</span>
                <div className="session-chip-row">
                  {availableTags.map((tag) => {
                    const isActive = selectedTags.includes(tag);
                    return (
                      <button
                        key={tag}
                        type="button"
                        className={`tag-filter-chip ${isActive ? "active" : ""}`}
                        onClick={() =>
                          setSelectedTags((currentValue) =>
                            currentValue.includes(tag)
                              ? currentValue.filter((item) => item !== tag)
                              : [...currentValue, tag],
                          )
                        }
                      >
                        #{tag}
                      </button>
                    );
                  })}
                </div>
              </section>
            ) : null}

            <div className="session-list">
              {filteredSessions.map((session) => (
                <article key={session.id} className="session-card">
                  <div className="session-card-header">
                    <div>
                      <h3>{getSessionTitle(session)}</h3>
                      <p>{session.id}</p>
                    </div>

                    <div className="session-chip-row">
                      {session.isArchived ? (
                        <span className="session-chip muted-chip">{t("session.archived")}</span>
                      ) : null}
                      {session.hasPlan ? (
                        <span className="session-chip">{t("session.hasPlan")}</span>
                      ) : null}
                      {session.parseError ? (
                        <span className="session-chip error-chip">{t("session.parseError")}</span>
                      ) : null}
                    </div>
                  </div>

                  <div className="session-meta-grid">
                    <div>
                      <span className="session-meta-label">{t("session.cwd")}</span>
                      <p>{session.cwd ?? t("session.uncategorized")}</p>
                    </div>
                    <div>
                      <span className="session-meta-label">{t("session.updatedAt")}</span>
                      <p>{session.updatedAt ?? "-"}</p>
                    </div>
                    <div>
                      <span className="session-meta-label">{t("session.createdAt")}</span>
                      <p>{session.createdAt ?? "-"}</p>
                    </div>
                    <div>
                      <span className="session-meta-label">{t("session.summaryCount")}</span>
                      <p>{session.summaryCount ?? 0}</p>
                    </div>
                  </div>

                  {session.notes ? (
                    <p className="session-notes">
                      <strong>{t("session.notes")}</strong> {session.notes}
                    </p>
                  ) : null}

                  {session.tags.length > 0 ? (
                    <div className="session-chip-row">
                      {session.tags.map((tag) => (
                        <span key={tag} className="session-chip">
                          #{tag}
                        </span>
                      ))}
                    </div>
                  ) : null}

                  <div className="session-actions">
                    <button type="button" onClick={() => handleOpenTerminal(session)}>
                      {t("session.actions.openTerminal")}
                    </button>
                    <button type="button" className="ghost-button" onClick={() => handleCopyCommand(session.id)}>
                      {t("session.actions.copyCommand")}
                    </button>
                    <button type="button" className="ghost-button" onClick={() => handleEditNotes(session)}>
                      {t("session.actions.editNotes")}
                    </button>
                    <button type="button" className="ghost-button" onClick={() => handleEditTags(session)}>
                      {t("session.actions.editTags")}
                    </button>
                    <button type="button" className="ghost-button" onClick={() => handleOpenPlan(session)}>
                      {t("session.actions.editPlan")}
                    </button>
                    <button
                      type="button"
                      className="ghost-button"
                      onClick={() => handleOpenPlanExternal(session)}
                    >
                      {t("session.actions.openPlanExternal")}
                    </button>
                    {!session.isArchived ? (
                      <button type="button" className="ghost-button" onClick={() => handleArchiveSession(session)}>
                        {t("session.actions.archive")}
                      </button>
                    ) : null}
                    <button type="button" className="danger-button" onClick={() => handleDeleteSession(session)}>
                      {t("session.actions.delete")}
                    </button>
                  </div>
                </article>
              ))}
            </div>

            {activePlanSession ? (
              <article className="info-card plan-editor-card">
                <div className="section-heading">
                  <h3>{t("plan.title")}</h3>
                  <span>{getSessionTitle(activePlanSession)}</span>
                </div>

                <div className="plan-editor-layout">
                  <label className="field-group">
                    <span>{t("plan.editor")}</span>
                    <textarea
                      className="plan-textarea"
                      value={planDraft}
                      onChange={(event) => setPlanDraft(event.currentTarget.value)}
                    />
                  </label>

                  <div className="plan-preview">
                    <span className="session-meta-label">{t("plan.preview")}</span>
                    <div
                      className="plan-preview-markdown"
                      dangerouslySetInnerHTML={{ __html: planPreviewHtml }}
                    />
                  </div>
                </div>

                <div className="settings-actions">
                  <button type="button" onClick={handleSavePlan}>
                    {t("plan.actions.save")}
                  </button>
                  <button
                    type="button"
                    className="ghost-button"
                    onClick={() => handleOpenPlanExternal(activePlanSession)}
                  >
                    {t("plan.actions.openExternal")}
                  </button>
                  <button
                    type="button"
                    className="ghost-button"
                    onClick={() => setActivePlanSession(null)}
                  >
                    {t("plan.actions.close")}
                  </button>
                </div>
              </article>
            ) : null}
          </section>
        ) : null}
      </section>

      {confirmDialog ? (
        <div className="dialog-backdrop">
          <article className="dialog-card">
            <h3>{confirmDialog.title}</h3>
            <p>{confirmDialog.message}</p>
            <div className="dialog-actions">
              <button type="button" className="ghost-button" onClick={() => setConfirmDialog(null)}>
                {t("dialog.cancel")}
              </button>
              <button
                type="button"
                className={confirmDialog.tone === "danger" ? "danger-button" : "dialog-confirm-button"}
                onClick={() => {
                  confirmDialog.onConfirm();
                  setConfirmDialog(null);
                }}
              >
                {confirmDialog.actionLabel}
              </button>
            </div>
          </article>
        </div>
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

      {toastMessage ? <div className="toast-banner">{toastMessage}</div> : null}
    </main>
  );
}

type EditDialogProps = {
  dialog: EditDialogState;
  onCancel: () => void;
  onConfirm: (value: string) => void;
};

function EditDialog({ dialog, onCancel, onConfirm }: EditDialogProps) {
  const { t } = useI18n();
  const [value, setValue] = useState(dialog.initialValue);

  useEffect(() => {
    setValue(dialog.initialValue);
  }, [dialog.initialValue]);

  return (
    <div className="dialog-backdrop">
      <article className="dialog-card">
        <h3>{dialog.title}</h3>
        <p>{dialog.message}</p>
        <div className="dialog-form">
          {dialog.multiline ? (
            <textarea
              className="dialog-input dialog-input-multiline"
              value={value}
              onChange={(event) => setValue(event.currentTarget.value)}
            />
          ) : (
            <input
              className="dialog-input"
              value={value}
              onChange={(event) => setValue(event.currentTarget.value)}
            />
          )}
        </div>
        <div className="dialog-actions">
          <button type="button" className="ghost-button" onClick={onCancel}>
            {t("dialog.cancel")}
          </button>
          <button type="button" className="dialog-confirm-button" onClick={() => onConfirm(value)}>
            {dialog.actionLabel}
          </button>
        </div>
      </article>
    </div>
  );
}

export default App;
