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
const HOOK_SCRIPT_VERSION: &str = "3";

// Node.js 主軌（.cjs，強制 CommonJS）
const MODULE_RECORD_EVENT_JS: &str =
    include_str!("../../../hooks/copilot/modules/record-event.cjs");
const MODULE_NOTIFY_JS: &str = include_str!("../../../hooks/copilot/modules/notify.cjs");
const SCRIPT_ON_SESSION_START_JS: &str =
    include_str!("../../../hooks/copilot/on-session-start.cjs");
const SCRIPT_ON_SESSION_END_JS: &str = include_str!("../../../hooks/copilot/on-session-end.cjs");
const SCRIPT_ON_USER_PROMPT_SUBMITTED_JS: &str =
    include_str!("../../../hooks/copilot/on-user-prompt-submitted.cjs");
const SCRIPT_ON_PRE_TOOL_USE_JS: &str =
    include_str!("../../../hooks/copilot/on-pre-tool-use.cjs");
const SCRIPT_ON_POST_TOOL_USE_JS: &str =
    include_str!("../../../hooks/copilot/on-post-tool-use.cjs");
const SCRIPT_ON_ERROR_OCCURRED_JS: &str =
    include_str!("../../../hooks/copilot/on-error-occurred.cjs");

// sh fallback（無 node 環境時的手動退路）
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

fn hook_script_entries() -> [(&'static str, &'static str); 15] {
    [
        ("modules/record-event.cjs", MODULE_RECORD_EVENT_JS),
        ("modules/notify.cjs", MODULE_NOTIFY_JS),
        ("on-session-start.cjs", SCRIPT_ON_SESSION_START_JS),
        ("on-session-end.cjs", SCRIPT_ON_SESSION_END_JS),
        (
            "on-user-prompt-submitted.cjs",
            SCRIPT_ON_USER_PROMPT_SUBMITTED_JS,
        ),
        ("on-pre-tool-use.cjs", SCRIPT_ON_PRE_TOOL_USE_JS),
        ("on-post-tool-use.cjs", SCRIPT_ON_POST_TOOL_USE_JS),
        ("on-error-occurred.cjs", SCRIPT_ON_ERROR_OCCURRED_JS),
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
    install_hook_scripts("Copilot", &root, &hook_script_entries(), HOOK_SCRIPT_VERSION)?;
    super::install_notification_binary(&root)?;
    Ok(root)
}

/// 移除 SessionHub 安裝的 Copilot hook 腳本（`~/.copilot/hooks`），保留使用者自訂檔案
fn remove_copilot_hook_scripts() {
    let Ok(root) = default_copilot_hook_scripts_root() else {
        return;
    };
    super::uninstall_hook_scripts(&root, &hook_script_entries());
    super::uninstall_notification_binary(&root);
}

fn sh_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// 產生 Node.js 主軌 hook 命令。Copilot 在各平台以 shell 執行 `command` 欄，
/// shell 會由 PATH 解析 `node`；無 node 環境時可改用磁碟上保留的 .sh 腳本手動退路。
fn render_copilot_hook_command(script_path: &Path, bridge_path: &Path) -> String {
    let script_literal = sh_single_quoted(&script_path.to_string_lossy());
    let bridge_literal = sh_single_quoted(&bridge_path.to_string_lossy());

    format!(
        "node {script} --bridge-path {bridge} --provider {provider} # {marker}",
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
            "sessionStart": [{ "type": "command", "command": render_copilot_hook_command(&hook_scripts_root.join("on-session-start.cjs"), bridge_path) }],
            "sessionEnd": [{ "type": "command", "command": render_copilot_hook_command(&hook_scripts_root.join("on-session-end.cjs"), bridge_path) }],
            "userPromptSubmitted": [{ "type": "command", "command": render_copilot_hook_command(&hook_scripts_root.join("on-user-prompt-submitted.cjs"), bridge_path) }],
            "preToolUse": [{ "type": "command", "command": render_copilot_hook_command(&hook_scripts_root.join("on-pre-tool-use.cjs"), bridge_path) }],
            "postToolUse": [{ "type": "command", "command": render_copilot_hook_command(&hook_scripts_root.join("on-post-tool-use.cjs"), bridge_path) }],
            "errorOccurred": [{ "type": "command", "command": render_copilot_hook_command(&hook_scripts_root.join("on-error-occurred.cjs"), bridge_path) }],
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
