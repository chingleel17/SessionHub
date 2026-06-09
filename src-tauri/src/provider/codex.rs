use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::db::ensure_parent_dir;
use crate::settings::resolve_codex_root;
use crate::types::*;

use super::bridge::read_bridge_diagnostics;
use super::{
    build_install_failure_status, build_provider_integration_status, managed_provider_metadata,
    validate_integration_target, validate_managed_metadata, write_provider_integration_file,
};

const CODEX_MANAGED_EVENTS: [&str; 3] = ["SessionStart", "PostToolUse", "Stop"];
const CODEX_NOOP_COMMAND: &str = "true";

pub(crate) fn resolve_codex_integration_path(codex_root: &Path) -> PathBuf {
    codex_root.join(CODEX_HOOK_FILE_NAME)
}

fn powershell_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn render_codex_hook_command(bridge_path: &Path, event_type: &str) -> String {
    let bridge_path_literal = powershell_single_quoted(&bridge_path.to_string_lossy());
    let bridge_parent_literal = powershell_single_quoted(
        &bridge_path
            .parent()
            .unwrap_or(bridge_path)
            .to_string_lossy(),
    );
    let event_type_literal = powershell_single_quoted(event_type);
    let provider_literal = powershell_single_quoted(CODEX_PROVIDER);

    format!(
        concat!(
            "$payload = [Console]::In.ReadToEnd(); ",
            "if ([string]::IsNullOrWhiteSpace($payload)) {{ exit 0 }}; ",
            "$event = $payload | ConvertFrom-Json; ",
            "$timestamp = [DateTimeOffset]::UtcNow.ToString('o'); ",
            "$sessionId = if ($null -ne $event.session_id -and -not [string]::IsNullOrWhiteSpace([string]$event.session_id)) {{ [string]$event.session_id }} else {{ $null }}; ",
            "$cwd = if ($null -ne $event.cwd -and -not [string]::IsNullOrWhiteSpace([string]$event.cwd)) {{ [string]$event.cwd }} else {{ $null }}; ",
            "$sourcePath = if ($null -ne $event.transcript_path -and -not [string]::IsNullOrWhiteSpace([string]$event.transcript_path)) {{ [string]$event.transcript_path }} else {{ $null }}; ",
            "$record = [ordered]@{{ version = {version}; provider = {provider}; eventType = {event_type}; timestamp = $timestamp; sessionId = $sessionId; cwd = $cwd; sourcePath = $sourcePath; title = $null; error = $null }}; ",
            "New-Item -ItemType Directory -Force -Path {bridge_parent} | Out-Null; ",
            "[System.IO.File]::AppendAllText({bridge_path}, (($record | ConvertTo-Json -Compress) + [Environment]::NewLine), [System.Text.UTF8Encoding]::new($false));"
        ),
        version = PROVIDER_INTEGRATION_VERSION,
        provider = provider_literal,
        event_type = event_type_literal,
        bridge_parent = bridge_parent_literal,
        bridge_path = bridge_path_literal,
    )
}

fn managed_hook_group(command_windows: String, matcher: Option<&str>) -> Value {
    let mut group = json!({
        "hooks": [{
            "type": "command",
            "command": CODEX_NOOP_COMMAND,
            "commandWindows": command_windows,
        }]
    });
    if let Some(matcher) = matcher {
        group["matcher"] = Value::String(matcher.to_string());
    }
    group
}

fn render_codex_integration(
    bridge_path: &Path,
    existing_content: Option<&str>,
) -> Result<String, String> {
    let mut root = existing_content
        .and_then(|content| serde_json::from_str::<Value>(content).ok())
        .unwrap_or_else(|| json!({}));
    if !root.is_object() {
        root = json!({});
    }

    root["sessionHub"] =
        serde_json::to_value(managed_provider_metadata(CODEX_PROVIDER, bridge_path))
            .map_err(|error| format!("failed to serialize Codex integration metadata: {error}"))?;

    if !root.get("hooks").is_some_and(Value::is_object) {
        root["hooks"] = json!({});
    }

    let managed_groups = [
        (
            "SessionStart",
            managed_hook_group(
                render_codex_hook_command(bridge_path, "session.started"),
                Some("startup|resume|clear|compact"),
            ),
        ),
        (
            "PostToolUse",
            managed_hook_group(render_codex_hook_command(bridge_path, "tool.post"), None),
        ),
        (
            "Stop",
            managed_hook_group(
                render_codex_hook_command(bridge_path, "session.updated"),
                None,
            ),
        ),
    ];

    for (event_name, managed_group) in managed_groups {
        let mut groups = root["hooks"]
            .get(event_name)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        groups.retain(|group| !is_managed_codex_group(group, bridge_path));
        groups.push(managed_group);
        root["hooks"][event_name] = Value::Array(groups);
    }

    serde_json::to_string_pretty(&root)
        .map_err(|error| format!("failed to serialize Codex integration: {error}"))
}

