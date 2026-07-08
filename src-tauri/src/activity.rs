use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::sessions::dir_mtime_secs;
use crate::settings::default_opencode_root;
use crate::types::*;

pub(crate) fn get_copilot_activity_status(
    session_dir: &Path,
    session_id: &str,
) -> SessionActivityStatus {
    let events_path = session_dir.join("events.jsonl");
    let not_started = SessionActivityStatus {
        session_id: session_id.to_string(),
        status: "idle".to_string(),
        detail: None,
        last_activity_at: None,
    };

    if !events_path.is_file() {
        return not_started;
    }

    // 使用 BufReader 逐行讀取，避免一次載入整個檔案到記憶體
    let file = match fs::File::open(&events_path) {
        Ok(f) => f,
        Err(_) => return not_started,
    };
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .map_while(Result::ok)
        .filter(|l| !l.trim().is_empty())
        .collect();

    // 只看最後 30 行，避免解析超大 events.jsonl
    let tail_start = lines.len().saturating_sub(30);
    let tail = &lines[tail_start..];

    let now = chrono::Utc::now();
    let mut last_type = String::new();
    let mut last_ts: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut last_tool: Option<String> = None;

    for raw in tail {
        let Ok(ev) = serde_json::from_str::<SessionEvent>(raw) else {
            continue;
        };
        if let Some(ref ts_str) = ev.timestamp {
            if let Ok(ts) = ts_str.parse::<chrono::DateTime<chrono::Utc>>() {
                last_ts = Some(ts);
            }
        }
        if ev.event_type == "tool.execution_start" {
            last_tool = ev
                .data
                .get("toolName")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }
        last_type = ev.event_type.clone();
    }

    let last_activity_at = last_ts.map(|t| t.to_rfc3339());
    let minutes_since = last_ts
        .map(|t| if t > now { 0 } else { (now - t).num_minutes() })
        .unwrap_or(i64::MAX);

    let (status, detail) = match last_type.as_str() {
        "session.task_complete" | "session.shutdown" => {
            ("done".to_string(), Some("completed".to_string()))
        }
        "assistant.turn_end" if minutes_since < 120 => ("waiting".to_string(), None),
        "tool.execution_start" | "assistant.turn_start" if minutes_since < 30 => {
            let detail = last_tool.as_deref().map(|tool| {
                match tool {
                    t if matches!(t, "edit" | "write" | "patch" | "create") => "file_op",
                    t if matches!(t, "task" | "subtask" | "call_omo_agent") => "sub_agent",
                    _ => "tool_call",
                }
                .to_string()
            });
            ("active".to_string(), detail)
        }
        _ if minutes_since < 30 && !last_type.is_empty() => {
            ("active".to_string(), Some("working".to_string()))
        }
        _ => ("idle".to_string(), None),
    };

    SessionActivityStatus {
        session_id: session_id.to_string(),
        status,
        detail,
        last_activity_at,
    }
}

