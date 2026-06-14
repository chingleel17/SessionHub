use std::fs;
use std::path::{Path, PathBuf};

use crate::settings::{default_copilot_hook_scripts_root, resolve_copilot_root};
use crate::types::*;

use super::bridge::{read_bridge_diagnostics, resolve_copilot_integration_path};
use super::{
    build_install_failure_status, build_provider_integration_status, install_hook_scripts,
    managed_provider_metadata, validate_integration_target, validate_managed_metadata,
    write_provider_integration_file,
};

const SESSIONHUB_HOOK_COMMAND_MARKER: &str = "sessionhub-provider-event-bridge";
const HOOK_SCRIPT_VERSION: &str = "2";

const MODULE_RECORD_EVENT: &str =
    include_str!("../../../hooks/copilot/modules/record-event.psm1");
const SCRIPT_ON_SESSION_START: &str = include_str!("../../../hooks/copilot/on-session-start.ps1");
const SCRIPT_ON_SESSION_END: &str = include_str!("../../../hooks/copilot/on-session-end.ps1");
const SCRIPT_ON_USER_PROMPT_SUBMITTED: &str =
    include_str!("../../../hooks/copilot/on-user-prompt-submitted.ps1");
const SCRIPT_ON_PRE_TOOL_USE: &str = include_str!("../../../hooks/copilot/on-pre-tool-use.ps1");
const SCRIPT_ON_POST_TOOL_USE: &str =
    include_str!("../../../hooks/copilot/on-post-tool-use.ps1");
const SCRIPT_ON_ERROR_OCCURRED: &str =
    include_str!("../../../hooks/copilot/on-error-occurred.ps1");

const MODULE_RECORD_EVENT_SH: &str =
    include_str!("../../../hooks/copilot/modules/record-event.sh");
const SCRIPT_ON_SESSION_START_SH: &str =
    include_str!("../../../hooks/copilot/on-session-start.sh");
const SCRIPT_ON_SESSION_END_SH: &str = include_str!("../../../hooks/copilot/on-session-end.sh");
const SCRIPT_ON_USER_PROMPT_SUBMITTED_SH: &str =
    include_str!("../../../hooks/copilot/on-user-prompt-submitted.sh");
const SCRIPT_ON_PRE_TOOL_USE_SH: &str =
    include_str!("../../../hooks/copilot/on-pre-tool-use.sh");
const SCRIPT_ON_POST_TOOL_USE_SH: &str =
    include_str!("../../../hooks/copilot/on-post-tool-use.sh");
const SCRIPT_ON_ERROR_OCCURRED_SH: &str =
    include_str!("../../../hooks/copilot/on-error-occurred.sh");

fn hook_script_entries() -> [(&'static str, &'static str); 14] {
    [
        ("modules/record-event.psm1", MODULE_RECORD_EVENT),
        ("on-session-start.ps1", SCRIPT_ON_SESSION_START),
        ("on-session-end.ps1", SCRIPT_ON_SESSION_END),
        (
            "on-user-prompt-submitted.ps1",
            SCRIPT_ON_USER_PROMPT_SUBMITTED,
        ),
        ("on-pre-tool-use.ps1", SCRIPT_ON_PRE_TOOL_USE),
        ("on-post-tool-use.ps1", SCRIPT_ON_POST_TOOL_USE),
        ("on-error-occurred.ps1", SCRIPT_ON_ERROR_OCCURRED),
        ("modules/record-event.sh", MODULE_RECORD_EVENT_SH),
        ("on-session-start.sh", SCRIPT_ON_SESSION_START_SH),
        ("on-session-end.sh", SCRIPT_ON_SESSION_END_SH),
        (
            "on-user-prompt-submitted.sh",
            SCRIPT_ON_USER_PROMPT_SUBMITTED_SH,
        ),
        ("on-pre-tool-use.sh", SCRIPT_ON_PRE_TOOL_USE_SH),
        ("on-post-tool-use.sh", SCRIPT_ON_POST_TOOL_USE_SH),
        ("on-error-occurred.sh", SCRIPT_ON_ERROR_OCCURRED_SH),
    ]
}

pub(crate) fn ensure_copilot_hook_scripts_installed() -> Result<PathBuf, String> {
    let root = default_copilot_hook_scripts_root()?;
    install_hook_scripts("Copilot", &root, &hook_script_entries(), HOOK_SCRIPT_VERSION)
}

/// 移除 SessionHub 安裝的 Copilot hook 腳本（`~/.copilot/hooks`），保留使用者自訂檔案
fn remove_copilot_hook_scripts() {
    let Ok(root) = default_copilot_hook_scripts_root() else {
        return;
    };
    super::uninstall_hook_scripts(&root, &hook_script_entries());
}

