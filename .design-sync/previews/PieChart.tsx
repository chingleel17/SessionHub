import { PieChart } from "session-hub";

const projectSlices = [
  { label: "SessionHub", value: 482100, color: "#6366f1" },
  { label: "quota-hooks", value: 213400, color: "#14b8a6" },
  { label: "infra-scripts", value: 98100, color: "#f59e0b" },
  { label: "docs-site", value: 45200, color: "#ef4444" },
];

export const ProjectDistribution = () => (
  <PieChart
    title="Project distribution"
    slices={projectSlices}
    emptyLabel="No analytics data yet"
    ariaLabel="Output token distribution across projects"
  />
);

export const TwoSlices = () => (
  <PieChart
    title="Provider share"
    slices={[
      { label: "claude", value: 61, color: "#8b5cf6" },
      { label: "copilot", value: 39, color: "#06b6d4" },
    ]}
    emptyLabel="No analytics data yet"
    ariaLabel="Session share by provider"
  />
);

export const SingleSlice = () => (
  <PieChart
    title="Model usage"
    slices={[{ label: "claude-sonnet-5", value: 128, color: "#6366f1" }]}
    emptyLabel="No analytics data yet"
    ariaLabel="Interactions by model"
  />
);

export const Empty = () => (
  <PieChart
    title="Project distribution"
    slices={[]}
    emptyLabel="No analytics data yet"
    ariaLabel="Output token distribution across projects"
  />
);
