import { lazy, Suspense, useEffect, useRef, useState } from "react";

import { useI18n } from "../i18n/I18nProvider";
import type {
  AnalyticsGroupBy,
  AnalyticsMetric,
  AnalyticsCoverageStatus,
  AnalyticsQuery,
  AnalyticsReport,
  AnalyticsSessionDetail,
  AnalyticsSessionPage,
  AnalyticsTokenField,
  ManualModelPricingInput,
  ModelPricingEntry,
  ProjectGroup,
  QuotaSnapshot,
} from "../types";
import { AnalyticsRankingList } from "./AnalyticsRankingList";
import { AnalyticsSessionDrawer } from "./AnalyticsSessionDrawer";
import { AnalyticsSessionList } from "./AnalyticsSessionList";
import { ChevronLeftIcon } from "./Icons";
import { ModelPricingView } from "./ModelPricingView";
import { QuotaOverview } from "./QuotaOverview";
import { Checkbox } from "./ui/Checkbox";
import { Select } from "./ui/Select";
import {
  formatAnalyticsPlatformLabel,
  formatAnalyticsSupplierLabel,
} from "../utils/analyticsProviderLabels";

const AnalyticsTrendChart = lazy(async () => {
  const module = await import("./AnalyticsTrendChart");
  return { default: module.AnalyticsTrendChart };
});

export type UsageAnalyticsViewProps = {
  query: AnalyticsQuery | null;
  report: AnalyticsReport | undefined;
  sessionPage: AnalyticsSessionPage | undefined;
  pageNumber: number;
  pageSize: number;
  isStaleResult: boolean;
  availableProviders: string[];
  projects: ProjectGroup[];
  models: string[];
  isLoading: boolean;
  isRefreshing: boolean;
  errorMessage: string | null;
  onQueryChange: (query: AnalyticsQuery) => void;
  onRetry: () => void;
  onRefresh: () => void;
  onUseUtcFallback: () => void;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
  onOpenSession: (provider: string, sessionId: string) => void;
  quotaSnapshots: QuotaSnapshot[];
  quotaEnabled: boolean;
  quotaError: string | null;
  onRefreshQuota: (provider?: string) => void;
  onConsumeResetCredit: () => void;
  resetCreditBusy: boolean;
  fixedCwd?: string;
  hideProjectRanking?: boolean;
  pricingEntries: ModelPricingEntry[];
  pricingLoading: boolean;
  pricingSaving: boolean;
  pricingError: string | null;
  onSavePricing: (input: ManualModelPricingInput) => Promise<void>;
  onDeletePricing: (provider: string, model: string) => void;
  onPricingVisibilityChange: (provider: string, model: string, hidden: boolean) => void;
};

type QuickRange = "last7d" | "thisWeek" | "last30d" | "thisMonth";

function dateInTimeZone(now: Date, timeZone: string): Date {
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(now);
  const value = (type: Intl.DateTimeFormatPartTypes) =>
    Number(parts.find((part) => part.type === type)?.value ?? 0);
  return new Date(Date.UTC(value("year"), value("month") - 1, value("day")));
}

function formatDateInTimeZone(timestamp: string, timeZone: string): string | null {
  const date = new Date(timestamp);
  if (!Number.isFinite(date.getTime())) return null;
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(date);
  const value = (type: Intl.DateTimeFormatPartTypes) =>
    parts.find((part) => part.type === type)?.value;
  const year = value("year");
  const month = value("month");
  const day = value("day");
  return year && month && day ? `${year}-${month}-${day}` : null;
}

function formatDate(date: Date): string {
  return `${date.getUTCFullYear()}-${String(date.getUTCMonth() + 1).padStart(2, "0")}-${String(date.getUTCDate()).padStart(2, "0")}`;
}

function quickRange(range: QuickRange, timeZone: string): { startDate: string; endDate: string } {
  const today = dateInTimeZone(new Date(), timeZone);
  const start = new Date(today);
  if (range === "last7d") {
    start.setUTCDate(start.getUTCDate() - 6);
  } else if (range === "last30d") {
    start.setUTCDate(start.getUTCDate() - 29);
  } else if (range === "thisWeek") {
    start.setUTCDate(start.getUTCDate() - ((start.getUTCDay() + 6) % 7));
  } else {
    start.setUTCDate(1);
  }
  return { startDate: formatDate(start), endDate: formatDate(today) };
}

