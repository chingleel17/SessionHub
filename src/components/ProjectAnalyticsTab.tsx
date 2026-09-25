import { UsageAnalyticsView, type UsageAnalyticsViewProps } from "./UsageAnalyticsView";

type Props = Omit<UsageAnalyticsViewProps, "fixedCwd" | "hideProjectRanking"> & {
  cwd: string;
};

export function ProjectAnalyticsTab({ cwd, ...analyticsProps }: Props) {
  return <UsageAnalyticsView {...analyticsProps} fixedCwd={cwd} hideProjectRanking />;
}
