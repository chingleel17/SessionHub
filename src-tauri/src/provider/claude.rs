use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::db::ensure_parent_dir;
use crate::settings::{resolve_claude_settings_path, resolve_effective_hook_scripts_root};
use crate::types::*;

use super::bridge::read_bridge_diagnostics;
use super::{build_install_failure_status, build_provider_integration_status};

const SESSIONHUB_HOOK_COMMAND_MARKER: &str = "sessionhub-provider-event-bridge";
const HOOK_SCRIPT_VERSION: &str = "2";

const CLAUDE_MANAGED_EVENTS: [&str; 5] = [
    "SessionStart",
    "PreToolUse",
    "PostToolUse",
    "UserPromptSubmit",
    "Stop",
];

const MODULE_DB_OPS: &str = include_str!("../../../.claude/hooks/modules/db-ops.psm1");
const MODULE_TASK_QUEUE: &str = include_str!("../../../.claude/hooks/modules/task-queue.psm1");
const MODULE_RECORD_EVENT: &str = include_str!("../../../.claude/hooks/modules/record-event.psm1");
const SCRIPT_ON_SESSION_START: &str = include_str!("../../../.claude/hooks/on-session-start.ps1");
const SCRIPT_ON_PRE_TOOL_USE: &str = include_str!("../../../.claude/hooks/on-pre-tool-use.ps1");
const SCRIPT_ON_POST_TOOL_USE: &str = include_str!("../../../.claude/hooks/on-post-tool-use.ps1");
const SCRIPT_ON_USER_PROMPT_SUBMIT: &str =
    include_str!("../../../.claude/hooks/on-user-prompt-submit.ps1");
const SCRIPT_ON_STOP: &str = include_str!("../../../.claude/hooks/on-stop.ps1");

const MODULE_DB_OPS_SH: &str = include_str!("../../../.claude/hooks/modules/db-ops.sh");
const MODULE_TASK_QUEUE_SH: &str = include_str!("../../../.claude/hooks/modules/task-queue.sh");
const MODULE_RECORD_EVENT_SH: &str =
    include_str!("../../../.claude/hooks/modules/record-event.sh");
const SCRIPT_ON_SESSION_START_SH: &str =
    include_str!("../../../.claude/hooks/on-session-start.sh");
const SCRIPT_ON_PRE_TOOL_USE_SH: &str =
    include_str!("../../../.claude/hooks/on-pre-tool-use.sh");
const SCRIPT_ON_POST_TOOL_USE_SH: &str =
    include_str!("../../../.claude/hooks/on-post-tool-use.sh");
const SCRIPT_ON_USER_PROMPT_SUBMIT_SH: &str =
    include_str!("../../../.claude/hooks/on-user-prompt-submit.sh");
const SCRIPT_ON_STOP_SH: &str = include_str!("../../../.claude/hooks/on-stop.sh");

fn hook_script_entries() -> [(&'static str, &'static str); 16] {
    [
        ("modules/db-ops.psm1", MODULE_DB_OPS),
        ("modules/task-queue.psm1", MODULE_TASK_QUEUE),
        ("modules/record-event.psm1", MODULE_RECORD_EVENT),
        ("on-session-start.ps1", SCRIPT_ON_SESSION_START),
        ("on-pre-tool-use.ps1", SCRIPT_ON_PRE_TOOL_USE),
        ("on-post-tool-use.ps1", SCRIPT_ON_POST_TOOL_USE),
        ("on-user-prompt-submit.ps1", SCRIPT_ON_USER_PROMPT_SUBMIT),
        ("on-stop.ps1", SCRIPT_ON_STOP),
        ("modules/db-ops.sh", MODULE_DB_OPS_SH),
        ("modules/task-queue.sh", MODULE_TASK_QUEUE_SH),
        ("modules/record-event.sh", MODULE_RECORD_EVENT_SH),
        ("on-session-start.sh", SCRIPT_ON_SESSION_START_SH),
        ("on-pre-tool-use.sh", SCRIPT_ON_PRE_TOOL_USE_SH),
        ("on-post-tool-use.sh", SCRIPT_ON_POST_TOOL_USE_SH),
        ("on-user-prompt-submit.sh", SCRIPT_ON_USER_PROMPT_SUBMIT_SH),
        ("on-stop.sh", SCRIPT_ON_STOP_SH),
    ]
}

