import { useMemo, useState } from "react";

import { useI18n } from "../i18n/I18nProvider";
import type { AnalyticsDataPoint, AnalyticsGroupBy, SessionInfo, SessionStats } from "../types";
import { PieChart } from "./PieChart";
import { TrendChart } from "./TrendChart";

type Props = {
  sessions: SessionInfo[];
  sessionStats: Record<string, SessionStats | undefined>;
  onFetchAnalytics: (
    cwd: string | null,
    startDate: string,
    endDate: string,
    groupBy: AnalyticsGroupBy,
  ) => Promise<AnalyticsDataPoint[] | null>;
};

const PIE_COLORS = ["#6366f1", "#14b8a6", "#f59e0b", "#ef4444", "#8b5cf6", "#06b6d4"];

function formatDateInput(date: Date) {
  return date.toISOString().slice(0, 10);
}

function buildQuickRange(type: "7d" | "30d" | "month") {
  const now = new Date();
  const endDate = formatDateInput(now);
  if (type === "month") {
    return { startDate: formatDateInput(new Date(now.getFullYear(), now.getMonth(), 1)), endDate };
  }
  const days = type === "7d" ? 6 : 29;
  const start = new Date(now);
  start.setDate(now.getDate() - days);
  return { startDate: formatDateInput(start), endDate };
}

function isSessionInRange(session: SessionInfo, startDate: string, endDate: string) {
  if (!session.updatedAt) return false;
  const value = session.updatedAt.slice(0, 10);
  return value >= startDate && value <= endDate;
}

export function ProjectAnalyticsTab({ sessions, sessionStats, onFetchAnalytics }: Props) {
  const { t } = useI18n();
  const initialRange = useMemo(() => buildQuickRange("7d"), []);
  const [startDate, setStartDate] = useState(initialRange.startDate);
  const [endDate, setEndDate] = useState(initialRange.endDate);
  const [groupBy, setGroupBy] = useState<AnalyticsGroupBy>("day");
  const [analyticsData, setAnalyticsData] = useState<AnalyticsDataPoint[]>([]);
  const [hasFetched, setHasFetched] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  const handleQuickRange = (type: "7d" | "30d" | "month") => {
    const next = buildQuickRange(type);
    setStartDate(next.startDate);
    setEndDate(next.endDate);
  };

  const handleGenerate = async () => {
    setIsLoading(true);
    try {
      const data = await onFetchAnalytics(sessions[0]?.cwd ?? null, startDate, endDate, groupBy);
      if (data) {
        setAnalyticsData(data);
        setHasFetched(true);
      }
    } finally {
      setIsLoading(false);
    }
  };

  const modelSlices = useMemo(() => {
    const totals = new Map<string, number>();
    sessions
      .filter((session) => isSessionInRange(session, startDate, endDate))
      .forEach((session) => {
        const stats = sessionStats[session.id];
        if (!stats) return;
        Object.entries(stats.modelMetrics ?? {}).forEach(([model, metric]) => {
          totals.set(model, (totals.get(model) ?? 0) + metric.inputTokens + metric.outputTokens);
        });
      });

    return Array.from(totals.entries()).map(([label, value], index) => ({
      label,
      value,
      color: PIE_COLORS[index % PIE_COLORS.length],
    }));
  }, [endDate, sessionStats, sessions, startDate]);

  return (
    <section className="project-analytics-tab">
      <section className="toolbar-card analytics-controls-card">
        <div className="analytics-controls-grid">
          <div className="analytics-quick-actions">
            <button type="button" className="ghost-button" onClick={() => handleQuickRange("7d")}>
              7D
            </button>
            <button type="button" className="ghost-button" onClick={() => handleQuickRange("30d")}>
              30D
            </button>
            <button type="button" className="ghost-button" onClick={() => handleQuickRange("month")}>
              {t("analytics.quickRange.month")}
            </button>
          </div>

          <label className="field-group compact-field">
            <span>{t("analytics.filters.startDate")}</span>
            <input type="date" value={startDate} onChange={(event) => setStartDate(event.currentTarget.value)} />
          </label>

          <label className="field-group compact-field">
            <span>{t("analytics.filters.endDate")}</span>
            <input type="date" value={endDate} onChange={(event) => setEndDate(event.currentTarget.value)} />
          </label>

          <label className="field-group compact-field">
            <span>{t("analytics.filters.groupBy")}</span>
            <select value={groupBy} onChange={(event) => setGroupBy(event.currentTarget.value as AnalyticsGroupBy)}>
              <option value="day">{t("analytics.groupBy.day")}</option>
              <option value="week">{t("analytics.groupBy.week")}</option>
              <option value="month">{t("analytics.groupBy.month")}</option>
            </select>
          </label>

          <div className="analytics-generate-action">
            <button type="button" className="primary-button" onClick={() => void handleGenerate()} disabled={isLoading}>
              {isLoading ? t("analytics.actions.loading") : t("analytics.actions.generate")}
            </button>
          </div>
        </div>
      </section>

      {!hasFetched ? (
        <div className="analytics-empty-state">{t("analytics.empty.projectPrompt")}</div>
      ) : (
        <div className="analytics-grid">
          <TrendChart
            title={`${t("analytics.charts.projectTrend")} · ${startDate} ~ ${endDate}`}
            data={analyticsData}
            lines={[
              { key: "outputTokens", label: t("analytics.metrics.outputTokens"), colorClass: "trend-chart-series--primary" },
              { key: "interactionCount", label: t("analytics.metrics.interactions"), colorClass: "trend-chart-series--secondary" },
              { key: "costPoints", label: t("analytics.metrics.costPoints"), colorClass: "trend-chart-series--accent" },
            ]}
            emptyLabel={t("analytics.empty.noData")}
            ariaLabel={t("analytics.aria.projectTrend")}
          />
          <PieChart
            title={`${t("analytics.charts.modelDistribution")} · ${startDate} ~ ${endDate}`}
            slices={modelSlices}
            emptyLabel={t("analytics.empty.noData")}
            ariaLabel={t("analytics.aria.modelDistribution")}
          />
        </div>
      )}
    </section>
  );
}
