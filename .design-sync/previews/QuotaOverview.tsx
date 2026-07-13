import { QuotaOverview } from "session-hub";

const noop = () => {};

const hoursFromNow = (h) => new Date(Date.now() + h * 3600 * 1000).toISOString();
const minutesAgo = (m) => new Date(Date.now() - m * 60 * 1000).toISOString();

const claudeOk = {
  provider: "claude",
  status: "ok",
  source: "remote_api",
  fetchedAt: minutesAgo(3),
  windows: [
    { windowKey: "5h", label: "5-hour window", utilization: 0.47, resetsAt: hoursFromNow(2.6) },
    { windowKey: "7d", label: "Weekly window", utilization: 0.72, resetsAt: hoursFromNow(88) },
    { windowKey: "7d_opus", label: "Weekly (Opus)", utilization: 0.31, resetsAt: hoursFromNow(88) },
  ],
  extraCredits: { isEnabled: true, monthlyLimit: 5000, usedCredits: 12.4, utilization: 0.25 },
};

const antigravityOk = {
  provider: "antigravity",
  status: "ok",
  source: "remote_api",
  fetchedAt: minutesAgo(11),
  windows: [
    { windowKey: "gemini-5h", label: "5-hour window", utilization: 0.93, resetsAt: hoursFromNow(1.1), group: "Gemini Models" },
    { windowKey: "gemini-7d", label: "Weekly window", utilization: 0.61, resetsAt: hoursFromNow(50), group: "Gemini Models" },
    { windowKey: "claude-5h", label: "5-hour window", utilization: 0.28, resetsAt: hoursFromNow(3.4), group: "Claude Models" },
    { windowKey: "gpt-5h", label: "5-hour window", utilization: 0.12, resetsAt: hoursFromNow(4.2), group: "GPT Models" },
  ],
};

const codexLocal = {
  provider: "codex",
  status: "ok",
  source: "local_scan",
  fetchedAt: minutesAgo(25),
  windows: [],
  localTokens: {
    inputTokens: 2_940_000,
    outputTokens: 611_000,
    periodLabel: "2026-07-01 ~ 2026-07-31",
  },
};

export const MultiProvider = () => (
  <QuotaOverview
    snapshots={[claudeOk, antigravityOk, codexLocal]}
    onRefresh={noop}
    onRefreshProvider={noop}
  />
);

export const AntigravityGroups = () => (
  <QuotaOverview snapshots={[antigravityOk]} onRefresh={noop} onRefreshProvider={noop} />
);

const copilotNoAuth = {
  provider: "copilot",
  status: "no_auth",
  source: "remote_api",
  fetchedAt: minutesAgo(2),
  windows: [],
};

const opencodeError = {
  provider: "opencode",
  status: "error",
  source: "remote_api",
  fetchedAt: minutesAgo(6),
  errorMessage: "GET /v1/usage returned 503 Service Unavailable (upstream quota service timed out after 10s)",
  windows: [],
};

export const AuthAndErrorStates = () => (
  <QuotaOverview
    snapshots={[copilotNoAuth, opencodeError]}
    onRefresh={noop}
    onRefreshProvider={noop}
  />
);
