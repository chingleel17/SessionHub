use tauri::State;

use crate::provider::{
    install_or_update_provider_integration, recheck_provider_integration_status,
    uninstall_provider_integration as do_uninstall_provider_integration,
};
use crate::types::*;

use super::restart_provider_watchers_after_integration_change;

#[tauri::command]
pub fn install_provider_integration(
    app: tauri::AppHandle,
    watcher_state: State<'_, WatcherState>,
    provider: String,
    copilot_root: Option<String>,
    codex_root: Option<String>,
    claude_root: Option<String>,
) -> Result<ProviderIntegrationStatus, String> {
    let status = install_or_update_provider_integration(
        &provider,
        copilot_root.as_deref(),
        codex_root.as_deref(),
        claude_root.as_deref(),
    )?;
    restart_provider_watchers_after_integration_change(
        &app,
        &watcher_state,
        copilot_root.as_deref(),
        codex_root.as_deref(),
    )?;
    Ok(status)
}

#[tauri::command]
pub fn update_provider_integration(
    app: tauri::AppHandle,
    watcher_state: State<'_, WatcherState>,
    provider: String,
    copilot_root: Option<String>,
    codex_root: Option<String>,
    claude_root: Option<String>,
) -> Result<ProviderIntegrationStatus, String> {
    let status = install_or_update_provider_integration(
        &provider,
        copilot_root.as_deref(),
        codex_root.as_deref(),
        claude_root.as_deref(),
    )?;
    restart_provider_watchers_after_integration_change(
        &app,
        &watcher_state,
        copilot_root.as_deref(),
        codex_root.as_deref(),
    )?;
    Ok(status)
}

#[tauri::command]
pub fn recheck_provider_integration(
    app: tauri::AppHandle,
    watcher_state: State<'_, WatcherState>,
    provider: String,
    copilot_root: Option<String>,
    codex_root: Option<String>,
    claude_root: Option<String>,
) -> Result<ProviderIntegrationStatus, String> {
    let status = recheck_provider_integration_status(
        &provider,
        copilot_root.as_deref(),
        codex_root.as_deref(),
        claude_root.as_deref(),
    )?;
    restart_provider_watchers_after_integration_change(
        &app,
        &watcher_state,
        copilot_root.as_deref(),
        codex_root.as_deref(),
    )?;
    Ok(status)
}

#[tauri::command]
pub fn uninstall_provider_integration(
    app: tauri::AppHandle,
    watcher_state: State<'_, WatcherState>,
    provider: String,
    copilot_root: Option<String>,
    codex_root: Option<String>,
    claude_root: Option<String>,
) -> Result<ProviderIntegrationStatus, String> {
    let status = do_uninstall_provider_integration(
        &provider,
        copilot_root.as_deref(),
        codex_root.as_deref(),
        claude_root.as_deref(),
    )?;
    restart_provider_watchers_after_integration_change(
        &app,
        &watcher_state,
        copilot_root.as_deref(),
        codex_root.as_deref(),
    )?;
    Ok(status)
}
