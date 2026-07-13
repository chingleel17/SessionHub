import { SessionStatsPanel } from "session-hub";

const claudeStats = {
  outputTokens: 48210,
  inputTokens: 152400,
  interactionCount: 36,
  toolCallCount: 88,
  durationMinutes: 74,
  modelsUsed: ["claude-sonnet-5", "claude-haiku-4.5"],
  reasoningCount: 12,
  toolBreakdown: { Read: 33, Bash: 31, Edit: 24, Grep: 14, Write: 6 },
  modelMetrics: {},
  isLive: false,
};

export const ClaudeSession = () => (
  <SessionStatsPanel stats={claudeStats} provider="claude" />
);

export const LiveSession = () => (
  <SessionStatsPanel
    stats={{
      ...claudeStats,
      outputTokens: 9120,
      inputTokens: 30400,
      interactionCount: 7,
      toolCallCount: 15,
      durationMinutes: 12,
      reasoningCount: 3,
      toolBreakdown: { Read: 8, Edit: 4, Bash: 3 },
      isLive: true,
    }}
    provider="claude"
  />
);

export const CopilotWithModelCost = () => (
  <SessionStatsPanel
    stats={{
      outputTokens: 1284000,
      inputTokens: 3620000,
      interactionCount: 142,
      toolCallCount: 305,
      durationMinutes: 318,
      modelsUsed: ["gpt-5-codex", "claude-sonnet-5"],
      reasoningCount: 0,
      toolBreakdown: { Read: 96, Bash: 84, Edit: 61, Grep: 40, Glob: 24 },
      modelMetrics: {
        "gpt-5-codex": { requestsCount: 96, requestsCost: 12.4, inputTokens: 2410000, outputTokens: 861000 },
        "claude-sonnet-5": { requestsCount: 46, requestsCost: 5.75, inputTokens: 1210000, outputTokens: 423000 },
      },
      isLive: false,
    }}
    provider="copilot"
  />
);

export const MinimalSession = () => (
  <SessionStatsPanel
    stats={{
      outputTokens: 420,
      inputTokens: 0,
      interactionCount: 2,
      toolCallCount: 0,
      durationMinutes: 1,
      modelsUsed: [],
      reasoningCount: 0,
      toolBreakdown: {},
      modelMetrics: {},
      isLive: false,
    }}
    provider="opencode"
  />
);
