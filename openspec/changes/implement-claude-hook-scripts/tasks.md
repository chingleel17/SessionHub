## 1. Prepare Hook Script Infrastructure

- [x] 1.1 Create `.claude/hooks/` directory structure in project root
- [x] 1.2 Create PowerShell helper module for event recording (record-event.psm1)
- [x] 1.3 Create PowerShell helper module for async task queueing (task-queue.psm1)
- [x] 1.4 Create PowerShell helper module for SQLite operations with retry logic (db-ops.psm1)

## 2. Implement Event Recording Hook Scripts

- [x] 2.1 Create `on-session-start.ps1` to record SessionStart events
- [x] 2.2 Create `on-pre-tool-use.ps1` to prepare tool use tracking
- [x] 2.3 Create `on-post-tool-use.ps1` to record tool execution events (token delta, model, tool name)
- [x] 2.4 Create `on-user-prompt-submit.ps1` to record prompt submission events
- [x] 2.5 Create `on-stop.ps1` to record session termination events
- [x] 2.6 Add error handling and logging to all hook scripts (write to `AppData\Roaming\SessionHub\logs\hook-errors.log`)

## 3. Implement Background Task Queue System

- [ ] 3.1 Design in-memory task queue structure in Rust (VecDeque<Task> with bounded depth)
- [ ] 3.2 Implement task enqueue logic in Rust backend
- [ ] 3.3 Implement task processor thread that runs continuously
- [ ] 3.4 Add task deduplication for cache invalidation tasks (same session within 100ms coalesces)
- [ ] 3.5 Add task batching for stats aggregation tasks
- [ ] 3.6 Implement idle timeout (5 seconds) and sleep mode for background processor

## 4. Implement Cache Invalidation Phase

- [ ] 4.1 Create `InvalidateSessionCache` task type in Rust
- [ ] 4.2 Implement cache invalidation handler in background processor
- [ ] 4.3 Update ScanCache is_live flag on PostToolUse events
- [ ] 4.4 Refresh session_updated_at timestamp in AppState
- [ ] 4.5 Test cache consistency: ensure in-memory matches metadata.db

## 5. Implement Stats Aggregation Phase

- [ ] 5.1 Create `AggregateSessionStats` task type in Rust
- [ ] 5.2 Implement token count accumulation (input + output totals)
- [ ] 5.3 Implement tool call tracking and breakdown (tool_name → count map)
- [ ] 5.4 Implement model usage tracking (models_used array with distribution)
- [ ] 5.5 Implement interaction count calculation (prompts + completions)
- [ ] 5.6 Add handling for missing/partial event data (defaults to zero/unknown)

## 6. Database Schema and Migration

- [ ] 6.1 Design event_log table schema (event_type, session_id, provider, timestamp_ms, payload JSON)
- [ ] 6.2 Add event_log table to metadata.db with indexes on session_id and timestamp_ms
- [ ] 6.3 Create migration script for existing databases
- [ ] 6.4 Update session_stats table to support new fields (tool_breakdown JSON, models_used JSON)
- [ ] 6.5 Ensure SQLite locking strategy handles concurrent hook writes (COMMIT with retries)

## 7. Hook Configuration and Settings Integration

- [x] 7.1 Update `settings.json` schema to include hook command paths
- [ ] 7.2 Update hook configuration in `default-settings.json` to point to development path (`~/.claude/hooks/`)
- [x] 7.3 Add AppSettings field for hook_scripts_path (defaults to `~/.claude/hooks/`)
- [x] 7.4 Create fallback logic: if development path doesn't exist, check production path

## 8. Installation and Setup

- [x] 8.1 Create setup utility function to copy hook scripts from bundled location to `AppData\Roaming\SessionHub\.claude\hooks\`
- [x] 8.2 Add version marker file in hooks directory for future migrations
- [x] 8.3 Call setup utility in app initialization (check if hooks exist; if not, copy from bundle)
- [ ] 8.4 Create uninstall logic to remove hook scripts on app removal
- [ ] 8.5 Test that hook scripts copy correctly on first run

## 9. Integration with Claude Code

- [x] 9.1 Update Claude integration hook handlers to validate hook script paths
- [ ] 9.2 Ensure hook scripts are executable (check permissions on Windows)
- [ ] 9.3 Add error handling for failed hook invocations (log to hook-errors.log)
- [ ] 9.4 Test hook execution with simulated Claude Code events

## 10. Testing and Validation

- [ ] 10.1 Write unit tests for task queue logic (enqueue, dequeue, deduplication)
- [ ] 10.2 Write integration tests for event recording (PostToolUse → DB write)
- [ ] 10.3 Write integration tests for cache invalidation (event → ScanCache refresh)
- [ ] 10.4 Write integration tests for stats aggregation (multiple events → correct totals)
- [ ] 10.5 Test error scenarios: database lock, permission denied, missing fields
- [ ] 10.6 Test performance: measure hook execution time and background task latency
- [ ] 10.7 Manual test: fire synthetic hook events and verify end-to-end flow

## 11. Logging and Monitoring

- [ ] 11.1 Create `AppData\Roaming\SessionHub\logs\` directory structure
- [ ] 11.2 Implement structured logging for hook events (JSON lines format)
- [ ] 11.3 Add debug-level logging to task queue operations
- [ ] 11.4 Implement log rotation (keep last 30 days of logs)
- [ ] 11.5 Add admin UI option to view hook logs in SessionHub settings

## 12. Documentation

- [ ] 12.1 Document hook script API (event payload schema for each hook type)
- [ ] 12.2 Document task queue behavior and deduplication rules
- [ ] 12.3 Create troubleshooting guide for hook failures
- [x] 12.4 Update README with hook script information
