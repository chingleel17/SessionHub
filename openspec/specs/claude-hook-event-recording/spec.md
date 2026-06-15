## ADDED Requirements

### Requirement: Record session events on PostToolUse hook
The system SHALL capture and store tool invocation metadata when Claude Code fires the PostToolUse event, including tool name, token usage delta, model identifier, and timestamp.

#### Scenario: Tool use event recorded to database
- **WHEN** Claude Code fires PostToolUse hook with tool metadata
- **THEN** system writes record to session_stats table with tool_name, input_tokens, output_tokens, model, timestamp

#### Scenario: Multiple sequential tool uses tracked
- **WHEN** multiple PostToolUse events fire in quick succession
- **THEN** each event is recorded as a separate row with independent token counts and timestamps

### Requirement: Record session events on UserPromptSubmit hook
The system SHALL capture user prompt submission events, including prompt length, submission timestamp, and associated session context.

#### Scenario: User prompt tracked
- **WHEN** Claude Code fires UserPromptSubmit hook
- **THEN** system records prompt_char_count, submission_timestamp to session_stats

#### Scenario: Prompt recorded even if no tool follows
- **WHEN** user submits prompt but no tool is invoked
- **THEN** system still creates event record for analytics

### Requirement: Record session events on SessionStart hook
The system SHALL capture session initialization metadata, including session ID, provider, project context, and start timestamp.

#### Scenario: Session start event recorded
- **WHEN** Claude Code fires SessionStart hook
- **THEN** system records session_id, provider, session_start_time, initial_model to session_stats

#### Scenario: Session metadata linked to project
- **WHEN** SessionStart fires with project context available
- **THEN** system associates session with parent project for analytics aggregation

### Requirement: Record session events on Stop hook
The system SHALL capture session termination metadata, including end timestamp, final token counts, and session duration.

#### Scenario: Session termination recorded
- **WHEN** Claude Code fires Stop hook (user closes session)
- **THEN** system writes session_end_time, final_output_tokens, final_input_tokens, duration_minutes

#### Scenario: Incomplete session marked
- **WHEN** user terminates session before natural completion
- **THEN** system records is_incomplete flag for session lifecycle tracking

### Requirement: Event records include structured metadata
All event records SHALL include: event_type, session_id, provider (claude|opencode), timestamp_ms (Unix milliseconds), and event-specific payload as JSON.

#### Scenario: Structured event payload stored
- **WHEN** event is recorded
- **THEN** system stores event_type, session_id, provider, timestamp_ms, and payload JSON (tool_name, tokens, model, etc.)

#### Scenario: Event timestamps in UTC milliseconds
- **WHEN** event is recorded from any timezone
- **THEN** timestamp is stored as UTC Unix milliseconds for consistency

### Requirement: Handle event recording failures gracefully
If event recording fails (database lock, disk full, permission error), the system SHALL log the error and continue execution without blocking Claude Code.

#### Scenario: Database lock during event write
- **WHEN** session_stats table is locked by another process
- **THEN** system retries write with exponential backoff (50ms → 100ms → 200ms) and logs retry count

#### Scenario: Event loss under critical failure
- **WHEN** database write fails after all retries
- **THEN** system logs error to `AppData\Roaming\SessionHub\logs\hook-errors.log` and exits gracefully without crashing Claude Code
