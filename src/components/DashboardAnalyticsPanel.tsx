import { lazy, Suspense, useEffect, useState } from "react";

import { useI18n } from "../i18n/I18nProvider";
import type { AnalyticsReport } from "../types";

const AnalyticsTrendChart = lazy(async () => {
  const module = await import("./AnalyticsTrendChart");
  return { default: module.AnalyticsTrendChart };
});

type Props = {
  report: AnalyticsReport | undefined;
  displayedPeriod: "week" | "month";
  isStaleResult: boolean;
  timeZoneUnavailable: boolean;
  collapsed: boolean;
  isLoading: boolean;
  isRefreshing: boolean;
  errorMessage: string | null;
  onRetry: () => void;
  onToggleCollapsed: () => void;
  onOpenAnalytics: (bucket?: { startAt: string; endAt: string }) => void;
  onUseUtcFallback: () => void;
};

export function DashboardAnalyticsPanel({
  report,
  displayedPeriod,
  isStaleResult,
  timeZoneUnavailable,
  collapsed,
  isLoading,
  isRefreshing,
  errorMessage,
  onRetry,
  onToggleCollapsed,
  onOpenAnalytics,
  onUseUtcFallback,
}: Props) {
  const { t, locale } = useI18n();
  const [showPreviousPeriodHint, setShowPreviousPeriodHint] = useState(false);
  useEffect(() => {
    setShowPreviousPeriodHint(false);
    if (!isStaleResult) return;

    const timeout = window.setTimeout(() => setShowPreviousPeriodHint(true), 250);
    return () => window.clearTimeout(timeout);
  }, [isStaleResult]);
  const formatTokenValue = (value: number | null) =>
    value === null ? "—" : new Intl.NumberFormat(locale, { maximumFractionDigits: 0 }).format(value);
  const formatUsdValue = (value: number | null) =>
    value === null
      ? "—"
      : new Intl.NumberFormat(locale, { style: "currency", currency: "USD", maximumFractionDigits: 2 }).format(value);
  const formatPointsValue = (value: number | null) =>
    value === null
      ? "—"
      : new Intl.NumberFormat(locale, { maximumFractionDigits: 2 }).format(value);

  return (
    <section className="analytics-panel">
      <div className="analytics-panel-header">
        <div>
          <h3>{t("analytics.dashboard.title")}</h3>
          <p>{isRefreshing ? t("analytics.dashboard.refreshing") : t("analytics.dashboard.subtitle")}</p>
        </div>
        <div className="analytics-panel-actions">
          <button type="button" className="ghost-button" onClick={() => onOpenAnalytics()}>
            {t("analytics.actions.viewFull")}
          </button>
          <button type="button" className="ghost-button" onClick={onToggleCollapsed}>
            {collapsed ? t("analytics.actions.expand") : t("analytics.actions.collapse")}
          </button>
        </div>
      </div>

      {!collapsed ? (
        <>
          {errorMessage ? (
            <div className="analytics-error-banner" role="alert">
              <span>{errorMessage}</span>
              <button type="button" className="ghost-button" onClick={onRetry}>
                {t("analytics.actions.retry")}
              </button>
            </div>
          ) : null}
          {timeZoneUnavailable ? (
            <div className="analytics-empty-state">
              <p>{t("analytics.timeZone.unavailable")}</p>
              <button type="button" className="ghost-button" onClick={onUseUtcFallback}>
                {t("analytics.timeZone.useUtc")}
              </button>
            </div>
          ) : null}
          {isLoading && !report ? (
            <div className="analytics-empty-state" role="status">{t("analytics.actions.loading")}</div>
          ) : report ? (
            <>
              <dl className="analytics-summary-strip analytics-summary-strip--compact">
                <div>
                  <dt>{t("analytics.summary.tokens")}</dt>
                  <dd>{formatTokenValue(report.summary.totalTokens.knownValue)}</dd>
                </div>
                <div>
                  <dt>{t("analytics.summary.estimatedUsd")}</dt>
                  <dd>{formatUsdValue(report.summary.estimatedUsd.knownValue)}</dd>
                </div>
                <div>
                  <dt>{t("analytics.summary.copilotPoints")}</dt>
                  <dd>{formatPointsValue(report.summary.copilotPoints.knownValue)}</dd>
                </div>
              </dl>
              {report.series.length > 0 ? (
                <Suspense fallback={<div className="analytics-chart-loading" role="status">{t("analytics.actions.loading")}</div>}>
                  <AnalyticsTrendChart
                    title={t("analytics.charts.dashboardTrend")}
                    data={report.series}
                    metric="tokens"
                    tokenField="total"
                    locale={locale}
                    unitLabel={t("analytics.tokenField.total")}
                    onSelectBucket={(startAt, endAt) => onOpenAnalytics({ startAt, endAt })}
                  />
                </Suspense>
              ) : (
                <p className="analytics-empty-state">{t("analytics.empty.noData")}</p>
              )}
              <p className="dashboard-analytics-period">
                {showPreviousPeriodHint ? t("analytics.workspace.previousPeriod") : null}
                {displayedPeriod === "week" ? t("analytics.quickRange.thisWeek") : t("analytics.quickRange.month")}
              </p>
            </>
          ) : null}
        </>
      ) : null}
    </section>
  );
}