pub(crate) fn ensure_claude_hook_scripts_installed(
    hook_scripts_path: Option<&str>,
) -> Result<PathBuf, String> {
    let root = resolve_effective_hook_scripts_root(hook_scripts_path)?;
    for (relative_path, content) in hook_script_entries() {
        let path = root.join(relative_path);
        ensure_parent_dir(&path)?;
        fs::write(&path, content).map_err(|error| {
            format!(
                "failed to write Claude hook script {}: {error}",
                path.display()
            )
        })?;
    }

    let version_path = root.join(".version");
    fs::write(&version_path, HOOK_SCRIPT_VERSION).map_err(|error| {
        format!(
            "failed to write Claude hook version marker {}: {error}",
            version_path.display()
        )
    })?;

    Ok(root)
}

/// 支援新巢狀格式 `{ "matcher": "...", "hooks": [{...}] }` 與舊扁平格式
fn is_sessionhub_hook_group(group: &Value) -> bool {
    let contains_marker = |v: &Value| {
        v.as_str()
            .is_some_and(|s| s.contains(SESSIONHUB_HOOK_COMMAND_MARKER))
    };
    if let Some(inner) = group.get("hooks").and_then(Value::as_array) {
        if inner.iter().any(|h| {
            h.get("command").is_some_and(&contains_marker)
                || h.get("commandWindows").is_some_and(&contains_marker)
        }) {
            return true;
        }
    }
    group.get("command").is_some_and(&contains_marker)
        || group.get("commandWindows").is_some_and(&contains_marker)
}

fn powershell_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn render_claude_hook_command(script_path: &Path, bridge_path: &Path) -> String {
    let script_literal = powershell_single_quoted(&script_path.to_string_lossy());
    let bridge_literal = powershell_single_quoted(&bridge_path.to_string_lossy());
    let provider_literal = powershell_single_quoted(CLAUDE_PROVIDER);

    format!(
        "pwsh -NoProfile -ExecutionPolicy Bypass -File {script} -BridgePath {bridge} -Provider {provider} # {marker}",
        script = script_literal,
        bridge = bridge_literal,
        provider = provider_literal,
        marker = SESSIONHUB_HOOK_COMMAND_MARKER,
    )
}

fn sh_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn render_claude_hook_command_sh(script_path: &Path, bridge_path: &Path) -> String {
    let script_literal = sh_single_quoted(&script_path.to_string_lossy());
    let bridge_literal = sh_single_quoted(&bridge_path.to_string_lossy());
    format!(
        "sh {script} --bridge-path {bridge} --provider {provider} # {marker}",
        script = script_literal,
        bridge = bridge_literal,
        provider = CLAUDE_PROVIDER,
        marker = SESSIONHUB_HOOK_COMMAND_MARKER,
    )
}

fn managed_hook_group(command_windows: String, command_sh: String, matcher: Option<&str>) -> Value {
    let mut group = json!({
        "hooks": [{
            "type": "command",
            "command": command_sh,
            "commandWindows": command_windows,
        }]
    });
    if let Some(matcher) = matcher {
        group["matcher"] = Value::String(matcher.to_string());
    }
    group
}

fn render_claude_integration(
    bridge_path: &Path,
    hook_scripts_root: &Path,
    existing_content: Option<&str>,
    config_path: &Path,
) -> Result<String, String> {
    let mut root: Value = match existing_content {
        None => json!({}),
        Some(c) if c.trim().is_empty() => json!({}),
        Some(c) => serde_json::from_str(c).map_err(|e| {
            format!(
                "settings.json 不是合法的 JSON 格式（可能含有 JSONC 備注）：{e}\n檔案路徑：{}\n請先備份並移除備注後重試，或手動刪除此檔案。",
                config_path.display()
            )
        })?,
    };

    if !root.is_object() {
        root = json!({});
    }

    if !root.get("hooks").is_some_and(Value::is_object) {
        root["hooks"] = json!({});
    }

    let managed_groups: &[(&str, &str, &str, Option<&str>)] = &[
        (
            "SessionStart",
            "on-session-start.ps1",
            "on-session-start.sh",
            Some("startup|resume|clear|compact"),
        ),
        ("PreToolUse", "on-pre-tool-use.ps1", "on-pre-tool-use.sh", None),
        ("PostToolUse", "on-post-tool-use.ps1", "on-post-tool-use.sh", None),
        ("UserPromptSubmit", "on-user-prompt-submit.ps1", "on-user-prompt-submit.sh", None),
        ("Stop", "on-stop.ps1", "on-stop.sh", None),
    ];

    for (event_name, script_ps1, script_sh, matcher) in managed_groups {
        let mut groups = root["hooks"]
            .get(event_name)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        groups.retain(|group| !is_sessionhub_hook_group(group));
        groups.push(managed_hook_group(
            render_claude_hook_command(&hook_scripts_root.join(script_ps1), bridge_path),
            render_claude_hook_command_sh(&hook_scripts_root.join(script_sh), bridge_path),
            *matcher,
        ));
        root["hooks"][event_name] = Value::Array(groups);
    }

    serde_json::to_string_pretty(&root).map_err(|e| format!("無法序列化 Claude settings.json：{e}"))
}

