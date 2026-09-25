import type { AnalyticsRankingEntry } from "../types";
import { useI18n } from "../i18n/I18nProvider";

type Props = {
  title: string;
  entries: AnalyticsRankingEntry[];
  locale: string;
  unitLabel: string;
  onSelect?: (key: string) => void;
};

export function AnalyticsRankingList({ title, entries, locale, unitLabel, onSelect }: Props) {
  const { t } = useI18n();
  const formatter = new Intl.NumberFormat(locale, { maximumFractionDigits: 2 });
  const percentFormatter = new Intl.NumberFormat(locale, {
    style: "percent",
    maximumFractionDigits: 1,
  });

  return (
    <section className="analytics-ranking-section" aria-label={title}>
      <h4>{title}</h4>
      {entries.length === 0 ? (
        <p className="analytics-ranking-empty">—</p>
      ) : (
        <div className="analytics-ranking-list">
          {entries.map((entry) => {
            const row = (
              <>
              <span className="analytics-ranking-row-heading">
                <span className="analytics-ranking-name" title={entry.label}>{entry.label}</span>
                <span className="analytics-ranking-value">
                  {entry.value.knownValue === null ? "—" : formatter.format(entry.value.knownValue)}
                  <span>{unitLabel}</span>
                </span>
              </span>
              <span className="analytics-ranking-track" aria-hidden="true">
                <span
                  className="analytics-ranking-fill"
                  style={{ width: `${Math.max(0, Math.min(1, entry.shareOfKnownValue ?? 0)) * 100}%` }}
                />
              </span>
              <span className="analytics-ranking-row-meta">
                <span>{t("analytics.ranking.sessions", { count: entry.sessionCount })}</span>
                <span>
                  {entry.shareOfKnownValue === null
                    ? "—"
                    : percentFormatter.format(entry.shareOfKnownValue)}
                </span>
                {entry.value.missingFieldEventCount > 0 ? <span>{t("analytics.coverage.partial")}</span> : null}
              </span>
              </>
            );
            return onSelect ? (
              <button
                key={entry.key}
                type="button"
                className="analytics-ranking-row"
                onClick={() => onSelect(entry.key)}
                aria-label={t("analytics.ranking.ariaLabel", {
                  label: entry.label,
                  value: entry.value.knownValue === null ? t("analytics.value.unknown") : formatter.format(entry.value.knownValue),
                  unit: unitLabel,
                })}
              >
                {row}
              </button>
            ) : (
              <div key={entry.key} className="analytics-ranking-row analytics-ranking-row-static">{row}</div>
            );
          })}
        </div>
      )}
    </section>
  );
}
