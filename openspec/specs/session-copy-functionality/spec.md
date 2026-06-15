## Requirements

### Requirement: Session copy button behavior

The system SHALL allow users to copy CLI commands to open their sessions in the appropriate platform-specific tools.

#### Scenario: Copy button visibility for supported platforms
- **WHEN** a session card is displayed for a Copilot, OpenCode, Codex, or Claude session
- **THEN** the copy button is visible and enabled

#### Scenario: Copy button hidden for unsupported platforms
- **WHEN** a session card is displayed for an unsupported platform provider
- **THEN** the copy button is hidden

#### Scenario: Successful copy confirmation
- **WHEN** user clicks the copy button and the command is successfully copied to clipboard
- **THEN** a toast notification shows "Command copied" to confirm the action

#### Scenario: Unavailable command notification
- **WHEN** user clicks the copy button on a session where the command cannot be generated
- **THEN** a toast notification shows "Command unavailable for this platform"
