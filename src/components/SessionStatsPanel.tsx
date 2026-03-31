import { useI18n } from "../i18n/I18nProvider";
import type { SessionStats } from "../types";

type Props = {
  stats: SessionStats;
};

function formatCompactNumber(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(value >= 10_000_000 ? 0 : 1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(value >= 10_000 ? 0 : 1)}K`;
  return String(value);
}

export function SessionStatsPanel({ stats }: Props) {
  const { t } = useI18n();
  const toolEntries = Object.entries(stats.toolBreakdown).sort((a, b) => b[1] - a[1]);
  const averageTokens = stats.interactionCount > 0 ? Math.round(stats.outputTokens / stats.interactionCount) : 0;

  return (
    <section className="stats-panel">
      <div className="stats-panel-row">
        <strong>{formatCompactNumber(stats.toolCallCount)}</strong>
        <span>{t("stats.detail.toolCalls")}</span>
      </div>
      <div className="stats-panel-row">
        <strong>{formatCompactNumber(stats.reasoningCount)}</strong>
        <span>{t("stats.detail.reasoning")}</span>
      </div>
      <div className="stats-panel-row">
        <strong>{formatCompactNumber(averageTokens)}</strong>
        <span>{t("stats.detail.avgTokens")}</span>
      </div>
      <div className="stats-panel-row stats-panel-row-block">
        <strong>{t("stats.detail.models")}</strong>
        <span>{stats.modelsUsed.length > 0 ? stats.modelsUsed.join(", ") : t("stats.noData")}</span>
      </div>
      {stats.isLive ? <div className="stats-panel-live">{t("stats.detail.live")}</div> : null}
      {toolEntries.length > 0 ? (
        <div className="stats-tool-table">
          {toolEntries.map(([toolName, count]) => (
            <div key={toolName} className="stats-tool-row">
              <span>{toolName}</span>
              <strong>{formatCompactNumber(count)}</strong>
            </div>
          ))}
        </div>
      ) : null}
    </section>
  );
}
