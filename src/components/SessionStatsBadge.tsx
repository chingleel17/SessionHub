import { useI18n } from "../i18n/I18nProvider";
import type { SessionStats } from "../types";

type Props = {
  stats?: SessionStats;
  isLoading: boolean;
};

function formatCompactNumber(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(value >= 10_000_000 ? 0 : 1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(value >= 10_000 ? 0 : 1)}K`;
  return String(value);
}

function formatDuration(minutes: number): string {
  if (minutes < 60) return `${minutes}m`;
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return m === 0 ? `${h}h` : `${h}h${m}m`;
}

export function SessionStatsBadge({ stats, isLoading }: Props) {
  const { t } = useI18n();

  if (isLoading) {
    return <div className="stats-badge-row"><span className="stats-badge stats-badge-loading">...</span></div>;
  }

  if (!stats || (stats.outputTokens === 0 && stats.interactionCount === 0 && stats.durationMinutes === 0)) {
    return <div className="stats-badge-row"><span className="stats-badge">{t("stats.noData")}</span></div>;
  }

  return (
    <div className="stats-badge-row">
      <span className="stats-badge">{formatCompactNumber(stats.interactionCount)} {t("stats.turns")}</span>
      <span className="stats-badge">{formatCompactNumber(stats.outputTokens)} {t("stats.tokens")}</span>
      <span className="stats-badge">{formatDuration(stats.durationMinutes)}</span>
      {stats.isLive ? <span className="stats-badge">LIVE</span> : null}
    </div>
  );
}
