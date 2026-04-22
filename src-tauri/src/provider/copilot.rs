use std::fs;
use std::path::Path;

use crate::db::ensure_parent_dir;
use crate::settings::resolve_copilot_root;
use crate::types::*;

use super::bridge::{read_bridge_diagnostics, resolve_copilot_integration_path};
use super::{
    build_install_failure_status, build_provider_integration_status, managed_provider_metadata,
    validate_integration_target, validate_managed_metadata, write_provider_integration_file,
};

fn powershell_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn render_copilot_hook_ps(
    bridge_path: &Path,
    event_type: &str,
    title_assignment: &str,
    error_assignment: &str,
) -> String {
    let bridge_path_literal = powershell_single_quoted(&bridge_path.to_string_lossy());
    let bridge_parent_literal = powershell_single_quoted(
        &bridge_path
            .parent()
            .unwrap_or(bridge_path)
            .to_string_lossy(),
    );
    let event_type_literal = powershell_single_quoted(event_type);
    let provider_literal = powershell_single_quoted(COPILOT_PROVIDER);

    format!(
        concat!(
            "$payload = [Console]::In.ReadToEnd(); ",
            "if ([string]::IsNullOrWhiteSpace($payload)) {{ exit 0 }}; ",
            "$event = $payload | ConvertFrom-Json; ",
            "$timestamp = if ($event.timestamp) {{ [DateTimeOffset]::FromUnixTimeMilliseconds([int64]$event.timestamp).UtcDateTime.ToString('o') }} else {{ [DateTimeOffset]::UtcNow.ToString('o') }}; ",
            "$cwd = if ($null -ne $event.cwd -and -not [string]::IsNullOrWhiteSpace([string]$event.cwd)) {{ [string]$event.cwd }} else {{ $null }}; ",
            "{title_assignment} ",
            "{error_assignment} ",
            "$record = [ordered]@{{ version = {version}; provider = {provider}; eventType = {event_type}; timestamp = $timestamp; sessionId = $null; cwd = $cwd; sourcePath = $null; title = $title; error = $error }}; ",
            "New-Item -ItemType Directory -Force -Path {bridge_parent} | Out-Null; ",
            "[System.IO.File]::AppendAllText({bridge_path}, (($record | ConvertTo-Json -Compress) + [Environment]::NewLine), [System.Text.UTF8Encoding]::new($false));",
        ),
        title_assignment = title_assignment,
        error_assignment = error_assignment,
        version = PROVIDER_INTEGRATION_VERSION,
        provider = provider_literal,
        event_type = event_type_literal,
        bridge_parent = bridge_parent_literal,
        bridge_path = bridge_path_literal,
    )
}

fn render_copilot_integration(bridge_path: &Path) -> Result<String, String> {
    let hook_session_start = render_copilot_hook_ps(
        bridge_path,
        "session.started",
        "$title = $null;",
        "$error = $null;",
    );
    let hook_session_end = render_copilot_hook_ps(
        bridge_path,
        "session.ended",
        "$title = $null;",
        "$error = if ($event.reason -eq 'error') { 'copilot session ended with error' } else { $null };",
    );
    let hook_prompt_submitted = render_copilot_hook_ps(
        bridge_path,
        "prompt.submitted",
        "$title = if ($event.prompt) { $s = [string]$event.prompt; if ($s.Length -gt 80) { $s.Substring(0, 80) } else { $s } } else { $null };",
        "$error = $null;",
    );
    let hook_pre_tool = render_copilot_hook_ps(
        bridge_path,
        "tool.pre",
        "$title = if ($event.toolName) { [string]$event.toolName } else { $null };",
        "$error = $null;",
    );
    let hook_post_tool = render_copilot_hook_ps(
        bridge_path,
        "tool.post",
        "$title = if ($event.toolName) { [string]$event.toolName } else { $null };",
        "$rt = if ($event.toolResult) { [string]$event.toolResult.resultType } else { $null }; $error = if ($rt -eq 'failure' -or $rt -eq 'denied') { 'tool ' + [string]$event.toolName + ' ' + $rt } else { $null };",
    );
    let hook_error_occurred = render_copilot_hook_ps(
        bridge_path,
        "session.errored",
        "$title = if ($event.error -and $event.error.name) { [string]$event.error.name } else { $null };",
        "$error = if ($event.error -and $event.error.message) { [string]$event.error.message } else { 'unknown error' };",
    );

    let integration = serde_json::json!({
        "version": 1,
        "sessionHub": managed_provider_metadata(COPILOT_PROVIDER, bridge_path),
        "hooks": {
            "sessionStart":         [{ "type": "command", "powershell": hook_session_start }],
            "sessionEnd":           [{ "type": "command", "powershell": hook_session_end }],
            "userPromptSubmitted":  [{ "type": "command", "powershell": hook_prompt_submitted }],
            "preToolUse":           [{ "type": "command", "powershell": hook_pre_tool }],
            "postToolUse":          [{ "type": "command", "powershell": hook_post_tool }],
            "errorOccurred":        [{ "type": "command", "powershell": hook_error_occurred }],
        }
    });

    serde_json::to_string_pretty(&integration)
        .map_err(|error| format!("failed to serialize Copilot integration: {error}"))
}

