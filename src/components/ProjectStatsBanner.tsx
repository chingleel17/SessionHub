import { useMemo } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { SessionInfo, SessionStats } from "../types";

type Props = {
  sessions: SessionInfo[];
  sessionStats: Record<string, SessionStats | undefined>;
  sessionStatsLoading: Record<string, boolean | undefined>;
};

function formatCompactNumber(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(value >= 10_000_000 ? 0 : 1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(value >= 10_000 ? 0 : 1)}K`;
  return String(value);
}

export function ProjectStatsBanner({ sessions, sessionStats, sessionStatsLoading }: Props) {
  const { t } = useI18n();
  const totals = useMemo(() => sessions.reduce(
    (acc, session) => {
      const stats = sessionStats[session.id];
      if (!stats) return acc;
      acc.tokens += stats.outputTokens;
      acc.interactions += stats.interactionCount;
      acc.cost += Object.values(stats.modelMetrics ?? {}).reduce(
        (sum, metric) => sum + metric.requestsCost,
        0,
      );
      return acc;
    },
    { tokens: 0, interactions: 0, cost: 0 },
  ), [sessionStats, sessions]);

  if (sessions.length === 0) return null;
  const loadingCount = sessions.filter((session) => sessionStatsLoading[session.id]).length;

  return (
    <div className="project-stats-banner">
      <strong>{t("stats.projectBanner").replace("{count}", String(sessions.length))}</strong>
      <span>{formatCompactNumber(totals.interactions)} {t("stats.turns")}</span>
      <span>{formatCompactNumber(totals.tokens)} {t("stats.tokens")}</span>
      <span>{t("stats.detail.totalCost")} {totals.cost.toFixed(2).replace(/\.00$/, "")}</span>
      {loadingCount > 0 ? <span>{t("dashboard.stats.loadingState")}: {loadingCount}</span> : null}
    </div>
  );
}
