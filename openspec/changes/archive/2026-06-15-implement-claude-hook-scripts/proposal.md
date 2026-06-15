## Why

Currently, Claude Code hook events trigger placeholder commands with no actual execution logic. Without proper hook script handling, SessionHub cannot record detailed session telemetry (tool use, token consumption, model usage), update runtime caches, or notify users of operation completion. This limits observability and real-time session tracking across Claude platforms. Implementing hook scripts aligned with the hybrid architecture (AppData + .claude) will enable full event-driven session tracking and system responsiveness.

## What Changes

- Add PowerShell hook script infrastructure in `C:\Users\kere4\.claude\hooks\` for development/testing
- Create hook scripts that handle Claude session events: `PostToolUse`, `PreToolUse`, `UserPromptSubmit`, `SessionStart`, `Stop`
- Implement event processing pipeline: record session events → update cache → trigger statistics updates
- Modularize hook scripts to avoid concurrent execution bottlenecks; run event recording first, then queue cache/stats updates asynchronously
- Include hook script copying logic in application setup (copy to `AppData\Roaming\SessionHub\.claude\hooks\`)
- Update `settings.json` hook configuration to point to installed hook script paths

## Capabilities

### New Capabilities
- `claude-hook-event-recording`: Record detailed session events (tool use, token counts, model usage, completion notifications) to session metadata
- `claude-hook-cache-update`: Update in-memory session cache upon hook events to reflect live session state
- `claude-hook-stats-aggregation`: Aggregate and update session statistics (token counters, tool call counts, model usage) following events
- `claude-hook-async-pipeline`: Decouple synchronous event recording from asynchronous cache/stats updates to prevent system blocking

### Modified Capabilities
- `claude-integration`: Hook configuration now points to installed script paths; `settings.json` schema updated to reference hook script locations

## Impact

- **Rust Backend**: Adds setup logic to copy hook scripts; hook commands in `settings.json` now point to persistent script paths
- **Frontend**: No breaking changes; hook events processed transparently in backend
- **SQLite**: Leverages existing `session_stats` and `session_cache` tables for event recording
- **Setup/Installation**: New step to deploy hook scripts to `AppData\Roaming\SessionHub\.claude\hooks\`
- **Dependencies**: PowerShell (already available on Windows; no new external dependencies)
