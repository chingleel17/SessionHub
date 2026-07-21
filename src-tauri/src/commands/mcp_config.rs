use crate::mcp_config::{
    delete_mcp_server_internal, is_codex_project_trusted, list_mcp_configs_internal,
    set_mcp_server_enabled_internal, upsert_mcp_server_internal, McpProviderConfig, McpScope,
};

#[tauri::command]
pub async fn list_mcp_configs(scope: McpScope) -> Result<Vec<McpProviderConfig>, String> {
    tauri::async_runtime::spawn_blocking(move || list_mcp_configs_internal(&scope))
        .await
        .map_err(|error| format!("failed to join list MCP configs task: {error}"))?
}

#[tauri::command]
pub async fn upsert_mcp_server(
    scope: McpScope,
    provider: String,
    name: String,
    original_name: Option<String>,
    config_json: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        upsert_mcp_server_internal(
            &scope,
            &provider,
            &name,
            original_name.as_deref(),
            &config_json,
        )
    })
    .await
    .map_err(|error| format!("failed to join upsert MCP server task: {error}"))?
}

#[tauri::command]
pub async fn delete_mcp_server(
    scope: McpScope,
    provider: String,
    name: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        delete_mcp_server_internal(&scope, &provider, &name)
    })
    .await
    .map_err(|error| format!("failed to join delete MCP server task: {error}"))?
}

#[tauri::command]
pub async fn set_mcp_server_enabled(
    scope: McpScope,
    provider: String,
    name: String,
    enabled: bool,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        set_mcp_server_enabled_internal(&scope, &provider, &name, enabled)
    })
    .await
    .map_err(|error| format!("failed to join set MCP server enabled task: {error}"))?
}

#[tauri::command]
pub async fn check_codex_project_trust(project_cwd: String) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || is_codex_project_trusted(&project_cwd))
        .await
        .map_err(|error| format!("failed to join codex trust check task: {error}"))?
}
