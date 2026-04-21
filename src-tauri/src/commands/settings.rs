use tauri::State;

use crate::settings::{
    default_opencode_root, detect_terminal_path, detect_vscode_path, load_settings_internal,
    save_settings_internal, validate_terminal_path_internal, collect_provider_integration_statuses,
};
use crate::types::{default_enabled_providers, AppSettings, WatcherState};
use crate::watcher::restart_session_watcher_internal;

pub(crate) fn get_settings_internal() -> Result<AppSettings, String> {
    let mut settings = load_settings_internal()?;
    if settings.opencode_root.trim().is_empty() {
        if let Ok(default_root) = default_opencode_root() {
            settings.opencode_root = default_root.to_string_lossy().to_string();
        }
    }
    settings.provider_integrations =
        collect_provider_integration_statuses(Some(settings.copilot_root.as_str()));
    Ok(settings)
}

#[tauri::command]
pub fn get_settings() -> Result<AppSettings, String> {
    get_settings_internal()
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    save_settings_internal(&settings)
}

#[tauri::command]
pub fn detect_terminal() -> Result<Option<String>, String> {
    detect_terminal_path()
}

#[tauri::command]
pub fn detect_vscode() -> Result<Option<String>, String> {
    detect_vscode_path()
}

#[tauri::command]
pub fn validate_terminal_path(path: String) -> bool {
    validate_terminal_path_internal(&path)
}

#[tauri::command]
pub fn restart_session_watcher(
    app: tauri::AppHandle,
    watcher_state: State<'_, WatcherState>,
    copilot_root: Option<String>,
    opencode_root: Option<String>,
    enabled_providers: Option<Vec<String>>,
) -> Result<(), String> {
    let providers = enabled_providers.unwrap_or_else(default_enabled_providers);
    restart_session_watcher_internal(
        &app,
        &watcher_state,
        copilot_root.as_deref(),
        opencode_root.as_deref(),
        &providers,
    )
}
