pub mod agents_config;
pub mod analytics;
pub mod notifications;
pub mod plan;
pub mod provider;
pub mod session_todos;
pub mod sessions;
pub mod settings;
pub mod tools;

pub(crate) use agents_config::*;
pub(crate) use analytics::*;
pub(crate) use notifications::*;
pub(crate) use plan::*;
pub(crate) use provider::*;
pub(crate) use session_todos::*;
pub(crate) use sessions::*;
pub(crate) use settings::*;
pub(crate) use tools::*;

pub mod quota;
pub(crate) use quota::*;

use crate::settings::{
    default_claude_root, default_codex_root, default_hook_scripts_root, default_opencode_root,
};
use crate::types::{
    default_enabled_providers, default_enabled_providers_all, AppSettings, WatcherState,
};
use crate::watcher::restart_session_watcher_internal;

use self::settings::get_settings_internal;

pub(super) fn restart_provider_watchers_after_integration_change(
    app: &tauri::AppHandle,
    watcher_state: &WatcherState,
    copilot_root_override: Option<&str>,
    codex_root_override: Option<&str>,
) -> Result<(), String> {
    let settings = get_settings_internal().unwrap_or_else(|_| AppSettings {
        copilot_root: copilot_root_override.unwrap_or_default().to_string(),
        opencode_root: default_opencode_root()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default(),
        codex_root: codex_root_override.map(str::to_string).unwrap_or_else(|| {
            default_codex_root()
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_default()
        }),
        claude_root: default_claude_root()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default(),
        hook_scripts_path: default_hook_scripts_root()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default(),
        claude_quota_reset_day: 1,
        minimize_to_tray: false,
        terminal_path: None,
        external_editor_path: None,
        show_archived: false,
        pinned_projects: Vec::new(),
        enabled_providers: default_enabled_providers(),
        provider_integrations: Vec::new(),
        default_launcher: None,
        enable_intervention_notification: true,
        enable_session_end_notification: false,
        show_status_bar: true,
        analytics_refresh_interval: 30,
        analytics_panel_collapsed: false,
        enable_quota_monitoring: true,
        quota_enabled_providers: default_enabled_providers_all(),
        allow_create_project_config_dir: false,
        agents_source_root: String::new(),
    });

    let copilot_root = copilot_root_override.unwrap_or(settings.copilot_root.as_str());
    restart_session_watcher_internal(
        app,
        watcher_state,
        Some(copilot_root),
        Some(settings.opencode_root.as_str()),
        Some(settings.codex_root.as_str()),
        Some(settings.claude_root.as_str()),
        &settings.enabled_providers,
    )
}
