import { TrendChart } from "session-hub";

const weekData = [
  { label: "07/06", outputTokens: 41200, inputTokens: 118400, interactionCount: 32, costPoints: 18, sessionCount: 4, missingCount: 0 },
  { label: "07/07", outputTokens: 68400, inputTokens: 201300, interactionCount: 51, costPoints: 27, sessionCount: 6, missingCount: 0 },
  { label: "07/08", outputTokens: 25300, inputTokens: 90200, interactionCount: 19, costPoints: 11, sessionCount: 3, missingCount: 1 },
  { label: "07/09", outputTokens: 90210, inputTokens: 264800, interactionCount: 64, costPoints: 35, sessionCount: 7, missingCount: 0 },
  { label: "07/10", outputTokens: 57800, inputTokens: 172500, interactionCount: 44, costPoints: 22, sessionCount: 5, missingCount: 0 },
  { label: "07/11", outputTokens: 12400, inputTokens: 45100, interactionCount: 9, costPoints: 5, sessionCount: 2, missingCount: 0 },
  { label: "07/12", outputTokens: 73900, inputTokens: 210600, interactionCount: 55, costPoints: 30, sessionCount: 6, missingCount: 0 },
];

const allLines = [
  { key: "outputTokens", label: "Output tokens", colorClass: "trend-chart-series--primary" },
  { key: "interactionCount", label: "Interactions", colorClass: "trend-chart-series--secondary" },
  { key: "costPoints", label: "Cost points", colorClass: "trend-chart-series--accent" },
];

export const WeeklyTrend = () => (
  <TrendChart
    title="Weekly activity"
    data={weekData}
    lines={allLines}
    emptyLabel="No analytics data yet"
    ariaLabel="Weekly activity trend of tokens, interactions and cost points"
  />
);

export const SingleMetric = () => (
  <TrendChart
    title="Output tokens"
    data={weekData}
    lines={[{ key: "outputTokens", label: "Output tokens", colorClass: "trend-chart-series--primary" }]}
    emptyLabel="No analytics data yet"
    ariaLabel="Output tokens over the last week"
  />
);

export const SinglePoint = () => (
  <TrendChart
    title="Today"
    data={[weekData[6]]}
    lines={allLines}
    emptyLabel="No analytics data yet"
    ariaLabel="Activity for a single day"
  />
);

export const Empty = () => (
  <TrendChart
    title="Weekly activity"
    data={[]}
    lines={allLines}
    emptyLabel="No analytics data yet"
    ariaLabel="Weekly activity trend"
  />
);
