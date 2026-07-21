import { BridgeEventMonitorDialog } from "session-hub";

const noop = () => {};

const events = [
  {
    id: "evt-001",
    provider: "claude",
    eventType: "SessionStart",
    timestamp: "2026-07-12T14:20:05+08:00",
    cwd: "H:/Code/DIY/SessionHub",
    sessionId: "0198c2f4-8a31-7c02-b7e9-3d5a12e04f77",
    title: "Refactor quota chip",
    error: null,
    status: "targeted",
  },
  {
    id: "evt-002",
    provider: "claude",
    eventType: "PostToolUse",
    timestamp: "2026-07-12T14:21:38+08:00",
    cwd: "H:/Code/DIY/SessionHub",
    sessionId: "0198c2f4-8a31-7c02-b7e9-3d5a12e04f77",
    title: null,
    error: null,
    status: "skipped_dedup",
  },
  {
    id: "evt-003",
    provider: "codex",
    eventType: "session.updated",
    timestamp: "2026-07-12T14:23:11+08:00",
    cwd: "H:/Code/DIY/hook-lab",
    sessionId: "c0dex-77aa41e09b21",
    title: null,
    error: null,
    status: "fallback",
  },
  {
    id: "evt-004",
    provider: "antigravity",
    eventType: "quota.refresh",
    timestamp: "2026-07-12T14:25:02+08:00",
    cwd: null,
    sessionId: null,
    title: null,
    error: "quota endpoint 回應 429，改用快取資料",
    status: "skipped_rate_limit",
  },
  {
    id: "evt-005",
    provider: "opencode",
    eventType: "session.idle",
    timestamp: "2026-07-12T14:27:44+08:00",
    cwd: "H:/Code/DIY/SessionHub",
    sessionId: "oc-31b22c02a001",
    title: null,
    error: null,
    status: "activity_hint",
  },
  {
    id: "evt-006",
    provider: "claude",
    eventType: "Stop",
    timestamp: "2026-07-12T14:30:19+08:00",
    cwd: "H:/Code/DIY/SessionHub",
    sessionId: "0198c2f4-8a31-7c02-b7e9-3d5a12e04f77",
    title: null,
    error: null,
    status: "full_refresh",
  },
];

export const WithEvents = () => (
  <BridgeEventMonitorDialog events={events} onClose={noop} onClear={noop} />
);

export const EmptyLog = () => (
  <BridgeEventMonitorDialog events={[]} onClose={noop} onClear={noop} />
);
