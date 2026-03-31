## ADDED Requirements

### Requirement: Dashboard shows aggregate token and interaction stats
The Dashboard SHALL display aggregate statistics across all sessions: total output tokens, total interactions, and total tool calls in addition to existing session and project counts.

#### Scenario: Dashboard renders with token totals
- **WHEN** the user navigates to the Dashboard and session stats have been loaded
- **THEN** the Dashboard SHALL show total output tokens (K/M formatted) and total interaction count across all loaded sessions
