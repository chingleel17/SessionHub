import type {
  AnalyticsMetric,
  AnalyticsSessionDetail,
  AnalyticsSessionPage,
  AnalyticsTokenField,
} from "../types";
import { useI18n } from "../i18n/I18nProvider";
import {
  formatAnalyticsPlatformLabel,
  formatAnalyticsSupplierLabel,
} from "../utils/analyticsProviderLabels";
import { Select } from "./ui/Select";

type Props = {
  page: AnalyticsSessionPage | undefined;
  pageNumber: number;
  pageSize: number;
  metric: AnalyticsMetric;
  tokenField: AnalyticsTokenField;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
  onSelect: (detail: AnalyticsSessionDetail, trigger: HTMLButtonElement) => void;
};

function selectedValue(detail: AnalyticsSessionDetail, metric: AnalyticsMetric, tokenField: AnalyticsTokenField): number | null {
  if (metric === "estimated_usd") return detail.values.estimatedUsd.knownValue;
  if (metric === "copilot_points") return detail.values.copilotPoints.knownValue;
  if (tokenField === "input") return detail.values.inputTokens.knownValue;
  if (tokenField === "output") return detail.values.outputTokens.knownValue;
  return detail.values.totalTokens.knownValue;
}

function statusKey(status: AnalyticsSessionDetail["status"]):
  | "analytics.coverage.complete"
  | "analytics.coverage.partial"
  | "analytics.coverage.pending"
  | "analytics.coverage.unsupported"
  | "analytics.coverage.error"
  | "analytics.coverage.summaryOnly" {
  switch (status) {
    case "complete": return "analytics.coverage.complete";
    case "partial": return "analytics.coverage.partial";
    case "pending": return "analytics.coverage.pending";
    case "unsupported": return "analytics.coverage.unsupported";
    case "error": return "analytics.coverage.error";
    case "summary_only": return "analytics.coverage.summaryOnly";
  }
}

export function AnalyticsSessionList({
  page,
  pageNumber,
  pageSize,
  metric,
  tokenField,
  onPageChange,
  onPageSizeChange,
  onSelect,
}: Props) {
  const { t, locale } = useI18n();
  const totalPages = Math.max(1, Math.ceil((page?.totalCount ?? 0) / pageSize));

  if (!page || page.items.length === 0) {
    return <p className="analytics-session-empty">{t("analytics.sessionDetail.noSessions")}</p>;
  }

  return (
    <section className="analytics-session-list" aria-labelledby="analytics-session-list-title">
          <div className="analytics-session-list-heading">
            <h4 id="analytics-session-list-title">{t("analytics.sessionDetail.sessions")}</h4>
            <label className="analytics-session-page-size">
              <span>{t("analytics.sessionDetail.pageSize")}</span>
              <Select
                aria-label={t("analytics.sessionDetail.pageSize")}
                value={String(pageSize)}
                onChange={(event) => onPageSizeChange(Number(event.currentTarget.value))}
              >
                {[10, 20, 50, 100].map((size) => (
                  <option key={size} value={size}>{size}</option>
                ))}
              </Select>
            </label>
          </div>
      <div className="analytics-session-table-wrap">
        <table className="analytics-session-table">
          <thead>
            <tr>
              <th scope="col">{t("analytics.sessionDetail.session")}</th>
              <th scope="col">{t("analytics.sessionDetail.platform")}</th>
              <th scope="col">{t("analytics.sessionDetail.supplier")}</th>
              <th scope="col">{t("analytics.sessionDetail.model")}</th>
              <th scope="col">{t("analytics.sessionDetail.usage")}</th>
              <th scope="col">{t("analytics.sessionDetail.lastEvent")}</th>
              <th scope="col">{t("analytics.sessionDetail.status")}</th>
            </tr>
          </thead>
          <tbody>
            {page.items.map((detail) => {
              const value = selectedValue(detail, metric, tokenField);
              const formattedValue = value === null
                ? "—"
                : new Intl.NumberFormat(locale, {
                  maximumFractionDigits: metric === "tokens" ? 0 : 6,
                }).format(value);
              return (
                <tr key={`${detail.provider}:${detail.sessionId}`}>
                  <td>
                    <button
                      type="button"
                      className="analytics-session-open-button"
                      onClick={(event) => onSelect(detail, event.currentTarget)}
                    >
                      {detail.sessionId}
                    </button>
                    {detail.cwd ? <small>{detail.cwd}</small> : null}
                  </td>
                  <td>{formatAnalyticsPlatformLabel(detail.provider)}</td>
                  <td>{detail.modelProviderIds.length > 0
                    ? detail.modelProviderIds.map(formatAnalyticsSupplierLabel).join(", ")
                    : t("analytics.value.unknown")}</td>
                  <td>{detail.model ?? t("analytics.value.unknown")}</td>
                  <td className="analytics-session-value">{formattedValue}</td>
                  <td>{detail.lastEventAt && Number.isFinite(new Date(detail.lastEventAt).getTime())
                    ? new Intl.DateTimeFormat(locale, { dateStyle: "medium", timeStyle: "short" }).format(new Date(detail.lastEventAt))
                    : "—"}</td>
                  <td>{t(statusKey(detail.status))}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      <div className="analytics-session-pagination">
        <span>{t("analytics.sessionDetail.pageOf", { page: pageNumber, totalPages })}</span>
        <div>
          <button type="button" className="ghost-button" disabled={pageNumber <= 1} onClick={() => onPageChange(pageNumber - 1)}>
            {t("analytics.sessionDetail.previousPage")}
          </button>
          <button type="button" className="ghost-button" disabled={pageNumber >= totalPages} onClick={() => onPageChange(pageNumber + 1)}>
            {t("analytics.sessionDetail.nextPage")}
          </button>
        </div>
      </div>
    </section>
  );
}
