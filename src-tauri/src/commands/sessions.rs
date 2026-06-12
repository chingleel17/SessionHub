use std::collections::HashMap;

use tauri::State;

use crate::activity::get_session_activity_statuses_internal;
use crate::db::{
    delete_session_meta_internal, load_sessions_cache_from_db, open_db_connection,
    upsert_session_meta_internal, DbState,
};
use crate::sessions::{
    archive_session_internal, delete_empty_sessions_internal, delete_session_internal,
    directory_exists, find_session_by_cwd_internal, get_sessions_internal, open_terminal_internal,
    unarchive_session_internal,
};
use crate::settings::resolve_copilot_root;
use crate::stats::{backfill_missing_stats_internal, get_session_stats_internal};
use crate::types::*;

pub(crate) fn get_sessions_cached_internal(
    connection: &rusqlite::Connection,
    show_archived: Option<bool>,
    enabled_providers: Option<Vec<String>>,
) -> Result<Vec<SessionInfo>, String> {
    let include_archived = show_archived.unwrap_or(false);
    let enabled_providers = enabled_providers.unwrap_or_else(default_enabled_providers);
    let mut sessions = load_sessions_cache_from_db(connection, None)?;
    sessions.retain(|session| {
        enabled_providers
            .iter()
            .any(|provider| provider == &session.provider)
            && (include_archived || !session.is_archived)
    });
    sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(sessions)
}

pub(crate) fn get_all_session_stats_internal(
    connection: &rusqlite::Connection,
    session_dirs: &[String],
) -> HashMap<String, SessionStats> {
    let mut stats_map = HashMap::with_capacity(session_dirs.len());
    for session_dir in session_dirs {
        if let Ok(stats) = get_session_stats_internal(connection, session_dir) {
            stats_map.insert(session_dir.clone(), stats);
        }
    }
    stats_map
}

#[tauri::command]
pub fn get_sessions(
    root_dir: Option<String>,
    opencode_root: Option<String>,
    codex_root: Option<String>,
    claude_root: Option<String>,
    show_archived: Option<bool>,
    enabled_providers: Option<Vec<String>>,
    force_full: Option<bool>,
    scan_cache: State<'_, ScanCache>,
    db: State<'_, DbState>,
) -> Result<Vec<SessionInfo>, String> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    get_sessions_internal(
        root_dir,
        opencode_root,
        codex_root,
        claude_root,
        show_archived,
        enabled_providers,
        force_full,
        scan_cache.inner(),
        &*conn,
    )
}

#[tauri::command]
pub fn get_sessions_cached(
    show_archived: Option<bool>,
    enabled_providers: Option<Vec<String>>,
    db: State<'_, DbState>,
) -> Result<Vec<SessionInfo>, String> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    get_sessions_cached_internal(&*conn, show_archived, enabled_providers)
}

#[tauri::command]
pub fn archive_session(root_dir: Option<String>, session_id: String) -> Result<(), String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    archive_session_internal(&resolved_root, &session_id)
}

#[tauri::command]
pub fn unarchive_session(root_dir: Option<String>, session_id: String) -> Result<(), String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    unarchive_session_internal(&resolved_root, &session_id)
}

#[tauri::command]
pub fn delete_session(
    root_dir: Option<String>,
    session_id: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    delete_session_internal(&resolved_root, &session_id, &*conn)
}

#[tauri::command]
pub fn delete_empty_sessions(
    root_dir: Option<String>,
    db: State<'_, DbState>,
) -> Result<usize, String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    delete_empty_sessions_internal(&resolved_root.to_string_lossy(), &*conn)
}

#[tauri::command]
pub fn open_terminal(
    terminal_path: String,
    cwd: String,
    _session_id: String,
) -> Result<(), String> {
    open_terminal_internal(&terminal_path, &cwd)
}

#[tauri::command]
pub fn check_directory_exists(path: String) -> bool {
    directory_exists(&path)
}

#[tauri::command]
pub fn get_session_activity_statuses(
    sessions: Vec<serde_json::Value>,
    opencode_root: Option<String>,
    scan_cache: State<'_, ScanCache>,
) -> Vec<SessionActivityStatus> {
    get_session_activity_statuses_internal(&sessions, opencode_root.as_deref(), &scan_cache.activity)
}

#[tauri::command]
pub fn get_session_stats(
    session_dir: String,
    db: State<'_, DbState>,
) -> Result<SessionStats, String> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    get_session_stats_internal(&*conn, &session_dir)
}

#[tauri::command]
pub fn get_all_session_stats(
    session_dirs: Vec<String>,
    db: State<'_, DbState>,
) -> Result<HashMap<String, SessionStats>, String> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    Ok(get_all_session_stats_internal(&*conn, &session_dirs))
}

#[tauri::command]
pub async fn trigger_stats_backfill(
    root_dir: Option<String>,
    _db: State<'_, DbState>,
) -> Result<usize, String> {
    let copilot_root = resolve_copilot_root(root_dir.as_deref())?;
    tauri::async_runtime::spawn_blocking(move || {
        let connection = open_db_connection()?;
        backfill_missing_stats_internal(&connection, &copilot_root)
    })
    .await
    .map_err(|error| format!("failed to join stats backfill task: {error}"))?
}

#[tauri::command]
pub fn upsert_session_meta(
    session_id: String,
    notes: Option<String>,
    tags: Vec<String>,
    db: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    upsert_session_meta_internal(&*conn, &session_id, notes, tags)
}

#[tauri::command]
pub fn delete_session_meta(session_id: String, db: State<'_, DbState>) -> Result<(), String> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    delete_session_meta_internal(&*conn, &session_id)
}

#[tauri::command]
pub fn get_session_by_cwd(
    cwd: String,
    root_dir: Option<String>,
    db: State<'_, DbState>,
) -> Result<Option<SessionInfo>, String> {
    let copilot_root = resolve_copilot_root(root_dir.as_deref())?;
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    find_session_by_cwd_internal(&copilot_root, &cwd, &*conn)
}