fn is_managed_codex_group(group: &Value, bridge_path: &Path) -> bool {
    let Some(hooks) = group.get("hooks").and_then(Value::as_array) else {
        return false;
    };
    let expected_bridge_path = bridge_path.to_string_lossy();
    hooks.iter().any(|hook| {
        hook.get("commandWindows")
            .or_else(|| hook.get("command"))
            .and_then(Value::as_str)
            .is_some_and(|command| {
                command.contains(expected_bridge_path.as_ref())
                    && command.contains("provider = 'codex'")
            })
    })
}

pub(crate) fn install_or_update_codex_integration(
    codex_root: Option<&str>,
) -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(CODEX_PROVIDER);
    let codex_root = match resolve_codex_root(codex_root) {
        Ok(path) => path,
        Err(error) => {
            return build_provider_integration_status(
                CODEX_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                None,
                Some(error),
            );
        }
    };
    let config_path = resolve_codex_integration_path(&codex_root);
    let Some(bridge_path) = diagnostics.bridge_path.clone() else {
        return build_provider_integration_status(
            CODEX_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            Some(config_path),
            diagnostics,
            None,
            Some("failed to resolve Codex bridge path".to_string()),
        );
    };

    let existing_content = fs::read_to_string(&config_path).ok();
    let content = match render_codex_integration(&bridge_path, existing_content.as_deref()) {
        Ok(content) => content,
        Err(error) => {
            return build_install_failure_status(
                CODEX_PROVIDER,
                Some(config_path),
                diagnostics,
                error,
            );
        }
    };

    if let Err(error) = ensure_parent_dir(&bridge_path)
        .and_then(|_| write_provider_integration_file(&config_path, &content))
    {
        return build_install_failure_status(CODEX_PROVIDER, Some(config_path), diagnostics, error);
    }

    detect_codex_integration_status(codex_root.to_str())
}

pub(crate) fn detect_codex_integration_status(
    codex_root: Option<&str>,
) -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(CODEX_PROVIDER);
    let codex_root = match resolve_codex_root(codex_root) {
        Ok(path) => path,
        Err(error) => {
            return build_provider_integration_status(
                CODEX_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                None,
                Some(error),
            );
        }
    };
    let config_path = resolve_codex_integration_path(&codex_root);

    if let Err(error) = validate_integration_target(&config_path) {
        return build_provider_integration_status(
            CODEX_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            Some(config_path),
            diagnostics,
            None,
            Some(error),
        );
    }

    let Some(expected_bridge_path) = diagnostics.bridge_path.clone() else {
        return build_provider_integration_status(
            CODEX_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            Some(config_path),
            diagnostics,
            None,
            Some("failed to resolve Codex bridge path".to_string()),
        );
    };

    if !config_path.exists() {
        return build_provider_integration_status(
            CODEX_PROVIDER,
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
            return build_provider_integration_status(
                CODEX_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path.clone()),
                diagnostics,
                None,
                Some(format!(
                    "failed to read Codex integration file {}: {error}",
                    config_path.display()
                )),
            );
        }
    };

    let parsed = match serde_json::from_str::<CopilotIntegrationConfig>(&content) {
        Ok(parsed) => parsed,
        Err(error) => {
            return build_provider_integration_status(
                CODEX_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path.clone()),
                diagnostics,
                None,
                Some(format!(
                    "failed to parse Codex integration file {}: {error}",
                    config_path.display()
                )),
            );
        }
    };

    let Some(metadata) = parsed.session_hub else {
        return build_provider_integration_status(
            CODEX_PROVIDER,
            ProviderIntegrationState::Outdated,
            Some(config_path),
            diagnostics,
            None,
            Some("missing SessionHub integration metadata".to_string()),
        );
    };

    let root: Value = match serde_json::from_str(&content) {
        Ok(root) => root,
        Err(error) => {
            return build_provider_integration_status(
                CODEX_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path.clone()),
                diagnostics,
                None,
                Some(format!(
                    "failed to parse Codex integration hooks {}: {error}",
                    config_path.display()
                )),
            );
        }
    };

    if !has_all_managed_codex_events(&root, &expected_bridge_path) {
        return build_provider_integration_status(
            CODEX_PROVIDER,
            ProviderIntegrationState::Outdated,
            Some(config_path),
            diagnostics,
            Some(metadata.integration_version),
            Some("missing managed Codex hook entries".to_string()),
        );
    }

    let installed_version = Some(metadata.integration_version);
    match validate_managed_metadata(&metadata, CODEX_PROVIDER, &expected_bridge_path) {
        Ok(()) => build_provider_integration_status(
            CODEX_PROVIDER,
            ProviderIntegrationState::Installed,
            Some(config_path),
            diagnostics,
            installed_version,
            None,
        ),
        Err(error) => build_provider_integration_status(
            CODEX_PROVIDER,
            ProviderIntegrationState::Outdated,
            Some(config_path),
            diagnostics,
            installed_version,
            Some(error),
        ),
    }
}

