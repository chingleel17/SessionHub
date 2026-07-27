use std::path::Path;

use tauri::State;

use crate::db::{delete_path_remap, list_path_remaps, upsert_path_remap, DbState};
use crate::types::ProjectPathRemap;

pub(crate) fn path_remap_directory_exists_internal(path: &str) -> bool {
    Path::new(path).is_dir()
}

pub(crate) fn upsert_path_remap_internal(
    db: &DbState,
    old_path: &str,
    new_path: &str,
) -> Result<(), String> {
    if !path_remap_directory_exists_internal(new_path) {
        return Err(format!("directory does not exist: {new_path}"));
    }
    let connection = db
        .conn
        .lock()
        .map_err(|error| format!("db lock poisoned: {error}"))?;
    upsert_path_remap(&connection, old_path, new_path)
}

#[tauri::command]
pub fn list_project_path_remaps(db: State<'_, DbState>) -> Result<Vec<ProjectPathRemap>, String> {
    let connection = db
        .conn
        .lock()
        .map_err(|error| format!("db lock poisoned: {error}"))?;
    list_path_remaps(&connection)
}

#[tauri::command]
pub fn upsert_project_path_remap(
    old_path: String,
    new_path: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    upsert_path_remap_internal(&db, &old_path, &new_path)
}

#[tauri::command]
pub fn delete_project_path_remap(old_path: String, db: State<'_, DbState>) -> Result<(), String> {
    let connection = db
        .conn
        .lock()
        .map_err(|error| format!("db lock poisoned: {error}"))?;
    delete_path_remap(&connection, &old_path)
}

#[tauri::command]
pub fn check_path_remap_directory_exists(path: String) -> bool {
    path_remap_directory_exists_internal(&path)
}