fn remove_claude_integration_from_content(
    content: &str,
    config_path: &Path,
) -> Result<String, String> {
    let mut root: Value = serde_json::from_str(content)
        .map_err(|e| format!("settings.json 格式不合法（{}）：{e}", config_path.display()))?;

    if let Some(hooks) = root.get_mut("hooks").and_then(Value::as_object_mut) {
        for event_name in CLAUDE_MANAGED_EVENTS {
            if let Some(arr) = hooks.get_mut(event_name).and_then(Value::as_array_mut) {
                arr.retain(|g| !is_sessionhub_hook_group(g));
                if arr.is_empty() {
                    hooks.remove(event_name);
                }
            }
        }
        if hooks.is_empty() {
            root.as_object_mut().map(|o| o.remove("hooks"));
        }
    }

    serde_json::to_string_pretty(&root).map_err(|e| format!("無法序列化 Claude settings.json：{e}"))
}

fn has_all_managed_claude_events(root: &Value) -> bool {
    CLAUDE_MANAGED_EVENTS.iter().all(|event_name| {
        root.get("hooks")
            .and_then(|hooks| hooks.get(*event_name))
            .and_then(Value::as_array)
            .is_some_and(|arr| arr.iter().any(is_sessionhub_hook_group))
    })
}

fn has_any_managed_claude_event(root: &Value) -> bool {
    CLAUDE_MANAGED_EVENTS.iter().any(|event_name| {
        root.get("hooks")
            .and_then(|hooks| hooks.get(*event_name))
            .and_then(Value::as_array)
            .is_some_and(|arr| arr.iter().any(is_sessionhub_hook_group))
    })
}

fn effective_hook_status_path(hook_scripts_path: Option<&str>) -> Option<PathBuf> {
    resolve_effective_hook_scripts_root(hook_scripts_path).ok()
}

pub(crate) fn detect_claude_integration_status(
    hook_scripts_path: Option<&str>,
) -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(CLAUDE_PROVIDER);
    let config_path = match resolve_claude_settings_path() {
        Ok(p) => p,
        Err(error) => {
            return build_provider_integration_status(
                CLAUDE_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                None,
                Some(error),
            );
        }
    };

    let status_path = effective_hook_status_path(hook_scripts_path).or(Some(config_path.clone()));

    if !config_path.exists() {
        return build_provider_integration_status(
            CLAUDE_PROVIDER,
            ProviderIntegrationState::Missing,
            status_path,
            diagnostics,
            None,
            None,
        );
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(error) => {
            return build_provider_integration_status(
                CLAUDE_PROVIDER,
                ProviderIntegrationState::Error,
                status_path,
                diagnostics,
                None,
                Some(format!(
                    "無法讀取 Claude settings.json（{}）：{error}",
                    config_path.display()
                )),
            );
        }
    };

    let root: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(error) => {
            return build_provider_integration_status(
                CLAUDE_PROVIDER,
                ProviderIntegrationState::Error,
                status_path,
                diagnostics,
                None,
                Some(format!(
                    "settings.json 格式不合法（{}）：{error}。\n若檔案使用 JSONC 格式（含有 // 或 /* 備注），請移除備注後重試。",
                    config_path.display()
                )),
            );
        }
    };

    if has_all_managed_claude_events(&root) {
        build_provider_integration_status(
            CLAUDE_PROVIDER,
            ProviderIntegrationState::Installed,
            status_path,
            diagnostics,
            Some(PROVIDER_INTEGRATION_VERSION),
            None,
        )
    } else if has_any_managed_claude_event(&root) {
        build_provider_integration_status(
            CLAUDE_PROVIDER,
            ProviderIntegrationState::Outdated,
            status_path,
            diagnostics,
            None,
            Some("缺少部分 Claude hook 事件，請重新安裝以完整支援所有事件。".to_string()),
        )
    } else {
        build_provider_integration_status(
            CLAUDE_PROVIDER,
            ProviderIntegrationState::Missing,
            status_path,
            diagnostics,
            None,
            None,
        )
    }
}