pub(crate) fn get_opencode_activity_status(
    opencode_root: &Path,
    session_id: &str,
) -> SessionActivityStatus {
    let not_started = SessionActivityStatus {
        session_id: session_id.to_string(),
        status: "idle".to_string(),
        detail: None,
        last_activity_at: None,
    };

    let msg_dir = opencode_root
        .join("storage")
        .join("message")
        .join(session_id);

    if !msg_dir.is_dir() {
        return not_started;
    }

    let mut msg_files: Vec<PathBuf> = match fs::read_dir(&msg_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
            .collect(),
        Err(_) => return not_started,
    };
    msg_files.sort();

    let last_msg_path = match msg_files.last() {
        Some(p) => p,
        None => return not_started,
    };

    let msg: OpenCodeMessageFile = match fs::read_to_string(last_msg_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
    {
        Some(m) => m,
        None => return not_started,
    };

    let now = chrono::Utc::now();
    let completed_ms = msg.time.as_ref().and_then(|t| t.completed.or(t.created));
    let last_ts = completed_ms.and_then(|ms| {
        chrono::DateTime::from_timestamp(ms / 1000, ((ms % 1000) * 1_000_000) as u32)
    });
    let last_activity_at = last_ts.map(|t| t.to_rfc3339());
    let minutes_since = last_ts
        .map(|t| if t > now { 0 } else { (now - t).num_minutes() })
        .unwrap_or(i64::MAX);

    let (status, detail) = match msg.role.as_deref() {
        _ if minutes_since > 1440 => ("done".to_string(), Some("completed".to_string())),
        Some("assistant") if msg.finish.as_deref() == Some("stop") && minutes_since < 120 => {
            ("waiting".to_string(), None)
        }
        Some("user") if minutes_since < 30 => ("active".to_string(), Some("working".to_string())),
        Some("assistant") if msg.finish.as_deref() == Some("tool-calls") && minutes_since < 30 => {
            ("active".to_string(), Some("tool_call".to_string()))
        }
        _ if minutes_since < 30 => ("active".to_string(), Some("working".to_string())),
        _ => ("idle".to_string(), None),
    };

    SessionActivityStatus {
        session_id: session_id.to_string(),
        status,
        detail,
        last_activity_at,
    }
}

pub(crate) fn get_codex_activity_status(
    session_file: &Path,
    session_id: &str,
) -> SessionActivityStatus {
    let not_started = SessionActivityStatus {
        session_id: session_id.to_string(),
        status: "idle".to_string(),
        detail: None,
        last_activity_at: None,
    };

    if !session_file.is_file() {
        return not_started;
    }

    let file = match fs::File::open(session_file) {
        Ok(file) => file,
        Err(_) => return not_started,
    };
    let reader = BufReader::new(file);
    let mut last_ts: Option<chrono::DateTime<chrono::Utc>> = None;

    for line in reader.lines().map_while(Result::ok) {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        let Some(timestamp) = value.get("timestamp").and_then(|item| item.as_str()) else {
            continue;
        };
        if let Ok(parsed) = timestamp.parse::<chrono::DateTime<chrono::Utc>>() {
            last_ts = Some(parsed);
        }
    }

    let now = chrono::Utc::now();
    let last_activity_at = last_ts.map(|value| value.to_rfc3339());
    let minutes_since = last_ts
        .map(|value| {
            if value > now {
                0
            } else {
                (now - value).num_minutes()
            }
        })
        .unwrap_or(i64::MAX);

    let (status, detail) = if minutes_since < 30 {
        ("active".to_string(), Some("working".to_string()))
    } else {
        ("idle".to_string(), None)
    };

    SessionActivityStatus {
        session_id: session_id.to_string(),
        status,
        detail,
        last_activity_at,
    }
}

/// 取得 events.jsonl 或 session dir 的修改時間（秒），失敗回傳 0
fn events_mtime(session_dir: &Path, provider: &str) -> i64 {
    match provider {
        CODEX_PROVIDER => {
            // Codex: session_dir 本身是 JSONL 檔案
            fs::metadata(session_dir)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        }
        OPENCODE_PROVIDER => {
            // OpenCode: 以目錄 mtime 作為 staleness 判斷（粗略）
            dir_mtime_secs(session_dir)
        }
        _ => {
            // Copilot/Claude: events.jsonl 的 mtime
            let events_path = session_dir.join("events.jsonl");
            fs::metadata(&events_path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        }
    }
}

pub(crate) fn get_session_activity_statuses_internal(
    sessions: &[serde_json::Value],
    opencode_root: Option<&str>,
    activity_cache: &std::sync::Mutex<
        std::collections::HashMap<String, (i64, SessionActivityStatus)>,
    >,
) -> Vec<SessionActivityStatus> {
    let oc_root = opencode_root
        .map(PathBuf::from)
        .unwrap_or_else(|| default_opencode_root().unwrap_or_default());

    let mut cache = activity_cache.lock().unwrap_or_else(|e| e.into_inner());

    sessions
        .iter()
        .filter_map(|s| {
            let id = s.get("id")?.as_str()?;
            let provider = s
                .get("provider")
                .and_then(|v| v.as_str())
                .unwrap_or("copilot");
            let session_dir_str = s.get("sessionDir").and_then(|v| v.as_str()).unwrap_or("");

            let dir = match provider {
                OPENCODE_PROVIDER => oc_root.join("storage").join("message").join(id),
                _ => PathBuf::from(session_dir_str),
            };

            let current_mtime = events_mtime(&dir, provider);

            // 如果快取存在且 mtime 未變，直接回傳快取結果
            if let Some((cached_mtime, cached_status)) = cache.get(id) {
                if *cached_mtime == current_mtime && current_mtime > 0 {
                    return Some(cached_status.clone());
                }
            }

            let status = match provider {
                OPENCODE_PROVIDER => get_opencode_activity_status(&oc_root, id),
                CODEX_PROVIDER => get_codex_activity_status(&dir, id),
                _ => get_copilot_activity_status(&dir, id),
            };

            cache.insert(id.to_string(), (current_mtime, status.clone()));
            Some(status)
        })
        .collect()
}
