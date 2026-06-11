use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db::read_session_meta;
use crate::sessions::dir_mtime_secs;
use crate::types::*;

fn claude_projects_root(claude_root: &Path) -> PathBuf {
    claude_root.join("projects")
}

fn collect_claude_session_files(projects_root: &Path) -> Result<Vec<PathBuf>, String> {
    if !projects_root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let mut stack = vec![projects_root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 跳過 subagents 目錄，避免掃描子 agent 的 JSONL 檔案
                let dir_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
                if dir_name != "subagents" {
                    stack.push(path);
                }
            } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                files.push(path);
            }
        }
    }

    files.sort();
    Ok(files)
}

fn parse_claude_session_file(session_path: &Path, meta: SessionMeta) -> SessionInfo {
    let fallback_id = session_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| session_path.to_string_lossy().to_string());

    let fallback_updated = fs::metadata(session_path)
        .and_then(|m| m.modified())
        .ok()
        .map(|t| {
            chrono::DateTime::<chrono::Utc>::from(t)
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        });

    let file = match fs::File::open(session_path) {
        Ok(f) => f,
        Err(_) => {
            return SessionInfo {
                id: fallback_id,
                provider: CLAUDE_PROVIDER.to_string(),
                cwd: None,
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
    let mut summary: Option<String> = None;

    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let Ok(entry) = serde_json::from_str::<ClaudeEntry>(trimmed) else {
            continue;
        };

        has_events = true;

        if let Some(ts) = &entry.timestamp {
            updated_at = Some(ts.clone());
            if created_at.is_none() {
                created_at = Some(ts.clone());
            }
        }

        if session_id.is_none() {
            session_id = entry.session_id.clone();
        }

        if cwd.is_none() {
            cwd = entry.cwd.clone();
        }

        if entry.entry_type == "user" && summary.is_none() {
            if let Some(msg) = &entry.message {
                if let Some(role) = &msg.role {
                    if role == "user" {
                        summary = entry.session_id.clone();
                    }
                }
            }
        }
    }

    let resolved_id = session_id.unwrap_or_else(|| fallback_id.clone());
    let parse_error = !has_events;

    SessionInfo {
        id: resolved_id.clone(),
        provider: CLAUDE_PROVIDER.to_string(),
        cwd,
        summary: Some(resolved_id),
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

pub(crate) fn build_claude_session_mtimes(sessions: &[SessionInfo]) -> HashMap<String, i64> {
    sessions
        .iter()
        .map(|s| {
            (
                s.session_dir.clone(),
                dir_mtime_secs(Path::new(&s.session_dir)),
            )
        })
        .collect()
}

pub(crate) fn scan_claude_sessions_internal(
    claude_root: &Path,
    _show_archived: bool,
    connection: &Connection,
) -> Result<Vec<SessionInfo>, String> {
    let projects_root = claude_projects_root(claude_root);
    let session_files = collect_claude_session_files(&projects_root)?;
    let mut sessions = Vec::new();

    for session_path in session_files {
        let mut session = parse_claude_session_file(
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

pub(crate) fn scan_claude_incremental_internal(
    claude_root: &Path,
    _show_archived: bool,
    connection: &Connection,
    cache: &mut ProviderCache,
) -> Result<(), String> {
    let projects_root = claude_projects_root(claude_root);
    let session_files = collect_claude_session_files(&projects_root)?;
    let mut current_ids: HashSet<String> = HashSet::new();

    for session_path in session_files {
        let cache_key = session_path.to_string_lossy().to_string();
        let current_mtime = dir_mtime_secs(&session_path);
        let cached_mtime = cache.session_mtimes.get(&cache_key).copied().unwrap_or(-1);

        if current_mtime != cached_mtime {
            let mut info = parse_claude_session_file(
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
            if let Some(pos) = cache.sessions.iter().position(|s| s.id == info.id) {
                cache.sessions[pos] = info.clone();
            } else {
                cache.sessions.push(info.clone());
            }
            cache.session_mtimes.insert(cache_key, current_mtime);
        } else if let Some(session) = cache
            .sessions
            .iter()
            .find(|s| s.session_dir == cache_key)
        {
            current_ids.insert(session.id.clone());
        }
    }

    cache
        .sessions
        .retain(|s| current_ids.contains(&s.id));
    let current_paths: HashSet<String> = cache
        .sessions
        .iter()
        .map(|s| s.session_dir.clone())
        .collect();
    cache
        .session_mtimes
        .retain(|k, _| current_paths.contains(k));

    Ok(())
}
