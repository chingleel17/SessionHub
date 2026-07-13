import { SessionStatsBadge } from "session-hub";

const noop = () => {};

const session = {
  id: "0198c2f4-8a31-7c02-b7e9-3d5a12e04f77",
  provider: "claude",
  cwd: "H:/Code/DIY/SessionHub",
  repoRoot: "H:/Code/DIY/SessionHub",
  repoName: "SessionHub",
  gitBranch: "main",
  summary: "Refactor status bar quota chip into SVG ring",
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
  toolBreakdown: { Read: 33, Bash: 31, Edit: 24 },
  modelMetrics: {},
  isLive: false,
};

const todos = [
  { id: "todo-1", title: "Extract quota ring into shared component", status: "done", updatedAt: "2026-07-12T13:10:00Z" },
  { id: "todo-2", title: "Wire percentage color thresholds", status: "in_progress", updatedAt: "2026-07-12T14:02:00Z" },
  { id: "todo-3", title: "Add antigravity provider color tokens", status: "pending", updatedAt: null },
  { id: "todo-4", title: "Update locale keys for quota window labels", status: "pending", updatedAt: null },
];

export const StatsOnly = () => (
  <SessionStatsBadge
    session={session}
    stats={stats}
    isLoading={false}
    todos={[]}
    todosLoading={false}
    onOpenTodos={noop}
  />
);

export const LiveWithTodos = () => (
  <SessionStatsBadge
    session={session}
    stats={{ ...stats, interactionCount: 9, outputTokens: 8120, inputTokens: 21400, durationMinutes: 145, isLive: true }}
    isLoading={false}
    todos={todos}
    todosLoading={false}
    onOpenTodos={noop}
  />
);

export const TodosOnly = () => (
  <SessionStatsBadge
    session={session}
    stats={undefined}
    isLoading={false}
    todos={[
      ...todos,
      { id: "todo-5", title: "Verify hook script on Windows", status: "blocked", updatedAt: "2026-07-11T18:44:00Z" },
    ]}
    todosLoading={false}
    onOpenTodos={noop}
  />
);

export const NoData = () => (
  <SessionStatsBadge
    session={session}
    stats={undefined}
    isLoading={false}
    todos={[]}
    todosLoading={false}
    onOpenTodos={noop}
  />
);

export const Loading = () => (
  <SessionStatsBadge
    session={session}
    stats={undefined}
    isLoading={true}
    todos={[]}
    todosLoading={false}
    onOpenTodos={noop}
  />
);
