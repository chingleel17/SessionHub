import { ContentViewer } from "session-hub";

const noopToggle = async () => {};

const proposalMd = `# Proposal: Quota ring in the status bar

## Why

The current text-only percentage is easy to miss when a provider window
is close to its limit. A compact SVG ring gives an at-a-glance signal.

## What Changes

- Replace the quota chip text with an SVG ring + graded color
- Add provider brand colors for \`claude\` / \`antigravity\` tags
- Localize the shared quota window labels

## Impact

| Area | Change |
| --- | --- |
| StatusBar | new ring renderer |
| QuotaOverview | shared label helper |
`;

const tasksMd = `# Tasks

- [x] Add SVG ring renderer to StatusBar
- [x] Wire graded color thresholds (60% / 80%)
- [ ] Localize quota window labels
- [ ] Add Antigravity model-group display
`;

export const MarkdownDocument = () => (
  <div style={{ height: 420 }}>
    <ContentViewer
      content={proposalMd}
      filePath="openspec/changes/quota-ring/proposal.md"
      filePathType="openspec"
      isLoading={false}
      error={null}
      isTaskSaving={false}
      onToggleTask={noopToggle}
    />
  </div>
);

export const InteractiveTaskList = () => (
  <div style={{ height: 320 }}>
    <ContentViewer
      content={tasksMd}
      filePath="openspec/changes/quota-ring/tasks.md"
      filePathType="openspec"
      isLoading={false}
      error={null}
      isTaskSaving={false}
      onToggleTask={noopToggle}
    />
  </div>
);

export const LoadingState = () => (
  <div style={{ height: 180 }}>
    <ContentViewer
      content={null}
      filePath={null}
      filePathType={null}
      isLoading
      error={null}
      isTaskSaving={false}
      onToggleTask={noopToggle}
    />
  </div>
);

export const ErrorState = () => (
  <div style={{ height: 180 }}>
    <ContentViewer
      content={null}
      filePath="openspec/specs/missing.md"
      filePathType="openspec"
      isLoading={false}
      error="ENOENT: no such file or directory"
      isTaskSaving={false}
      onToggleTask={noopToggle}
    />
  </div>
);
