import { SessionCard } from "session-hub";

const noop = () => {};

const baseSession = {
  id: "0198c2f4-8a31-7c02-b7e9-3d5a12e04f77",
  provider: "claude",
  cwd: "H:/Code/DIY/SessionHub",
  repoRoot: "H:/Code/DIY/SessionHub",
  repoName: "SessionHub",
  gitBranch: "main",
  summary: "Refactor status bar quota chip into SVG ring with color-graded percentage",
  summaryCount: 12,
  createdAt: "2026-07-10T09:12:00Z",
  updatedAt: "2026-07-12T14:30:00Z",
  sessionDir: "C:/Users/dev/.claude/projects/sessionhub",
  parseError: false,
  isArchived: false,
  notes: null,
  tags: ["ui", "quota"],
  hasPlan: true,
  hasEvents: true,
};

const stats = {
  outputTokens: 48210,
  inputTokens: 152400,
  interactionCount: 36,
  toolCallCount: 88,
  durationMinutes: 74,
  modelsUsed: ["claude-sonnet-5"],
  reasoningCount: 12,
  toolBreakdown: { Edit: 24, Bash: 31, Read: 33 },
  modelMetrics: {
    "claude-sonnet-5": { requestsCount: 36, requestsCost: 1.84, inputTokens: 152400, outputTokens: 48210 },
  },
  isLive: false,
};

const handlers = {
  onCopyCommand: noop,
  onEditNotes: noop,
  onEditTags: noop,
  onEditTag: noop,
  onOpenPlan: noop,
  onOpenTodos: noop,
  onArchive: noop,
  onUnarchive: noop,
  onDelete: noop,
  onResumeSession: noop,
  onFocusTerminal: noop,
};

export const ActiveSession = () => (
  <SessionCard
    {...handlers}
    session={baseSession}
    stats={stats}
    statsLoading={false}
    todos={[]}
    todosLoading={false}
    activityStatus={{ sessionId: baseSession.id, status: "active", detail: "tool_call", lastActivityAt: "2026-07-12T14:29:40Z" }}
  />
);

export const IdleWithTags = () => (
  <SessionCard
    {...handlers}
    session={{
      ...baseSession,
      id: "0198c2f4-1b22-7c02-a001-77aa41e09b21",
      provider: "opencode",
      summary: "Investigate flaky session scanning on the OpenCode SQLite backend",
      tags: ["bug", "backend", "sqlite"],
      hasPlan: false,
    }}
    stats={undefined}
    statsLoading={false}
    todos={[]}
    todosLoading={false}
    activityStatus={{ sessionId: "0198c2f4-1b22-7c02-a001-77aa41e09b21", status: "idle" }}
  />
);

export const ArchivedSession = () => (
  <SessionCard
    {...handlers}
    session={{
      ...baseSession,
      id: "0198b1d0-4c19-7c02-b1f4-90ce2277d001",
      provider: "codex",
      summary: "Migrate hook scripts to the Node CLI runner",
      isArchived: true,
      tags: ["hooks"],
      hasPlan: false,
    }}
    stats={undefined}
    statsLoading={false}
    todos={[]}
    todosLoading={false}
  />
);

export const ParseError = () => (
  <SessionCard
    {...handlers}
    session={{
      ...baseSession,
      id: "0198a0ff-0000-7c02-ffff-13579bdf2468",
      provider: "copilot",
      summary: null,
      parseError: true,
      tags: [],
      hasPlan: false,
      hasEvents: false,
    }}
    stats={undefined}
    statsLoading={false}
    todos={[]}
    todosLoading={false}
  />
);
