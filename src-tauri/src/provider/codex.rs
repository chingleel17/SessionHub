use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::settings::{default_codex_hook_scripts_root, resolve_codex_root};
use crate::types::*;

use super::bridge::read_bridge_diagnostics;
use super::{
    build_install_failure_status, build_provider_integration_status, install_hook_scripts,
    managed_provider_metadata, validate_integration_target, validate_managed_metadata,
    write_provider_integration_file,
};

const SESSIONHUB_HOOK_COMMAND_MARKER: &str = "sessionhub-provider-event-bridge";
const HOOK_SCRIPT_VERSION: &str = "3";
const CODEX_MANAGED_EVENTS: [&str; 5] = [
    "SessionStart",
    "PreToolUse",
    "PostToolUse",
    "UserPromptSubmit",
    "Stop",
];

// Node.js 主軌（.cjs，強制 CommonJS）
const MODULE_RECORD_EVENT_JS: &str = include_str!("../../../hooks/codex/modules/record-event.cjs");
const MODULE_NOTIFY_JS: &str = include_str!("../../../hooks/codex/modules/notify.cjs");
const SCRIPT_ON_SESSION_START_JS: &str = include_str!("../../../hooks/codex/on-session-start.cjs");
const SCRIPT_ON_PRE_TOOL_USE_JS: &str = include_str!("../../../hooks/codex/on-pre-tool-use.cjs");
const SCRIPT_ON_POST_TOOL_USE_JS: &str = include_str!("../../../hooks/codex/on-post-tool-use.cjs");
const SCRIPT_ON_USER_PROMPT_SUBMIT_JS: &str =
    include_str!("../../../hooks/codex/on-user-prompt-submit.cjs");
const SCRIPT_ON_STOP_JS: &str = include_str!("../../../hooks/codex/on-stop.cjs");

// sh fallback（無 node 環境時的手動退路）
const MODULE_RECORD_EVENT_SH: &str = include_str!("../../../hooks/codex/modules/record-event.sh");
const SCRIPT_ON_SESSION_START_SH: &str = include_str!("../../../hooks/codex/on-session-start.sh");
const SCRIPT_ON_PRE_TOOL_USE_SH: &str = include_str!("../../../hooks/codex/on-pre-tool-use.sh");
const SCRIPT_ON_POST_TOOL_USE_SH: &str = include_str!("../../../hooks/codex/on-post-tool-use.sh");
const SCRIPT_ON_USER_PROMPT_SUBMIT_SH: &str =
    include_str!("../../../hooks/codex/on-user-prompt-submit.sh");
const SCRIPT_ON_STOP_SH: &str = include_str!("../../../hooks/codex/on-stop.sh");

fn hook_script_entries() -> [(&'static str, &'static str); 13] {
    [
        ("modules/record-event.cjs", MODULE_RECORD_EVENT_JS),
        ("modules/notify.cjs", MODULE_NOTIFY_JS),
        ("on-session-start.cjs", SCRIPT_ON_SESSION_START_JS),
        ("on-pre-tool-use.cjs", SCRIPT_ON_PRE_TOOL_USE_JS),
        ("on-post-tool-use.cjs", SCRIPT_ON_POST_TOOL_USE_JS),
        ("on-user-prompt-submit.cjs", SCRIPT_ON_USER_PROMPT_SUBMIT_JS),
        ("on-stop.cjs", SCRIPT_ON_STOP_JS),
        ("modules/record-event.sh", MODULE_RECORD_EVENT_SH),
        ("on-session-start.sh", SCRIPT_ON_SESSION_START_SH),
        ("on-pre-tool-use.sh", SCRIPT_ON_PRE_TOOL_USE_SH),
        ("on-post-tool-use.sh", SCRIPT_ON_POST_TOOL_USE_SH),
        ("on-user-prompt-submit.sh", SCRIPT_ON_USER_PROMPT_SUBMIT_SH),
        ("on-stop.sh", SCRIPT_ON_STOP_SH),
    ]
}

pub(crate) fn ensure_codex_hook_scripts_installed() -> Result<PathBuf, String> {
    let root = default_codex_hook_scripts_root()?;
    install_hook_scripts("Codex", &root, &hook_script_entries(), HOOK_SCRIPT_VERSION)?;
    super::install_notification_binary(&root)?;
    Ok(root)
}

/// 移除 SessionHub 安裝的 Codex hook 腳本（`~/.codex/hooks`），保留使用者自訂檔案
fn remove_codex_hook_scripts() {
    let Ok(root) = default_codex_hook_scripts_root() else {
        return;
    };
    super::uninstall_hook_scripts(&root, &hook_script_entries());
    super::uninstall_notification_binary(&root);
}

pub(crate) fn resolve_codex_integration_path(codex_root: &Path) -> PathBuf {
    codex_root.join(CODEX_HOOK_FILE_NAME)
}

