## Requirements

### Requirement: Generate platform-specific CLI commands

The system SHALL generate the correct CLI command for opening a session based on the session's platform provider type.

#### Scenario: Copy command for Copilot session
- **WHEN** user clicks copy button on a Copilot session card
- **THEN** the command `copilot --resume=<session-id>` is copied to clipboard

#### Scenario: Copy command for OpenCode session
- **WHEN** user clicks copy button on an OpenCode session card
- **THEN** the command `opencode session <session-id>` is copied to clipboard

#### Scenario: Copy command for Codex session
- **WHEN** user clicks copy button on a Codex session card
- **THEN** the command `codex session <session-id>` is copied to clipboard

#### Scenario: Copy command for Claude session
- **WHEN** user clicks copy button on a Claude session card
- **THEN** the command `claude code --session=<session-id>` is copied to clipboard

#### Scenario: Handle unknown platform gracefully
- **WHEN** user clicks copy button on a session with an unrecognized provider
- **THEN** the system shows "Command unavailable for this platform" toast notification and does not copy anything
