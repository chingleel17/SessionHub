import { SyncConflictDialog } from "session-hub";

const noop = () => {};

export const MultipleConflicts = () => (
  <SyncConflictDialog
    conflicts={[
      {
        source: "H:/Code/DIY/SessionHub/AGENTS.md",
        target: "H:/Code/DIY/SessionHub/CLAUDE.md",
        reason: "兩側檔案在上次同步後都有修改（hash 不一致）",
      },
      {
        source: "C:/Users/dev/.claude/agents/code-reviewer.md",
        target: "H:/Code/DIY/SessionHub/.claude/agents/code-reviewer.md",
        reason: "目標檔案較新（mtime 晚於來源 2 小時）",
      },
    ]}
    canRememberChoice={true}
    onResolve={noop}
    onCancel={noop}
  />
);

export const SingleConflict = () => (
  <SyncConflictDialog
    conflicts={[
      {
        source: "C:/Users/dev/.claude/skills/verify/SKILL.md",
        target: "H:/Code/DIY/SessionHub/.claude/skills/verify/SKILL.md",
      },
    ]}
    canRememberChoice={false}
    onResolve={noop}
    onCancel={noop}
  />
);
