use std::fs;

use serde_json::{json, Value};

use crate::db::ensure_parent_dir;
use crate::settings::resolve_claude_settings_path;
use crate::types::*;

use super::bridge::read_bridge_diagnostics;
use super::{build_install_failure_status, build_provider_integration_status};

const SESSIONHUB_HOOK_COMMAND_MARKER: &str = "sessionhub-provider-event-bridge";

const CLAUDE_MANAGED_EVENTS: [&str; 5] = [
    "SessionStart",
    "PreToolUse",
    "PostToolUse",
    "UserPromptSubmit",
    "Stop",
];

/// 支援新巢狀格式 `{ "matcher": "...", "hooks": [{...}] }` 與舊扁平格式
fn is_sessionhub_hook_group(group: &Value) -> bool {
    let contains_marker = |v: &Value| {
        v.as_str()
            .is_some_and(|s| s.contains(SESSIONHUB_HOOK_COMMAND_MARKER))
    };
    // 新格式：{ "matcher": "...", "hooks": [{ "type": "command", "command": "...", "commandWindows": "..." }] }
    if let Some(inner) = group.get("hooks").and_then(Value::as_array) {
        if inner.iter().any(|h| {
            h.get("command").is_some_and(&contains_marker)
                || h.get("commandWindows").is_some_and(&contains_marker)
        }) {
            return true;
        }
    }
    // 舊格式：{ "type": "command", "command": "...", "commandWindows": "..." }
    group.get("command").is_some_and(&contains_marker)
        || group.get("commandWindows").is_some_and(&contains_marker)
}

fn powershell_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// 生成 Claude hook 的 PowerShell 命令。
/// `title_snippet` 是設定 `$title` 變數的 PowerShell 片段，各 event 不同。
fn render_claude_hook_command(
    bridge_path: &std::path::Path,
    event_type: &str,
    title_snippet: &str,
) -> String {
    let bridge_path_literal =
        powershell_single_quoted(&bridge_path.to_string_lossy().replace('\\', "/"));
    let bridge_parent_literal = powershell_single_quoted(
        &bridge_path
            .parent()
            .unwrap_or(bridge_path)
            .to_string_lossy()
            .replace('\\', "/"),
    );
    let event_type_literal = powershell_single_quoted(event_type);
    let provider_literal = powershell_single_quoted(CLAUDE_PROVIDER);

    format!(
        concat!(
            "$payload = [Console]::In.ReadToEnd(); ",
            "if ([string]::IsNullOrWhiteSpace($payload)) {{ exit 0 }}; ",
            "$event = $payload | ConvertFrom-Json; ",
            "$timestamp = [DateTimeOffset]::UtcNow.ToString('o'); ",
            "$sessionId = if ($null -ne $event.session_id -and -not [string]::IsNullOrWhiteSpace([string]$event.session_id)) {{ [string]$event.session_id }} else {{ $null }}; ",
            "$cwd = if ($null -ne $event.cwd -and -not [string]::IsNullOrWhiteSpace([string]$event.cwd)) {{ [string]$event.cwd }} else {{ $null }}; ",
            "$sourcePath = if ($null -ne $event.transcript_path -and -not [string]::IsNullOrWhiteSpace([string]$event.transcript_path)) {{ [string]$event.transcript_path }} else {{ $null }}; ",
            "{title_snippet}",
            "$record = [ordered]@{{ version = {version}; provider = {provider}; eventType = {event_type}; timestamp = $timestamp; sessionId = $sessionId; cwd = $cwd; sourcePath = $sourcePath; title = $title; error = $null }}; ",
            "New-Item -ItemType Directory -Force -Path {bridge_parent} | Out-Null; ",
            "[System.IO.File]::AppendAllText({bridge_path}, (($record | ConvertTo-Json -Compress) + [Environment]::NewLine), [System.Text.UTF8Encoding]::new($false)); ",
            "# {marker}"
        ),
        title_snippet = title_snippet,
        version = PROVIDER_INTEGRATION_VERSION,
        provider = provider_literal,
        event_type = event_type_literal,
        bridge_parent = bridge_parent_literal,
        bridge_path = bridge_path_literal,
        marker = SESSIONHUB_HOOK_COMMAND_MARKER,
    )
}

