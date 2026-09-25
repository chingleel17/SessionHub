import { useEffect, type RefObject } from "react";

import { useI18n } from "../i18n/I18nProvider";
import type { AnalyticsSessionDetail, AnalyticsValues } from "../types";
import {
  formatAnalyticsPlatformLabel,
  formatAnalyticsSupplierLabel,
} from "../utils/analyticsProviderLabels";
import { CloseIcon } from "./Icons";

type Props = {
  detail: AnalyticsSessionDetail;
  closeButtonRef: RefObject<HTMLButtonElement | null>;
  onClose: () => void;
  onOpenSession: (provider: string, sessionId: string) => void;
};

function formatValue(value: number | null, digits: number, locale: string): string {
  return value === null ? "—" : new Intl.NumberFormat(locale, { maximumFractionDigits: digits }).format(value);
}

function ValueList({ values }: { values: AnalyticsValues }) {
  const { t, locale } = useI18n();
  return (
    <dl className="analytics-session-drawer-values">
      <div><dt>{t("analytics.sessionDetail.totalTokens")}</dt><dd>{formatValue(values.totalTokens.knownValue, 0, locale)}</dd></div>
      <div><dt>{t("analytics.sessionDetail.inputTokens")}</dt><dd>{formatValue(values.inputTokens.knownValue, 0, locale)}</dd></div>
      <div><dt>{t("analytics.sessionDetail.outputTokens")}</dt><dd>{formatValue(values.outputTokens.knownValue, 0, locale)}</dd></div>
      <div><dt>{t("analytics.sessionDetail.cacheRead")}</dt><dd>{formatValue(values.cacheReadTokens.knownValue, 0, locale)}</dd></div>
      <div><dt>{t("analytics.sessionDetail.cacheWrite")}</dt><dd>{formatValue(values.cacheWriteTokens.knownValue, 0, locale)}</dd></div>
      <div><dt>{t("analytics.sessionDetail.reasoning")}</dt><dd>{formatValue(values.reasoningTokens.knownValue, 0, locale)}</dd></div>
      <div><dt>{t("analytics.sessionDetail.estimatedUsd")}</dt><dd>{formatValue(values.estimatedUsd.knownValue, 6, locale)}</dd></div>
      <div><dt>{t("analytics.sessionDetail.copilotPoints")}</dt><dd>{formatValue(values.copilotPoints.knownValue, 6, locale)}</dd></div>
    </dl>
  );
}

export function AnalyticsSessionDrawer({ detail, closeButtonRef, onClose, onOpenSession }: Props) {
  const { t } = useI18n();

  useEffect(() => {
    closeButtonRef.current?.focus();
  }, [closeButtonRef]);

  return (
    <aside
      className="analytics-session-drawer"
      role="dialog"
      aria-modal="true"
      aria-labelledby="analytics-session-drawer-title"
    >
      <header className="analytics-session-drawer-header">
        <div>
          <h3 id="analytics-session-drawer-title">{detail.sessionId}</h3>
          <p>{formatAnalyticsPlatformLabel(detail.provider)} · {detail.model ?? t("analytics.value.unknown")}</p>
          {detail.modelProviderIds.length > 0 ? (
            <p>{t("analytics.sessionDetail.supplier")}: {detail.modelProviderIds.map(formatAnalyticsSupplierLabel).join(", ")}</p>
          ) : null}
        </div>
        <button
          ref={closeButtonRef}
          type="button"
          className="icon-button"
          aria-label={t("analytics.sessionDetail.close")}
          title={t("analytics.sessionDetail.close")}
          onClick={onClose}
        >
          <CloseIcon size={16} />
        </button>
      </header>
      <div className="analytics-session-drawer-body">
        <section>
          <h4>{t("analytics.sessionDetail.rangeUsage")}</h4>
          <ValueList values={detail.values} />
        </section>
        {detail.sessionSummaryOnly ? (
          <section>
            <h4>{t("analytics.sessionDetail.fullSessionSummary")}</h4>
            <ValueList values={detail.sessionSummaryOnly} />
            <p className="analytics-session-drawer-note">{t("analytics.sessionDetail.fullSessionNote")}</p>
          </section>
        ) : null}
        <p className="analytics-session-drawer-meta">
          {detail.firstEventAt ?? "—"} – {detail.lastEventAt ?? "—"}
        </p>
        <button
          type="button"
          className="ghost-button"
          onClick={() => onOpenSession(detail.provider, detail.sessionId)}
        >
          {t("analytics.sessionDetail.openSession")}
        </button>
      </div>
    </aside>
  );
}
