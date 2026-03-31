import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import DOMPurify from "dompurify";
import { marked } from "marked";

import { useI18n } from "./i18n/I18nProvider";
import type { AppSettings, ConfirmDialogState, EditDialogState, ProjectGroup, SessionInfo } from "./types";
import { formatDateTime } from "./utils/formatDate";

import { ConfirmDialog } from "./components/ConfirmDialog";
import { DashboardView } from "./components/DashboardView";
import { EditDialog } from "./components/EditDialog";
import { PlanEditor } from "./components/PlanEditor";
import { ProjectView } from "./components/ProjectView";
import { SettingsView } from "./components/SettingsView";
import { Sidebar } from "./components/Sidebar";

// ─── helpers ─────────────────────────────────────────────────────────────────

function getProjectKey(session: SessionInfo, uncategorizedLabel: string) {
  return session.cwd?.trim() || uncategorizedLabel;
}

function buildProjectGroups(sessions: SessionInfo[], uncategorizedLabel: string, locale: string): ProjectGroup[] {
  const groupMap = new Map<string, SessionInfo[]>();

  for (const session of sessions) {
    const key = getProjectKey(session, uncategorizedLabel);
    const bucket = groupMap.get(key) ?? [];
    bucket.push(session);
    groupMap.set(key, bucket);
  }

  const getTitle = (pathLabel: string) => {
    if (pathLabel === uncategorizedLabel) return pathLabel;
    const parts = pathLabel.split("\\").filter(Boolean);
    return parts[parts.length - 1] ?? pathLabel;
  };

  return Array.from(groupMap.entries())
    .map(([pathLabel, groupedSessions]) => ({
      key: pathLabel,
      title: getTitle(pathLabel),
      pathLabel,
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

// ─── App ─────────────────────────────────────────────────────────────────────

function App() {
  const { t, locale } = useI18n();
  const queryClient = useQueryClient();

  const [openProjectKeys, setOpenProjectKeys] = useState<string[]>([]);
  const [activeView, setActiveView] = useState<string>("dashboard");
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);

  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [confirmDialog, setConfirmDialog] = useState<ConfirmDialogState | null>(null);
  const [editDialog, setEditDialog] = useState<EditDialogState | null>(null);

  const [realtimeStatus, setRealtimeStatus] = useState<"connecting" | "active" | "error">(
    "connecting",
  );
  const [lastRealtimeSyncAt, setLastRealtimeSyncAt] = useState<string | null>(null);

  const [openPlanKeys, setOpenPlanKeys] = useState<string[]>([]);
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

  const activePlanSession = useMemo(() => {
    if (!activeView.startsWith("plan:")) return null;
    const sessionId = activeView.replace("plan:", "");
    return sessionsQuery.data?.find((s) => s.id === sessionId) ?? null;
  }, [activeView, sessionsQuery.data]);

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
        terminalPath: settingsQuery.data.terminalPath ?? "",
        externalEditorPath: settingsQuery.data.externalEditorPath ?? "",
        showArchived: settingsQuery.data.showArchived,
      });
    }
  }, [settingsQuery.data]);

  useEffect(() => {
    if (activePlanSession) {
      setPlanDraft(planQuery.data ?? "");
    }
  }, [activePlanSession, planQuery.data]);

  useEffect(() => {
    if (!settingsQuery.data) return undefined;
    void invoke("restart_session_watcher", { copilotRoot: settingsQuery.data.copilotRoot })
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
    let mounted = true;

    const setup = async () => {
      const unlistenSessions = await listen("sessions-updated", async () => {
        await queryClient.invalidateQueries({ queryKey: ["sessions"] });
        if (mounted) {
          setRealtimeStatus("active");
          setLastRealtimeSyncAt(new Date().toLocaleTimeString("zh-TW", { hour12: false }));
          showToast(t("toast.sessionsUpdated"));
        }
      });

      const unlistenPlan = await listen<string>("plan-file-changed", async (event) => {
        if (!activePlanSession || event.payload !== activePlanSession.sessionDir) return;
        await queryClient.invalidateQueries({ queryKey: ["plan", activePlanSession.sessionDir] });
        if (mounted) {
          setRealtimeStatus("active");
          showToast(t("toast.planReloaded"));
        }
      });

      return () => { unlistenSessions(); unlistenPlan(); };
    };

    let cleanup: (() => void) | undefined;
    void setup().then((dispose) => { cleanup = dispose; });
    return () => { mounted = false; cleanup?.(); };
  }, [activePlanSession, queryClient, t]);

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
        await invoke("restart_session_watcher", { copilotRoot: settingsForm.copilotRoot.trim() });
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
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (sessionId: string) =>
      invoke("delete_session", { rootDir: settingsQuery.data?.copilotRoot, sessionId }),
    onSuccess: async () => {
      showToast(t("toast.sessionDeleted"));
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
    () => buildProjectGroups(sessionsQuery.data ?? [], uncategorizedLabel, locale),
    [sessionsQuery.data, uncategorizedLabel, locale],
  );

  const activeProject = useMemo(
    () => groupedProjects.find((p) => p.key === activeView) ?? null,
    [activeView, groupedProjects],
  );

  const recentSessions = useMemo(
    () =>
      [...(sessionsQuery.data ?? [])]
        .sort((a, b) => (b.updatedAt ?? "").localeCompare(a.updatedAt ?? ""))
        .slice(0, 10),
    [sessionsQuery.data],
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

  const closeProjectTab = (projectKey: string) => {
    setOpenProjectKeys((v) => v.filter((k) => k !== projectKey));
    setActiveView((v) => (v === projectKey ? "dashboard" : v));
  };

  const closePlanTab = (planKey: string) => {
    setOpenPlanKeys((v) => v.filter((k) => k !== planKey));
    setActiveView((v) => (v === planKey ? "dashboard" : v));
  };

  const handleSaveSettings = async () => {
    const next: AppSettings = {
      copilotRoot: settingsForm.copilotRoot.trim(),
      terminalPath: settingsForm.terminalPath?.trim() || null,
      externalEditorPath: settingsForm.externalEditorPath?.trim() || null,
      showArchived: settingsForm.showArchived,
    };

    if (!next.copilotRoot) {
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

  const handleBrowseDirectory = async (field: "copilotRoot") => {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") setSettingsForm((v) => ({ ...v, [field]: selected }));
  };

  const handleBrowseFile = async (field: "terminalPath" | "externalEditorPath") => {
    const selected = await open({ directory: false, multiple: false });
    if (typeof selected === "string") setSettingsForm((v) => ({ ...v, [field]: selected }));
  };

  const handleToggleArchived = async (nextValue: boolean) => {
    const next: AppSettings = {
      copilotRoot: settingsForm.copilotRoot.trim(),
      terminalPath: settingsForm.terminalPath?.trim() || null,
      externalEditorPath: settingsForm.externalEditorPath?.trim() || null,
      showArchived: nextValue,
    };
    setSettingsForm((v) => ({ ...v, showArchived: nextValue }));
    settingsMutation.mutate(next);
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

  const handleDeleteSession = (session: SessionInfo) => {
    setConfirmDialog({
      title: t("dialog.deleteTitle"),
      message: `${t("session.confirm.delete")} ${session.summary?.trim() || session.id}?`,
      actionLabel: t("session.actions.delete"),
      tone: "danger",
      onConfirm: () => deleteMutation.mutate(session.id),
    });
  };

  const handleCopyCommand = async (sessionId: string) => {
    await navigator.clipboard.writeText(`copilot --resume=${sessionId}`);
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
    const planKey = `plan:${session.id}`;
    setOpenPlanKeys((v) => (v.includes(planKey) ? v : [...v, planKey]));
    setActiveView(planKey);
  };

  const handleSavePlan = () => {
    if (!activePlanSession) return;
    savePlanMutation.mutate({ sessionDir: activePlanSession.sessionDir, content: planDraft });
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

  return (
    <main className={`app-shell ${isSidebarCollapsed ? "sidebar-collapsed" : ""}`}>
      <Sidebar
        activeView={activeView}
        isSidebarCollapsed={isSidebarCollapsed}
        realtimeStatus={realtimeStatus}
        lastRealtimeSyncAt={lastRealtimeSyncAt}
        onNavigate={(view) => setActiveView(view)}
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
                    : activeView.startsWith("plan:") && activePlanSession
                      ? t("plan.title")
                      : activeProject?.title ?? ""}
              </h2>
              <p className="workspace-subtitle">
                {activeView === "settings"
                  ? t("settings.subtitle")
                  : activeView.startsWith("plan:") && activePlanSession
                    ? (activePlanSession.summary?.trim() || activePlanSession.id)
                    : activeProject?.pathLabel ?? ""}
              </p>
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
              const project = groupedProjects.find((p) => p.key === projectKey);
              if (!project) return null;
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

            {openPlanKeys.map((planKey) => {
              const sessionId = planKey.replace("plan:", "");
              const session = sessionsQuery.data?.find((s) => s.id === sessionId);
              if (!session) return null;
              const tabTitle = session.summary?.trim() || session.id.slice(0, 8);
              return (
                <div
                  key={planKey}
                  className={`tab-item tab-item-project ${activeView === planKey ? "active" : ""}`}
                >
                  <button
                    type="button"
                    className="tab-label"
                    onClick={() => setActiveView(planKey)}
                  >
                    {t("plan.tab")} · {tabTitle}
                  </button>
                  <button
                    type="button"
                    className="tab-close"
                    onClick={() => closePlanTab(planKey)}
                    aria-label={`${t("tabs.close")} ${tabTitle}`}
                  >
                    ×
                  </button>
                </div>
              );
            })}
          </section>
        ) : null}

        <div className="workspace-content">
          {activeView === "dashboard" ? (
            <DashboardView
              sessionsIsLoading={sessionsQuery.isLoading}
              sessionsIsError={sessionsQuery.isError}
              sessionsError={sessionsQuery.error}
              groupedProjects={groupedProjects}
              recentSessions={recentSessions}
              onOpenProject={openProjectTab}
              onOpenRecentSession={(session) =>
                openProjectTab(getProjectKey(session, uncategorizedLabel))
              }
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
            />
          ) : null}

          {activeProject ? (
            <ProjectView
              project={activeProject}
              showArchived={settingsForm.showArchived}
              onToggleArchived={(v) => void handleToggleArchived(v)}
              onOpenTerminal={(s) => void handleOpenTerminal(s)}
              onCopyCommand={(id) => void handleCopyCommand(id)}
              onEditNotes={handleEditNotes}
              onEditTags={handleEditTags}
              onOpenPlan={handleOpenPlan}
              onArchive={handleArchiveSession}
              onDelete={handleDeleteSession}
            />
          ) : null}

          {activeView.startsWith("plan:") && activePlanSession ? (
            <PlanEditor
              session={activePlanSession}
              planDraft={planDraft}
              planPreviewHtml={planPreviewHtml}
              onDraftChange={setPlanDraft}
              onSave={handleSavePlan}
              onOpenExternal={(s) => void handleOpenPlanExternal(s)}
              onClose={() => closePlanTab(activeView)}
            />
          ) : null}
        </div>
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

      {toastMessage ? <div className="toast-banner">{toastMessage}</div> : null}
    </main>
  );
}

export default App;
