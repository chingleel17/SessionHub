import { ExplorerTree } from "session-hub";

const noop = () => {};

const changeTree = [
  {
    id: "changes",
    label: "Changes",
    defaultOpen: true,
    children: [
      {
        id: "change-quota-ring",
        label: "add-quota-ring-status-bar",
        icon: "change",
        tone: "in_progress",
        badge: "7/12",
        defaultOpen: true,
        children: [
          {
            id: "change-quota-ring/proposal",
            label: "proposal.md",
            icon: "proposal",
            tone: "done",
            filePath: "openspec/changes/add-quota-ring-status-bar/proposal.md",
            filePathType: "openspec",
          },
          {
            id: "change-quota-ring/design",
            label: "design.md",
            icon: "design",
            tone: "done",
            filePath: "openspec/changes/add-quota-ring-status-bar/design.md",
            filePathType: "openspec",
          },
          {
            id: "change-quota-ring/tasks",
            label: "tasks.md",
            icon: "tasks",
            tone: "in_progress",
            badge: "7/12",
            filePath: "openspec/changes/add-quota-ring-status-bar/tasks.md",
            filePathType: "openspec",
          },
        ],
      },
      {
        id: "change-antigravity",
        label: "add-antigravity-provider",
        icon: "change",
        tone: "not_started",
        badge: "0/9",
        defaultOpen: false,
        children: [
          {
            id: "change-antigravity/proposal",
            label: "proposal.md",
            icon: "proposal",
            tone: "neutral",
            filePath: "openspec/changes/add-antigravity-provider/proposal.md",
            filePathType: "openspec",
          },
        ],
      },
    ],
  },
  {
    id: "specs",
    label: "Specs",
    defaultOpen: true,
    children: [
      {
        id: "spec-session-scanning",
        label: "session-scanning",
        icon: "spec",
        tone: "neutral",
        filePath: "openspec/specs/session-scanning/spec.md",
        filePathType: "openspec",
      },
      {
        id: "spec-quota-monitoring",
        label: "quota-monitoring",
        icon: "spec",
        tone: "neutral",
        filePath: "openspec/specs/quota-monitoring/spec.md",
        filePathType: "openspec",
      },
      {
        id: "spec-bridge-events",
        label: "bridge-events",
        icon: "spec",
        tone: "neutral",
        filePath: "openspec/specs/bridge-events/spec.md",
        filePathType: "openspec",
      },
    ],
  },
];

export const OpenSpecChanges = () => (
  <ExplorerTree
    nodes={changeTree}
    selectedId="change-quota-ring/tasks"
    onSelect={noop}
  />
);

const planTree = [
  {
    id: "plans",
    label: "Plans",
    defaultOpen: true,
    children: [
      {
        id: "plan-mcp-config",
        label: "mcp-config-manager.plan.md",
        icon: "plan",
        tone: "done",
        badge: "done",
        filePath: "plans/mcp-config-manager.plan.md",
        filePathType: "absolute",
      },
      {
        id: "plan-linear-ui",
        label: "linear-design-refresh.plan.md",
        icon: "plan",
        tone: "in_progress",
        badge: "wip",
        filePath: "plans/linear-design-refresh.plan.md",
        filePathType: "absolute",
      },
    ],
  },
  {
    id: "notes",
    label: "Notes & Evidence",
    defaultOpen: true,
    children: [
      {
        id: "note-quota-endpoint",
        label: "antigravity-quota-endpoint.md",
        icon: "note",
        tone: "muted",
        filePath: "notes/antigravity-quota-endpoint.md",
        filePathType: "absolute",
      },
      {
        id: "evidence-hook-log",
        label: "hook-latency-trace.md",
        icon: "evidence",
        tone: "muted",
        filePath: "notes/hook-latency-trace.md",
        filePathType: "absolute",
      },
      {
        id: "draft-agents",
        label: "AGENTS.md (draft)",
        icon: "draft",
        tone: "neutral",
      },
    ],
  },
];

export const PlansAndNotes = () => (
  <ExplorerTree nodes={planTree} selectedId="plan-linear-ui" onSelect={noop} />
);

const archiveTree = [
  {
    id: "archive",
    label: "Archive",
    defaultOpen: true,
    children: [
      {
        id: "archived-mcp",
        label: "add-mcp-config-manager",
        icon: "change",
        tone: "done",
        badge: "14/14",
        defaultOpen: false,
        children: [
          {
            id: "archived-mcp/tasks",
            label: "tasks.md",
            icon: "tasks",
            tone: "done",
            badge: "14/14",
            filePath: "openspec/changes/archive/add-mcp-config-manager/tasks.md",
            filePathType: "openspec",
          },
        ],
      },
      {
        id: "archived-theme",
        label: "unify-design-tokens",
        icon: "change",
        tone: "done",
        badge: "21/21",
        defaultOpen: false,
        children: [],
      },
    ],
  },
];

export const ArchivedChanges = () => (
  <ExplorerTree nodes={archiveTree} selectedId={null} onSelect={noop} />
);
