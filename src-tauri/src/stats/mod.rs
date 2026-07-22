use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use rusqlite::{params, Connection};

use crate::types::*;

// ── Copilot stats cache ──────────────────────────────────────────────────────

pub(crate) fn get_session_stats_cache(
    connection: &Connection,
    session_id: &str,
) -> Result<Option<(i64, SessionStats)>, String> {
    let mut statement = connection
        .prepare(
            "
            SELECT events_mtime, output_tokens, interaction_count, tool_call_count,
                     duration_minutes, models_used, reasoning_count, tool_breakdown, input_tokens,
                     model_metrics
            FROM session_stats
            WHERE session_id = ?1
            ",
        )
        .map_err(|error| format!("failed to prepare session stats cache query: {error}"))?;

    let mut rows = statement
        .query(params![session_id])
        .map_err(|error| format!("failed to query session stats cache: {error}"))?;

    match rows
        .next()
        .map_err(|error| format!("failed to read session stats cache row: {error}"))?
    {
        Some(row) => {
            let events_mtime: i64 = row
                .get(0)
                .map_err(|error| format!("failed to read events_mtime column: {error}"))?;
            let models_used_json: String = row
                .get(5)
                .map_err(|error| format!("failed to read models_used column: {error}"))?;
            let tool_breakdown_json: String = row
                .get(7)
                .map_err(|error| format!("failed to read tool_breakdown column: {error}"))?;

            let models_used = serde_json::from_str::<Vec<String>>(&models_used_json)
                .map_err(|error| format!("failed to deserialize cached models_used: {error}"))?;
            let tool_breakdown = serde_json::from_str::<BTreeMap<String, u32>>(
                &tool_breakdown_json,
            )
            .map_err(|error| format!("failed to deserialize cached tool_breakdown: {error}"))?;
            let model_metrics_json: String = row.get(9).unwrap_or_else(|_| "{}".to_string());
            let model_metrics =
                serde_json::from_str::<BTreeMap<String, ModelMetricsEntry>>(&model_metrics_json)
                    .unwrap_or_default();

            Ok(Some((
                events_mtime,
                SessionStats {
                    output_tokens: row
                        .get(1)
                        .map_err(|error| format!("failed to read output_tokens column: {error}"))?,
                    input_tokens: row
                        .get(8)
                        .map_err(|error| format!("failed to read input_tokens column: {error}"))?,
                    interaction_count: row.get(2).map_err(|error| {
                        format!("failed to read interaction_count column: {error}")
                    })?,
                    tool_call_count: row.get(3).map_err(|error| {
                        format!("failed to read tool_call_count column: {error}")
                    })?,
                    duration_minutes: row.get(4).map_err(|error| {
                        format!("failed to read duration_minutes column: {error}")
                    })?,
                    models_used,
                    reasoning_count: row.get(6).map_err(|error| {
                        format!("failed to read reasoning_count column: {error}")
                    })?,
                    tool_breakdown,
                    model_metrics,
                    is_live: false,
                },
            )))
        }
        None => Ok(None),
    }
}

