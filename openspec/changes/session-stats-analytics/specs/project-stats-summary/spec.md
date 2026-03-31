## ADDED Requirements

### Requirement: Project stats summary banner
The system SHALL display a stats summary banner at the top of the ProjectView (within the toolbar-card section) showing: total session count (visible + archived), total output tokens across all sessions, and total interaction count across all sessions.

#### Scenario: Banner renders with loaded stats
- **WHEN** the ProjectView is rendered and all session stats have been loaded
- **THEN** the banner SHALL show: "N sessions · X turns · Y tokens total" (with K/M formatting)

#### Scenario: Stats partially loaded
- **WHEN** some session stats are still loading
- **THEN** the banner SHALL show totals for available sessions with a loading indicator for the rest

#### Scenario: Project has no sessions
- **WHEN** the project has zero sessions
- **THEN** the stats banner SHALL be hidden
