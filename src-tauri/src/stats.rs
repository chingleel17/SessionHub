use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use rusqlite::{params, Connection};

use crate::sessions::dir_mtime_secs;
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
            let tool_breakdown =
                serde_json::from_str::<BTreeMap<String, u32>>(&tool_breakdown_json)
                    .map_err(|error| {
                        format!("failed to deserialize cached tool_breakdown: {error}")
                    })?;
            let model_metrics_json: String = row.get(9).unwrap_or_else(|_| "{}".to_string());
            let model_metrics = serde_json::from_str::<BTreeMap<String, ModelMetricsEntry>>(
                &model_metrics_json,
            )
            .unwrap_or_default();

            Ok(Some((
                events_mtime,
                SessionStats {
                    output_tokens: row.get(1).map_err(|error| {
                        format!("failed to read output_tokens column: {error}")
                    })?,
                    input_tokens: row.get(8).map_err(|error| {
                        format!("failed to read input_tokens column: {error}")
                    })?,
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

// ── OpenCode: message 目錄解析 ────────────────────────────────────────────────

/// 判斷 session_dir 是否為 OpenCode session
pub(crate) fn is_opencode_session_dir(session_dir: &Path) -> bool {
    let name = session_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if name.starts_with("ses_") {
        return true;
    }
    // ULID：26 個字元，全為大寫英數字（0-9A-Z）
    if name.len() == 26
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return true;
    }
    false
}

/// 判斷 OpenCode message 目錄是否為活躍狀態（最近 5 分鐘內有修改）
pub(crate) fn is_opencode_session_live(message_dir: &Path) -> bool {
    let mtime = dir_mtime_secs(message_dir);
    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    if mtime > now {
        false
    } else {
        now - mtime < 300
    }
}

fn parse_opencode_message_json(path: &Path) -> Option<OpencodeMessage> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str::<OpencodeMessage>(&content).ok()
}

/// 掃描 storage/message/ 全量，回傳屬於 session_id 的所有訊息
fn scan_opencode_messages_for_session(
    storage_root: &Path,
    session_id: &str,
) -> Vec<OpencodeMessage> {
    let message_storage = storage_root.join("message");

    let direct_dir = message_storage.join(session_id);
    if direct_dir.is_dir() {
        let Ok(entries) = fs::read_dir(&direct_dir) else {
            return Vec::new();
        };
        return entries
            .flatten()
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
            .filter_map(|e| parse_opencode_message_json(&e.path()))
            .collect();
    }

    let Ok(all_dirs) = fs::read_dir(&message_storage) else {
        return Vec::new();
    };
    let mut messages = Vec::new();
    for dir_entry in all_dirs.flatten() {
        let dir = dir_entry.path();
        if !dir.is_dir() {
            continue;
        }
        let Ok(files) = fs::read_dir(&dir) else {
            continue;
        };
        for file_entry in files.flatten() {
            let path = file_entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Some(msg) = parse_opencode_message_json(&path) {
                if msg.session_id.as_deref() == Some(session_id) {
                    messages.push(msg);
                }
            }
        }
    }
    messages
}

/// 掃描 storage/part/{messageID}/ 目錄，回傳 type=tool 的工具名稱清單
fn scan_opencode_parts_for_message(storage_root: &Path, message_id: &str) -> Vec<String> {
    let part_dir = storage_root.join("part").join(message_id);
    if !part_dir.exists() {
        return Vec::new();
    }
    let Ok(entries) = fs::read_dir(&part_dir) else {
        return Vec::new();
    };
    let mut tools = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };
        if value.get("type").and_then(|v| v.as_str()) == Some("tool") {
            let tool_name = value
                .pointer("/state/tool")
                .or_else(|| value.get("tool"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            tools.push(tool_name);
        }
    }
    tools
}

/// 計算 OpenCode session 的統計資料
pub(crate) fn calculate_opencode_session_stats(message_dir: &Path) -> Result<SessionStats, String> {
    let storage_root = message_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "cannot determine storage root from message_dir".to_string())?;

    let session_id = message_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let messages = scan_opencode_messages_for_session(storage_root, session_id);

    if messages.is_empty() {
        return Ok(SessionStats::default());
    }

    let mut stats = SessionStats::default();
    let mut models_used = BTreeSet::new();
    let mut min_time: Option<i64> = None;
    let mut max_time: Option<i64> = None;

    for msg in &messages {
        if let Some(time) = msg.time() {
            let t = time.completed.or(time.created);
            if let Some(t) = t {
                min_time = Some(min_time.map_or(t, |m: i64| m.min(t)));
                max_time = Some(max_time.map_or(t, |m: i64| m.max(t)));
            }
        }

        match msg.role.as_str() {
            "user" => {
                stats.interaction_count += 1;
            }
            "assistant" => {
                if let Some(tokens) = msg.tokens() {
                    stats.output_tokens += tokens.effective_output();
                    stats.input_tokens += tokens.effective_input();
                    if tokens.effective_reasoning() > 0 {
                        stats.reasoning_count += 1;
                    }
                }
                if let Some(model) = msg.model_id() {
                    models_used.insert(model.to_string());
                }
                let tool_names = scan_opencode_parts_for_message(storage_root, &msg.id);
                for tool_name in tool_names {
                    stats.tool_call_count += 1;
                    *stats.tool_breakdown.entry(tool_name).or_insert(0) += 1;
                }
            }
            _ => {}
        }
    }

    stats.models_used = models_used.into_iter().collect();

    if let (Some(min_t), Some(max_t)) = (min_time, max_time) {
        let diff_ms = max_t - min_t;
        if diff_ms > 0 {
            stats.duration_minutes = (diff_ms as u64) / 60_000;
        }
    }

    Ok(stats)
}

pub(crate) fn get_opencode_session_stats_internal(
    connection: &Connection,
    message_dir: &Path,
    session_id: &str,
) -> Result<SessionStats, String> {
    if !message_dir.exists() {
        return Ok(SessionStats::default());
    }

    let is_live = is_opencode_session_live(message_dir);
    let dir_mtime = dir_mtime_secs(message_dir);

    if !is_live {
        if let Some((cached_mtime, mut cached_stats)) =
            get_session_stats_cache(connection, session_id)?
        {
            if cached_mtime == dir_mtime {
                cached_stats.is_live = false;
                return Ok(cached_stats);
            }
        }
    }

    let mut stats = calculate_opencode_session_stats(message_dir)?;
    stats.is_live = is_live;

    if !stats.is_live {
        upsert_session_stats_cache(connection, session_id, dir_mtime, &stats)?;
    }

    Ok(stats)
}

pub(crate) fn get_session_stats_internal(
    connection: &Connection,
    session_dir: &str,
) -> Result<SessionStats, String> {
    let session_path = PathBuf::from(session_dir);
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
