use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

use crate::db::read_session_meta;
use crate::types::*;

/// 將 unix timestamp（毫秒）轉換為 ISO 8601 字串
pub(crate) fn unix_ms_to_iso8601(timestamp_ms: i64) -> Option<String> {
    let timestamp_secs = timestamp_ms / 1000;
    let nanos = ((timestamp_ms % 1000) * 1_000_000) as u32;

    chrono::DateTime::from_timestamp(timestamp_secs, nanos)
        .map(|datetime| datetime.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}

fn opencode_db_path(opencode_root: &Path) -> PathBuf {
    opencode_root.join("opencode.db")
}

fn message_dir_for_session(opencode_root: &Path, session_id: &str) -> PathBuf {
    opencode_root
        .join("storage")
        .join("message")
        .join(session_id)
}

fn summary_count(
    additions: Option<i64>,
    deletions: Option<i64>,
    files: Option<i64>,
) -> Option<u32> {
    let total = additions.unwrap_or(0) + deletions.unwrap_or(0) + files.unwrap_or(0);
    if total > 0 {
        Some(total as u32)
    } else {
        None
    }
}

fn open_opencode_db(opencode_root: &Path) -> Result<Option<Connection>, String> {
    let db_path = opencode_db_path(opencode_root);
    if !db_path.exists() {
        return Ok(None);
    }

    Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map(Some)
        .map_err(|error| format!("failed to open OpenCode db {}: {error}", db_path.display()))
}

fn has_table(connection: &Connection, table: &str) -> Result<bool, String> {
    connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1 LIMIT 1",
            [table],
            |_row| Ok(()),
        )
        .map(|_| true)
        .or_else(|error| {
            if matches!(error, rusqlite::Error::QueryReturnedNoRows) {
                Ok(false)
            } else {
                Err(format!(
                    "failed to inspect OpenCode db table {table}: {error}"
                ))
            }
        })
}

fn build_opencode_session_info(
    opencode_root: &Path,
    metadata_conn: &Connection,
    id: String,
    cwd: Option<String>,
    title: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
    is_archived: bool,
    summary_count: Option<u32>,
    has_events: bool,
) -> SessionInfo {
    let meta = read_session_meta(metadata_conn, &id).unwrap_or(SessionMeta {
        notes: None,
        tags: Vec::new(),
    });

    SessionInfo {
        id: id.clone(),
        provider: OPENCODE_PROVIDER.to_string(),
        cwd,
        repo_root: None,
        repo_name: None,
        git_branch: None,
        summary: title,
        summary_count,
        created_at,
        updated_at,
        session_dir: message_dir_for_session(opencode_root, &id)
            .to_string_lossy()
            .to_string(),
        parse_error: false,
        is_archived,
        notes: meta.notes,
        tags: meta.tags,
        has_plan: false,
        has_events,
    }
}

fn scan_opencode_sessions_from_db_internal(
    opencode_root: &Path,
    show_archived: bool,
    metadata_conn: &Connection,
) -> Result<Option<Vec<SessionInfo>>, String> {
    let Some(connection) = open_opencode_db(opencode_root)? else {
        return Ok(None);
    };
    if !has_table(&connection, "session")? || !has_table(&connection, "project")? {
        return Ok(None);
    }

    let mut statement = connection
        .prepare(
            "
            SELECT
              s.id,
              s.title,
              s.directory,
              s.time_created,
              s.time_updated,
              s.time_archived,
              s.summary_additions,
              s.summary_deletions,
              s.summary_files,
              p.worktree,
              EXISTS(SELECT 1 FROM message m WHERE m.session_id = s.id) AS has_events
            FROM session s
            LEFT JOIN project p ON p.id = s.project_id
            ORDER BY s.time_updated DESC
            ",
        )
        .map_err(|error| format!("failed to prepare OpenCode db session query: {error}"))?;

    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, Option<i64>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, i64>(10)?,
            ))
        })
        .map_err(|error| format!("failed to query OpenCode db sessions: {error}"))?;

    let mut sessions = Vec::new();
    for row in rows {
        let (
            id,
            title,
            directory,
            created_ms,
            updated_ms,
            archived_ms,
            adds,
            dels,
            files,
            worktree,
            has_events,
        ) = row.map_err(|error| format!("failed to read OpenCode db session row: {error}"))?;
        let is_archived = archived_ms.is_some();
        if !show_archived && is_archived {
            continue;
        }

        sessions.push(build_opencode_session_info(
            opencode_root,
            metadata_conn,
            id,
            Some(directory).or(worktree),
            Some(title),
            unix_ms_to_iso8601(created_ms),
            unix_ms_to_iso8601(updated_ms),
            is_archived,
            summary_count(adds, dels, files),
            has_events != 0,
        ));
    }

    Ok(Some(sessions))
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
            break;
        }
    }
    set
}

