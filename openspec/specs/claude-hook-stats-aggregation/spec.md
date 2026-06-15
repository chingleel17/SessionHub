## ADDED Requirements

### Requirement: Aggregate token counts from events
The system SHALL track cumulative input and output tokens across all events for a session and maintain running totals in session_stats.

#### Scenario: Token totals accumulated over session
- **WHEN** multiple PostToolUse events fire each with token deltas
- **THEN** system sums all token deltas to compute session total_input_tokens and total_output_tokens

#### Scenario: Token counts survive session restart
- **WHEN** user resumes a session
- **THEN** system retrieves existing token totals from session_stats and adds new event deltas

### Requirement: Track tool call count and breakdown by tool
The system SHALL maintain a count of all tool invocations and categorize by tool name (e.g., tool_call_breakdown: {read: 5, grep: 3, bash: 2}).

#### Scenario: Tool call count incremented per PostToolUse
- **WHEN** PostToolUse event fires
- **THEN** system increments tool_call_count and updates tool_breakdown[tool_name]++

#### Scenario: Tool breakdown available for analytics
- **WHEN** frontend requests session stats
- **THEN** system returns tool_breakdown map with count per tool

### Requirement: Track model usage distribution
The system SHALL record which models were used during a session and count invocations per model (e.g., models_used: ["claude-opus-4", "claude-sonnet-4"]).

#### Scenario: Model recorded on each tool use
- **WHEN** PostToolUse event includes model field
- **THEN** system adds model to models_used set if not already present

#### Scenario: Model switch tracked
- **WHEN** user switches models mid-session (e.g., opus → sonnet)
- **THEN** system records both models in models_used array and timestamps of switch

### Requirement: Calculate interaction count
The system SHALL count user-initiated interactions (prompts + tool completions) as interaction_count.

#### Scenario: Interaction count incremented on user prompt
- **WHEN** UserPromptSubmit event fires
- **THEN** system increments interaction_count

#### Scenario: Interaction count reflects full round-trip
- **WHEN** user submits prompt and tool executes
- **THEN** system counts as single interaction (not two)

### Requirement: Defer stats aggregation to avoid blocking
Stats aggregation operations SHALL run asynchronously after event recording and not block the initial hook execution.

#### Scenario: Hook exits before stats computation
- **WHEN** PostToolUse hook fires
- **THEN** event recording completes within <5ms; stats aggregation queued as background task

#### Scenario: Stats updates coalesce
- **WHEN** multiple events fire within 500ms window
- **WHEN** system batches stats aggregation into single background task instead of N separate updates

### Requirement: Handle missing or partial event data
If event data is incomplete (missing token count, model field), the system SHALL use reasonable defaults and continue aggregation.

#### Scenario: Missing token count defaults to zero
- **WHEN** event lacks output_tokens field
- **THEN** system uses 0 for that event's token delta and logs warning to hook-errors.log

#### Scenario: Missing model defaults to "unknown"
- **WHEN** event lacks model field
- **THEN** system records model="unknown" and excludes from models_used analytics
