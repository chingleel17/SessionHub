use tauri::State;

use crate::activity::get_session_activity_statuses_internal;
use crate::db::{DbState, upsert_session_meta_internal, delete_session_meta_internal};
use crate::sessions::{
    archive_session_internal, delete_empty_sessions_internal, delete_session_internal,
    directory_exists, find_session_by_cwd_internal, get_sessions_internal, open_terminal_internal,
    unarchive_session_internal,
};
use crate::settings::resolve_copilot_root;
use crate::stats::get_session_stats_internal;
use crate::types::*;

#[tauri::command]
pub fn get_sessions(
    root_dir: Option<String>,
    opencode_root: Option<String>,
    show_archived: Option<bool>,
    enabled_providers: Option<Vec<String>>,
    force_full: Option<bool>,
    scan_cache: State<'_, ScanCache>,
    db: State<'_, DbState>,
) -> Result<Vec<SessionInfo>, String> {
    let conn = db.conn.lock().map_err(|e| format!("db lock poisoned: {e}"))?;
    get_sessions_internal(
        root_dir,
        opencode_root,
        show_archived,
        enabled_providers,
        force_full,
        scan_cache.inner(),
        &*conn,
    )
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
pub fn delete_session(root_dir: Option<String>, session_id: String, db: State<'_, DbState>) -> Result<(), String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    let conn = db.conn.lock().map_err(|e| format!("db lock poisoned: {e}"))?;
    delete_session_internal(&resolved_root, &session_id, &*conn)
}

#[tauri::command]
pub fn delete_empty_sessions(root_dir: Option<String>, db: State<'_, DbState>) -> Result<usize, String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    let conn = db.conn.lock().map_err(|e| format!("db lock poisoned: {e}"))?;
    delete_empty_sessions_internal(&resolved_root.to_string_lossy(), &*conn)
}

#[tauri::command]
pub fn open_terminal(terminal_path: String, cwd: String, _session_id: String) -> Result<(), String> {
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
) -> Vec<SessionActivityStatus> {
    get_session_activity_statuses_internal(&sessions, opencode_root.as_deref())
}

#[tauri::command]
pub fn get_session_stats(session_dir: String, db: State<'_, DbState>) -> Result<SessionStats, String> {
    let conn = db.conn.lock().map_err(|e| format!("db lock poisoned: {e}"))?;
    get_session_stats_internal(&*conn, &session_dir)
}

#[tauri::command]
pub fn upsert_session_meta(
    session_id: String,
    notes: Option<String>,
    tags: Vec<String>,
    db: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("db lock poisoned: {e}"))?;
    upsert_session_meta_internal(&*conn, &session_id, notes, tags)
}

#[tauri::command]
pub fn delete_session_meta(session_id: String, db: State<'_, DbState>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("db lock poisoned: {e}"))?;
    delete_session_meta_internal(&*conn, &session_id)
}

#[tauri::command]
pub fn get_session_by_cwd(
    cwd: String,
    root_dir: Option<String>,
    db: State<'_, DbState>,
) -> Result<Option<SessionInfo>, String> {
    let copilot_root = resolve_copilot_root(root_dir.as_deref())?;
    let conn = db.conn.lock().map_err(|e| format!("db lock poisoned: {e}"))?;
    find_session_by_cwd_internal(&copilot_root, &cwd, &*conn)
}