fn has_all_managed_codex_events(root: &Value, bridge_path: &Path) -> bool {
    CODEX_MANAGED_EVENTS.iter().all(|event_name| {
        root.get("hooks")
            .and_then(|hooks| hooks.get(*event_name))
            .and_then(Value::as_array)
            .is_some_and(|groups| {
                groups
                    .iter()
                    .any(|group| is_managed_codex_group(group, bridge_path))
            })
    })
}

pub(crate) fn uninstall_codex_integration(
    codex_root: Option<&str>,
) -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(CODEX_PROVIDER);
    let codex_root = match resolve_codex_root(codex_root) {
        Ok(path) => path,
        Err(error) => {
            return build_provider_integration_status(
                CODEX_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                None,
                Some(error),
            );
        }
    };
    let config_path = resolve_codex_integration_path(&codex_root);

    if !config_path.exists() {
        return build_provider_integration_status(
            CODEX_PROVIDER,
            ProviderIntegrationState::Missing,
            Some(config_path),
            diagnostics,
            None,
            None,
        );
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(error) => {
            return build_install_failure_status(
                CODEX_PROVIDER,
                Some(config_path),
                diagnostics,
                format!("failed to read Codex integration file: {error}"),
            );
        }
    };

    let mut root: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(error) => {
            return build_install_failure_status(
                CODEX_PROVIDER,
                Some(config_path),
                diagnostics,
                format!("failed to parse Codex integration file: {error}"),
            );
        }
    };

    if let Some(obj) = root.as_object_mut() {
        obj.remove("sessionHub");
    }
    if let Some(hooks) = root.get_mut("hooks").and_then(Value::as_object_mut) {
        for event_name in CODEX_MANAGED_EVENTS {
            if let Some(groups) = hooks.get_mut(event_name).and_then(Value::as_array_mut) {
                groups.retain(|g| {
                    !g.get("hooks").and_then(Value::as_array).is_some_and(|inner| {
                        inner.iter().any(|h| {
                            h.get("commandWindows")
                                .or_else(|| h.get("command"))
                                .and_then(Value::as_str)
                                .is_some_and(|cmd| cmd.contains("provider = 'codex'"))
                        })
                    })
                });
                if groups.is_empty() {
                    hooks.remove(event_name);
                }
            }
        }
    }

    let new_content = match serde_json::to_string_pretty(&root) {
        Ok(c) => c,
        Err(error) => {
            return build_install_failure_status(
                CODEX_PROVIDER,
                Some(config_path),
                diagnostics,
                format!("failed to serialize Codex integration: {error}"),
            );
        }
    };

    if let Err(error) = write_provider_integration_file(&config_path, &new_content) {
        return build_install_failure_status(CODEX_PROVIDER, Some(config_path), diagnostics, error);
    }

    build_provider_integration_status(
        CODEX_PROVIDER,
        ProviderIntegrationState::Missing,
        Some(config_path),
        diagnostics,
        None,
        None,
    )
}
