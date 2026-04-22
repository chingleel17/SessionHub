use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use rusqlite::Connection;

use crate::db::read_session_meta;
use crate::types::*;

/// 將 unix timestamp（毫秒）轉換為 ISO 8601 字串
pub(crate) fn unix_ms_to_iso8601(timestamp_ms: i64) -> Option<String> {
    let timestamp_secs = timestamp_ms / 1000;
    let nanos = ((timestamp_ms % 1000) * 1_000_000) as u32;

    chrono::DateTime::from_timestamp(timestamp_secs, nanos)
        .map(|datetime| datetime.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}

/// 讀取 storage/project/*.json，回傳 HashMap<project_id, worktree>
pub(crate) fn load_opencode_projects(opencode_root: &Path) -> HashMap<String, Option<String>> {
    let project_dir = opencode_root.join("storage").join("project");
    let mut map = HashMap::new();
    let Ok(entries) = fs::read_dir(&project_dir) else {
        return map;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(project) = serde_json::from_str::<OpencodeProjectJson>(&content) else {
            continue;
        };
        map.insert(project.id, project.worktree);
    }
    map
}

/// 掃描 storage/message/，回傳有訊息的 session_id HashSet
pub(crate) fn build_opencode_events_index(opencode_root: &Path) -> HashSet<String> {
    let message_dir = opencode_root.join("storage").join("message");
    let mut set = HashSet::new();
    let Ok(entries) = fs::read_dir(&message_dir) else {
        return set;
    };
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let Ok(files) = fs::read_dir(&dir) else {
            continue;
        };
        for file_entry in files.flatten() {
            let file_path = file_entry.path();
            if file_path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Ok(content) = fs::read_to_string(&file_path) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
                continue;
            };
            if let Some(session_id) = value.get("sessionID").and_then(|v| v.as_str()) {
                set.insert(session_id.to_string());
            }
            break; // 每個 message dir 只需讀一個 JSON
        }
    }
    set
}

/// 掃描 OpenCode JSON 檔案中的 session，映射為 Vec<SessionInfo>
pub(crate) fn scan_opencode_sessions_internal(
    opencode_root: &Path,
    show_archived: bool,
    metadata_conn: &Connection,
) -> Result<Vec<SessionInfo>, String> {
    let storage_root = opencode_root.join("storage");
    let session_storage = storage_root.join("session");

    if !session_storage.exists() {
        return Ok(Vec::new());
    }

    let projects = load_opencode_projects(opencode_root);
    let events_index = build_opencode_events_index(opencode_root);

    let mut sessions = Vec::new();

    let Ok(project_dirs) = fs::read_dir(&session_storage) else {
        return Ok(Vec::new());
    };

    for project_entry in project_dirs.flatten() {
        let project_dir = project_entry.path();
        if !project_dir.is_dir() {
            continue;
        }

        let project_id = project_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let worktree = projects.get(&project_id).and_then(|w| w.clone());

        let Ok(session_files) = fs::read_dir(&project_dir) else {
            continue;
        };

        for session_entry in session_files.flatten() {
            let session_path = session_entry.path();
            if session_path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let Ok(content) = fs::read_to_string(&session_path) else {
                continue;
            };
            let Ok(session) = serde_json::from_str::<OpencodeSessionJson>(&content) else {
                continue;
            };

            let time = session.time.as_ref();
            let is_archived = time.and_then(|t| t.archived).is_some();

            if !show_archived && is_archived {
                continue;
            }

            let created_at = time.and_then(|t| t.created).and_then(unix_ms_to_iso8601);
            let updated_at = time.and_then(|t| t.updated).and_then(unix_ms_to_iso8601);

            let summary_count = session.summary.as_ref().and_then(|s| {
                let total =
                    s.additions.unwrap_or(0) + s.deletions.unwrap_or(0) + s.files.unwrap_or(0);
                if total > 0 {
                    Some(total as u32)
                } else {
                    None
                }
            });

            let cwd = session.directory.or(worktree.clone());

            let meta = read_session_meta(metadata_conn, &session.id).unwrap_or(SessionMeta {
                notes: None,
                tags: Vec::new(),
            });

            let message_dir = storage_root.join("message").join(&session.id);
            let has_events = events_index.contains(&session.id)
                || message_dir
                    .read_dir()
                    .map(|mut e| e.next().is_some())
                    .unwrap_or(false);

            sessions.push(SessionInfo {
                id: session.id.clone(),
                provider: "opencode".to_string(),
                cwd,
                summary: session.title,
                summary_count,
                created_at,
                updated_at,
                session_dir: message_dir.to_string_lossy().to_string(),
                parse_error: false,
                is_archived,
                notes: meta.notes,
                tags: meta.tags,
                has_plan: false,
                has_events,
            });
        }
    }

    Ok(sessions)
}

