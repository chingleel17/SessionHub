## 1. Implement platform-specific command generation

- [x] 1.1 Create helper function `getSessionOpenCommand(provider: string, sessionId: string): string` in `src/App.tsx`
- [x] 1.2 Add case handlers for "copilot", "opencode", "codex", and "claude" providers
- [x] 1.3 Return empty string for unknown providers

## 2. Update handleCopyCommand function

- [x] 2.1 Modify `handleCopyCommand` to use the new `getSessionOpenCommand` helper function
- [x] 2.2 Test that command generation works correctly for all supported platforms

## 3. Update SessionCard copy button logic

- [x] 3.1 Verify that the `supportsCommandCopy` check on line 130 of SessionCard.tsx correctly excludes unsupported platforms
- [x] 3.2 Ensure copy button visibility is consistent with command generation

## 4. Testing and verification

- [x] 4.1 Test copying command from a Copilot session - verify `copilot --resume=<id>` is copied
- [x] 4.2 Test copying command from an OpenCode session - verify `opencode session <id>` is copied
- [x] 4.3 Test copying command from a Codex session - verify `codex session <id>` is copied
- [x] 4.4 Test copying command from a Claude session - verify `claude code --session=<id>` is copied
- [x] 4.5 Verify toast notifications show correct messages for success and unsupported platforms
