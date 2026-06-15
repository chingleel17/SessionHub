## ADDED Requirements

### Requirement: Update in-memory session cache on session state changes
The system SHALL invalidate and refresh in-memory ScanCache when events indicate session state changes (new session start, session termination, metadata updates).

#### Scenario: Cache invalidated on new session
- **WHEN** SessionStart hook fires for a new Claude session
- **THEN** system marks ScanCache as stale and re-scans session directory on next query

#### Scenario: Cache invalidated on session end
- **WHEN** Stop hook fires
- **THEN** system invalidates any cached session stats for that session_id

### Requirement: Refresh session metadata in runtime state
When session metadata changes (notes, tags, archive status, is_live flag), the system SHALL update the in-memory AppState to reflect changes without requiring full re-scan.

#### Scenario: is_live flag updated
- **WHEN** PostToolUse event fires indicating session activity
- **THEN** system sets is_live = true in AppState for that session

#### Scenario: Session activity timestamp updated
- **WHEN** any event (PostToolUse, UserPromptSubmit) fires
- **THEN** system updates session_updated_at timestamp in both memory and metadata.db

### Requirement: Defer cache updates to avoid blocking hooks
Cache refresh operations SHALL be queued as low-priority background tasks and not block the initial hook execution.

#### Scenario: Hook returns quickly despite cache invalidation
- **WHEN** event hook triggers cache update
- **THEN** hook exits in <10ms; cache refresh queued for background execution

#### Scenario: Multiple cache updates coalesce
- **WHEN** several events fire in rapid succession (within 100ms)
- **THEN** system coalesces multiple cache invalidation requests into single background task

### Requirement: Maintain cache consistency with database
Cache updates SHALL read from session_stats and session_cache tables to ensure frontend receives current data.

#### Scenario: Cache reflects latest database state
- **WHEN** cache update task runs
- **THEN** in-memory cache synchronized with metadata.db session_stats and session_cache tables

#### Scenario: Stale cache detected and refreshed
- **WHEN** cache mtime differs from database update time by >1 second
- **THEN** system triggers full cache refresh before returning to frontend