fn managed_hook_group(command_windows: String, matcher: Option<&str>) -> Value {
    // command = "true" 為非 Windows 平台的 NOOP（bash 可執行）
    // commandWindows 為 Windows 平台執行的 PowerShell 腳本
    let mut group = json!({
        "hooks": [{
            "type": "command",
            "command": "true",
            "commandWindows": command_windows,
        }]
    });
    if let Some(matcher) = matcher {
        group["matcher"] = Value::String(matcher.to_string());
    }
    group
}

fn render_claude_integration(
    bridge_path: &std::path::Path,
    existing_content: Option<&str>,
    config_path: &std::path::Path,
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

    // SessionStart → source 欄位（startup/resume/clear/compact）作為 title
    let session_start_title = concat!(
        "$title = if ($null -ne $event.source -and -not [string]::IsNullOrWhiteSpace([string]$event.source)) ",
        "{ [string]$event.source } else { $null }; "
    );
    // PreToolUse / PostToolUse → tool_name 作為 title
    let tool_title = concat!(
        "$title = if ($null -ne $event.tool_name -and -not [string]::IsNullOrWhiteSpace([string]$event.tool_name)) ",
        "{ [string]$event.tool_name } else { $null }; "
    );
    // UserPromptSubmit → prompt 前 80 字作為 title
    let prompt_title = concat!(
        "$title = if ($null -ne $event.prompt -and -not [string]::IsNullOrWhiteSpace([string]$event.prompt)) ",
        "{ $s = [string]$event.prompt; if ($s.Length -gt 80) { $s.Substring(0, 80) } else { $s } } else { $null }; "
    );
    let null_title = "$title = $null; ";

    let managed_groups: &[(&str, &str, Option<&str>, &str)] = &[
        ("SessionStart", "session.started", Some("startup|resume|clear|compact"), session_start_title),
        ("PreToolUse", "tool.pre", None, tool_title),
        ("PostToolUse", "tool.post", None, tool_title),
        ("UserPromptSubmit", "prompt.submitted", None, prompt_title),
        ("Stop", "session.stop", None, null_title),
    ];

    for (event_name, event_type, matcher, title_snippet) in managed_groups {
        let mut groups = root["hooks"]
            .get(event_name)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        groups.retain(|group| !is_sessionhub_hook_group(group));
        groups.push(managed_hook_group(
            render_claude_hook_command(bridge_path, event_type, title_snippet),
            *matcher,
        ));
        root["hooks"][event_name] = Value::Array(groups);
    }

    serde_json::to_string_pretty(&root)
        .map_err(|e| format!("無法序列化 Claude settings.json：{e}"))
}

fn remove_claude_integration_from_content(
    content: &str,
    config_path: &std::path::Path,
) -> Result<String, String> {
    let mut root: Value = serde_json::from_str(content).map_err(|e| {
        format!(
            "settings.json 格式不合法（{}）：{e}",
            config_path.display()
        )
    })?;

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

    serde_json::to_string_pretty(&root)
        .map_err(|e| format!("無法序列化 Claude settings.json：{e}"))
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

pub(crate) fn detect_claude_integration_status(
    _claude_root: Option<&str>,
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

    if !config_path.exists() {
        return build_provider_integration_status(
            CLAUDE_PROVIDER,
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
            return build_provider_integration_status(
                CLAUDE_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path.clone()),
                diagnostics,
                None,
                Some(format!("無法讀取 Claude settings.json（{}）：{error}", config_path.display())),
            );
        }
    };

    let root: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(error) => {
            return build_provider_integration_status(
                CLAUDE_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path.clone()),
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
            Some(config_path),
            diagnostics,
            Some(PROVIDER_INTEGRATION_VERSION),
            None,
        )
    } else if has_any_managed_claude_event(&root) {
        // 只有部分事件（舊版安裝），需要更新
        build_provider_integration_status(
            CLAUDE_PROVIDER,
            ProviderIntegrationState::Outdated,
            Some(config_path),
            diagnostics,
            None,
            Some("缺少部分 Claude hook 事件，請重新安裝以完整支援所有事件。".to_string()),
        )
    } else {
        build_provider_integration_status(
            CLAUDE_PROVIDER,
            ProviderIntegrationState::Missing,
            Some(config_path),
            diagnostics,
            None,
            None,
        )
    }
}

