use tauri::{Manager, State};

use crate::quota::cache::prune_disabled_provider_quota;
use crate::settings::{
    collect_provider_integration_statuses, default_codex_root, default_hook_scripts_root,
    default_opencode_root, detect_terminal_path, detect_vscode_path, load_settings_internal,
    resolve_copilot_root, save_settings_internal, validate_terminal_path_internal,
};
use crate::types::{default_enabled_providers, AppSettings, QuotaCache, WatcherState};
use crate::watcher::restart_session_watcher_internal;
use crate::DbState;

pub(crate) fn get_settings_internal() -> Result<AppSettings, String> {
    let mut settings = load_settings_internal()?;
    if settings.opencode_root.trim().is_empty() {
        if let Ok(default_root) = default_opencode_root() {
            settings.opencode_root = default_root.to_string_lossy().to_string();
        }
    }
    if settings.codex_root.trim().is_empty() {
        if let Ok(default_root) = default_codex_root() {
            settings.codex_root = default_root.to_string_lossy().to_string();
        }
    }
    if settings.hook_scripts_path.trim().is_empty() {
        if let Ok(default_root) = default_hook_scripts_root() {
            settings.hook_scripts_path = default_root.to_string_lossy().to_string();
        }
    }
    settings.provider_integrations = collect_provider_integration_statuses(
        Some(settings.copilot_root.as_str()),
        Some(settings.codex_root.as_str()),
        Some(settings.hook_scripts_path.as_str()),
    );
    Ok(settings)
}

fn save_settings_and_sync_internal(
    settings: &AppSettings,
    sync_autostart: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    save_settings_internal(settings)?;
    sync_autostart()
}

#[tauri::command]
pub fn get_settings() -> Result<AppSettings, String> {
    get_settings_internal()
}

#[tauri::command]
pub fn save_settings(
    app: tauri::AppHandle,
    db_state: State<'_, DbState>,
    quota_cache: State<'_, QuotaCache>,
    settings: AppSettings,
) -> Result<(), String> {
    save_settings_and_sync_internal(&settings, || {
        crate::app_setup::sync_autostart_registration(&app, &settings)
            .map_err(|error| format!("failed to sync launch on startup: {error}"))
    })?;

    // Prune quota cache & DB for providers the user just disabled
    {
        let conn = db_state
            .conn
            .lock()
            .map_err(|_| "failed to lock db".to_string())?;
        let _ =
            prune_disabled_provider_quota(&conn, &quota_cache, &settings.quota_enabled_providers);
    }

    // tray / overlay 設定即時生效：重繪系統匣圖示、依開關建立/關閉 overlay 並套用鎖定與透明度
    crate::tray_icon::update_tray_from_cache(&app, &settings);
    crate::sync_overlay_visibility(&app, &settings);
    if let Some(overlay) = app.get_webview_window(crate::QUOTA_OVERLAY_LABEL) {
        crate::apply_overlay_locked(&overlay, settings.quota_overlay_locked);
        crate::emit_overlay_settings_changed(&app, &settings);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_failure_preserves_the_saved_launch_on_startup_setting() {
        let _guard = crate::shared_env_test_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let appdata_dir = std::env::temp_dir().join(format!(
            "session-hub-autostart-settings-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&appdata_dir).expect("create app data dir");
        unsafe {
            std::env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", &appdata_dir);
        }

        let mut settings = AppSettings::default().expect("default settings");
        settings.launch_on_startup = true;
        let result =
            save_settings_and_sync_internal(&settings, || Err("registration failed".to_string()));

        assert_eq!(result, Err("registration failed".to_string()));
        assert!(
            load_settings_internal()
                .expect("load settings")
                .launch_on_startup
        );

        unsafe {
            std::env::remove_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE");
        }
        std::fs::remove_dir_all(&appdata_dir).expect("remove app data dir");
    }

    #[test]
    fn settings_without_autostart_fields_use_compatible_defaults() {
        let settings = AppSettings::default().expect("default settings");
        let mut value = serde_json::to_value(settings).expect("serialize settings");
        let object = value.as_object_mut().expect("settings object");
        object.remove("launchOnStartup");
        object.remove("startMinimizedOnStartup");

        let parsed = serde_json::from_value::<AppSettings>(value).expect("parse old settings");
        assert!(!parsed.launch_on_startup);
        assert!(parsed.start_minimized_on_startup);
    }
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
    codex_root: Option<String>,
    claude_root: Option<String>,
    hook_scripts_path: Option<String>,
    enabled_providers: Option<Vec<String>>,
) -> Result<(), String> {
    let providers = enabled_providers.unwrap_or_else(default_enabled_providers);

    // 更新 integration 狀態快取，讓 watcher 判斷時不需再重讀磁碟
    let resolved_copilot = resolve_copilot_root(copilot_root.as_deref())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let hook_path = hook_scripts_path.unwrap_or_default();
    let integrations = collect_provider_integration_statuses(
        Some(resolved_copilot.as_str()),
        codex_root.as_deref(),
        Some(hook_path.as_str()),
    );
    if let Ok(mut known) = watcher_state.known_integrations.lock() {
        *known = integrations;
    }

    restart_session_watcher_internal(
        &app,
        &watcher_state,
        copilot_root.as_deref(),
        opencode_root.as_deref(),
        codex_root.as_deref(),
        claude_root.as_deref(),
        &providers,
    )
}