pub(crate) fn install_or_update_claude_integration(
    hook_scripts_path: Option<&str>,
) -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(CLAUDE_PROVIDER);
    let config_path = match resolve_claude_settings_path() {
        Ok(p) => p,
        Err(error) => {
            return build_provider_integration_status(
                CLAUDE_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                None,
                Some(error),
            );
        }
    };

    let Some(bridge_path) = diagnostics.bridge_path.clone() else {
        return build_provider_integration_status(
            CLAUDE_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            effective_hook_status_path(hook_scripts_path).or(Some(config_path)),
            diagnostics,
            None,
            Some(
                "無法解析 Claude bridge 路徑。\
                bridge 檔案將在第一次 hook 事件觸發時自動建立（通常位於 <copilot_root>/claude-events.jsonl）。\
                請確認 Claude Root 目錄設定正確後重試。"
                    .to_string(),
            ),
        );
    };

    if let Err(error) = ensure_claude_hook_scripts_installed(hook_scripts_path) {
        return build_install_failure_status(
            CLAUDE_PROVIDER,
            effective_hook_status_path(hook_scripts_path).or(Some(config_path)),
            diagnostics,
            error,
        );
    }

    let hook_scripts_root = match resolve_effective_hook_scripts_root(hook_scripts_path) {
        Ok(path) => path,
        Err(error) => {
            return build_install_failure_status(
                CLAUDE_PROVIDER,
                Some(config_path),
                diagnostics,
                error,
            );
        }
    };

    let existing_content = fs::read_to_string(&config_path).ok();
    let content = match render_claude_integration(
        &bridge_path,
        &hook_scripts_root,
        existing_content.as_deref(),
        &config_path,
    ) {
        Ok(c) => c,
        Err(error) => {
            return build_install_failure_status(
                CLAUDE_PROVIDER,
                Some(config_path),
                diagnostics,
                error,
            );
        }
    };

    if let Err(error) = ensure_parent_dir(&config_path)
        .and_then(|_| super::write_provider_integration_file(&config_path, &content))
    {
        return build_install_failure_status(
            CLAUDE_PROVIDER,
            Some(config_path),
            diagnostics,
            error,
        );
    }

    detect_claude_integration_status(hook_scripts_path)
}

/// 移除 SessionHub 安裝的 Claude hook 腳本（原生 `~/.claude/hooks` 或使用者自訂路徑），
/// 保留同目錄中使用者自訂的其他 hook 檔案
fn remove_hook_scripts(hook_scripts_path: Option<&str>) {
    let Ok(root) = resolve_effective_hook_scripts_root(hook_scripts_path) else {
        return;
    };
    super::uninstall_hook_scripts(&root, &hook_script_entries());
}

