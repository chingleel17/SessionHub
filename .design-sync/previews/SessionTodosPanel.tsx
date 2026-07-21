import { SessionTodosPanel } from "session-hub";

const activeTodos = [
  {
    id: "todo-1",
    title: "Scan Antigravity session directory for .pb transcripts",
    status: "done",
    description: "Walk ~/.antigravity/sessions and index conversation metadata",
    updatedAt: "2026-07-12T09:42:00Z",
  },
  {
    id: "todo-2",
    title: "Map Gemini model groups onto QuotaWindow.group",
    status: "in_progress",
    description: "Group 5h / weekly windows under \"Gemini Models\" in the dashboard",
    updatedAt: "2026-07-12T10:15:00Z",
  },
  {
    id: "todo-3",
    title: "Wire status bar chip to the new quota snapshot store",
    status: "pending",
    updatedAt: "2026-07-12T10:15:00Z",
  },
  {
    id: "todo-4",
    title: "Verify hook payload against bridge event schema",
    status: "blocked",
    description: "Waiting on hook-runner CLI fix for Windows paths",
    updatedAt: "2026-07-11T18:03:00Z",
  },
];

export const ActiveSessionTodos = () => (
  <SessionTodosPanel todos={activeTodos} isLoading={false} />
);

export const Loading = () => <SessionTodosPanel todos={[]} isLoading={true} />;

export const Empty = () => <SessionTodosPanel todos={[]} isLoading={false} />;
