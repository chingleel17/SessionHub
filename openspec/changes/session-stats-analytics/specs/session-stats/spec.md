## ADDED Requirements

### Requirement: Parse session statistics from events.jsonl
The system SHALL parse `events.jsonl` in a session directory to extract usage statistics including output tokens, interaction count, tool call count, duration, models used, reasoning count, and per-tool breakdown.

#### Scenario: Successful parse with full data
- **WHEN** `get_session_stats` is invoked with a valid session directory containing `events.jsonl`
- **THEN** the system returns a `SessionStats` object with `outputTokens`, `interactionCount`, `toolCallCount`, `durationMinutes`, `modelsUsed`, `reasoningCount`, and `toolBreakdown`

#### Scenario: Missing events.jsonl
- **WHEN** `get_session_stats` is invoked with a directory that has no `events.jsonl`
- **THEN** the system returns a `SessionStats` object with all numeric fields as `0` and empty arrays/maps

#### Scenario: Malformed lines in events.jsonl
- **WHEN** `events.jsonl` contains lines that are not valid JSON or missing expected fields
- **THEN** the system SHALL skip malformed lines and continue parsing, returning partial stats without error

### Requirement: Cache parsed statistics in SQLite
The system SHALL store parsed `SessionStats` in a `session_stats` SQLite table keyed by `session_id`, including the `events_mtime` of the parsed file.

#### Scenario: Cache hit - file unchanged
- **WHEN** `get_session_stats` is called and the stored `events_mtime` matches the current `events.jsonl` file modification time
- **THEN** the system returns cached stats without re-parsing the file

#### Scenario: Cache miss - file updated
- **WHEN** `get_session_stats` is called and `events.jsonl` mtime differs from the cached value
- **THEN** the system re-parses the file, updates the cache, and returns fresh stats

#### Scenario: Cache miss - no cache entry
- **WHEN** `get_session_stats` is called for a session with no cached entry
- **THEN** the system parses the file, inserts a new cache entry, and returns stats

### Requirement: Exclude subagent events from top-level stats
The system SHALL count only top-level events (those without `parentToolCallId` in their data) when computing `interactionCount`, `toolCallCount`, and `reasoningCount`.

#### Scenario: Session with subagent tool calls
- **WHEN** `events.jsonl` contains `tool.execution_start` events where `data.parentToolCallId` is set
- **THEN** those events SHALL NOT be counted in `toolCallCount`

### Requirement: Mark live sessions
The system SHALL set `isLive: true` on `SessionStats` when an `inuse.*.lock` file exists in the session directory, and SHALL NOT cache stats for live sessions.

#### Scenario: Session with lock file
- **WHEN** `get_session_stats` is called on a directory containing `inuse.*.lock`
- **THEN** `isLive` is `true` and the result is not written to SQLite cache
