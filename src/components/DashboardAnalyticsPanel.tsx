import { useI18n } from "../i18n/I18nProvider";
import type { AnalyticsDataPoint } from "../types";
import { PieChart } from "./PieChart";
import { TrendChart } from "./TrendChart";

type PieSlice = {
  label: string;
  value: number;
  color: string;
};

type Props = {
  data: AnalyticsDataPoint[];
  projectSlices: PieSlice[];
  collapsed: boolean;
  isLoading: boolean;
  isRefreshing: boolean;
  errorMessage: string | null;
  onRetry: () => void;
  onToggleCollapsed: () => void;
};

export function DashboardAnalyticsPanel({
  data,
  projectSlices,
  collapsed,
  isLoading,
  isRefreshing,
  errorMessage,
  onRetry,
  onToggleCollapsed,
}: Props) {
  const { t } = useI18n();

  return (
    <section className="analytics-panel">
      <div className="analytics-panel-header">
        <div>
          <h3>{t("analytics.dashboard.title")}</h3>
          <p>{isRefreshing ? t("analytics.dashboard.refreshing") : t("analytics.dashboard.subtitle")}</p>
        </div>
        <button type="button" className="ghost-button" onClick={onToggleCollapsed}>
          {collapsed ? t("analytics.actions.expand") : t("analytics.actions.collapse")}
        </button>
      </div>

      {!collapsed ? (
        <>
          {errorMessage ? (
            <div className="analytics-error-banner">
              <span>{errorMessage}</span>
              <button type="button" className="ghost-button" onClick={onRetry}>
                {t("analytics.actions.retry")}
              </button>
            </div>
          ) : null}

          {isLoading && data.length === 0 ? (
            <div className="analytics-empty-state">{t("analytics.actions.loading")}</div>
          ) : (
            <div className="analytics-grid">
              <TrendChart
                title={t("analytics.charts.dashboardTrend")}
                data={data}
                lines={[
                  { key: "outputTokens", label: t("analytics.metrics.outputTokens"), colorClass: "trend-chart-series--primary" },
                  { key: "interactionCount", label: t("analytics.metrics.interactions"), colorClass: "trend-chart-series--secondary" },
                  { key: "costPoints", label: t("analytics.metrics.costPoints"), colorClass: "trend-chart-series--accent" },
                ]}
                emptyLabel={t("analytics.empty.noData")}
                ariaLabel={t("analytics.aria.dashboardTrend")}
              />
              <PieChart
                title={t("analytics.charts.projectDistribution")}
                slices={projectSlices}
                emptyLabel={t("analytics.empty.noData")}
                ariaLabel={t("analytics.aria.projectDistribution")}
              />
            </div>
          )}
        </>
      ) : null}
    </section>
  );
}
