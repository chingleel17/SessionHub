use std::fs;

use serde_json::{json, Value};

use crate::db::ensure_parent_dir;
use crate::settings::resolve_claude_settings_path;
use crate::types::*;

use super::bridge::read_bridge_diagnostics;
use super::{build_install_failure_status, build_provider_integration_status};

const SESSIONHUB_HOOK_COMMAND_MARKER: &str = "sessionhub-provider-event-bridge";

/// 支援新巢狀格式 `{ "matcher": "...", "hooks": [{...}] }` 與舊扁平格式
fn is_sessionhub_hook_group(group: &Value) -> bool {
    // 新格式：{ "matcher": "...", "hooks": [{ "type": "command", "command": "..." }] }
    if let Some(inner) = group.get("hooks").and_then(Value::as_array) {
        if inner.iter().any(|h| {
            h.get("command")
                .and_then(Value::as_str)
                .is_some_and(|cmd| cmd.contains(SESSIONHUB_HOOK_COMMAND_MARKER))
        }) {
            return true;
        }
    }
    // 舊格式：{ "type": "command", "command": "...", "commandWindows": "..." }
    group
        .get("command")
        .or_else(|| group.get("commandWindows"))
        .and_then(Value::as_str)
        .is_some_and(|cmd| cmd.contains(SESSIONHUB_HOOK_COMMAND_MARKER))
}

fn render_claude_stop_hook_command(bridge_path: &std::path::Path) -> String {
    let bridge_path_str = bridge_path.to_string_lossy().replace('\\', "/");
    let bridge_parent_str = bridge_path
        .parent()
        .unwrap_or(bridge_path)
        .to_string_lossy()
        .replace('\\', "/");
    // 用 bash 單引號包住 PowerShell 腳本，避免 Git Bash 展開 $ 變數
    // PowerShell 腳本內使用雙引號作字串字面量
    format!(
        concat!(
            "powershell.exe -NoProfile -NonInteractive -Command ",
            "'$d=[Console]::In.ReadToEnd();",
            "try{{$j=$d|ConvertFrom-Json -EA 0}}catch{{$j=[PSCustomObject]@{{}}}};",
            "$ts=[DateTimeOffset]::UtcNow.ToString(\"o\");",
            "$sid=if($j.session_id){{$j.session_id}}else{{$null}};",
            "$cwd=if($j.cwd){{$j.cwd}}else{{(Get-Location).Path}};",
            "$rec=[ordered]@{{version={version};provider=\"claude\";eventType=\"session.stop\";timestamp=$ts;sessionId=$sid;cwd=$cwd;sourcePath=$null;title=$null;error=$null}};",
            "$null=New-Item -ItemType Directory -Force -Path \"{bridge_parent}\";",
            "[System.IO.File]::AppendAllText(\"{bridge_path}\",($rec|ConvertTo-Json -Compress)+[System.Environment]::NewLine,[System.Text.UTF8Encoding]::new($false));",
            "# {marker}'"
        ),
        version = PROVIDER_INTEGRATION_VERSION,
        bridge_parent = bridge_parent_str,
        bridge_path = bridge_path_str,
        marker = SESSIONHUB_HOOK_COMMAND_MARKER,
    )
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

    let hooks_obj = root["hooks"].as_object_mut().ok_or("hooks 欄位不是物件")?;
    let stop_arr = hooks_obj.entry("Stop").or_insert_with(|| json!([]));
    if !stop_arr.is_array() {
        *stop_arr = json!([]);
    }
    let arr = stop_arr.as_array_mut().ok_or("hooks.Stop 不是陣列")?;

    arr.retain(|group| !is_sessionhub_hook_group(group));

    // 使用 Claude Code 規格的巢狀格式：{ "matcher": "", "hooks": [{...}] }
    arr.push(json!({
        "matcher": "",
        "hooks": [{
            "type": "command",
            "command": render_claude_stop_hook_command(bridge_path),
        }]
    }));

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
        if let Some(stop) = hooks.get_mut("Stop").and_then(Value::as_array_mut) {
            stop.retain(|g| !is_sessionhub_hook_group(g));
            if stop.is_empty() {
                hooks.remove("Stop");
            }
        }
        if hooks.is_empty() {
            root.as_object_mut().map(|o| o.remove("hooks"));
        }
    }

    serde_json::to_string_pretty(&root)
        .map_err(|e| format!("無法序列化 Claude settings.json：{e}"))
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

    let has_hook = root
        .get("hooks")
        .and_then(|h| h.get("Stop"))
        .and_then(Value::as_array)
        .is_some_and(|arr| arr.iter().any(is_sessionhub_hook_group));

    if has_hook {
        build_provider_integration_status(
            CLAUDE_PROVIDER,
            ProviderIntegrationState::Installed,
            Some(config_path),
            diagnostics,
            Some(PROVIDER_INTEGRATION_VERSION),
            None,
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