fn sh_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// 產生 Node.js 主軌 hook 命令。Codex 以 shell 執行 `command` 欄，
/// shell 會由 PATH 解析 `node`；無 node 環境時可改用磁碟上保留的 .sh 腳本手動退路。
fn render_codex_hook_command(script_path: &Path, bridge_path: &Path) -> String {
    let script_literal = sh_single_quoted(&script_path.to_string_lossy());
    let bridge_literal = sh_single_quoted(&bridge_path.to_string_lossy());

    format!(
        "node {script} --bridge-path {bridge} --provider {provider} # {marker}",
        script = script_literal,
        bridge = bridge_literal,
        provider = CODEX_PROVIDER,
        marker = SESSIONHUB_HOOK_COMMAND_MARKER,
    )
}

fn is_sessionhub_hook_group(group: &Value) -> bool {
    let contains_marker = |v: &Value| {
        v.as_str()
            .is_some_and(|s| s.contains(SESSIONHUB_HOOK_COMMAND_MARKER))
    };
    // 舊版 v4 內嵌 PowerShell group 特徵：commandWindows 含 "provider = 'codex'"
    let is_legacy_codex_group = |v: &Value| {
        v.as_str()
            .is_some_and(|s| s.contains("provider = 'codex'"))
    };
    let matches_hook = |h: &Value| {
        h.get("command").is_some_and(&contains_marker)
            || h.get("commandWindows").is_some_and(&contains_marker)
            || h.get("commandWindows").is_some_and(&is_legacy_codex_group)
    };
    if let Some(inner) = group.get("hooks").and_then(Value::as_array) {
        if inner.iter().any(matches_hook) {
            return true;
        }
    }
    contains_marker(group.get("command").unwrap_or(&Value::Null))
        || contains_marker(group.get("commandWindows").unwrap_or(&Value::Null))
        || is_legacy_codex_group(group.get("commandWindows").unwrap_or(&Value::Null))
}

fn managed_hook_group(command: String, matcher: Option<&str>) -> Value {
    let mut group = json!({
        "hooks": [{
            "type": "command",
            "command": command,
        }]
    });
    if let Some(matcher) = matcher {
        group["matcher"] = Value::String(matcher.to_string());
    }
    group
}

fn render_codex_integration(
    bridge_path: &Path,
    hook_scripts_root: &Path,
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

    let managed_groups: &[(&str, &str, Option<&str>)] = &[
        (
            "SessionStart",
            "on-session-start.cjs",
            Some("startup|resume|clear|compact"),
        ),
        ("PreToolUse", "on-pre-tool-use.cjs", None),
        ("PostToolUse", "on-post-tool-use.cjs", None),
        ("UserPromptSubmit", "on-user-prompt-submit.cjs", None),
        ("Stop", "on-stop.cjs", None),
    ];

    for (event_name, script_cjs, matcher) in managed_groups {
        let mut groups = root["hooks"]
            .get(event_name)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        groups.retain(|group| !is_sessionhub_hook_group(group));
        groups.push(managed_hook_group(
            render_codex_hook_command(&hook_scripts_root.join(script_cjs), bridge_path),
            *matcher,
        ));
        root["hooks"][event_name] = Value::Array(groups);
    }

    serde_json::to_string_pretty(&root)
        .map_err(|error| format!("failed to serialize Codex integration: {error}"))
}

fn has_all_managed_codex_events(root: &Value) -> bool {
    CODEX_MANAGED_EVENTS.iter().all(|event_name| {
        root.get("hooks")
            .and_then(|hooks| hooks.get(*event_name))
            .and_then(Value::as_array)
            .is_some_and(|arr| arr.iter().any(is_sessionhub_hook_group))
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

    if let Err(error) = ensure_codex_hook_scripts_installed() {
        return build_install_failure_status(
            CODEX_PROVIDER,
            Some(config_path.clone()),
            diagnostics,
            error,
        );
    }

    let hook_scripts_root = match default_codex_hook_scripts_root() {
        Ok(path) => path,
        Err(error) => {
            return build_install_failure_status(
                CODEX_PROVIDER,
                Some(config_path),
                diagnostics,
                error,
            );
        }
    };

    let existing_content = fs::read_to_string(&config_path).ok();
    let content =
        match render_codex_integration(&bridge_path, &hook_scripts_root, existing_content.as_deref())
        {
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

    if let Err(error) = write_provider_integration_file(&config_path, &content) {
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

    if !has_all_managed_codex_events(&root) {
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

pub(crate) fn uninstall_codex_integration(codex_root: Option<&str>) -> ProviderIntegrationStatus {
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

    remove_codex_hook_scripts();

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
                groups.retain(|g| !is_sessionhub_hook_group(g));
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