function isValidDate(value: string): boolean {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return false;
  const timestamp = Date.parse(`${value}T00:00:00Z`);
  return Number.isFinite(timestamp) && new Date(timestamp).toISOString().slice(0, 10) === value;
}

function sameStringValues(left: string[], right: string[]): boolean {
  return left.length === right.length && left.every((value) => right.includes(value));
}

function sameAnalyticsQuery(left: AnalyticsQuery, right: AnalyticsQuery): boolean {
  return left.startDate === right.startDate
    && left.endDate === right.endDate
    && left.timeZone === right.timeZone
    && left.groupBy === right.groupBy
    && sameStringValues(left.providers, right.providers)
    && left.cwd === right.cwd
    && left.model === right.model
    && sameStringValues(left.cwds, right.cwds)
    && sameStringValues(left.models, right.models)
    && left.includeArchived === right.includeArchived
    && left.metric === right.metric
    && left.tokenField === right.tokenField
    && left.breakdown === right.breakdown;
}

export function UsageAnalyticsView({
  query,
  report,
  sessionPage,
  pageNumber,
  pageSize,
  isStaleResult,
  availableProviders,
  projects,
  models,
  isLoading,
  isRefreshing,
  errorMessage,
  onQueryChange,
  onRetry,
  onRefresh,
  onUseUtcFallback,
  onPageChange,
  onPageSizeChange,
  onOpenSession,
  quotaSnapshots,
  quotaEnabled,
  quotaError,
  onRefreshQuota,
  onConsumeResetCredit,
  resetCreditBusy,
  fixedCwd,
  hideProjectRanking = false,
  pricingEntries,
  pricingLoading,
  pricingSaving,
  pricingError,
  onSavePricing,
  onDeletePricing,
  onPricingVisibilityChange,
}: UsageAnalyticsViewProps) {
  const { t, locale } = useI18n();
  const [draftStartDate, setDraftStartDate] = useState(query?.startDate ?? "");
  const [draftEndDate, setDraftEndDate] = useState(query?.endDate ?? "");
  const [resultQuery, setResultQuery] = useState(query);
  const [showPreviousQueryHint, setShowPreviousQueryHint] = useState(false);
  const [showPricingPage, setShowPricingPage] = useState(false);
  const [drilldownBaseQuery, setDrilldownBaseQuery] = useState<AnalyticsQuery | null>(null);
  const [selectedSession, setSelectedSession] = useState<AnalyticsSessionDetail | null>(null);
  const detailTriggerRef = useRef<HTMLButtonElement | null>(null);
  const closeButtonRef = useRef<HTMLButtonElement | null>(null);
  const wasDrawerOpen = useRef(false);
  const committedStartDate = query?.startDate;
  const committedEndDate = query?.endDate;
  const isPreviousQueryDisplayed = Boolean(
    query && resultQuery && isStaleResult && !sameAnalyticsQuery(resultQuery, query),
  );

  useEffect(() => {
    setShowPreviousQueryHint(false);
    if (!isPreviousQueryDisplayed) return;

    const timeout = window.setTimeout(() => setShowPreviousQueryHint(true), 250);
    return () => window.clearTimeout(timeout);
  }, [isPreviousQueryDisplayed]);

  useEffect(() => {
    if (!committedStartDate || !committedEndDate) return;
    setDraftStartDate(committedStartDate);
    setDraftEndDate(committedEndDate);
  }, [committedStartDate, committedEndDate]);

  useEffect(() => {
    if (query && report && !isStaleResult) setResultQuery(query);
  }, [isStaleResult, query, report]);

  useEffect(() => {
    if (selectedSession) {
      wasDrawerOpen.current = true;
      const handleKeyDown = (event: KeyboardEvent) => {
        if (event.key === "Escape") setSelectedSession(null);
      };
      window.addEventListener("keydown", handleKeyDown);
      return () => window.removeEventListener("keydown", handleKeyDown);
    }
    if (wasDrawerOpen.current) {
      wasDrawerOpen.current = false;
      window.setTimeout(() => detailTriggerRef.current?.focus(), 0);
    }
  }, [selectedSession]);

  if (showPricingPage) {
    return (
      <section className="analytics-workspace-view pricing-page" aria-label={t("pricing.title")}>
        <header className="pricing-page-header">
          <button type="button" className="ghost-button pricing-back-button" onClick={() => setShowPricingPage(false)}>
            <ChevronLeftIcon />
            {t("pricing.backToAnalytics")}
          </button>
          <div>
            <h3>{t("pricing.title")}</h3>
            <p>{t("pricing.subtitle")}</p>
          </div>
        </header>
        <ModelPricingView
          entries={pricingEntries}
          isLoading={pricingLoading}
          isSaving={pricingSaving}
          errorMessage={pricingError}
          onSave={onSavePricing}
          onDelete={onDeletePricing}
          onVisibilityChange={onPricingVisibilityChange}
        />
      </section>
    );
  }

  if (!query) {
    return (
      <section className="analytics-workspace-view" aria-label={t("analytics.workspace.title")}>
        <div className="analytics-empty-state">
          <p>{t("analytics.timeZone.unavailable")}</p>
          <button type="button" className="ghost-button" onClick={onUseUtcFallback}>
            {t("analytics.timeZone.useUtc")}
          </button>
        </div>
      </section>
    );
  }

  const submitDateRange = (startDate: string, endDate: string) => {
    setDraftStartDate(startDate);
    setDraftEndDate(endDate);
    if (!isValidDate(startDate) || !isValidDate(endDate) || startDate > endDate) return;
    changeQuery({ ...query, startDate, endDate });
  };

  const changeQuery = (nextQuery: AnalyticsQuery) => {
    setDrilldownBaseQuery(null);
    onQueryChange(fixedCwd ? { ...nextQuery, cwd: fixedCwd } : nextQuery);
  };

  const applyDrilldown = (nextQuery: AnalyticsQuery) => {
    setDrilldownBaseQuery((previous) => previous ?? query);
    onQueryChange(nextQuery);
  };

  const applyQuickRange = (range: QuickRange) => {
    const dates = quickRange(range, query.timeZone);
    submitDateRange(dates.startDate, dates.endDate);
  };

  const toggleProvider = (provider: string) => {
    const providers = query.providers.includes(provider)
      ? query.providers.filter((item) => item !== provider)
      : [...query.providers, provider];
    changeQuery({ ...query, providers });
  };

  const dateRangeInvalid =
    !isValidDate(draftStartDate) || !isValidDate(draftEndDate) || draftStartDate > draftEndDate;
  const projectOptions = projects;
  const modelOptions = [...new Set(["unknown", ...models])].sort((left, right) => left.localeCompare(right));
  const formatTokenValue = (value: number | null) =>
    value === null ? "—" : new Intl.NumberFormat(locale, { maximumFractionDigits: 0 }).format(value);
  const formatUsdValue = (value: number | null) =>
    value === null
      ? "—"
      : new Intl.NumberFormat(locale, { style: "currency", currency: "USD", maximumFractionDigits: 2 }).format(value);
  const formatPointsValue = (value: number | null) =>
    value === null
      ? "—"
      : `${new Intl.NumberFormat(locale, { maximumFractionDigits: 2 }).format(value)} ${t("analytics.metrics.costPoints")}`;
  const comparisonLabel = report
    ? report.comparison.status === "comparable"
      ? t("analytics.comparison.change", {
        value: new Intl.NumberFormat(locale, { style: "percent", maximumFractionDigits: 1 })
          .format((report.comparison.percentChange ?? 0) / 100),
      })
      : report.comparison.status === "new_usage"
        ? t("analytics.comparison.newUsage")
        : report.comparison.status === "no_change"
          ? t("analytics.comparison.noChange")
          : t("analytics.comparison.notComparable")
    : null;

  return (
    <section className="analytics-workspace-view" aria-label={t("analytics.workspace.title")}>
      <div className="analytics-filter-row" aria-label={t("analytics.filters.title")}>
        <div className="analytics-filter-row-primary">
          <div className="analytics-quick-ranges" role="group" aria-label={t("analytics.filters.quickRange")}>
            {(["last7d", "thisWeek", "last30d", "thisMonth"] as const).map((range) => (
              <button key={range} type="button" className="ghost-button" onClick={() => applyQuickRange(range)}>
                {t(
                  range === "last7d"
                    ? "analytics.quickRange.last7d"
                    : range === "thisWeek"
                      ? "analytics.quickRange.thisWeek"
                      : range === "last30d"
                        ? "analytics.quickRange.last30d"
                        : "analytics.quickRange.month",
                )}
              </button>
            ))}
          </div>

          <label className="analytics-filter-field">
            <span>{t("analytics.filters.startDate")}</span>
            <input
              type="date"
              value={draftStartDate}
              onChange={(event) => submitDateRange(event.currentTarget.value, draftEndDate)}
            />
          </label>
          <label className="analytics-filter-field">
            <span>{t("analytics.filters.endDate")}</span>
            <input
              type="date"
              value={draftEndDate}
              onChange={(event) => submitDateRange(draftStartDate, event.currentTarget.value)}
            />
          </label>
          <label className="analytics-filter-field">
            <span>{t("analytics.filters.groupBy")}</span>
            <Select
              value={query.groupBy}
              aria-label={t("analytics.filters.groupBy")}
              onChange={(event) => changeQuery({ ...query, groupBy: event.currentTarget.value as AnalyticsGroupBy })}
            >
              <option value="day">{t("analytics.groupBy.day")}</option>
              <option value="week">{t("analytics.groupBy.week")}</option>
              <option value="month">{t("analytics.groupBy.month")}</option>
            </Select>
          </label>
        </div>

        <div className="analytics-filter-row-secondary">
          {fixedCwd ? (
            <div className="analytics-filter-locked">
              <span>{t("analytics.filters.project")}</span>
              <strong title={fixedCwd}>{fixedCwd}</strong>
            </div>
          ) : (
            <label className="analytics-filter-field">
              <span>{t("analytics.filters.project")}</span>
              <Select
                multiple
                value={query.cwds}
                aria-label={t("analytics.filters.project")}
                filterable
                filterPlaceholder={t("analytics.filters.searchProject")}
                filterAriaLabel={t("analytics.filters.searchProject")}
                filterEmptyLabel={t("analytics.filters.noMatches")}
                onChange={(event) => changeQuery({
                  ...query,
                  cwd: null,
                  cwds: Array.from(event.currentTarget.selectedOptions)
                    .map((option) => option.value)
                    .filter(Boolean),
                })}
              >
                <option value="">{t("analytics.filters.allProjects")}</option>
                {projectOptions.map((project) => (
                  <option key={project.key} value={project.pathLabel} title={project.pathLabel}>
                    {project.branchLabel?.trim()
                      ? `${project.title} · ${project.branchLabel.trim()}`
                      : `${project.title} · ${project.pathLabel}`}
                  </option>
                ))}
              </Select>
            </label>
          )}
          <label className="analytics-filter-field">
            <span>{t("analytics.filters.model")}</span>
            <Select
              multiple
              value={query.models}
              aria-label={t("analytics.filters.model")}
              filterable
              filterPlaceholder={t("analytics.filters.searchModel")}
              filterAriaLabel={t("analytics.filters.searchModel")}
              filterEmptyLabel={t("analytics.filters.noMatches")}
              onChange={(event) => changeQuery({
                ...query,
                model: null,
                models: Array.from(event.currentTarget.selectedOptions)
                  .map((option) => option.value)
                  .filter(Boolean),
              })}
            >
              <option value="">{t("analytics.filters.allModels")}</option>
              {modelOptions.map((model) => (
                <option key={model} value={model}>{model}</option>
              ))}
            </Select>
          </label>
          <label className="analytics-filter-field">
            <span>{t("analytics.filters.metric")}</span>
            <Select
              value={query.metric}
              aria-label={t("analytics.filters.metric")}
              onChange={(event) => changeQuery({
                ...query,
                metric: event.currentTarget.value as AnalyticsMetric,
                tokenField: "total",
              })}
            >
              <option value="tokens">{t("analytics.metrics.tokens")}</option>
              <option value="estimated_usd">{t("analytics.metrics.estimatedUsd")}</option>
              <option value="copilot_points">{t("analytics.metrics.costPoints")}</option>
            </Select>
          </label>
          {query.metric === "tokens" ? (
            <label className="analytics-filter-field">
              <span>{t("analytics.filters.tokenField")}</span>
              <Select
                value={query.tokenField}
                aria-label={t("analytics.filters.tokenField")}
                onChange={(event) => changeQuery({ ...query, tokenField: event.currentTarget.value as AnalyticsTokenField })}
              >
                <option value="total">{t("analytics.tokenField.total")}</option>
                <option value="input">{t("analytics.tokenField.input")}</option>
                <option value="output">{t("analytics.tokenField.output")}</option>
              </Select>
            </label>
          ) : null}
          <div className="analytics-filter-secondary-actions">
            <button type="button" className="ghost-button" onClick={() => setShowPricingPage(true)}>
              {t("pricing.open")}
            </button>
            <button type="button" className="ghost-button analytics-refresh-button" onClick={onRefresh}>
              {t("analytics.actions.refresh")}
            </button>
          </div>
        </div>
      </div>
      {dateRangeInvalid ? <p className="analytics-filter-hint">{t("analytics.filters.invalidDateRange")}</p> : null}
      {drilldownBaseQuery ? (
        <button type="button" className="ghost-button analytics-clear-drilldown" onClick={() => {
          changeQuery(drilldownBaseQuery);
          setDrilldownBaseQuery(null);
        }}>
          {t("analytics.actions.clearDrilldown")}
        </button>
      ) : null}
      <fieldset className="analytics-provider-filter">
        <legend>{t("analytics.filters.providers")}</legend>
        {availableProviders.map((provider) => (
          <Checkbox
            key={provider}
            className="analytics-provider-checkbox"
              checked={query.providers.includes(provider)}
              onChange={() => toggleProvider(provider)}
          >
            {formatAnalyticsPlatformLabel(provider)}
          </Checkbox>
        ))}
        <Checkbox
          className="analytics-archive-filter analytics-archive-filter--provider"
          checked={query.includeArchived}
          onChange={(event) => changeQuery({ ...query, includeArchived: event.currentTarget.checked })}
        >
          {t("analytics.filters.includeArchived")}
        </Checkbox>
      </fieldset>
      {errorMessage ? (
        <div className="analytics-error-banner" role="alert">
          <span>{errorMessage}</span>
          <button type="button" className="ghost-button" onClick={onRetry}>
            {t("analytics.actions.retry")}
          </button>
        </div>
      ) : null}
      {isPreviousQueryDisplayed && showPreviousQueryHint && resultQuery ? (
        <p className="analytics-stale-hint" role="status" aria-live="polite">
          {t("analytics.workspace.previousRange", {
            startDate: resultQuery.startDate,
            endDate: resultQuery.endDate,
          })}
        </p>
      ) : null}
      {sessionPage?.status === "stale_revision" ? (
        <p className="analytics-filter-hint" role="status">{t("analytics.workspace.staleRevision")}</p>
      ) : null}
      {isLoading && !report ? (
        <p className="analytics-empty-state" role="status">{t("analytics.actions.loading")}</p>
      ) : null}
      {report ? (
        <>
          <dl className="analytics-summary-strip">
            <div>
              <dt>{t("analytics.summary.tokens")}</dt>
              <dd>{formatTokenValue(report.summary.totalTokens.knownValue)}</dd>
              <small>{t("analytics.coverage.eligibleEvents", {
                count: report.summary.totalTokens.eligibleEventCount,
              })}</small>
            </div>
            <div>
              <dt>{t("analytics.summary.estimatedUsd")}</dt>
              <dd>{formatUsdValue(report.summary.estimatedUsd.knownValue)}</dd>
              <small>{report.summary.estimatedUsd.priceVersions.length > 1
                ? t("analytics.summary.mixedPriceVersions")
                : t("analytics.coverage.eligibleEvents", {
                  count: report.summary.estimatedUsd.eligibleEventCount,
                })}</small>
            </div>
            <div>
              <dt>{t("analytics.summary.copilotPoints")}</dt>
              <dd>{formatPointsValue(report.summary.copilotPoints.knownValue)}</dd>
              <small>{t("analytics.coverage.eligibleEvents", {
                count: report.summary.copilotPoints.eligibleEventCount,
              })}</small>
            </div>
          </dl>
          <p className={`analytics-comparison analytics-comparison--${report.comparison.status}`}>
            {comparisonLabel}
          </p>
          <div className="analytics-workspace-status">
            <span>{isRefreshing ? t("analytics.dashboard.refreshing") : report.timeZone}</span>
            <span>{t("analytics.workspace.revision", { revision: report.revision })}</span>
            <button type="button" className="ghost-button" onClick={onRefresh}>
              {t("analytics.actions.refresh")}
            </button>
            <span className="analytics-workspace-session-count" aria-live="polite">
              {t("analytics.workspace.sessions", { count: sessionPage?.totalCount ?? 0 })}
            </span>
          </div>
          {report.series.length === 0 ? (
            <div className="analytics-empty-state">{t("analytics.empty.noData")}</div>
          ) : (
            <Suspense fallback={<p className="analytics-empty-state" role="status">{t("analytics.actions.loading")}</p>}>
              <AnalyticsTrendChart
                title={t("analytics.charts.analyticsTrend")}
                data={report.series}
                metric={query.metric}
                tokenField={query.tokenField}
                locale={locale}
                onSelectBucket={(startAt, endAt) => {
                  const startDate = formatDateInTimeZone(startAt, query.timeZone);
                  const endDate = formatDateInTimeZone(
                    new Date(new Date(endAt).getTime() - 1).toISOString(),
                    query.timeZone,
                  );
                  if (!startDate || !endDate) return;
                  applyDrilldown({ ...query, startDate, endDate });
                }}
                unitLabel={query.metric === "tokens"
                  ? t(query.tokenField === "total"
                    ? "analytics.tokenField.total"
                    : query.tokenField === "input"
                      ? "analytics.tokenField.input"
                      : "analytics.tokenField.output")
                  : query.metric === "estimated_usd"
                    ? t("analytics.metrics.estimatedUsd")
                    : t("analytics.metrics.costPoints")}
              />
            </Suspense>
          )}
          <div className="analytics-ranking-grid">
            {!hideProjectRanking ? <AnalyticsRankingList
              title={t("analytics.ranking.projects")}
              entries={report.projectRanking}
              locale={locale}
              unitLabel={query.metric === "tokens"
                ? t(query.tokenField === "total"
                  ? "analytics.tokenField.total"
                  : query.tokenField === "input"
                    ? "analytics.tokenField.input"
                    : "analytics.tokenField.output")
                : query.metric === "estimated_usd"
                  ? t("analytics.metrics.estimatedUsd")
                  : t("analytics.metrics.costPoints")}
              onSelect={(cwd) => applyDrilldown({ ...query, cwd: null, cwds: [cwd] })}
            /> : null}
            <AnalyticsRankingList
              title={t("analytics.ranking.models")}
              entries={report.modelRanking}
              locale={locale}
              unitLabel={query.metric === "tokens"
                ? t(query.tokenField === "total"
                  ? "analytics.tokenField.total"
                  : query.tokenField === "input"
                    ? "analytics.tokenField.input"
                    : "analytics.tokenField.output")
                : query.metric === "estimated_usd"
                  ? t("analytics.metrics.estimatedUsd")
                  : t("analytics.metrics.costPoints")}
              onSelect={(model) => applyDrilldown({ ...query, model: null, models: [model] })}
            />
            <AnalyticsRankingList
              title={t("analytics.ranking.platforms")}
              entries={report.platformRanking.map((entry) => ({
                ...entry,
                label: formatAnalyticsPlatformLabel(entry.key),
              }))}
              locale={locale}
              unitLabel={query.metric === "tokens"
                ? t(query.tokenField === "total"
                  ? "analytics.tokenField.total"
                  : query.tokenField === "input"
                    ? "analytics.tokenField.input"
                    : "analytics.tokenField.output")
                : query.metric === "estimated_usd"
                  ? t("analytics.metrics.estimatedUsd")
                  : t("analytics.metrics.costPoints")}
              onSelect={(platform) => changeQuery({ ...query, providers: [platform] })}
            />
            <AnalyticsRankingList
              title={t("analytics.ranking.suppliers")}
              entries={report.supplierRanking.map((entry) => ({
                ...entry,
                label: formatAnalyticsSupplierLabel(entry.key),
              }))}
              locale={locale}
              unitLabel={query.metric === "tokens"
                ? t(query.tokenField === "total"
                  ? "analytics.tokenField.total"
                  : query.tokenField === "input"
                    ? "analytics.tokenField.input"
                    : "analytics.tokenField.output")
                : query.metric === "estimated_usd"
                  ? t("analytics.metrics.estimatedUsd")
                  : t("analytics.metrics.costPoints")}
            />
          </div>
          <AnalyticsSessionList
            page={sessionPage}
            pageNumber={pageNumber}
            pageSize={pageSize}
            metric={query.metric}
            tokenField={query.tokenField}
            onPageChange={onPageChange}
            onPageSizeChange={onPageSizeChange}
            onSelect={(detail, trigger) => {
              detailTriggerRef.current = trigger;
              setSelectedSession(detail);
            }}
          />
        </>
      ) : null}
      {!isPreviousQueryDisplayed && report?.coverage.providers.some((provider) => provider.status !== "complete") ? (
        <ul className="analytics-coverage-list" aria-label={t("analytics.coverage.title")}>
          {report.coverage.providers
            .filter((provider) => provider.status !== "complete")
            .map((provider) => (
              <li key={provider.provider}>
                <span>{provider.provider}</span>
                <span>{coverageStatusLabel(provider.status, t)}</span>
                {provider.periodUnknownSessionCount > 0 ? (
                  <span>{t("analytics.coverage.periodUnknown", { count: provider.periodUnknownSessionCount })}</span>
                ) : null}
              </li>
            ))}
        </ul>
      ) : null}
      <details className="analytics-quota-section" open>
        <summary>{t("analytics.quota.title")}</summary>
        <p>{t("analytics.quota.subtitle")}</p>
        {quotaError ? <p className="analytics-filter-hint" role="alert">{quotaError}</p> : null}
        {quotaEnabled ? (
          <QuotaOverview
            snapshots={quotaSnapshots}
            onRefresh={() => onRefreshQuota()}
            onRefreshProvider={onRefreshQuota}
            onConsumeResetCredit={onConsumeResetCredit}
            resetBusy={resetCreditBusy}
            storageKey="quota-overview-analytics-provider"
          />
        ) : (
          <p className="analytics-session-empty">{t("analytics.quota.disabled")}</p>
        )}
      </details>
      {selectedSession ? (
        <AnalyticsSessionDrawer
          detail={selectedSession}
          closeButtonRef={closeButtonRef}
          onClose={() => setSelectedSession(null)}
          onOpenSession={(provider, sessionId) => {
            setSelectedSession(null);
            onOpenSession(provider, sessionId);
          }}
        />
      ) : null}
    </section>
  );
}

function coverageStatusLabel(
  status: AnalyticsCoverageStatus,
  t: ReturnType<typeof useI18n>["t"],
): string {
  switch (status) {
    case "complete": return t("analytics.coverage.complete");
    case "partial": return t("analytics.coverage.partial");
    case "pending": return t("analytics.coverage.pending");
    case "unsupported": return t("analytics.coverage.unsupported");
    case "error": return t("analytics.coverage.error");
    case "summary_only": return t("analytics.coverage.summaryOnly");
  }
}
