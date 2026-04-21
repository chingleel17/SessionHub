use tauri::State;

use crate::sessions::{open_plan_external_internal, read_plan_internal, write_plan_internal};
use crate::types::*;
use crate::watcher::watch_plan_file_internal;

#[tauri::command]
pub fn watch_plan_file(
    app: tauri::AppHandle,
    watcher_state: State<'_, WatcherState>,
    session_dir: String,
) -> Result<(), String> {
    watch_plan_file_internal(&app, &watcher_state, &session_dir)
}

#[tauri::command]
pub fn stop_plan_watch(watcher_state: State<'_, WatcherState>) -> Result<(), String> {
    let mut plan_watcher = watcher_state
        .plan
        .lock()
        .map_err(|_| "failed to lock plan watcher state".to_string())?;
    *plan_watcher = None;
    Ok(())
}

#[tauri::command]
pub fn read_plan(session_dir: String) -> Result<Option<String>, String> {
    read_plan_internal(&session_dir)
}

#[tauri::command]
pub fn write_plan(session_dir: String, content: String) -> Result<(), String> {
    write_plan_internal(&session_dir, &content)
}

#[tauri::command]
pub fn open_plan_external(session_dir: String, editor_cmd: Option<String>) -> Result<(), String> {
    open_plan_external_internal(&session_dir, editor_cmd.as_deref())
}

#[tauri::command]
pub fn read_plan_content(file_path: String) -> Result<String, String> {
    let path = std::path::Path::new(&file_path);
    if !path.is_file() {
        return Err(format!("File not found: {}", file_path));
    }
    std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
}
