use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db::read_session_meta;
use crate::sessions::dir_mtime_secs;
use crate::types::*;

fn codex_sessions_root(codex_root: &Path) -> PathBuf {
    codex_root.join("sessions")
}

fn is_codex_session_file(path: &Path) -> bool {
    path.is_file() && path.extension().and_then(|value| value.to_str()) == Some("jsonl")
}

fn collect_codex_session_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir)
            .map_err(|error| format!("failed to read {}: {error}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("failed to read codex entry: {error}"))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if is_codex_session_file(&path) {
                files.push(path);
            }
        }
    }

    files.sort();
    Ok(files)
}

fn session_id_from_path(path: &Path) -> String {
    path.file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

fn parse_codex_timestamp(value: &serde_json::Value) -> Option<String> {
    value
        .get("timestamp")
        .and_then(|timestamp| timestamp.as_str())
        .map(|timestamp| timestamp.to_string())
}

fn parse_codex_session_file(session_path: &Path, meta: SessionMeta) -> SessionInfo {
    let fallback_id = session_id_from_path(session_path);
    let fallback_updated = fs::metadata(session_path)
        .and_then(|value| value.modified())
        .ok()
        .map(|value| {
            chrono::DateTime::<chrono::Utc>::from(value)
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        })
        .or_else(|| Some(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)));

    let file = match fs::File::open(session_path) {
        Ok(file) => file,
        Err(_) => {
            return SessionInfo {
                id: fallback_id,
                provider: CODEX_PROVIDER.to_string(),
                cwd: None,
                repo_root: None,
                repo_name: None,
                git_branch: None,
                summary: None,
                summary_count: None,
                created_at: None,
                updated_at: fallback_updated,
                session_dir: session_path.to_string_lossy().to_string(),
                parse_error: true,
                is_archived: false,
                notes: meta.notes,
                tags: meta.tags,
                has_plan: false,
                has_events: false,
            };
        }
    };

    let reader = BufReader::new(file);
    let mut session_id: Option<String> = None;
    let mut cwd: Option<String> = None;
    let mut created_at: Option<String> = None;
    let mut updated_at = fallback_updated;
    let mut has_events = false;

    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        has_events = true;

        let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
            continue;
        };

        if let Some(timestamp) = parse_codex_timestamp(&value) {
            updated_at = Some(timestamp.clone());
            if created_at.is_none() {
                created_at = Some(timestamp);
            }
        }

        if value.get("type").and_then(|kind| kind.as_str()) != Some("session_meta") {
            continue;
        }

        let payload = value.get("payload").and_then(|payload| payload.as_object());
        if let Some(payload) = payload {
            if session_id.is_none() {
                session_id = payload
                    .get("id")
                    .and_then(|item| item.as_str())
                    .map(|item| item.to_string());
            }
            if cwd.is_none() {
                cwd = payload
                    .get("cwd")
                    .and_then(|item| item.as_str())
                    .map(|item| item.to_string());
            }
            if let Some(timestamp) = payload
                .get("timestamp")
                .and_then(|item| item.as_str())
                .map(|item| item.to_string())
            {
                created_at = Some(timestamp);
            }
        }
    }

    let resolved_id = session_id.unwrap_or_else(|| fallback_id.clone());
    let parse_error = resolved_id == fallback_id && cwd.is_none() && created_at.is_none();

    SessionInfo {
        id: resolved_id,
        provider: CODEX_PROVIDER.to_string(),
        cwd,
        repo_root: None,
        repo_name: None,
        git_branch: None,
        summary: Some(fallback_id),
        summary_count: None,
        created_at,
        updated_at,
        session_dir: session_path.to_string_lossy().to_string(),
        parse_error,
        is_archived: false,
        notes: meta.notes,
        tags: meta.tags,
        has_plan: false,
        has_events,
    }
}

pub(crate) fn build_codex_session_mtimes(sessions: &[SessionInfo]) -> HashMap<String, i64> {
    sessions
        .iter()
        .map(|session| {
            (
                session.session_dir.clone(),
                dir_mtime_secs(Path::new(&session.session_dir)),
            )
        })
        .collect()
}

