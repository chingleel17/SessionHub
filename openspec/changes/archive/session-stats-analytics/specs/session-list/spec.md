## ADDED Requirements

### Requirement: Session card shows stats summary badges
Each session card in the session list SHALL display a compact stats badge row showing interaction count, output token total, and session duration when stats are available.

#### Scenario: Stats available on card render
- **WHEN** a session card is displayed and its stats have been loaded
- **THEN** the card SHALL show badges for turns, tokens (K-formatted), and duration (minutes)

#### Scenario: Session count shown in project header
- **WHEN** the user views a project tab
- **THEN** the project view SHALL display the count of visible sessions (respecting current filters)
