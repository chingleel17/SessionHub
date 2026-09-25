import { CartesianGrid, Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";

import type {
  AnalyticsMetric,
  AnalyticsSeriesPoint,
  AnalyticsTokenField,
} from "../types";
import { useI18n } from "../i18n/I18nProvider";

type Props = {
  title: string;
  data: AnalyticsSeriesPoint[];
  metric: AnalyticsMetric;
  tokenField: AnalyticsTokenField;
  locale: string;
  unitLabel: string;
  onSelectBucket: (startAt: string, endAt: string) => void;
};

type ChartDataPoint = {
  label: string;
  value: number | null;
  inputTokens: number | null;
  outputTokens: number | null;
  cacheReadTokens: number | null;
  cacheWriteTokens: number | null;
  reasoningTokens: number | null;
  startAt: string;
  endAt: string;
};

function selectValue(point: AnalyticsSeriesPoint, metric: AnalyticsMetric, tokenField: AnalyticsTokenField): number | null {
  if (metric === "estimated_usd") return point.values.estimatedUsd.knownValue;
  if (metric === "copilot_points") return point.values.copilotPoints.knownValue;
  if (tokenField === "input") return point.values.inputTokens.knownValue;
  if (tokenField === "output") return point.values.outputTokens.knownValue;
  return point.values.totalTokens.knownValue;
}

function formatValue(value: number, metric: AnalyticsMetric, locale: string): string {
  const maximumFractionDigits = metric === "tokens" ? 0 : 6;
  return new Intl.NumberFormat(locale, { maximumFractionDigits }).format(value);
}

export function AnalyticsTrendChart({
  title,
  data,
  metric,
  tokenField,
  locale,
  unitLabel,
  onSelectBucket,
}: Props) {
  const { t } = useI18n();
  const chartData: ChartDataPoint[] = data.map((point) => ({
    label: point.label,
    value: selectValue(point, metric, tokenField),
    inputTokens: point.values.inputTokens.knownValue,
    outputTokens: point.values.outputTokens.knownValue,
    cacheReadTokens: point.values.cacheReadTokens.knownValue,
    cacheWriteTokens: point.values.cacheWriteTokens.knownValue,
    reasoningTokens: point.values.reasoningTokens.knownValue,
    startAt: point.startAt,
    endAt: point.endAt,
  }));
  const textColor = "var(--color-text-secondary)";

  return (
    <div className="analytics-trend-chart" role="group" aria-label={title}>
      <div className="analytics-trend-chart-plot">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart
            data={chartData}
            title={title}
            accessibilityLayer
            margin={{ top: 12, right: 18, bottom: 2, left: 2 }}
            onClick={(state) => {
              const label = state?.activeLabel;
              const point = chartData.find((item) => item.label === String(label));
              if (point) onSelectBucket(point.startAt, point.endAt);
            }}
          >
            <CartesianGrid stroke="var(--color-border-subtle)" strokeDasharray="3 5" vertical={false} />
            <XAxis
              dataKey="label"
              tick={{ fill: textColor, fontSize: 11 }}
              axisLine={{ stroke: "var(--color-border-subtle)" }}
              tickLine={false}
              minTickGap={18}
            />
            <YAxis
              width={64}
              tick={{ fill: textColor, fontSize: 11 }}
              axisLine={false}
              tickLine={false}
              tickFormatter={(value: number) => formatValue(value, metric, locale)}
            />
            <Tooltip
              content={({ active, payload, label }) => {
                const chartValue = payload?.[0]?.payload;
                if (!active || !chartValue || typeof chartValue !== "object") return null;
                const point = chartValue as ChartDataPoint;
                const formatNullable = (value: number | null) =>
                  value === null ? "—" : formatValue(value, metric, locale);
                return (
                  <div className="analytics-chart-tooltip">
                    <strong>{String(label ?? point.label)}</strong>
                    <span>{formatNullable(point.value)} {unitLabel}</span>
                    {metric === "tokens" ? (
                      <dl>
                        <div><dt>{t("analytics.tokenField.input")}</dt><dd>{formatNullable(point.inputTokens)}</dd></div>
                        <div><dt>{t("analytics.tokenField.output")}</dt><dd>{formatNullable(point.outputTokens)}</dd></div>
                        <div><dt>{t("analytics.tokenField.cacheRead")}</dt><dd>{formatNullable(point.cacheReadTokens)}</dd></div>
                        <div><dt>{t("analytics.tokenField.cacheWrite")}</dt><dd>{formatNullable(point.cacheWriteTokens)}</dd></div>
                        <div><dt>{t("analytics.tokenField.reasoning")}</dt><dd>{formatNullable(point.reasoningTokens)}</dd></div>
                      </dl>
                    ) : null}
                  </div>
                );
              }}
              contentStyle={{
                border: "1px solid var(--color-border-subtle)",
                borderRadius: "var(--radius-input)",
                background: "var(--color-surface-panel)",
                color: "var(--color-text-primary)",
                boxShadow: "var(--shadow-panel)",
              }}
              labelStyle={{ color: "var(--color-text-secondary)" }}
            />
            <Line
              type="monotone"
              dataKey="value"
              name={unitLabel}
              stroke="var(--color-action-primary)"
              strokeWidth={2}
              dot={{ r: 3, fill: "var(--color-action-primary)" }}
              activeDot={{ r: 5, fill: "var(--color-action-primary)" }}
              connectNulls={false}
              isAnimationActive={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
      <details className="analytics-chart-data-details">
        <summary>{t("analytics.chart.dataTable")}</summary>
        <table className="analytics-chart-data-table">
          <caption>{title}</caption>
          <thead>
            <tr><th scope="col">{t("analytics.chart.date")}</th><th scope="col">{unitLabel}</th></tr>
          </thead>
          <tbody>
            {chartData.map((point) => (
              <tr key={point.label}>
                <th scope="row">
                  <button
                    type="button"
                    className="analytics-chart-point-button"
                    onClick={() => onSelectBucket(point.startAt, point.endAt)}
                  >
                    {point.label}
                  </button>
                </th>
                <td>{point.value === null ? "—" : formatValue(point.value, metric, locale)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </details>
    </div>
  );
}