pub(crate) fn scan_codex_sessions_internal(
    codex_root: &Path,
    _show_archived: bool,
    connection: &Connection,
) -> Result<Vec<SessionInfo>, String> {
    let sessions_root = codex_sessions_root(codex_root);
    let session_files = collect_codex_session_files(&sessions_root)?;
    let mut sessions = Vec::new();

    for session_path in session_files {
        let mut session = parse_codex_session_file(
            &session_path,
            SessionMeta {
                notes: None,
                tags: Vec::new(),
            },
        );
        let meta = read_session_meta(connection, &session.id)?;
        session.notes = meta.notes;
        session.tags = meta.tags;
        sessions.push(session);
    }

    Ok(sessions)
}

pub(crate) fn scan_codex_incremental_internal(
    codex_root: &Path,
    _show_archived: bool,
    connection: &Connection,
    cache: &mut ProviderCache,
) -> Result<(), String> {
    let sessions_root = codex_sessions_root(codex_root);
    let session_files = collect_codex_session_files(&sessions_root)?;
    let mut current_ids: HashSet<String> = HashSet::new();

    for session_path in session_files {
        let cache_key = session_path.to_string_lossy().to_string();
        let current_mtime = dir_mtime_secs(&session_path);
        let cached_mtime = cache.session_mtimes.get(&cache_key).copied().unwrap_or(-1);

        if current_mtime != cached_mtime {
            let mut info = parse_codex_session_file(
                &session_path,
                SessionMeta {
                    notes: None,
                    tags: Vec::new(),
                },
            );
            let meta = read_session_meta(connection, &info.id)?;
            info.notes = meta.notes;
            info.tags = meta.tags;
            current_ids.insert(info.id.clone());
            if let Some(pos) = cache
                .sessions
                .iter()
                .position(|session| session.id == info.id)
            {
                cache.sessions[pos] = info.clone();
            } else {
                cache.sessions.push(info.clone());
            }
            cache.session_mtimes.insert(cache_key, current_mtime);
        } else {
            if let Some(session) = cache
                .sessions
                .iter()
                .find(|session| session.session_dir == cache_key)
            {
                current_ids.insert(session.id.clone());
            }
        }
    }

    cache
        .sessions
        .retain(|session| current_ids.contains(&session.id));
    let current_paths: HashSet<String> = cache
        .sessions
        .iter()
        .map(|session| session.session_dir.clone())
        .collect();
    cache
        .session_mtimes
        .retain(|session_dir, _| current_paths.contains(session_dir));

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::types::{SessionMeta, CODEX_PROVIDER};

    use super::parse_codex_session_file;

    fn create_temp_file(name: &str, content: &str) -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("sessionhub-codex-{name}-{suffix}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        let file_path = dir.join("sample.jsonl");
        fs::write(&file_path, content).expect("write codex session file");
        file_path
    }

    #[test]
    fn parse_codex_session_file_reads_session_meta_and_last_timestamp() {
        let file_path = create_temp_file(
            "parse",
            concat!(
                r#"{"timestamp":"2026-06-02T03:52:56.263Z","type":"session_meta","payload":{"id":"codex-001","timestamp":"2026-06-02T03:48:25.741Z","cwd":"D:\\repo"}}"#,
                "\n",
                r#"{"timestamp":"2026-06-02T03:59:00.000Z","type":"event_msg","payload":{"type":"user_message"}}"#,
                "\n"
            ),
        );

        let session = parse_codex_session_file(
            &file_path,
            SessionMeta {
                notes: None,
                tags: Vec::new(),
            },
        );

        assert_eq!(session.id, "codex-001");
        assert_eq!(session.provider, CODEX_PROVIDER);
        assert_eq!(session.cwd.as_deref(), Some("D:\\repo"));
        assert_eq!(
            session.created_at.as_deref(),
            Some("2026-06-02T03:48:25.741Z")
        );
        assert_eq!(
            session.updated_at.as_deref(),
            Some("2026-06-02T03:59:00.000Z")
        );
        assert!(!session.parse_error);
        assert!(session.has_events);

        fs::remove_file(&file_path).ok();
        fs::remove_dir_all(file_path.parent().expect("temp parent")).ok();
    }
}
