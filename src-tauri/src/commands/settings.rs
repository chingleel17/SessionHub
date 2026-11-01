use tauri::Manager;

use crate::quota::cache::prune_disabled_provider_quota;
use crate::settings::{
    collect_provider_integration_statuses, default_codex_root, default_hook_scripts_root,
    default_opencode_root, detect_terminal_path, detect_vscode_path, load_settings_file_internal,
    load_settings_internal, resolve_copilot_root, validate_terminal_path_internal,
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
    update_reset_credits: bool,
    sync_autostart: impl FnOnce() -> Result<(), String>,
) -> Result<SettingsEffects, String> {
    let previous = load_settings_file_internal()?;
    crate::settings::save_settings_with_reset_credit_mode(settings, update_reset_credits)?;
    let effects = settings_effects(&previous, &load_settings_file_internal()?);
    if effects.autostart {
        sync_autostart()?;
    }
    Ok(effects)
}

#[derive(Debug, PartialEq)]
struct SettingsEffects {
    autostart: bool,
    quota: bool,
    tray: bool,
    overlay: bool,
    automation: bool,
}

fn settings_effects(previous: &AppSettings, next: &AppSettings) -> SettingsEffects {
    let quota = previous.enable_quota_monitoring != next.enable_quota_monitoring
        || previous.quota_enabled_providers != next.quota_enabled_providers;
    let tray = quota
        || previous.tray_quota_mode != next.tray_quota_mode
        || previous.tray_quota_primary_provider != next.tray_quota_primary_provider
        || previous.tray_quota_panel_enabled != next.tray_quota_panel_enabled;
    let overlay = quota
        || previous.quota_overlay_enabled != next.quota_overlay_enabled
        || previous.quota_overlay_locked != next.quota_overlay_locked
        || previous.quota_overlay_opacity != next.quota_overlay_opacity
        || previous.quota_overlay_providers != next.quota_overlay_providers
        || previous.quota_overlay_theme != next.quota_overlay_theme
        || previous.quota_overlay_style != next.quota_overlay_style
        || previous.quota_overlay_intervention_enabled != next.quota_overlay_intervention_enabled
        || previous.quota_overlay_intervention_style != next.quota_overlay_intervention_style
        || previous.quota_overlay_indicator_side != next.quota_overlay_indicator_side;
    SettingsEffects {
        autostart: previous.launch_on_startup != next.launch_on_startup,
        quota,
        tray,
        overlay,
        automation: crate::settings::reset_credit_settings_changed(previous, next),
    }
}

#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String> {
    tauri::async_runtime::spawn_blocking(get_settings_internal)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn save_settings(
    app: tauri::AppHandle,
    settings: AppSettings,
    update_reset_credit_settings: Option<bool>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        save_settings_command_internal(
            &app,
            &settings,
            update_reset_credit_settings.unwrap_or(false),
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

fn save_settings_command_internal(
    app: &tauri::AppHandle,
    settings: &AppSettings,
    update_reset_credits: bool,
) -> Result<(), String> {
    // 非同步 command 可能並行；檔案保存與原生後續工作的排程保持同一順序。
    static SAVE_EFFECTS_GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _save = SAVE_EFFECTS_GATE
        .lock()
        .map_err(|_| "無法鎖定設定儲存工作".to_string())?;
    let effects = save_settings_and_sync_internal(&settings, update_reset_credits, || {
        crate::app_setup::sync_autostart_registration(&app, &settings)
            .map_err(|error| format!("failed to sync launch on startup: {error}"))
    })?;
    if effects.automation {
        app.state::<crate::quota::reset_credit_automation::AutomationState>()
            .request_refresh();
    }

    // Prune quota cache & DB for providers the user just disabled
    if effects.quota {
        let db_state = app.state::<DbState>();
        let quota_cache = app.state::<QuotaCache>();
        let conn = db_state
            .conn
            .lock()
            .map_err(|_| "failed to lock db".to_string())?;
        let _ =
            prune_disabled_provider_quota(&conn, &quota_cache, &settings.quota_enabled_providers);
    }

    if effects.tray || effects.overlay {
        // 維持在 blocking worker 建立 WebView，原生事件迴圈可持續處理視窗初始化。
        let current = load_settings_file_internal()?;
        if effects.tray {
            crate::tray_icon::update_tray_from_cache(app, &current);
        }
        if effects.overlay {
            crate::sync_overlay_visibility(app, &current);
            if let Some(overlay) = app.get_webview_window(crate::QUOTA_OVERLAY_LABEL) {
                crate::apply_overlay_locked(&overlay, current.quota_overlay_locked);
                crate::emit_overlay_settings_changed(app, &current);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_credit_save_skips_unrelated_native_effects() {
        let previous: AppSettings =
            serde_json::from_str(r#"{"copilotRoot":"", "showArchived":false}"#).expect("settings");
        let mut next = previous.clone();
        next.codex_reset_credit_reminder_enabled = true;
        next.codex_reset_credit_auto_use_enabled = true;
        next.codex_reset_credit_reminder_time = "08:30".into();
        assert_eq!(
            settings_effects(&previous, &next),
            SettingsEffects {
                autostart: false,
                quota: false,
                tray: false,
                overlay: false,
                automation: true,
            }
        );
        next.launch_on_startup = !previous.launch_on_startup;
        assert!(settings_effects(&previous, &next).autostart);
        next.quota_overlay_locked = !previous.quota_overlay_locked;
        assert!(settings_effects(&previous, &next).overlay);
    }

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
        let result = save_settings_and_sync_internal(&settings, true, || {
            Err("registration failed".to_string())
        });

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
pub async fn validate_terminal_path(path: String, launcher: Option<String>) -> bool {
    tauri::async_runtime::spawn_blocking(move || {
        validate_terminal_path_internal(&path, launcher.as_deref())
    })
    .await
    .unwrap_or(false)
}

#[tauri::command]
pub async fn restart_session_watcher(
    app: tauri::AppHandle,
    copilot_root: Option<String>,
    opencode_root: Option<String>,
    codex_root: Option<String>,
    claude_root: Option<String>,
    hook_scripts_path: Option<String>,
    enabled_providers: Option<Vec<String>>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        restart_session_watcher_command_internal(
            &app,
            copilot_root,
            opencode_root,
            codex_root,
            claude_root,
            hook_scripts_path,
            enabled_providers,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

fn restart_session_watcher_command_internal(
    app: &tauri::AppHandle,
    copilot_root: Option<String>,
    opencode_root: Option<String>,
    codex_root: Option<String>,
    claude_root: Option<String>,
    hook_scripts_path: Option<String>,
    enabled_providers: Option<Vec<String>>,
) -> Result<(), String> {
    let watcher_state = app.state::<WatcherState>();
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