pub(crate) fn uninstall_claude_integration(
    hook_scripts_path: Option<&str>,
) -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(CLAUDE_PROVIDER);
    let config_path = match resolve_claude_settings_path() {
        Ok(p) => p,
        Err(error) => {
            return build_provider_integration_status(
                CLAUDE_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                None,
                Some(error),
            );
        }
    };

    remove_hook_scripts(hook_scripts_path);

    let status_path = effective_hook_status_path(hook_scripts_path).or(Some(config_path.clone()));

    if !config_path.exists() {
        return build_provider_integration_status(
            CLAUDE_PROVIDER,
            ProviderIntegrationState::Missing,
            status_path,
            diagnostics,
            None,
            None,
        );
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(error) => {
            return build_provider_integration_status(
                CLAUDE_PROVIDER,
                ProviderIntegrationState::Error,
                status_path,
                diagnostics,
                None,
                Some(format!(
                    "無法讀取 Claude settings.json（{}）：{error}",
                    config_path.display()
                )),
            );
        }
    };

    let new_content = match remove_claude_integration_from_content(&content, &config_path) {
        Ok(c) => c,
        Err(error) => {
            return build_install_failure_status(CLAUDE_PROVIDER, status_path, diagnostics, error);
        }
    };

    if let Err(error) = super::write_provider_integration_file(&config_path, &new_content) {
        return build_install_failure_status(CLAUDE_PROVIDER, status_path, diagnostics, error);
    }

    build_provider_integration_status(
        CLAUDE_PROVIDER,
        ProviderIntegrationState::Missing,
        status_path,
        diagnostics,
        None,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_bridge_path() -> PathBuf {
        PathBuf::from("C:/Users/test/AppData/Roaming/SessionHub/provider-bridge/claude.jsonl")
    }

    fn fake_hook_root() -> PathBuf {
        PathBuf::from("C:/Users/test/.claude/hooks")
    }

    #[test]
    fn render_integration_has_all_events() {
        let bridge = fake_bridge_path();
        let result =
            render_claude_integration(&bridge, &fake_hook_root(), None, Path::new("settings.json"))
                .unwrap();
        let v: Value = serde_json::from_str(&result).unwrap();
        for event in CLAUDE_MANAGED_EVENTS {
            assert!(
                v["hooks"][event].is_array(),
                "missing event hook array: {event}"
            );
            let arr = v["hooks"][event].as_array().unwrap();
            assert!(
                arr.iter().any(is_sessionhub_hook_group),
                "no sessionhub group for event: {event}"
            );
        }
    }

    #[test]
    fn idempotent_reinstall_does_not_duplicate() {
        let bridge = fake_bridge_path();
        let first =
            render_claude_integration(&bridge, &fake_hook_root(), None, Path::new("settings.json"))
                .unwrap();
        let second = render_claude_integration(
            &bridge,
            &fake_hook_root(),
            Some(&first),
            Path::new("settings.json"),
        )
        .unwrap();
        let v: Value = serde_json::from_str(&second).unwrap();
        for event in CLAUDE_MANAGED_EVENTS {
            let arr = v["hooks"][event].as_array().unwrap();
            let count = arr.iter().filter(|g| is_sessionhub_hook_group(g)).count();
            assert_eq!(count, 1, "duplicate sessionhub group for event: {event}");
        }
    }

    #[test]
    fn uninstall_removes_all_events() {
        let bridge = fake_bridge_path();
        let installed =
            render_claude_integration(&bridge, &fake_hook_root(), None, Path::new("settings.json"))
                .unwrap();
        let removed =
            remove_claude_integration_from_content(&installed, Path::new("settings.json")).unwrap();
        let v: Value = serde_json::from_str(&removed).unwrap();
        assert!(
            !has_any_managed_claude_event(&v),
            "hooks should be empty after uninstall"
        );
    }

    #[test]
    fn detect_partial_install_returns_outdated() {
        let marker = SESSIONHUB_HOOK_COMMAND_MARKER;
        let old_settings = format!(
            "{{\"hooks\":{{\"Stop\":[{{\"matcher\":\"\",\"hooks\":[{{\"type\":\"command\",\"command\":\"# {marker}\"}}]}}]}}}}"
        );
        let v: Value = serde_json::from_str(&old_settings).unwrap();
        assert!(
            has_any_managed_claude_event(&v),
            "should detect partial install"
        );
        assert!(
            !has_all_managed_claude_events(&v),
            "should not report as fully installed"
        );
    }

    #[test]
    fn session_start_hook_has_matcher() {
        let bridge = fake_bridge_path();
        let result =
            render_claude_integration(&bridge, &fake_hook_root(), None, Path::new("settings.json"))
                .unwrap();
        let v: Value = serde_json::from_str(&result).unwrap();
        let arr = v["hooks"]["SessionStart"].as_array().unwrap();
        let group = arr.iter().find(|g| is_sessionhub_hook_group(g)).unwrap();
        assert_eq!(group["matcher"], "startup|resume|clear|compact");
    }

    #[test]
    fn render_integration_uses_script_file_commands() {
        let bridge = fake_bridge_path();
        let result =
            render_claude_integration(&bridge, &fake_hook_root(), None, Path::new("settings.json"))
                .unwrap();
        assert!(result.contains("on-session-start.ps1"));
        assert!(result.contains("on-post-tool-use.ps1"));
        assert!(result.contains("on-session-start.sh"));
        assert!(result.contains("on-post-tool-use.sh"));
    }

    #[test]
    fn render_integration_has_both_command_and_command_windows() {
        let bridge = fake_bridge_path();
        let result =
            render_claude_integration(&bridge, &fake_hook_root(), None, Path::new("settings.json"))
                .unwrap();
        let v: Value = serde_json::from_str(&result).unwrap();
        for event in CLAUDE_MANAGED_EVENTS {
            let arr = v["hooks"][event].as_array().unwrap();
            let group = arr.iter().find(|g| is_sessionhub_hook_group(g)).unwrap();
            let hooks = group["hooks"].as_array().unwrap();
            let hook = &hooks[0];
            assert!(
                hook.get("command").is_some(),
                "missing 'command' (sh) for event {event}"
            );
            assert!(
                hook.get("commandWindows").is_some(),
                "missing 'commandWindows' (ps1) for event {event}"
            );
        }
    }
}