pub(crate) fn install_or_update_claude_integration(
    _claude_root: Option<&str>,
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
            Some(config_path),
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

    let existing_content = fs::read_to_string(&config_path).ok();
    let content = match render_claude_integration(&bridge_path, existing_content.as_deref(), &config_path) {
        Ok(c) => c,
        Err(error) => {
            return build_install_failure_status(CLAUDE_PROVIDER, Some(config_path), diagnostics, error);
        }
    };

    if let Err(error) = ensure_parent_dir(&config_path)
        .and_then(|_| super::write_provider_integration_file(&config_path, &content))
    {
        return build_install_failure_status(CLAUDE_PROVIDER, Some(config_path), diagnostics, error);
    }

    detect_claude_integration_status(None)
}

pub(crate) fn uninstall_claude_integration(
    _claude_root: Option<&str>,
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

    if !config_path.exists() {
        return build_provider_integration_status(
            CLAUDE_PROVIDER,
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
            return build_provider_integration_status(
                CLAUDE_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path.clone()),
                diagnostics,
                None,
                Some(format!("無法讀取 Claude settings.json（{}）：{error}", config_path.display())),
            );
        }
    };

    let new_content = match remove_claude_integration_from_content(&content, &config_path) {
        Ok(c) => c,
        Err(error) => {
            return build_install_failure_status(CLAUDE_PROVIDER, Some(config_path), diagnostics, error);
        }
    };

    if let Err(error) = super::write_provider_integration_file(&config_path, &new_content) {
        return build_install_failure_status(CLAUDE_PROVIDER, Some(config_path), diagnostics, error);
    }

    build_provider_integration_status(
        CLAUDE_PROVIDER,
        ProviderIntegrationState::Missing,
        Some(config_path),
        diagnostics,
        None,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_bridge_path() -> std::path::PathBuf {
        std::path::PathBuf::from("C:/Users/test/AppData/Roaming/SessionHub/provider-bridge/claude.jsonl")
    }

    #[test]
    fn render_integration_has_all_events() {
        let bridge = fake_bridge_path();
        let result = render_claude_integration(&bridge, None, std::path::Path::new("settings.json")).unwrap();
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
        let first = render_claude_integration(&bridge, None, std::path::Path::new("settings.json")).unwrap();
        let second = render_claude_integration(&bridge, Some(&first), std::path::Path::new("settings.json")).unwrap();
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
        let installed = render_claude_integration(&bridge, None, std::path::Path::new("settings.json")).unwrap();
        let removed = remove_claude_integration_from_content(&installed, std::path::Path::new("settings.json")).unwrap();
        let v: Value = serde_json::from_str(&removed).unwrap();
        assert!(!has_any_managed_claude_event(&v), "hooks should be empty after uninstall");
    }

    #[test]
    fn detect_partial_install_returns_outdated() {
        // 模擬舊版只有 Stop hook 的 settings.json
        let marker = SESSIONHUB_HOOK_COMMAND_MARKER;
        let old_settings = format!(
            "{{\"hooks\":{{\"Stop\":[{{\"matcher\":\"\",\"hooks\":[{{\"type\":\"command\",\"command\":\"# {marker}\"}}]}}]}}}}"
        );
        let v: Value = serde_json::from_str(&old_settings).unwrap();
        assert!(has_any_managed_claude_event(&v), "should detect partial install");
        assert!(!has_all_managed_claude_events(&v), "should not report as fully installed");
    }

    #[test]
    fn session_start_hook_has_matcher() {
        let bridge = fake_bridge_path();
        let result = render_claude_integration(&bridge, None, std::path::Path::new("settings.json")).unwrap();
        let v: Value = serde_json::from_str(&result).unwrap();
        let arr = v["hooks"]["SessionStart"].as_array().unwrap();
        let group = arr.iter().find(|g| is_sessionhub_hook_group(g)).unwrap();
        assert_eq!(group["matcher"], "startup|resume|clear|compact");
    }
}