pub(crate) fn upsert_session_stats_cache(
    connection: &Connection,
    session_id: &str,
    events_mtime: i64,
    stats: &SessionStats,
) -> Result<(), String> {
    let models_used_json = serde_json::to_string(&stats.models_used)
        .map_err(|error| format!("failed to serialize models_used: {error}"))?;
    let tool_breakdown_json = serde_json::to_string(&stats.tool_breakdown)
        .map_err(|error| format!("failed to serialize tool_breakdown: {error}"))?;
    let model_metrics_json = serde_json::to_string(&stats.model_metrics)
        .map_err(|error| format!("failed to serialize model_metrics: {error}"))?;

    connection
        .execute(
            "
            INSERT INTO session_stats (
                session_id, events_mtime, output_tokens, input_tokens, interaction_count,
                tool_call_count, duration_minutes, models_used, reasoning_count, tool_breakdown,
                model_metrics
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(session_id) DO UPDATE SET
                events_mtime = excluded.events_mtime,
                output_tokens = excluded.output_tokens,
                input_tokens = excluded.input_tokens,
                interaction_count = excluded.interaction_count,
                tool_call_count = excluded.tool_call_count,
                duration_minutes = excluded.duration_minutes,
                models_used = excluded.models_used,
                reasoning_count = excluded.reasoning_count,
                tool_breakdown = excluded.tool_breakdown,
                model_metrics = excluded.model_metrics
            ",
            params![
                session_id,
                events_mtime,
                stats.output_tokens,
                stats.input_tokens,
                stats.interaction_count,
                stats.tool_call_count,
                stats.duration_minutes,
                models_used_json,
                stats.reasoning_count,
                tool_breakdown_json,
                model_metrics_json,
            ],
        )
        .map_err(|error| format!("failed to upsert session stats cache: {error}"))?;

    Ok(())
}

// ── 輔助函式 ─────────────────────────────────────────────────────────────────

pub(crate) fn session_events_mtime(events_path: &Path) -> Result<i64, String> {
    let modified = fs::metadata(events_path)
        .map_err(|error| {
            format!(
                "failed to read metadata for {}: {error}",
                events_path.display()
            )
        })?
        .modified()
        .map_err(|error| {
            format!(
                "failed to read modified time for {}: {error}",
                events_path.display()
            )
        })?;

    let seconds = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to convert modified time to unix epoch: {error}"))?
        .as_secs();

    i64::try_from(seconds).map_err(|error| format!("failed to convert modified time: {error}"))
}

pub(crate) fn session_id_from_dir(session_dir: &Path) -> String {
    session_dir
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub(crate) fn is_live_session(session_dir: &Path) -> Result<bool, String> {
    let entries = fs::read_dir(session_dir).map_err(|error| {
        format!(
            "failed to read session dir {}: {error}",
            session_dir.display()
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read session dir entry: {error}"))?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.starts_with("inuse.") && file_name.ends_with(".lock") {
            // 從 inuse.<pid>.lock 取出 PID 並驗證程序是否仍在執行
            let mid = &file_name["inuse.".len()..file_name.len() - ".lock".len()];
            if let Ok(pid) = mid.parse::<u32>() {
                if is_pid_alive(pid) {
                    return Ok(true);
                }
                // PID 已不存在 → 殭屍 lock 檔，繼續找下一個
            } else {
                // 非 PID 格式的 lock 檔 → 保守判定為 live
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// 檢查指定 PID 的程序是否仍在執行
#[cfg(windows)]
fn is_pid_alive(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    const STILL_ACTIVE: u32 = 259;
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle == std::ptr::null_mut() {
            return false;
        }
        let mut exit_code: u32 = 0;
        let ok = GetExitCodeProcess(handle, &mut exit_code);
        CloseHandle(handle);
        ok != 0 && exit_code == STILL_ACTIVE
    }
}

#[cfg(not(windows))]
fn is_pid_alive(pid: u32) -> bool {
    // Unix: kill -0 不送訊號，只檢查程序是否存在
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

// ── Copilot: 解析 events.jsonl ───────────────────────────────────────────────

pub(crate) fn parse_session_stats_internal(session_dir: &Path) -> Result<SessionStats, String> {
    let events_path = session_dir.join("events.jsonl");
    let mut stats = SessionStats {
        is_live: is_live_session(session_dir)?,
        ..SessionStats::default()
    };

    if !events_path.exists() {
        return Ok(stats);
    }

    let file = fs::File::open(&events_path)
        .map_err(|error| format!("failed to open {}: {error}", events_path.display()))?;
    let reader = BufReader::new(file);
    let mut start_time: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut last_timestamp: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut models_used = BTreeSet::new();

    for line in reader.lines() {
        let line = match line {
            Ok(value) => value,
            Err(_) => continue,
        };

        let event = match serde_json::from_str::<SessionEvent>(&line) {
            Ok(value) => value,
            Err(_) => continue,
        };

        if let Some(timestamp) = event.timestamp.as_deref() {
            if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(timestamp) {
                last_timestamp = Some(parsed.with_timezone(&chrono::Utc));
            }
        }

        match event.event_type.as_str() {
            "session.start" => {
                if let Ok(data) = serde_json::from_value::<SessionStartData>(event.data) {
                    if let Some(model) =
                        data.selected_model.filter(|value| !value.trim().is_empty())
                    {
                        models_used.insert(model);
                    }
                    if let Some(raw_time) = data.start_time {
                        if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&raw_time) {
                            start_time = Some(parsed.with_timezone(&chrono::Utc));
                        }
                    }
                }
            }
            "session.model_change" => {
                if let Ok(data) = serde_json::from_value::<SessionModelChangeData>(event.data) {
                    if let Some(model) = data.new_model.filter(|value| !value.trim().is_empty()) {
                        models_used.insert(model);
                    }
                }
            }
            "user.message" => {
                if let Ok(data) = serde_json::from_value::<TopLevelFilterData>(event.data) {
                    if data.parent_tool_call_id.is_none() {
                        stats.interaction_count += 1;
                    }
                }
            }
            "tool.execution_start" => {
                if let Ok(data) = serde_json::from_value::<ToolExecutionStartData>(event.data) {
                    if data.parent_tool_call_id.is_none() {
                        stats.tool_call_count += 1;
                        let tool_name = data.tool_name.unwrap_or_else(|| "unknown".to_string());
                        *stats.tool_breakdown.entry(tool_name).or_insert(0) += 1;
                    }
                }
            }
            "assistant.message" => {
                if let Ok(data) = serde_json::from_value::<AssistantMessageData>(event.data) {
                    if data.parent_tool_call_id.is_none() {
                        stats.output_tokens += data.output_tokens.unwrap_or(0);
                        if data.reasoning_opaque.is_some() {
                            stats.reasoning_count += 1;
                        }
                    }
                }
            }
            "session.shutdown" => {
                if let Ok(data) = serde_json::from_value::<SessionShutdownData>(event.data) {
                    for (model, metric) in data.model_metrics {
                        if model.trim().is_empty() {
                            continue;
                        }
                        models_used.insert(model.clone());
                        let requests = metric.requests;
                        let usage = metric.usage;
                        stats.model_metrics.insert(
                            model,
                            ModelMetricsEntry {
                                requests_count: requests
                                    .as_ref()
                                    .and_then(|v| v.count)
                                    .unwrap_or(0.0),
                                requests_cost: requests
                                    .as_ref()
                                    .and_then(|v| v.cost)
                                    .unwrap_or(0.0),
                                input_tokens: usage
                                    .as_ref()
                                    .and_then(|v| v.input_tokens)
                                    .unwrap_or(0),
                                output_tokens: usage
                                    .as_ref()
                                    .and_then(|v| v.output_tokens)
                                    .unwrap_or(0),
                            },
                        );
                    }
                }
            }
            _ => {}
        }
    }

    stats.models_used = models_used.into_iter().collect();

    if let (Some(start), Some(end)) = (start_time, last_timestamp) {
        let seconds = (end - start).num_seconds();
        if seconds > 0 {
            stats.duration_minutes = (seconds as u64) / 60;
        }
    }

    Ok(stats)
}

mod claude;
mod opencode;

pub(crate) use claude::*;
pub(crate) use opencode::*;

pub(crate) fn get_session_stats_internal(
    connection: &Connection,
    session_dir: &str,
) -> Result<SessionStats, String> {
    let session_path = PathBuf::from(session_dir);

    // Claude sessions: .jsonl files under ~/.claude/projects/
    if session_path.is_file() && is_claude_session_file(&session_path) {
        let session_id = session_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let current_mtime = session_events_mtime(&session_path)?;
        if let Some((cached_mtime, cached_stats)) =
            get_session_stats_cache(connection, &session_id)?
        {
            if cached_mtime == current_mtime {
                return Ok(cached_stats);
            }
        }
        let stats = compute_claude_stats(&session_path)?;
        upsert_session_stats_cache(connection, &session_id, current_mtime, &stats)?;
        return Ok(stats);
    }

    if session_path.is_file() {
        return Ok(SessionStats::default());
    }
    let session_id = session_id_from_dir(&session_path);

    if is_opencode_session_dir(&session_path) {
        return get_opencode_session_stats_internal(connection, &session_path, &session_id);
    }

    let events_path = session_path.join("events.jsonl");
    let is_live = is_live_session(&session_path)?;

    if !events_path.exists() {
        return Ok(SessionStats {
            is_live,
            ..SessionStats::default()
        });
    }

    let current_mtime = session_events_mtime(&events_path)?;
    if !is_live {
        if let Some((cached_mtime, mut cached_stats)) =
            get_session_stats_cache(connection, &session_id)?
        {
            if cached_mtime == current_mtime {
                cached_stats.is_live = false;
                return Ok(cached_stats);
            }
        }
    }

    let stats = parse_session_stats_internal(&session_path)?;
    if !stats.is_live {
        upsert_session_stats_cache(connection, &session_id, current_mtime, &stats)?;
    }

    Ok(stats)
}

pub(crate) fn backfill_missing_stats_internal(
    connection: &Connection,
    _copilot_root: &Path,
) -> Result<usize, String> {
    let mut statement = connection
        .prepare(
            "
            SELECT sc.session_id, sc.session_dir
            FROM sessions_cache sc
            LEFT JOIN session_stats ss ON ss.session_id = sc.session_id
            WHERE sc.provider = 'copilot'
              AND ss.session_id IS NULL
              AND sc.has_events = 1
              AND sc.session_dir IS NOT NULL
              AND sc.session_dir != ''
            ORDER BY COALESCE(sc.updated_at, sc.created_at, '') DESC
            LIMIT 50
            ",
        )
        .map_err(|error| format!("failed to prepare missing stats query: {error}"))?;

    let session_rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| format!("failed to query missing stats rows: {error}"))?;

    let mut session_targets = Vec::new();
    for row in session_rows {
        session_targets
            .push(row.map_err(|error| format!("failed to read missing stats row: {error}"))?);
    }

    let mut processed_count = 0usize;
    for (session_id, session_dir) in session_targets {
        let session_path = PathBuf::from(&session_dir);
        let events_path = session_path.join("events.jsonl");

        if !events_path.exists() {
            continue;
        }

        match is_live_session(&session_path) {
            Ok(true) => continue,
            Ok(false) => {}
            Err(error) => {
                eprintln!(
                    "[stats-backfill] failed to inspect live state for {}: {}",
                    session_path.display(),
                    error
                );
                continue;
            }
        }

        let events_mtime = match session_events_mtime(&events_path) {
            Ok(value) => value,
            Err(error) => {
                eprintln!(
                    "[stats-backfill] failed to read events mtime for {}: {}",
                    session_path.display(),
                    error
                );
                continue;
            }
        };

        let stats = match parse_session_stats_internal(&session_path) {
            Ok(value) => value,
            Err(error) => {
                eprintln!(
                    "[stats-backfill] failed to parse {}: {}",
                    session_path.display(),
                    error
                );
                continue;
            }
        };

        if stats.is_live {
            continue;
        }

        if let Err(error) =
            upsert_session_stats_cache(connection, &session_id, events_mtime, &stats)
        {
            eprintln!(
                "[stats-backfill] failed to cache {}: {}",
                session_path.display(),
                error
            );
            continue;
        }

        processed_count += 1;
    }

    Ok(processed_count)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
