## ADDED Requirements

### Requirement: Session card displays stats badge row
The system SHALL display a compact stats badge row at the bottom of each SessionCard showing: interaction count, output token count (formatted with K suffix), and duration in minutes.

#### Scenario: Stats loaded successfully
- **WHEN** a SessionCard is rendered and stats have been loaded
- **THEN** the card SHALL show badges for interaction count (e.g. "12 turns"), output tokens (e.g. "48K tokens"), and duration (e.g. "35 min")

#### Scenario: Stats loading
- **WHEN** a SessionCard is rendered and stats are still being fetched
- **THEN** the badge row SHALL show a skeleton/placeholder state (no layout shift)

#### Scenario: Events file absent
- **WHEN** stats are loaded but the session has no events.jsonl (all counts are 0)
- **THEN** the badge row SHALL be hidden or show a "no data" indicator

### Requirement: Session stats detail panel
The system SHALL provide an expandable detail panel for each session that shows full statistics including: per-tool breakdown table, models used list, reasoning count, average tokens per turn.

#### Scenario: User opens detail panel
- **WHEN** user clicks the stats detail icon-button on a SessionCard
- **THEN** a panel expands inline below the card showing full stats

#### Scenario: User closes detail panel
- **WHEN** user clicks the detail icon-button again or clicks a close button
- **THEN** the panel collapses

#### Scenario: Tool breakdown display
- **WHEN** the detail panel is open and toolBreakdown is non-empty
- **THEN** the panel SHALL show a list of tool names with their call counts, sorted descending by count

#### Scenario: Models used display
- **WHEN** the detail panel is open
- **THEN** the panel SHALL show all unique models used in the session

#### Scenario: Live session indicator
- **WHEN** the session stats have `isLive: true`
- **THEN** the detail panel SHALL display a "Session in progress" notice and stats are labeled as current snapshot
