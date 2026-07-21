import { ProjectStatsBanner } from "session-hub";

const makeSession = (id, provider, summary, branch) => ({
  id,
  provider,
  cwd: "H:/Code/DIY/SessionHub",
  repoRoot: "H:/Code/DIY/SessionHub",
  repoName: "SessionHub",
  gitBranch: branch,
  summary,
  summaryCount: 8,
  createdAt: "2026-07-10T08:00:00Z",
  updatedAt: "2026-07-12T13:20:00Z",
  sessionDir: "C:/Users/dev/.claude/projects/sessionhub",
  parseError: false,
  isArchived: false,
  notes: null,
  tags: [],
  hasPlan: false,
  hasEvents: true,
});

const makeStats = (inputTokens, outputTokens, interactionCount, toolCallCount) => ({
  outputTokens,
  inputTokens,
  interactionCount,
  toolCallCount,
  durationMinutes: 62,
  modelsUsed: ["claude-sonnet-5"],
  reasoningCount: 9,
  toolBreakdown: { Read: 20, Edit: 14, Bash: 12 },
  modelMetrics: {
    "claude-sonnet-5": {
      requestsCount: interactionCount,
      requestsCost: 1.2,
      inputTokens,
      outputTokens,
    },
  },
  isLive: false,
});

const sessions = [
  makeSession("s-01", "claude", "Redesign status bar quota ring", "feat/quota-ring"),
  makeSession("s-02", "claude", "Antigravity provider scanning", "feat/antigravity"),
  makeSession("s-03", "opencode", "Fix flaky SQLite session parser", "fix/sqlite-scan"),
  makeSession("s-04", "codex", "Migrate hook scripts to Node runner", "main"),
];

const fullStats = {
  "s-01": makeStats(152400, 48210, 36, 88),
  "s-02": makeStats(893200, 121500, 64, 143),
  "s-03": makeStats(64100, 18900, 22, 41),
  "s-04": makeStats(210300, 55600, 29, 77),
};

export const AllStatsLoaded = () => (
  <ProjectStatsBanner
    sessions={sessions}
    sessionStats={fullStats}
    sessionStatsLoading={{}}
  />
);

export const PartiallyLoading = () => (
  <ProjectStatsBanner
    sessions={sessions}
    sessionStats={{ "s-01": fullStats["s-01"], "s-03": fullStats["s-03"] }}
    sessionStatsLoading={{ "s-02": true, "s-04": true }}
  />
);

export const SingleSession = () => (
  <ProjectStatsBanner
    sessions={[sessions[0]]}
    sessionStats={{ "s-01": makeStats(48200, 12100, 14, 31) }}
    sessionStatsLoading={{}}
  />
);
