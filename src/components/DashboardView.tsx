import { useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { IdeLauncherType, ProjectGroup, SessionActivityStatus, SessionInfo, ToolAvailability } from "../types";

type Props = {
  sessionsIsLoading: boolean;
  sessionsIsFetching: boolean;
  sessionsIsError: boolean;
  sessionsError: unknown;
  groupedProjects: ProjectGroup[];
  recentSessions: SessionInfo[];
  dashboardPeriod: "week" | "month";
  onPeriodChange: (period: "week" | "month") => void;
  filteredTotalOutputTokens: number;
  filteredTotalInteractions: number;
  onOpenProject: (projectKey: string) => void;
  onOpenRecentSession: (session: SessionInfo) => void;
  activityStatusMap: Map<string, SessionActivityStatus>;
  onOpenInTool: (session: SessionInfo, tool: IdeLauncherType) => void;
  onFocusTerminal: (session: SessionInfo) => void;
  defaultLauncher: string | null;
  toolAvailability: ToolAvailability | null;
};

const RECENT_TITLE_MAX_LEN = 80;
const PROJECT_LAST_SESSION_MAX_LEN = 60;

function truncate(text: string, maxLen: number): string {
  return text.length > maxLen ? `${text.slice(0, maxLen)}…` : text;
}

function getSessionTitle(session: SessionInfo) {
  return session.summary?.trim() || session.id;
}

function getProjectShortName(cwd?: string | null): string | null {
  if (!cwd) return null;
  const parts = cwd.replace(/\//g, "\\").split("\\").filter(Boolean);
  return parts[parts.length - 1] ?? null;
}

function formatCompactNumber(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(value >= 10_000_000 ? 0 : 1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(value >= 10_000 ? 0 : 1)}K`;
  return String(value);
}

const LAUNCHER_OPTIONS: { type: IdeLauncherType; label: string; icon: string; availKey?: keyof ToolAvailability }[] = [
  { type: "terminal", label: "Terminal", icon: ">_" },
  { type: "copilot", label: "Copilot", icon: "C", availKey: "copilot" },
  { type: "opencode", label: "OpenCode", icon: "O", availKey: "opencode" },
  { type: "gemini", label: "Gemini", icon: "G", availKey: "gemini" },
  { type: "vscode", label: "VS Code", icon: "⌨", availKey: "vscode" },
  { type: "explorer", label: "Explorer", icon: "📁" },
];

function getActivityStatusLabel(t: (k: string) => string, status?: SessionActivityStatus): { label: string; cls: string } {
  if (!status) return { label: t("dashboard.kanban.status.idle"), cls: "kanban-status-idle" };
  switch (status.status) {
    case "active": return { label: t(`dashboard.kanban.detail.${status.detail ?? "working"}`), cls: "kanban-status-active" };
    case "waiting": return { label: t("dashboard.kanban.status.waiting"), cls: "kanban-status-waiting" };
    case "done": return { label: t("dashboard.kanban.status.done"), cls: "kanban-status-done" };
    default: return { label: t("dashboard.kanban.status.idle"), cls: "kanban-status-idle" };
  }
}

function KanbanCard({
  session,
  activityStatus,
  onOpenInTool,
  onFocusTerminal,
  defaultLauncher,
  toolAvailability,
}: {
  session: SessionInfo;
  activityStatus?: SessionActivityStatus;
  onOpenInTool: (session: SessionInfo, tool: IdeLauncherType) => void;
  onFocusTerminal: (session: SessionInfo) => void;
  defaultLauncher: string | null;
  toolAvailability: ToolAvailability | null;
}) {
  const { t } = useI18n();
  const [showLauncher, setShowLauncher] = useState(false);
  const { label, cls } = getActivityStatusLabel(t as (k: string) => string, activityStatus);
  const title = session.summary?.trim() || session.id;
  const projectName = session.cwd?.split("\\").pop() ?? null;

  const handleDefaultLaunch = () => {
    const tool = (defaultLauncher as IdeLauncherType) ?? "terminal";
    onOpenInTool(session, tool);
  };

  const isToolAvailable = (opt: typeof LAUNCHER_OPTIONS[number]) => {
    if (!opt.availKey || !toolAvailability) return true;
    return toolAvailability[opt.availKey];
  };

  return (
    <div className="kanban-card">
      <div className={`kanban-card-status-bar ${cls}`} />
      <div className="kanban-card-body">
        <p className="kanban-card-title" title={title}>{title.length > 60 ? `${title.slice(0, 60)}…` : title}</p>
        {projectName ? <p className="kanban-card-project">{projectName}</p> : null}
        <div className="kanban-card-provider">
          <span className={`provider-tag provider-tag--${session.provider}`}>
            {session.provider === "opencode" ? "OpenCode" : "Copilot"}
          </span>
          <span className={`kanban-activity-badge ${cls}`}>{label}</span>
        </div>
        <div className="kanban-card-actions">
          <button type="button" className="kanban-action-btn" onClick={handleDefaultLaunch} title={t("session.actions.openTool")}>
            ▶
          </button>
          <div style={{ position: "relative" }}>
            <button
              type="button"
              className="kanban-action-btn"
              onClick={() => setShowLauncher((v) => !v)}
              title={t("session.actions.chooseTool")}
            >
              ⋯
            </button>
            {showLauncher ? (
              <div className="kanban-launcher-menu">
                {LAUNCHER_OPTIONS.map((opt) => {
                  const available = isToolAvailable(opt);
                  return (
                    <button
                      key={opt.type}
                      type="button"
                      className={`kanban-launcher-option${!available ? " kanban-launcher-option--disabled" : ""}`}
                      disabled={!available}
                      onClick={() => { onOpenInTool(session, opt.type); setShowLauncher(false); }}
                    >
                      <span className="launcher-option-icon">{opt.icon}</span>
                      {opt.label}
                      {!available ? <span className="launcher-option-unavail"> —</span> : null}
                    </button>
                  );
                })}
              </div>
            ) : null}
          </div>
          <button type="button" className="kanban-action-btn" onClick={() => onFocusTerminal(session)} title={t("session.actions.focusTerminal")}>
            ⊙
          </button>
        </div>
      </div>
    </div>
  );
}

function KanbanBoard({
  groupedProjects,
  activityStatusMap,
  onOpenInTool,
  onFocusTerminal,
  defaultLauncher,
  toolAvailability,
}: {
  groupedProjects: ProjectGroup[];
  activityStatusMap: Map<string, SessionActivityStatus>;
  onOpenInTool: (session: SessionInfo, tool: IdeLauncherType) => void;
  onFocusTerminal: (session: SessionInfo) => void;
  defaultLauncher: string | null;
  toolAvailability: ToolAvailability | null;
}) {
  const { t } = useI18n();
  const allSessions = groupedProjects.flatMap((p) => p.sessions);

  const columns: { key: string; label: string; sessions: SessionInfo[] }[] = [
    { key: "active", label: t("dashboard.kanban.column.active"), sessions: [] },
    { key: "waiting", label: t("dashboard.kanban.column.waiting"), sessions: [] },
    { key: "idle", label: t("dashboard.kanban.column.idle"), sessions: [] },
    { key: "done", label: t("dashboard.kanban.column.done"), sessions: [] },
  ];

  for (const s of allSessions) {
    if (s.isArchived) { columns[3].sessions.push(s); continue; }
    const status = activityStatusMap.get(s.id)?.status ?? "idle";
    const col = columns.find((c) => c.key === status) ?? columns[2];
    col.sessions.push(s);
  }

  return (
    <div className="kanban-board">
      {columns.map((col) => (
        <div key={col.key} className="kanban-column">
          <div className="kanban-column-header">
            <span className="kanban-column-title">{col.label}</span>
            <span className="kanban-column-count">{col.sessions.length}</span>
          </div>
          <div className="kanban-column-cards">
            {col.sessions.map((session) => (
              <KanbanCard
                key={session.id}
                session={session}
                activityStatus={activityStatusMap.get(session.id)}
                onOpenInTool={onOpenInTool}
                onFocusTerminal={onFocusTerminal}
                defaultLauncher={defaultLauncher}
                toolAvailability={toolAvailability}
              />
            ))}
            {col.sessions.length === 0 ? (
              <p className="kanban-empty">{t("dashboard.kanban.empty")}</p>
            ) : null}
          </div>
        </div>
      ))}
    </div>
  );
}

export function DashboardView({
  sessionsIsLoading,
  sessionsIsFetching,
  sessionsIsError,
  sessionsError,
  groupedProjects,
  recentSessions,
  dashboardPeriod,
  onPeriodChange,
  filteredTotalOutputTokens,
  filteredTotalInteractions,
  onOpenProject,
  onOpenRecentSession,
  activityStatusMap,
  onOpenInTool,
  onFocusTerminal,
  defaultLauncher,
  toolAvailability,
}: Props) {
  const { t } = useI18n();
  const [viewMode, setViewMode] = useState<"list" | "kanban">("list");

  const totalSessionCount = groupedProjects.reduce((sum, p) => sum + p.sessions.length, 0);
  const activeProjectCount = groupedProjects.length;
  const allSessions = groupedProjects.flatMap((p) => p.sessions);
  const archivedCount = allSessions.filter((s) => s.isArchived).length;
  const parseErrorCount = allSessions.filter((s) => s.parseError).length;

  const loading = sessionsIsLoading;
  const fetching = sessionsIsFetching;

  return (
    <section className="dashboard-layout">
      {/* Stats cards row */}
      <div className="stat-cards-row">
        <div className="stat-card stat-card--sessions">
          <span className="stat-card-icon">🗂</span>
          <strong className="stat-card-value">{loading ? "…" : formatCompactNumber(totalSessionCount)}</strong>
          <span className="stat-card-label">{t("dashboard.stats.totalSessions")}</span>
        </div>
        <div className="stat-card stat-card--projects">
          <span className="stat-card-icon">📁</span>
          <strong className="stat-card-value">{loading ? "…" : activeProjectCount}</strong>
          <span className="stat-card-label">{t("dashboard.stats.activeProjects")}</span>
        </div>
        <div className="stat-card stat-card--archived">
          <span className="stat-card-icon">📦</span>
          <strong className="stat-card-value">{loading ? "…" : archivedCount}</strong>
          <span className="stat-card-label">{t("dashboard.stats.archivedSessions")}</span>
        </div>
        {parseErrorCount > 0 ? (
          <div className="stat-card stat-card--errors">
            <span className="stat-card-icon">⚠</span>
            <strong className="stat-card-value">{loading ? "…" : parseErrorCount}</strong>
            <span className="stat-card-label">{t("dashboard.stats.parseErrors")}</span>
          </div>
        ) : null}
        <div className="stat-card stat-card--tokens">
          <span className="stat-card-icon">🪙</span>
          <strong className="stat-card-value">{loading ? "…" : formatCompactNumber(filteredTotalOutputTokens)}</strong>
          <span className="stat-card-label">{t("dashboard.stats.totalTokens")}</span>
        </div>
        <div className="stat-card stat-card--interactions">
          <span className="stat-card-icon">💬</span>
          <strong className="stat-card-value">{loading ? "…" : formatCompactNumber(filteredTotalInteractions)}</strong>
          <span className="stat-card-label">{t("dashboard.stats.totalInteractions")}</span>
        </div>
        {/* Period + view mode toggles */}
        <div className="stat-card-toggles">
          <div className="period-toggle">
            <button
              type="button"
              className={`period-toggle-btn${dashboardPeriod === "week" ? " active" : ""}`}
              onClick={() => onPeriodChange("week")}
            >
              {t("dashboard.stats.period.week")}
            </button>
            <button
              type="button"
              className={`period-toggle-btn${dashboardPeriod === "month" ? " active" : ""}`}
              onClick={() => onPeriodChange("month")}
            >
              {t("dashboard.stats.period.month")}
            </button>
          </div>
          <div className="view-mode-toggle">
            <button
              type="button"
              className={`period-toggle-btn${viewMode === "list" ? " active" : ""}`}
              onClick={() => setViewMode("list")}
            >
              {t("dashboard.viewMode.list")}
            </button>
            <button
              type="button"
              className={`period-toggle-btn${viewMode === "kanban" ? " active" : ""}`}
              onClick={() => setViewMode("kanban")}
            >
              {t("dashboard.viewMode.kanban")}
            </button>
          </div>
        </div>
      </div>

      {sessionsIsError ? (
        <article className="info-card status-card status-card-error">
          <h3>{t("dashboard.status.errorTitle")}</h3>
          <p className="placeholder-copy">
            {sessionsError instanceof Error
              ? sessionsError.message
              : t("dashboard.status.errorDescription")}
          </p>
        </article>
      ) : null}

      {viewMode === "kanban" ? (
        <KanbanBoard
          groupedProjects={groupedProjects}
          activityStatusMap={activityStatusMap}
          onOpenInTool={onOpenInTool}
          onFocusTerminal={onFocusTerminal}
          defaultLauncher={defaultLauncher}
          toolAvailability={toolAvailability}
        />
      ) : (
        <section className="content-grid">
          <article className="info-card">
            <div className="section-heading">
              <h3>{t("dashboard.projects.title")}</h3>
              <span>{t("dashboard.projects.subtitle")}</span>
            </div>

            <div className="project-list">
              {groupedProjects.map((project) => {
                const lastSession = project.sessions[0];
                const lastSessionTitle = lastSession?.summary?.trim() || null;
                return (
                  <button
                    key={project.key}
                    type="button"
                    className="project-item"
                    onClick={() => onOpenProject(project.key)}
                  >
                    <div className="project-item-info">
                      <strong>{project.title}</strong>
                      <p>{project.pathLabel}</p>
                      {lastSessionTitle ? (
                        <p className="project-last-session-title">
                          {truncate(lastSessionTitle, PROJECT_LAST_SESSION_MAX_LEN)}
                        </p>
                      ) : null}
                    </div>

                    <div className="project-meta">
                      <span>
                        {project.sessions.length} {t("dashboard.projects.sessionCountSuffix")}
                      </span>
                      <span>{project.updatedAtLabel}</span>
                    </div>
                  </button>
                );
              })}
            </div>
          </article>

          <article className="info-card">
            <div className="section-heading">
              <h3>{t("dashboard.recent.title")}</h3>
              <span className="recent-subtitle-row">
                <span>{fetching ? t("dashboard.status.scanning") : t("dashboard.recent.subtitle")}</span>
              </span>
            </div>

            <ul className="feature-list feature-list-tight">
              {recentSessions.map((session) => {
                const fullTitle = getSessionTitle(session);
                const projectName = getProjectShortName(session.cwd);
                return (
                  <li key={session.id}>
                    <div className="recent-session-item">
                      <button
                        type="button"
                        className="inline-link"
                        title={fullTitle}
                        onClick={() => onOpenRecentSession(session)}
                      >
                        {truncate(fullTitle, RECENT_TITLE_MAX_LEN)}
                      </button>
                      {projectName ? (
                        <span className="recent-session-project">{projectName}</span>
                      ) : null}
                    </div>
                  </li>
                );
              })}
            </ul>
          </article>
        </section>
      )}
    </section>
  );
}