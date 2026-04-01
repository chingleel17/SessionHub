import { useI18n } from "../i18n/I18nProvider";
import type { ProjectGroup, SessionInfo } from "../types";

type Props = {
  sessionsIsLoading: boolean;
  sessionsIsError: boolean;
  sessionsError: unknown;
  groupedProjects: ProjectGroup[];
  recentSessions: SessionInfo[];
  totalOutputTokens: number;
  totalInteractions: number;
  onOpenProject: (projectKey: string) => void;
  onOpenRecentSession: (session: SessionInfo) => void;
};

function formatCompactNumber(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(value >= 10_000_000 ? 0 : 1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(value >= 10_000 ? 0 : 1)}K`;
  return String(value);
}

function getSessionTitle(session: SessionInfo) {
  return session.summary?.trim() || session.id;
}

export function DashboardView({
  sessionsIsLoading,
  sessionsIsError,
  sessionsError,
  groupedProjects,
  recentSessions,
  totalOutputTokens,
  totalInteractions,
  onOpenProject,
  onOpenRecentSession,
}: Props) {
  const { t } = useI18n();

  const loadingStatsValue = sessionsIsLoading
    ? "..."
    : groupedProjects.reduce((sum, p) => sum + p.sessions.length, 0);
  const activeProjectCount = groupedProjects.length;
  const allSessions = groupedProjects.flatMap((p) => p.sessions);
  const archivedCount = allSessions.filter((s) => s.isArchived).length;
  const parseErrorCount = allSessions.filter((s) => s.parseError).length;

  const providerCounts: Record<string, number> = {};
  for (const session of allSessions) {
    const key = session.provider ?? "copilot";
    providerCounts[key] = (providerCounts[key] ?? 0) + 1;
  }
  const providerEntries = Object.entries(providerCounts).sort(
    ([, a], [, b]) => b - a,
  );
  const hasMultipleProviders = providerEntries.length > 1;

  return (
    <section className="dashboard-layout">
      <section className="stats-grid">
        <article className="stat-card">
          <span className="stat-label">{t("dashboard.stats.totalSessions")}</span>
          <strong>{loadingStatsValue}</strong>
        </article>
        <article className="stat-card">
          <span className="stat-label">{t("dashboard.stats.activeProjects")}</span>
          <strong>{sessionsIsLoading ? "..." : activeProjectCount}</strong>
        </article>
        <article className="stat-card">
          <span className="stat-label">{t("dashboard.stats.archivedSessions")}</span>
          <strong>{sessionsIsLoading ? "..." : archivedCount}</strong>
        </article>
        <article className="stat-card">
          <span className="stat-label">{t("dashboard.stats.parseErrors")}</span>
          <strong>{sessionsIsLoading ? "..." : parseErrorCount}</strong>
        </article>
        <article className="stat-card">
          <span className="stat-label">{t("dashboard.stats.totalTokens")}</span>
          <strong>{sessionsIsLoading ? "..." : formatCompactNumber(totalOutputTokens)}</strong>
        </article>
        <article className="stat-card">
          <span className="stat-label">{t("dashboard.stats.totalInteractions")}</span>
          <strong>{sessionsIsLoading ? "..." : formatCompactNumber(totalInteractions)}</strong>
        </article>
        {hasMultipleProviders ? (
          <article className="stat-card">
            <span className="stat-label">{t("dashboard.stats.providerDistribution")}</span>
            <div style={{ marginTop: 8, display: "flex", flexWrap: "wrap", gap: 8 }}>
              {providerEntries.map(([provider, count]) => (
                <span key={provider} className={`provider-tag provider-tag--${provider}`}>
                  {provider === "copilot" ? "Copilot" : provider === "opencode" ? "OpenCode" : provider}: {count}
                </span>
              ))}
            </div>
          </article>
        ) : null}
      </section>

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
            {groupedProjects.map((project) => (
              <button
                key={project.key}
                type="button"
                className="project-item"
                onClick={() => onOpenProject(project.key)}
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
                  onClick={() => onOpenRecentSession(session)}
                >
                  {getSessionTitle(session)}
                </button>
              </li>
            ))}
          </ul>
        </article>
      </section>
    </section>
  );
}
