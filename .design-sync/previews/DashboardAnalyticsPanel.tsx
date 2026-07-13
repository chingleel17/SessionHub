import { DashboardAnalyticsPanel } from "session-hub";

const noop = () => {};

const data = [
  { label: "07/06", outputTokens: 41200, inputTokens: 118400, interactionCount: 32, costPoints: 18, sessionCount: 4, missingCount: 0 },
  { label: "07/07", outputTokens: 68400, inputTokens: 201300, interactionCount: 51, costPoints: 27, sessionCount: 6, missingCount: 0 },
  { label: "07/08", outputTokens: 25300, inputTokens: 90200, interactionCount: 19, costPoints: 11, sessionCount: 3, missingCount: 1 },
  { label: "07/09", outputTokens: 90210, inputTokens: 264800, interactionCount: 64, costPoints: 35, sessionCount: 7, missingCount: 0 },
  { label: "07/10", outputTokens: 57800, inputTokens: 172500, interactionCount: 44, costPoints: 22, sessionCount: 5, missingCount: 0 },
  { label: "07/11", outputTokens: 12400, inputTokens: 45100, interactionCount: 9, costPoints: 5, sessionCount: 2, missingCount: 0 },
  { label: "07/12", outputTokens: 73900, inputTokens: 210600, interactionCount: 55, costPoints: 30, sessionCount: 6, missingCount: 0 },
];

const projectSlices = [
  { label: "SessionHub", value: 482100, color: "#6366f1" },
  { label: "quota-hooks", value: 213400, color: "#14b8a6" },
  { label: "infra-scripts", value: 98100, color: "#f59e0b" },
  { label: "docs-site", value: 45200, color: "#ef4444" },
];

export const Expanded = () => (
  <DashboardAnalyticsPanel
    data={data}
    projectSlices={projectSlices}
    collapsed={false}
    isLoading={false}
    isRefreshing={false}
    errorMessage={null}
    onRetry={noop}
    onToggleCollapsed={noop}
  />
);

export const Collapsed = () => (
  <DashboardAnalyticsPanel
    data={data}
    projectSlices={projectSlices}
    collapsed={true}
    isLoading={false}
    isRefreshing={false}
    errorMessage={null}
    onRetry={noop}
    onToggleCollapsed={noop}
  />
);

export const Loading = () => (
  <DashboardAnalyticsPanel
    data={[]}
    projectSlices={[]}
    collapsed={false}
    isLoading={true}
    isRefreshing={false}
    errorMessage={null}
    onRetry={noop}
    onToggleCollapsed={noop}
  />
);

export const WithError = () => (
  <DashboardAnalyticsPanel
    data={data}
    projectSlices={projectSlices}
    collapsed={false}
    isLoading={false}
    isRefreshing={true}
    errorMessage="Failed to refresh analytics: session index is locked by another process"
    onRetry={noop}
    onToggleCollapsed={noop}
  />
);