fn powershell_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn sh_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn render_copilot_hook_command(script_path: &Path, bridge_path: &Path) -> String {
    let script_literal = powershell_single_quoted(&script_path.to_string_lossy());
    let bridge_literal = powershell_single_quoted(&bridge_path.to_string_lossy());
    let provider_literal = powershell_single_quoted(COPILOT_PROVIDER);

    format!(
        "pwsh -NoProfile -ExecutionPolicy Bypass -File {script} -BridgePath {bridge} -Provider {provider} # {marker}",
        script = script_literal,
        bridge = bridge_literal,
        provider = provider_literal,
        marker = SESSIONHUB_HOOK_COMMAND_MARKER,
    )
}

fn render_copilot_hook_command_sh(script_path: &Path, bridge_path: &Path) -> String {
    let script_literal = sh_single_quoted(&script_path.to_string_lossy());
    let bridge_literal = sh_single_quoted(&bridge_path.to_string_lossy());

    format!(
        "sh {script} --bridge-path {bridge} --provider {provider} # {marker}",
        script = script_literal,
        bridge = bridge_literal,
        provider = COPILOT_PROVIDER,
        marker = SESSIONHUB_HOOK_COMMAND_MARKER,
    )
}

fn render_copilot_integration(bridge_path: &Path, hook_scripts_root: &Path) -> Result<String, String> {
    let integration = serde_json::json!({
        "version": 1,
        "sessionHub": managed_provider_metadata(COPILOT_PROVIDER, bridge_path),
        "hooks": {
            "sessionStart": [{ "type": "command", "powershell": render_copilot_hook_command(&hook_scripts_root.join("on-session-start.ps1"), bridge_path), "command": render_copilot_hook_command_sh(&hook_scripts_root.join("on-session-start.sh"), bridge_path) }],
            "sessionEnd": [{ "type": "command", "powershell": render_copilot_hook_command(&hook_scripts_root.join("on-session-end.ps1"), bridge_path), "command": render_copilot_hook_command_sh(&hook_scripts_root.join("on-session-end.sh"), bridge_path) }],
            "userPromptSubmitted": [{ "type": "command", "powershell": render_copilot_hook_command(&hook_scripts_root.join("on-user-prompt-submitted.ps1"), bridge_path), "command": render_copilot_hook_command_sh(&hook_scripts_root.join("on-user-prompt-submitted.sh"), bridge_path) }],
            "preToolUse": [{ "type": "command", "powershell": render_copilot_hook_command(&hook_scripts_root.join("on-pre-tool-use.ps1"), bridge_path), "command": render_copilot_hook_command_sh(&hook_scripts_root.join("on-pre-tool-use.sh"), bridge_path) }],
            "postToolUse": [{ "type": "command", "powershell": render_copilot_hook_command(&hook_scripts_root.join("on-post-tool-use.ps1"), bridge_path), "command": render_copilot_hook_command_sh(&hook_scripts_root.join("on-post-tool-use.sh"), bridge_path) }],
            "errorOccurred": [{ "type": "command", "powershell": render_copilot_hook_command(&hook_scripts_root.join("on-error-occurred.ps1"), bridge_path), "command": render_copilot_hook_command_sh(&hook_scripts_root.join("on-error-occurred.sh"), bridge_path) }],
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

    if let Err(error) = ensure_copilot_hook_scripts_installed() {
        return build_install_failure_status(
            COPILOT_PROVIDER,
            Some(config_path.clone()),
            diagnostics,
            error,
        );
    }

    let hook_scripts_root = match default_copilot_hook_scripts_root() {
        Ok(path) => path,
        Err(error) => {
            return build_install_failure_status(
                COPILOT_PROVIDER,
                Some(config_path),
                diagnostics,
                error,
            );
        }
    };

    let content = match render_copilot_integration(&bridge_path, &hook_scripts_root) {
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

    if let Err(error) = write_provider_integration_file(&config_path, &content) {
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

pub(crate) fn uninstall_copilot_integration(
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

    remove_copilot_hook_scripts();

    if config_path.exists() {
        if let Err(error) = fs::remove_file(&config_path) {
            return build_install_failure_status(
                COPILOT_PROVIDER,
                Some(config_path),
                diagnostics,
                format!("failed to remove Copilot integration file: {error}"),
            );
        }
    }

    build_provider_integration_status(
        COPILOT_PROVIDER,
        ProviderIntegrationState::Missing,
        Some(config_path),
        diagnostics,
        None,
        None,
    )
}
