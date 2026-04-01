import { useI18n } from "../i18n/I18nProvider";
import type { ProjectGroup, SessionInfo } from "../types";

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
}: Props) {
  const { t } = useI18n();

  const totalSessionCount = groupedProjects.reduce((sum, p) => sum + p.sessions.length, 0);
  const activeProjectCount = groupedProjects.length;
  const allSessions = groupedProjects.flatMap((p) => p.sessions);
  const archivedCount = allSessions.filter((s) => s.isArchived).length;
  const parseErrorCount = allSessions.filter((s) => s.parseError).length;

  const loading = sessionsIsLoading;
  const fetching = sessionsIsFetching;

  return (
    <section className="dashboard-layout">
      <div className="stat-bar">
        <div className="stat-bar-item">
          <span className="stat-bar-icon">🗂</span>
          <strong className="stat-bar-value">{loading ? "..." : formatCompactNumber(totalSessionCount)}</strong>
          <span className="stat-bar-label">{t("dashboard.stats.totalSessions")}</span>
        </div>
        <div className="stat-bar-item">
          <span className="stat-bar-icon">📁</span>
          <strong className="stat-bar-value">{loading ? "..." : activeProjectCount}</strong>
          <span className="stat-bar-label">{t("dashboard.stats.activeProjects")}</span>
        </div>
        <div className="stat-bar-item">
          <span className="stat-bar-icon">📦</span>
          <strong className="stat-bar-value">{loading ? "..." : archivedCount}</strong>
          <span className="stat-bar-label">{t("dashboard.stats.archivedSessions")}</span>
        </div>
        <div className="stat-bar-item">
          <span className="stat-bar-icon">⚠</span>
          <strong className="stat-bar-value">{loading ? "..." : parseErrorCount}</strong>
          <span className="stat-bar-label">{t("dashboard.stats.parseErrors")}</span>
        </div>
        <div className="stat-bar-separator" />
        <div className="stat-bar-item">
          <span className="stat-bar-icon">🪙</span>
          <strong className="stat-bar-value">{loading ? "..." : formatCompactNumber(filteredTotalOutputTokens)}</strong>
          <span className="stat-bar-label">{t("dashboard.stats.totalTokens")}</span>
        </div>
        <div className="stat-bar-item">
          <span className="stat-bar-icon">💬</span>
          <strong className="stat-bar-value">{loading ? "..." : formatCompactNumber(filteredTotalInteractions)}</strong>
          <span className="stat-bar-label">{t("dashboard.stats.totalInteractions")}</span>
        </div>
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
    </section>
  );
}
