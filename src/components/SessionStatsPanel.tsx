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

  return (
    <section className="stats-panel">
      <div className="stats-panel-grid">
        <div className="stats-panel-col">
          <div className="stats-panel-metric">
            <span className="stats-panel-label">{t("stats.tokens")}</span>
            <strong>{formatCompactNumber(stats.outputTokens)}</strong>
          </div>
          {stats.inputTokens > 0 ? (
            <div className="stats-panel-metric">
              <span className="stats-panel-label">{t("stats.detail.inputTokens")}</span>
              <strong>{formatCompactNumber(stats.inputTokens)}</strong>
            </div>
          ) : null}
          <div className="stats-panel-metric">
            <span className="stats-panel-label">{t("stats.turns")}</span>
            <strong>{formatCompactNumber(stats.interactionCount)}</strong>
          </div>
          <div className="stats-panel-metric">
            <span className="stats-panel-label">{t("stats.detail.toolCalls")}</span>
            <strong>{formatCompactNumber(stats.toolCallCount)}</strong>
          </div>
          {stats.reasoningCount > 0 ? (
            <div className="stats-panel-metric">
              <span className="stats-panel-label">{t("stats.detail.reasoning")}</span>
              <strong>{formatCompactNumber(stats.reasoningCount)}</strong>
            </div>
          ) : null}
          <div className="stats-panel-metric">
            <span className="stats-panel-label">{t("stats.duration")}</span>
            <strong>{formatCompactNumber(stats.durationMinutes)}</strong>
          </div>
          {stats.isLive ? (
            <div className="stats-panel-live-row">
              <span className="stats-live-dot" />
              <span>{t("statsLiveIndicator")}</span>
            </div>
          ) : null}
        </div>
        <div className="stats-panel-col">
          <div className="stats-panel-section">
            <strong>{t("stats.detail.models")}</strong>
            <span className="stats-panel-text">
              {stats.modelsUsed.length > 0 ? stats.modelsUsed.join(", ") : t("stats.noData")}
            </span>
          </div>
          <div className="stats-panel-section">
            <strong>{t("stats.detail.toolCalls")}</strong>
            {toolEntries.length > 0 ? (
              <div className="stats-tool-breakdown-scroll">
                {toolEntries.map(([toolName, count]) => (
                  <div key={toolName} className="stats-tool-row">
                    <span>{toolName}</span>
                    <strong>{formatCompactNumber(count)}</strong>
                  </div>
                ))}
              </div>
            ) : (
              <span className="stats-panel-text">{t("stats.noData")}</span>
            )}
          </div>
        </div>
      </div>
    </section>
  );
}
