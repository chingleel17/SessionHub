pub mod notifications;
pub mod plan;
pub mod provider;
pub mod sessions;
pub mod settings;
pub mod tools;

pub(crate) use notifications::*;
pub(crate) use plan::*;
pub(crate) use provider::*;
pub(crate) use sessions::*;
pub(crate) use settings::*;
pub(crate) use tools::*;

use crate::settings::default_opencode_root;
use crate::types::{default_enabled_providers, AppSettings, WatcherState};
use crate::watcher::restart_session_watcher_internal;

use self::settings::get_settings_internal;

pub(super) fn restart_provider_watchers_after_integration_change(
    app: &tauri::AppHandle,
    watcher_state: &WatcherState,
    copilot_root_override: Option<&str>,
) -> Result<(), String> {
    let settings = get_settings_internal().unwrap_or_else(|_| AppSettings {
        copilot_root: copilot_root_override.unwrap_or_default().to_string(),
        opencode_root: default_opencode_root()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default(),
        terminal_path: None,
        external_editor_path: None,
        show_archived: false,
        pinned_projects: Vec::new(),
        enabled_providers: default_enabled_providers(),
        provider_integrations: Vec::new(),
        default_launcher: None,
        enable_intervention_notification: true,
        enable_session_end_notification: false,
    });

    let copilot_root = copilot_root_override.unwrap_or(settings.copilot_root.as_str());
    restart_session_watcher_internal(
        app,
        watcher_state,
        Some(copilot_root),
        Some(settings.opencode_root.as_str()),
        &settings.enabled_providers,
    )
}
