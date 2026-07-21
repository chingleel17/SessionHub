import { StatusBar } from "session-hub";

const noop = () => {};

const hoursFromNow = (h) => new Date(Date.now() + h * 3600 * 1000).toISOString();
const minutesAgo = (m) => new Date(Date.now() - m * 60 * 1000).toISOString();

const bridgeEvent = {
  entry: {
    id: "evt-8842",
    provider: "claude",
    eventType: "PostToolUse",
    timestamp: minutesAgo(1),
    cwd: "H:/Code/DIY/SessionHub",
    sessionId: "0198c2f4-8a31-7c02-b7e9-3d5a12e04f77",
    title: "Edit src/components/StatusBar.tsx",
    error: null,
    status: "targeted",
  },
  receivedAt: new Date(Date.now() - 45 * 1000),
};

const fallbackEvent = {
  entry: {
    id: "evt-8901",
    provider: "antigravity",
    eventType: "SessionEnd",
    timestamp: minutesAgo(3),
    cwd: "H:/Code/DIY/session-hub-docs",
    sessionId: null,
    title: null,
    error: null,
    status: "fallback",
  },
  receivedAt: new Date(Date.now() - 3 * 60 * 1000),
};

const claudeQuota = {
  provider: "claude",
  billingPeriod: "2026-06-15 ~ 2026-07-14",
  inputTokens: 18_400_000,
  outputTokens: 3_150_000,
  cacheCreationTokens: 2_210_000,
  cacheReadTokens: 41_800_000,
  costUsd: 84.62,
  monthlyLimitTokens: 30_000_000,
  monthlyLimitUsd: null,
  resetDay: 15,
  nextResetDate: "2026-07-15",
};

const codexQuota = {
  provider: "codex",
  billingPeriod: "2026-07-01 ~ 2026-07-31",
  inputTokens: 2_940_000,
  outputTokens: 611_000,
  cacheCreationTokens: 0,
  cacheReadTokens: 0,
  costUsd: 12.37,
  monthlyLimitTokens: null,
  monthlyLimitUsd: null,
  resetDay: 1,
  nextResetDate: "2026-08-01",
};

const claudeSnapshot = {
  provider: "claude",
  status: "ok",
  source: "remote_api",
  fetchedAt: minutesAgo(4),
  windows: [
    { windowKey: "5h", label: "5-hour window", utilization: 0.42, resetsAt: hoursFromNow(2.4) },
    { windowKey: "7d", label: "Weekly window", utilization: 0.63, resetsAt: hoursFromNow(96) },
  ],
};

const antigravitySnapshot = {
  provider: "antigravity",
  status: "ok",
  source: "remote_api",
  fetchedAt: minutesAgo(9),
  windows: [
    { windowKey: "gemini-5h", label: "5-hour window", utilization: 0.91, resetsAt: hoursFromNow(1.2), group: "Gemini Models" },
    { windowKey: "gemini-7d", label: "Weekly window", utilization: 0.58, resetsAt: hoursFromNow(52), group: "Gemini Models" },
    { windowKey: "claude-5h", label: "5-hour window", utilization: 0.33, resetsAt: hoursFromNow(3.1), group: "Claude Models" },
  ],
};

export const LiveEventWithQuotas = () => (
  <StatusBar
    lastBridgeEvent={bridgeEvent}
    onOpenEventMonitor={noop}
    activeSessions={3}
    waitingSessions={1}
    idleSessions={5}
    doneSessions={12}
    isLoadingSessions={false}
    providerQuotas={[claudeQuota, codexQuota]}
    quotaSnapshots={[claudeSnapshot, antigravitySnapshot]}
    quotaEnabledProviders={["claude", "codex", "antigravity"]}
  />
);

export const FallbackEventHighUsage = () => (
  <StatusBar
    lastBridgeEvent={fallbackEvent}
    onOpenEventMonitor={noop}
    activeSessions={1}
    waitingSessions={2}
    idleSessions={0}
    doneSessions={4}
    isLoadingSessions={false}
    providerQuotas={[{ ...claudeQuota, inputTokens: 26_800_000, outputTokens: 2_400_000, costUsd: 118.04 }]}
    quotaSnapshots={[antigravitySnapshot]}
    quotaEnabledProviders={["claude", "antigravity"]}
  />
);

export const LoadingNoEvent = () => (
  <StatusBar
    lastBridgeEvent={null}
    onOpenEventMonitor={noop}
    activeSessions={0}
    waitingSessions={0}
    idleSessions={0}
    doneSessions={0}
    isLoadingSessions={true}
    providerQuotas={[]}
    quotaSnapshots={[]}
    quotaEnabledProviders={[]}
  />
);
