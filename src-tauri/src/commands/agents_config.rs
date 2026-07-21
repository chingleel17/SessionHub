use crate::agents_config::{
    check_agents_root_link_internal, link_agents_root_internal, load_project_agents_prefs_internal,
    read_agents_file_internal, save_project_agents_prefs_internal, scan_agents_commands_internal,
    scan_agents_md_internal, scan_agents_skills_internal, scan_global_agents_md_internal,
    sync_agents_items_internal, write_agents_file_internal, AgentsMdScanResult,
    AgentsRootLinkStatus, AgentsScope, CommandsScanResult, ProjectAgentsPrefs,
    SaveProjectAgentsPrefsResult, SkillsScanResult, SyncReport, SyncRequest,
};
use crate::commands::settings::get_settings_internal;

#[tauri::command]
pub async fn scan_agents_md(project_cwd: String) -> Result<AgentsMdScanResult, String> {
    tauri::async_runtime::spawn_blocking(move || scan_agents_md_internal(&project_cwd))
        .await
        .map_err(|error| format!("failed to join agents md scan task: {error}"))?
}

#[tauri::command]
pub async fn scan_global_agents_md() -> Result<AgentsMdScanResult, String> {
    tauri::async_runtime::spawn_blocking(scan_global_agents_md_internal)
        .await
        .map_err(|error| format!("failed to join global agents md scan task: {error}"))?
}

#[tauri::command]
pub async fn scan_agents_skills(scope: AgentsScope) -> Result<SkillsScanResult, String> {
    tauri::async_runtime::spawn_blocking(move || scan_agents_skills_internal(&scope))
        .await
        .map_err(|error| format!("failed to join agents skills scan task: {error}"))?
}

#[tauri::command]
pub async fn scan_agents_commands(scope: AgentsScope) -> Result<CommandsScanResult, String> {
    tauri::async_runtime::spawn_blocking(move || scan_agents_commands_internal(&scope))
        .await
        .map_err(|error| format!("failed to join agents commands scan task: {error}"))?
}

#[tauri::command]
pub async fn sync_agents_items(request: SyncRequest) -> Result<SyncReport, String> {
    tauri::async_runtime::spawn_blocking(move || sync_agents_items_internal(&request))
        .await
        .map_err(|error| format!("failed to join agents sync task: {error}"))?
}

#[tauri::command]
pub fn read_agents_file(file_path: String) -> Result<String, String> {
    read_agents_file_internal(&file_path)
}

#[tauri::command]
pub fn write_agents_file(
    scope_root: String,
    file_path: String,
    content: String,
) -> Result<(), String> {
    write_agents_file_internal(&scope_root, &file_path, &content)
}

#[tauri::command]
pub fn load_project_agents_prefs(project_cwd: String) -> Result<ProjectAgentsPrefs, String> {
    load_project_agents_prefs_internal(&project_cwd)
}

#[tauri::command]
pub fn save_project_agents_prefs(
    project_cwd: String,
    prefs: ProjectAgentsPrefs,
) -> Result<SaveProjectAgentsPrefsResult, String> {
    let settings = get_settings_internal()?;
    save_project_agents_prefs_internal(
        &project_cwd,
        &prefs,
        settings.allow_create_project_config_dir,
    )
}

#[tauri::command]
pub async fn check_agents_root_link() -> Result<AgentsRootLinkStatus, String> {
    tauri::async_runtime::spawn_blocking(check_agents_root_link_internal)
        .await
        .map_err(|error| format!("failed to join agents root link check task: {error}"))?
}

#[tauri::command]
pub async fn link_agents_root() -> Result<AgentsRootLinkStatus, String> {
    tauri::async_runtime::spawn_blocking(link_agents_root_internal)
        .await
        .map_err(|error| format!("failed to join agents root link task: {error}"))?
}
