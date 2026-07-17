import { useState } from "react";
import { useI18n } from "../i18n/I18nProvider";
import type { SessionStats } from "../types";
import { formatDuration } from "../utils/formatDuration";

type Props = {
  stats: SessionStats;
};

function formatCompactNumber(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(value >= 10_000_000 ? 0 : 1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(value >= 10_000 ? 0 : 1)}K`;
  return String(value);
}

function formatCost(value: number) {
  return value.toFixed(2).replace(/\.00$/, "");
}

export function SessionStatsPanel({ stats }: Props) {
  const { t } = useI18n();
  const [showAllTools, setShowAllTools] = useState(false);
  const toolEntries = Object.entries(stats.toolBreakdown).sort((a, b) => b[1] - a[1]);
  const modelMetricEntries = Object.entries(stats.modelMetrics ?? {}).sort((a, b) => b[1].requestsCost - a[1].requestsCost);
  const visibleToolEntries = showAllTools ? toolEntries : toolEntries.slice(0, 5);
  const hiddenToolCount = toolEntries.length - 5;
  const hasModelCosts = modelMetricEntries.some(([, metric]) => metric.requestsCost > 0);
  const totalInputTokens = modelMetricEntries.reduce((sum, [, metric]) => sum + metric.inputTokens, 0);
  const totalOutputTokens = modelMetricEntries.reduce((sum, [, metric]) => sum + metric.outputTokens, 0);
  const totalModelCost = modelMetricEntries.reduce((sum, [, metric]) => sum + metric.requestsCost, 0);

  return (
    <section className="stats-panel">
      <div className="stats-panel-section stats-panel-summary">
        <div className="stats-panel-stat-row">
          <div className="stats-panel-stat">
            <strong>{formatCompactNumber(stats.outputTokens)}</strong>
            <span>{t("stats.detail.outputTokens")}</span>
          </div>
          {stats.inputTokens > 0 ? (
            <div className="stats-panel-stat">
              <strong>{formatCompactNumber(stats.inputTokens)}</strong>
              <span>{t("stats.detail.inputTokens")}</span>
            </div>
          ) : null}
          <div className="stats-panel-stat">
            <strong>{formatCompactNumber(stats.interactionCount)}</strong>
            <span>{t("stats.turns")}</span>
          </div>
          <div className="stats-panel-stat">
            <strong>{formatCompactNumber(stats.toolCallCount)}</strong>
            <span>{t("stats.detail.toolCalls")}</span>
          </div>
          {stats.reasoningCount > 0 ? (
            <div className="stats-panel-stat">
              <strong>{formatCompactNumber(stats.reasoningCount)}</strong>
              <span>{t("stats.detail.reasoning")}</span>
            </div>
          ) : null}
          <div className="stats-panel-stat">
            <strong>{formatDuration(stats.durationMinutes)}</strong>
            <span>{t("stats.duration")}</span>
          </div>
        </div>
        {stats.isLive ? (
          <div className="stats-panel-live-row">
            <span className="stats-live-dot" />
            <span>{t("stats.detail.live")}</span>
          </div>
        ) : null}
      </div>

      <div className="stats-panel-section">
        <strong>{t("stats.detail.models")}</strong>
        {modelMetricEntries.length > 0 ? (
          <div className="stats-model-table-wrap">
            <table className="stats-model-table">
              <thead>
                <tr>
                  <th>{t("stats.detail.models")}</th>
                  <th>{t("stats.detail.inputTokens")}</th>
                  <th>{t("stats.detail.outputTokens")}</th>
                  {hasModelCosts ? <th>{t("stats.detail.modelCost")}</th> : null}
                </tr>
              </thead>
              <tbody>
                {modelMetricEntries.map(([modelName, metric]) => (
                  <tr key={modelName}>
                    <td>{modelName}</td>
                    <td>{formatCompactNumber(metric.inputTokens)}</td>
                    <td>{formatCompactNumber(metric.outputTokens)}</td>
                    {hasModelCosts ? <td>{metric.requestsCost > 0 ? formatCost(metric.requestsCost) : "-"}</td> : null}
                  </tr>
                ))}
              </tbody>
              {modelMetricEntries.length > 1 ? (
                <tfoot>
                  <tr>
                    <th>{t("stats.detail.total")}</th>
                    <td>{formatCompactNumber(totalInputTokens)}</td>
                    <td>{formatCompactNumber(totalOutputTokens)}</td>
                    {hasModelCosts ? <td>{formatCost(totalModelCost)}</td> : null}
                  </tr>
                </tfoot>
              ) : null}
            </table>
          </div>
        ) : (
          <span className="stats-panel-text">
            {stats.modelsUsed.length > 0 ? stats.modelsUsed.join(", ") : t("stats.noData")}
          </span>
        )}
      </div>

      <div className="stats-panel-section">
        <strong>{t("stats.detail.toolCalls")}</strong>
        {toolEntries.length > 0 ? (
          <div className="stats-tool-breakdown">
            {visibleToolEntries.map(([toolName, count]) => (
              <div key={toolName} className="stats-tool-row">
                <span>{toolName}</span>
                <strong>{formatCompactNumber(count)}</strong>
              </div>
            ))}
            {hiddenToolCount > 0 ? (
              <button
                type="button"
                className="stats-panel-expand"
                aria-expanded={showAllTools}
                onClick={() => setShowAllTools((value) => !value)}
              >
                {showAllTools
                  ? t("stats.detail.showLess")
                  : t("stats.detail.showMore").replace("{count}", String(hiddenToolCount))}
              </button>
            ) : null}
          </div>
        ) : (
          <span className="stats-panel-text">{t("stats.noData")}</span>
        )}
      </div>
    </section>
  );
}
