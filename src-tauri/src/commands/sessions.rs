use tauri::State;

use crate::activity::get_session_activity_statuses_internal;
use crate::db::{init_db, open_db_connection, upsert_session_meta_internal, delete_session_meta_internal};
use crate::sessions::{
    archive_session_internal, delete_empty_sessions_internal, delete_session_internal,
    directory_exists, get_sessions_internal, open_terminal_internal, unarchive_session_internal,
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
) -> Result<Vec<SessionInfo>, String> {
    get_sessions_internal(
        root_dir,
        opencode_root,
        show_archived,
        enabled_providers,
        force_full,
        scan_cache.inner(),
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
pub fn delete_session(root_dir: Option<String>, session_id: String) -> Result<(), String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    delete_session_internal(&resolved_root, &session_id)
}

#[tauri::command]
pub fn delete_empty_sessions(root_dir: Option<String>) -> Result<usize, String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    delete_empty_sessions_internal(&resolved_root.to_string_lossy())
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
pub fn get_session_stats(session_dir: String) -> Result<SessionStats, String> {
    let connection = open_db_connection()?;
    init_db(&connection)?;
    get_session_stats_internal(&connection, &session_dir)
}

#[tauri::command]
pub fn upsert_session_meta(
    session_id: String,
    notes: Option<String>,
    tags: Vec<String>,
) -> Result<(), String> {
    let connection = open_db_connection()?;
    init_db(&connection)?;
    upsert_session_meta_internal(&connection, &session_id, notes, tags)
}

#[tauri::command]
pub fn delete_session_meta(session_id: String) -> Result<(), String> {
    let connection = open_db_connection()?;
    init_db(&connection)?;
    delete_session_meta_internal(&connection, &session_id)
}
