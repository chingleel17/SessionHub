## Context

SessionHub currently has hook event configurations in `settings.json` that point to placeholder commands (`"true"`), yielding no real event tracking. The hybrid architecture requires:
1. **Development path**: `C:\Users\kere4\.claude\hooks\` (for local testing and iteration)
2. **Production path**: `C:\Users\kere4\AppData\Roaming\SessionHub\.claude\hooks\` (installed with the app, versioned with releases)

Hook scripts must execute asynchronously without blocking Claude Code, and must handle five event types: `SessionStart`, `PreToolUse`, `PostToolUse`, `UserPromptSubmit`, `Stop`.

## Goals / Non-Goals

**Goals:**
- Implement modular PowerShell hook scripts that record session events without blocking
- Establish async event pipeline: synchronous event recording → asynchronous cache/stats updates
- Support both development (`~/.claude/hooks`) and production (`AppData/.claude/hooks`) paths
- Create installation/setup logic to deploy hook scripts to production location
- Decouple event recording from stats aggregation to prevent system bottlenecks
- Record event metadata (tool name, token deltas, model, timestamps) to SQLite for analytics

**Non-Goals:**
- Real-time dashboard updates via WebSocket (use polling or event bridge for now)
- Hook script configuration UI (users manage via settings.json)
- Cross-platform hook support (Windows PowerShell only; macOS/Linux out of scope)

## Decisions

### Decision 1: PowerShell + Async Task Queue
**Chosen**: PowerShell scripts + async background jobs for cache/stats updates
**Why**: PowerShell is built-in on Windows; async jobs prevent hook execution from blocking Claude Code. Event recording runs synchronously (fast), then queues stats updates.
**Alternatives**:
- Rust CLI tool: heavier to maintain, slower startup per event
- Batch scripts: less powerful, harder to handle complex IPC
- Direct Rust integration: would require modifying Claude Code itself (not feasible)

### Decision 2: Hybrid Directory Structure
**Chosen**: Development path `C:\Users\kere4\.claude\hooks\`; production path `C:\Users\kere4\AppData\Roaming\SessionHub\.claude\hooks\`
**Why**: Development path aligns with Claude Code standards, production path keeps app-related files isolated in AppData per Windows conventions. Setup process copies scripts at install time.
**Alternatives**:
- Single path in AppData: loses dev/test flexibility, harder to iterate
- Environment variable override: adds configuration complexity

### Decision 3: Event Recording → Cache Update → Stats Aggregation Pipeline
**Chosen**: Three-phase async pipeline with sequential execution per event
**Why**: Recording (write event to DB) is fast and must run synchronously. Cache updates and stats aggregation can be queued to avoid blocking. Sequential async phases prevent race conditions.
**Alternatives**:
- All-async: cache updates could run before recording completes, causing inconsistency
- Synchronous all: blocks Claude Code during stats aggregation (expensive)

### Decision 4: Hook Script Entry Points
**Chosen**: One script per event type (e.g., `on-post-tool-use.ps1`), each logs to event table and queues background jobs
**Why**: Clear separation of concerns, easy to test each event handler independently, reduces script complexity per file.
**Alternatives**:
- Monolithic script with switch statement: harder to maintain and test
- Shared library function: adds loader overhead per event

### Decision 5: IPC and Event Delivery
**Chosen**: Hook scripts invoke Tauri backend via CLI (e.g., `sessionhub record-event`) or write directly to metadata.db if SQLite access available
**Why**: CLI invocation respects Tauri's IPC security model; direct DB write is faster but requires careful locking.
**Alternatives**:
- File-based queue: slow and fragile under concurrent events
- Named pipes: complex to set up on Windows, harder for users to debug

## Risks / Trade-offs

- **[Risk]** PowerShell scripts vulnerable to path whitespace if hooks dir moved → **[Mitigation]** Use quoted paths in all script invocations; validate paths at setup time
- **[Risk]** Async queue overload under high event frequency (many tool uses in quick succession) → **[Mitigation]** Implement simple queue depth limit; drop oldest if queue > N items
- **[Risk]** Hook script errors could crash Claude Code if not caught → **[Mitigation]** Wrap all hook invocations in try-catch; log errors to file, never stderr to Claude
- **[Risk]** SQLite lock contention if multiple hook scripts write events simultaneously → **[Mitigation]** Use SQLite's built-in locking; add retry logic with exponential backoff
- **[Trade-off]** Async stats updates mean real-time session view may lag by milliseconds → **[Accepted]** Acceptable for UX; users will see updates within 1–2 seconds

## Migration Plan

1. **Phase 1 (Development)**: 
   - Create hook scripts in `openspec/hooks/` directory with full source
   - Update `settings.json` to reference development path (`~/.claude/hooks/`)
   - Test event recording and cache updates locally

2. **Phase 2 (Installation)**:
   - Add setup step in Tauri app initialization to copy hook scripts from bundled location to `AppData\Roaming\SessionHub\.claude\hooks\`
   - Update `settings.json` hook paths to production location
   - Create version marker file in hooks directory for future migration

3. **Phase 3 (Rollout)**:
   - Ship in next release; existing users upgrade automatically
   - Users with custom hook scripts in `~/.claude/hooks/` can migrate manually or continue using old path (dev path takes precedence if it exists)

## Open Questions

1. **Event Schema**: What fields should be stored in session_stats for each event type? (e.g., for `PostToolUse`: tool_name, tokens_used, model, latency_ms)
2. **Retention Policy**: How long should raw events be kept in metadata.db? (e.g., 30 days, rolling window)
3. **Error Logging**: Where should hook script errors be logged? (e.g., `AppData\Roaming\SessionHub\logs\hooks.log`)
4. **Queue Persistence**: Should async task queue be persisted to disk, or is in-memory acceptable?