/// 掃描 OpenCode session，優先讀取 DB，失敗時退回 JSON storage
pub(crate) fn scan_opencode_sessions_internal(
    opencode_root: &Path,
    show_archived: bool,
    metadata_conn: &Connection,
) -> Result<Vec<SessionInfo>, String> {
    if let Some(sessions) =
        scan_opencode_sessions_from_db_internal(opencode_root, show_archived, metadata_conn)?
    {
        return Ok(sessions);
    }

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
            let summary_count = session
                .summary
                .as_ref()
                .and_then(|s| summary_count(s.additions, s.deletions, s.files));
            let cwd = session.directory.or(worktree.clone());
            let message_dir = storage_root.join("message").join(&session.id);
            let has_events = events_index.contains(&session.id)
                || message_dir
                    .read_dir()
                    .map(|mut e| e.next().is_some())
                    .unwrap_or(false);

            sessions.push(build_opencode_session_info(
                opencode_root,
                metadata_conn,
                session.id,
                cwd,
                session.title,
                created_at,
                updated_at,
                is_archived,
                summary_count,
                has_events,
            ));
        }
    }

    Ok(sessions)
}

fn scan_opencode_incremental_from_db_internal(
    opencode_root: &Path,
    show_archived: bool,
    metadata_conn: &Connection,
    cache: &mut ProviderCache,
) -> Result<bool, String> {
    let Some(connection) = open_opencode_db(opencode_root)? else {
        return Ok(false);
    };
    if !has_table(&connection, "session")? || !has_table(&connection, "project")? {
        return Ok(false);
    }

    let mut new_cursor = cache.last_cursor;
    let mut statement = connection
        .prepare(
            "
            SELECT
              s.id,
              s.title,
              s.directory,
              s.time_created,
              s.time_updated,
              s.time_archived,
              s.summary_additions,
              s.summary_deletions,
              s.summary_files,
              p.worktree,
              EXISTS(SELECT 1 FROM message m WHERE m.session_id = s.id) AS has_events
            FROM session s
            LEFT JOIN project p ON p.id = s.project_id
            WHERE s.time_updated > ?1
            ORDER BY s.time_updated ASC
            ",
        )
        .map_err(|error| format!("failed to prepare OpenCode db incremental query: {error}"))?;

    let rows = statement
        .query_map([cache.last_cursor], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, Option<i64>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, i64>(10)?,
            ))
        })
        .map_err(|error| format!("failed to query OpenCode db incremental rows: {error}"))?;

    for row in rows {
        let (
            id,
            title,
            directory,
            created_ms,
            updated_ms,
            archived_ms,
            adds,
            dels,
            files,
            worktree,
            has_events,
        ) = row.map_err(|error| format!("failed to read OpenCode db incremental row: {error}"))?;
        let is_archived = archived_ms.is_some();
        new_cursor = new_cursor.max(updated_ms);
        if !show_archived && is_archived {
            continue;
        }

        let info = build_opencode_session_info(
            opencode_root,
            metadata_conn,
            id.clone(),
            Some(directory).or(worktree),
            Some(title),
            unix_ms_to_iso8601(created_ms),
            unix_ms_to_iso8601(updated_ms),
            is_archived,
            summary_count(adds, dels, files),
            has_events != 0,
        );

        if let Some(pos) = cache.sessions.iter().position(|session| session.id == id) {
            cache.sessions[pos] = info;
        } else {
            cache.sessions.push(info);
        }
    }

    cache.last_cursor = new_cursor;
    Ok(true)
}

/// OpenCode 增量掃描：優先從 DB 讀取 time_updated > last_cursor 的 session
pub(crate) fn scan_opencode_incremental_internal(
    opencode_root: &Path,
    show_archived: bool,
    metadata_conn: &Connection,
    cache: &mut ProviderCache,
) -> Result<(), String> {
    if scan_opencode_incremental_from_db_internal(
        opencode_root,
        show_archived,
        metadata_conn,
        cache,
    )? {
        return Ok(());
    }

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
            let is_archived = session.time.as_ref().and_then(|t| t.archived).is_some();
            if !show_archived && is_archived {
                new_cursor = new_cursor.max(time_updated);
                continue;
            }
            let created_at = session
                .time
                .as_ref()
                .and_then(|t| t.created)
                .and_then(unix_ms_to_iso8601);
            let updated_at = unix_ms_to_iso8601(time_updated);
            let session_summary = session.title.clone();
            let summary_count = session
                .summary
                .as_ref()
                .and_then(|s| summary_count(s.additions, s.deletions, s.files));
            let session_dir_path = message_dir_for_session(opencode_root, &session.id);
            let has_events = session_dir_path
                .read_dir()
                .map(|mut e| e.next().is_some())
                .unwrap_or(false);
            let info = build_opencode_session_info(
                opencode_root,
                metadata_conn,
                session.id.clone(),
                session.directory.or(worktree.clone()),
                session_summary,
                created_at,
                updated_at,
                is_archived,
                summary_count,
                has_events,
            );
            if let Some(pos) = cache.sessions.iter().position(|s| s.id == session.id) {
                cache.sessions[pos] = info;
            } else {
                cache.sessions.push(info);
            }
            new_cursor = new_cursor.max(time_updated);
        }
    }

    cache.last_cursor = new_cursor;
    Ok(())
}

/// 從 OpenCode 取得最大 time_updated 值
pub(crate) fn get_opencode_max_cursor(opencode_root: &Path) -> Result<i64, String> {
    if let Some(connection) = open_opencode_db(opencode_root)? {
        if has_table(&connection, "session")? {
            return connection
                .query_row(
                    "SELECT COALESCE(MAX(time_updated), 0) FROM session",
                    [],
                    |row| row.get(0),
                )
                .map_err(|error| format!("failed to query OpenCode db max cursor: {error}"));
        }
    }

    Ok(0)
}
