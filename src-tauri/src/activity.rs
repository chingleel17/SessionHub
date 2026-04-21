use std::fs;
use std::path::{Path, PathBuf};

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

    let content = match fs::read_to_string(&events_path) {
        Ok(c) => c,
        Err(_) => return not_started,
    };

    let lines: Vec<&str> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();

    let tail: Vec<&str> = lines.iter().rev().take(30).copied().collect();

    let now = chrono::Utc::now();
    let mut last_type = "";
    let mut last_ts: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut last_tool: Option<String> = None;

    for raw in tail.iter().rev() {
        let Ok(ev) = serde_json::from_str::<SessionEvent>(raw) else {
            continue;
        };
        if let Some(ref ts_str) = ev.timestamp {
            if let Ok(ts) = ts_str.parse::<chrono::DateTime<chrono::Utc>>() {
                last_ts = Some(ts);
            }
        }
        last_type = Box::leak(ev.event_type.clone().into_boxed_str());
        if ev.event_type == "tool.execution_start" {
            last_tool = ev
                .data
                .get("toolName")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }
    }

    let last_activity_at = last_ts.map(|t| t.to_rfc3339());
    let minutes_since = last_ts
        .map(|t| {
            if t > now {
                0
            } else {
                (now - t).num_minutes()
            }
        })
        .unwrap_or(i64::MAX);

    let (status, detail) = match last_type {
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
    let completed_ms = msg
        .time
        .as_ref()
        .and_then(|t| t.completed.or(t.created));
    let last_ts = completed_ms.and_then(|ms| {
        chrono::DateTime::from_timestamp(ms / 1000, ((ms % 1000) * 1_000_000) as u32)
    });
    let last_activity_at = last_ts.map(|t| t.to_rfc3339());
    let minutes_since = last_ts
        .map(|t| {
            if t > now {
                0
            } else {
                (now - t).num_minutes()
            }
        })
        .unwrap_or(i64::MAX);

    let (status, detail) = match msg.role.as_deref() {
        _ if minutes_since > 1440 => ("done".to_string(), Some("completed".to_string())),
        Some("assistant") if msg.finish.as_deref() == Some("stop") && minutes_since < 120 => {
            ("waiting".to_string(), None)
        }
        Some("user") if minutes_since < 30 => {
            ("active".to_string(), Some("working".to_string()))
        }
        Some("assistant")
            if msg.finish.as_deref() == Some("tool-calls") && minutes_since < 30 =>
        {
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

pub(crate) fn get_session_activity_statuses_internal(
    sessions: &[serde_json::Value],
    opencode_root: Option<&str>,
) -> Vec<SessionActivityStatus> {
    let oc_root = opencode_root
        .map(PathBuf::from)
        .unwrap_or_else(|| default_opencode_root().unwrap_or_default());

    sessions
        .iter()
        .filter_map(|s| {
            let id = s.get("id")?.as_str()?;
            let provider = s.get("provider").and_then(|v| v.as_str()).unwrap_or("copilot");
            let session_dir_str = s.get("sessionDir").and_then(|v| v.as_str()).unwrap_or("");

            if provider == "opencode" {
                Some(get_opencode_activity_status(&oc_root, id))
            } else {
                let dir = PathBuf::from(session_dir_str);
                Some(get_copilot_activity_status(&dir, id))
            }
        })
        .collect()
}
