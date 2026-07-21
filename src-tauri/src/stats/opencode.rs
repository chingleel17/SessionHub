use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use rusqlite::{Connection, OpenFlags};

use crate::sessions::dir_mtime_secs;
use crate::types::*;

use super::{get_session_stats_cache, upsert_session_stats_cache};

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

fn parse_opencode_message_row(
    session_id: &str,
    message_id: &str,
    content: &str,
) -> Option<OpencodeMessage> {
    let mut value = serde_json::from_str::<serde_json::Value>(content).ok()?;
    let object = value.as_object_mut()?;
    object
        .entry("id".to_string())
        .or_insert_with(|| serde_json::Value::String(message_id.to_string()));
    object
        .entry("sessionID".to_string())
        .or_insert_with(|| serde_json::Value::String(session_id.to_string()));
    serde_json::from_value::<OpencodeMessage>(value).ok()
}

fn open_opencode_db_for_stats(storage_root: &Path) -> Option<Connection> {
    let db_path = storage_root.parent()?.join("opencode.db");
    if !db_path.exists() {
        return None;
    }
    Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY).ok()
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

    if let Some(connection) = open_opencode_db_for_stats(storage_root) {
        let mut statement = match connection
            .prepare("SELECT id, data FROM message WHERE session_id = ?1 ORDER BY time_created ASC")
        {
            Ok(statement) => statement,
            Err(_) => return Vec::new(),
        };
        let rows = match statement.query_map([session_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }) {
            Ok(rows) => rows,
            Err(_) => return Vec::new(),
        };
        return rows
            .flatten()
            .filter_map(|(message_id, content)| {
                parse_opencode_message_row(session_id, &message_id, &content)
            })
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
        if let Some(connection) = open_opencode_db_for_stats(storage_root) {
            let mut statement = match connection
                .prepare("SELECT data FROM part WHERE message_id = ?1 ORDER BY time_created ASC")
            {
                Ok(statement) => statement,
                Err(_) => return Vec::new(),
            };
            let rows = match statement.query_map([message_id], |row| row.get::<_, String>(0)) {
                Ok(rows) => rows,
                Err(_) => return Vec::new(),
            };
            return rows
                .flatten()
                .filter_map(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
                .filter_map(|value| {
                    if value.get("type").and_then(|v| v.as_str()) != Some("tool") {
                        return None;
                    }
                    Some(
                        value
                            .pointer("/state/tool")
                            .or_else(|| value.get("tool"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                    )
                })
                .collect();
        }
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
    let storage_root = message_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "cannot determine storage root from message_dir".to_string())?;
    let message_dir_exists = message_dir.exists();
    let db_mtime_secs = open_opencode_db_for_stats(storage_root)
        .and_then(|db| {
            db.query_row(
                "SELECT COALESCE(MAX(time_updated), 0) FROM message WHERE session_id = ?1",
                [session_id],
                |row| row.get::<_, i64>(0),
            )
            .ok()
        })
        .unwrap_or(0)
        / 1000;

    if !message_dir_exists && db_mtime_secs == 0 {
        return Ok(SessionStats::default());
    }

    let is_live = if message_dir_exists {
        is_opencode_session_live(message_dir)
    } else {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        db_mtime_secs > 0 && db_mtime_secs <= now && now - db_mtime_secs < 300
    };
    let dir_mtime = if message_dir_exists {
        dir_mtime_secs(message_dir).max(db_mtime_secs)
    } else {
        db_mtime_secs
    };

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
