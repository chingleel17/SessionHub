## ADDED Requirements

### Requirement: Hook scripts execute in three phases asynchronously
When a hook event fires, the system SHALL execute three phases in sequence: (1) Event Recording (sync), (2) Cache Invalidation (async), (3) Stats Aggregation (async), ensuring event data is persisted before returning to Claude Code.

#### Scenario: Hook returns quickly to Claude Code
- **WHEN** event hook (e.g., PostToolUse) is triggered
- **THEN** hook process exits within 15ms, having completed event recording; cache/stats updates queued for background

#### Scenario: Phases execute in order
- **WHEN** background task processor starts
- **THEN** cache invalidation tasks run first, then stats aggregation (to ensure cache reflects latest stats)

### Requirement: Event recording runs synchronously within hook
Event Recording phase (writing to session_stats table) SHALL complete before the hook exits, ensuring no event data is lost if SessionHub terminates unexpectedly.

#### Scenario: Event persisted before hook exits
- **WHEN** PostToolUse hook writes event to session_stats
- **THEN** system flushes write to disk before hook exits (SQLite COMMIT/PRAGMA synchronous)

#### Scenario: Failed event recording blocks hook exit
- **WHEN** database write fails and all retries exhausted
- **THEN** hook logs error and exits with non-zero code (to notify Claude Code of failure)

### Requirement: Cache updates queue asynchronously
Cache Invalidation phase SHALL be queued as a background job and not block hook execution. Multiple requests within a time window SHALL be deduplicated.

#### Scenario: Cache update job queued
- **WHEN** event hook invalidates session cache
- **THEN** job added to internal queue (not executed immediately)

#### Scenario: Rapid events deduplicate cache jobs
- **WHEN** three PostToolUse events fire within 100ms for same session
- **THEN** system coalesces into single cache update task for that session

#### Scenario: Cache updates complete within 1 second
- **WHEN** background task processor picks up cache job
- **THEN** cache refresh completes within 1s and in-memory state updated

### Requirement: Stats aggregation queues asynchronously
Stats Aggregation phase SHALL be queued as a low-priority background task after cache updates are scheduled. Stats updates for same session within a window SHALL batch together.

#### Scenario: Stats task queued separately from cache
- **WHEN** event hook triggers both cache and stats updates
- **THEN** cache task queued immediately; stats task queued with lower priority (runs after cache)

#### Scenario: Stats updates batch efficiently
- **WHEN** session receives 10 PostToolUse events in rapid succession
- **THEN** system batches all 10 into single stats aggregation task (token sum, tool breakdown, etc.)

### Requirement: Background task queue with bounded depth
The system SHALL maintain a bounded queue of pending background tasks (max 100 items per session). Tasks beyond limit are logged and oldest dropped.

#### Scenario: Queue depth monitored
- **WHEN** background task queue exceeds 100 items
- **THEN** system logs warning and drops oldest pending task to prevent memory bloat

#### Scenario: User notified of queue overload
- **WHEN** queue drops tasks
- **THEN** system logs to hook-errors.log and optionally triggers frontend toast notification

### Requirement: Error handling without blocking hook
If a background task (cache or stats) fails, the system SHALL log error and continue processing remaining tasks without affecting hook performance.

#### Scenario: Cache task fails gracefully
- **WHEN** cache update task encounters database lock or permission error
- **THEN** system logs error to hook-errors.log, retries up to 3 times with backoff, then abandons task

#### Scenario: Stats task failure logged
- **WHEN** stats aggregation fails (missing event data, calculation error)
- **THEN** system logs error with event details and continues; frontend receives partial stats

### Requirement: Idle timeout for background processing
Background tasks SHALL be processed continuously while queued. If queue becomes empty and stays empty for 5 seconds, background processor MAY enter sleep mode to conserve resources.

#### Scenario: Background processor sleeps when idle
- **WHEN** queue is empty for 5+ seconds
- **THEN** processor thread enters low-power sleep

#### Scenario: Processor wakes on new event
- **WHEN** new event enqueues task while processor is sleeping
- **THEN** processor wakes immediately (< 10ms latency) and resumes processing
