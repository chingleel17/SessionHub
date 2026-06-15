## Context

Current implementation in `src/App.tsx` (line 1500-1513) only handles two cases: OpenCode gets `opencode session <id>`, everything else defaults to `copilot --resume=<id>`. This doesn't account for Codex and Claude platforms, which have their own CLI tools and command formats.

## Goals / Non-Goals

**Goals:**
- Generate correct CLI command for each supported platform (Copilot, OpenCode, Codex, Claude)
- Ensure the copied command matches the platform's actual CLI tool syntax
- Handle unsupported platforms gracefully by showing an error toast
- Keep the implementation simple and maintainable by using a mapping approach

**Non-Goals:**
- Validate that the CLI tool is actually installed on the user's system
- Support launching commands directly (only copy to clipboard)
- Add new CLI command formats beyond what each platform officially supports

## Decisions

**Decision 1: Use a Platform-to-Command Mapping Function**
- Rationale: Centralize the command generation logic in a pure function that takes provider and session ID, returns command string. This is:
  - Easy to test
  - Reusable if other components need the same logic
  - Maintainable as new platforms are added
- Alternative considered: Inline switch/if-else in `handleCopyCommand` (simpler initially, but harder to extend)

**Decision 2: Command Format for Each Platform**
- **Copilot**: `copilot --resume=<session-id>` (current format, unchanged)
- **OpenCode**: `opencode session <session-id>` (current format, unchanged)
- **Codex**: `codex session <session-id>` (tentative, needs verification of actual CLI syntax)
- **Claude**: `claude code --session=<session-id>` (tentative, needs verification of actual CLI syntax)
- Rationale: Maintain consistency with each platform's actual CLI tool conventions. If exact format differs, update based on platform documentation.

**Decision 3: Unsupported Platform Handling**
- Return empty string for unsupported platforms
- The existing `handleCopyCommand` checks for empty string and shows "commandUnavailable" toast
- No need to modify SessionCard logic (line 130 already has `supportsCommandCopy` guard)

## Risks / Trade-offs

**Risk 1: CLI Command Format Might Change**
- Mitigation: Document the expected format for each platform as a comment. When CLI tools update, this file is the single source of truth to update.

**Risk 2: Unknown Platforms**
- Mitigation: Log unknown providers and return empty string gracefully. User gets "command unavailable" message instead of crash.

**Risk 3: Session ID Format Varies by Platform**
- Current assumption: Session ID is the same across SessionInfo regardless of provider. If platform-specific ID formatting is needed, that requires backend changes to SessionInfo.
- Mitigation: No action needed now; this is a future concern if platforms use different ID schemes.

## Migration Plan

1. Add a new helper function `getSessionOpenCommand(provider: string, sessionId: string): string`
2. Update `handleCopyCommand` to use this new function
3. No database changes, no breaking changes to existing data
4. Deploy as-is; users immediately get correct commands for their platform