pub(crate) fn install_or_update_copilot_integration(
    copilot_root: Option<&str>,
) -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(COPILOT_PROVIDER);
    let copilot_root = match resolve_copilot_root(copilot_root) {
        Ok(path) => path,
        Err(error) => {
            return build_provider_integration_status(
                COPILOT_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                None,
                Some(error),
            );
        }
    };
    let config_path = resolve_copilot_integration_path(&copilot_root);
    let Some(bridge_path) = diagnostics.bridge_path.clone() else {
        return build_provider_integration_status(
            COPILOT_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            Some(config_path),
            diagnostics,
            None,
            Some("failed to resolve Copilot bridge path".to_string()),
        );
    };

    let content = match render_copilot_integration(&bridge_path) {
        Ok(content) => content,
        Err(error) => {
            return build_install_failure_status(
                COPILOT_PROVIDER,
                Some(config_path),
                diagnostics,
                error,
            );
        }
    };

    if let Err(error) = ensure_parent_dir(&bridge_path)
        .and_then(|_| write_provider_integration_file(&config_path, &content))
    {
        return build_install_failure_status(
            COPILOT_PROVIDER,
            Some(config_path),
            diagnostics,
            error,
        );
    }

    detect_copilot_integration_status(copilot_root.to_str())
}

pub(crate) fn detect_copilot_integration_status(
    copilot_root: Option<&str>,
) -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(COPILOT_PROVIDER);
    let copilot_root = match resolve_copilot_root(copilot_root) {
        Ok(path) => path,
        Err(error) => {
            return build_provider_integration_status(
                COPILOT_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                None,
                Some(error),
            );
        }
    };
    let config_path = resolve_copilot_integration_path(&copilot_root);

    if let Err(error) = validate_integration_target(&config_path) {
        return build_provider_integration_status(
            COPILOT_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            Some(config_path),
            diagnostics,
            None,
            Some(error),
        );
    }

    let Some(expected_bridge_path) = diagnostics.bridge_path.clone() else {
        return build_provider_integration_status(
            COPILOT_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            Some(config_path),
            diagnostics,
            None,
            Some("failed to resolve Copilot bridge path".to_string()),
        );
    };

    if !config_path.exists() {
        return build_provider_integration_status(
            COPILOT_PROVIDER,
            ProviderIntegrationState::Missing,
            Some(config_path),
            diagnostics,
            None,
            None,
        );
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(error) => {
            let error_message = format!(
                "failed to read Copilot integration file {}: {error}",
                config_path.display()
            );
            return build_provider_integration_status(
                COPILOT_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path),
                diagnostics,
                None,
                Some(error_message),
            );
        }
    };

    let parsed = match serde_json::from_str::<CopilotIntegrationConfig>(&content) {
        Ok(parsed) => parsed,
        Err(error) => {
            let error_message = format!(
                "failed to parse Copilot integration file {}: {error}",
                config_path.display()
            );
            return build_provider_integration_status(
                COPILOT_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path),
                diagnostics,
                None,
                Some(error_message),
            );
        }
    };

    let Some(metadata) = parsed.session_hub else {
        return build_provider_integration_status(
            COPILOT_PROVIDER,
            ProviderIntegrationState::Outdated,
            Some(config_path),
            diagnostics,
            None,
            Some("missing SessionHub integration metadata".to_string()),
        );
    };

    let installed_version = Some(metadata.integration_version);
    match validate_managed_metadata(&metadata, COPILOT_PROVIDER, &expected_bridge_path) {
        Ok(()) => build_provider_integration_status(
            COPILOT_PROVIDER,
            ProviderIntegrationState::Installed,
            Some(config_path),
            diagnostics,
            installed_version,
            None,
        ),
        Err(error) => build_provider_integration_status(
            COPILOT_PROVIDER,
            ProviderIntegrationState::Outdated,
            Some(config_path),
            diagnostics,
            installed_version,
            Some(error),
        ),
    }
}