/// OpenCode 增量掃描：只處理 time.updated > last_cursor 的 session
pub(crate) fn scan_opencode_incremental_internal(
    opencode_root: &Path,
    show_archived: bool,
    metadata_conn: &Connection,
    cache: &mut ProviderCache,
) -> Result<(), String> {
    let projects = load_opencode_projects(opencode_root);
    if projects.is_empty() {
        return Ok(());
    }

    let mut new_cursor = cache.last_cursor;

    for (project_id, worktree) in &projects {
        let session_dir = opencode_root
            .join("storage")
            .join("session")
            .join(project_id);
        let entries = match fs::read_dir(&session_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let session: OpencodeSessionJson = match serde_json::from_str(&content) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let time_updated = session.time.as_ref().and_then(|t| t.updated).unwrap_or(0);
            if time_updated <= cache.last_cursor {
                continue;
            }
            let time_archived = session.time.as_ref().and_then(|t| t.archived);
            let is_archived = time_archived.is_some();
            if !show_archived && is_archived {
                continue;
            }
            let time_created = session.time.as_ref().and_then(|t| t.created);
            let created_at = time_created.and_then(unix_ms_to_iso8601);
            let updated_at = unix_ms_to_iso8601(time_updated);
            let worktree_val = worktree.clone();
            let session_summary = session.title.clone();
            let summary_count = session.summary.as_ref().and_then(|s| {
                let total =
                    s.additions.unwrap_or(0) + s.deletions.unwrap_or(0) + s.files.unwrap_or(0);
                if total > 0 {
                    Some(total as u32)
                } else {
                    None
                }
            });
            let meta = read_session_meta(metadata_conn, &session.id).unwrap_or(SessionMeta {
                notes: None,
                tags: Vec::new(),
            });
            let session_dir_path = opencode_root
                .join("storage")
                .join("message")
                .join(&session.id);
            let has_events = session_dir_path
                .read_dir()
                .map(|mut e| e.next().is_some())
                .unwrap_or(false);
            let info = SessionInfo {
                id: session.id.clone(),
                provider: OPENCODE_PROVIDER.to_string(),
                cwd: worktree_val,
                summary: session_summary,
                summary_count,
                created_at,
                updated_at,
                session_dir: session_dir_path.to_string_lossy().to_string(),
                parse_error: false,
                is_archived,
                notes: meta.notes,
                tags: meta.tags,
                has_plan: false,
                has_events,
            };
            if let Some(pos) = cache.sessions.iter().position(|s| s.id == session.id) {
                cache.sessions[pos] = info;
            } else {
                cache.sessions.push(info);
            }
            if time_updated > new_cursor {
                new_cursor = time_updated;
            }
        }
    }

    cache.last_cursor = new_cursor;
    Ok(())
}

/// 從 OpenCode 取得最大 time_updated 值
pub(crate) fn get_opencode_max_cursor(_opencode_root: &Path) -> Result<i64, String> {
    // JSON-based storage: no SQL cursor; always return 0
    Ok(0)
}
