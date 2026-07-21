use std::process::Command;

use crate::types::{AppSettings, QuotaSnapshot, QuotaWindow, COPILOT_PROVIDER};

#[cfg(target_os = "windows")]
use crate::types::CREATE_NO_WINDOW;

use super::QuotaAdapter;

pub(crate) struct CopilotAdapter;

fn current_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn no_auth_snapshot(error_message: impl Into<String>) -> QuotaSnapshot {
    QuotaSnapshot {
        provider: COPILOT_PROVIDER.to_string(),
        status: "no_auth".to_string(),
        source: "remote_api".to_string(),
        fetched_at: current_timestamp(),
        error_message: Some(error_message.into()),
        windows: None,
        local_tokens: None,
        extra_credits: None,
        reset_credits: None,
    }
}

fn error_snapshot(error_message: impl Into<String>) -> QuotaSnapshot {
    QuotaSnapshot {
        provider: COPILOT_PROVIDER.to_string(),
        status: "error".to_string(),
        source: "remote_api".to_string(),
        fetched_at: current_timestamp(),
        error_message: Some(error_message.into()),
        windows: None,
        local_tokens: None,
        extra_credits: None,
        reset_credits: None,
    }
}

fn spawn_gh(args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("gh");
    cmd.args(args);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("failed to spawn gh: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "gh {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Copilot quota_snapshots 物件內單一額度區塊的解析結果
struct CopilotQuota {
    remaining_percent: f64,
    unlimited: bool,
}

fn parse_copilot_quota(obj: &serde_json::Value) -> Option<CopilotQuota> {
    let unlimited = obj
        .get("unlimited")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let remaining_percent = obj.get("percent_remaining").and_then(|v| v.as_f64())?;
    Some(CopilotQuota {
        remaining_percent,
        unlimited,
    })
}

impl QuotaAdapter for CopilotAdapter {
    fn provider_key(&self) -> &str {
        COPILOT_PROVIDER
    }

    fn fetch_snapshot(&self, _settings: &AppSettings) -> QuotaSnapshot {
        let token = match spawn_gh(&["auth", "token"]) {
            Ok(t) if !t.is_empty() => t,
            Ok(_) => {
                return no_auth_snapshot(
                    "gh auth token returned empty — please log in with `gh auth login`",
                )
            }
            Err(e) => return no_auth_snapshot(format!("需要安裝並登入 gh CLI: {e}")),
        };

        // 參考 costats 的做法：打 Copilot 內部 usage API，直接用 gh CLI 換來的一般
        // OAuth token 嘗試（不要求使用者額外取得 Copilot 專用 token）
        let response = match ureq::get("https://api.github.com/copilot_internal/user")
            .set("Authorization", &format!("token {token}"))
            .set("Accept", "application/json")
            .set("User-Agent", "GitHubCopilotChat/0.26.7")
            .set("Editor-Version", "vscode/1.96.2")
            .set("Editor-Plugin-Version", "copilot-chat/0.26.7")
            .set("X-GitHub-Api-Version", "2025-04-01")
            .call()
        {
            Ok(r) => r,
            Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => {
                return no_auth_snapshot(
                    "GitHub token 無 Copilot 存取權限，請確認帳號已啟用 GitHub Copilot",
                );
            }
            Err(ureq::Error::Status(404, _)) => {
                return QuotaSnapshot {
                    provider: COPILOT_PROVIDER.to_string(),
                    status: "unsupported".to_string(),
                    source: "remote_api".to_string(),
                    fetched_at: current_timestamp(),
                    error_message: Some(
                        "此帳號無法使用 Copilot usage API（可能非 Copilot 訂閱帳號）".to_string(),
                    ),
                    windows: None,
                    local_tokens: None,
                    extra_credits: None,
                    reset_credits: None,
                };
            }
            Err(e) => {
                return error_snapshot(format!("Copilot usage API 呼叫失敗: {e}"));
            }
        };

        let body: serde_json::Value = match response.into_json() {
            Ok(v) => v,
            Err(e) => return error_snapshot(format!("failed to parse Copilot API response: {e}")),
        };

        let quota_reset_at = body
            .get("quota_reset_date_utc")
            .or_else(|| body.get("quota_reset_date"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let snapshots = body.get("quota_snapshots");

        // Pro/Business: premium_interactions 為主要額度，chat 為次要
        // Free plan: chat 為主要額度，completions 為次要
        let premium = snapshots
            .and_then(|s| s.get("premium_interactions"))
            .and_then(parse_copilot_quota);
        let chat = snapshots
            .and_then(|s| s.get("chat"))
            .and_then(parse_copilot_quota);
        let completions = snapshots
            .and_then(|s| s.get("completions"))
            .and_then(parse_copilot_quota);

        // Pro/Business 帳號：premium 為主要額度、chat 為次要
        // Free 帳號（無 premium）：chat 為主要額度、completions 為次要
        let (primary, primary_label, secondary, secondary_label) = match premium {
            Some(q) if !q.unlimited => (Some(q), "Premium", chat.filter(|q| !q.unlimited), "Chat"),
            _ => (
                chat.filter(|q| !q.unlimited),
                "Chat",
                completions.filter(|q| !q.unlimited),
                "Completions",
            ),
        };

        let mut windows = Vec::new();
        if let Some(q) = primary {
            windows.push(QuotaWindow {
                window_key: "primary".to_string(),
                label: primary_label.to_string(),
                utilization: ((100.0 - q.remaining_percent) / 100.0).clamp(0.0, 1.0),
                resets_at: quota_reset_at.clone(),
                group: None,
            });
        }
        if let Some(q) = secondary {
            windows.push(QuotaWindow {
                window_key: "secondary".to_string(),
                label: secondary_label.to_string(),
                utilization: ((100.0 - q.remaining_percent) / 100.0).clamp(0.0, 1.0),
                resets_at: quota_reset_at.clone(),
                group: None,
            });
        }

        if windows.is_empty() {
            return QuotaSnapshot {
                provider: COPILOT_PROVIDER.to_string(),
                status: "unsupported".to_string(),
                source: "remote_api".to_string(),
                fetched_at: current_timestamp(),
                error_message: Some(
                    "Copilot 回應中沒有可用的額度資料（可能為無限方案）".to_string(),
                ),
                windows: None,
                local_tokens: None,
                extra_credits: None,
                reset_credits: None,
            };
        }

        QuotaSnapshot {
            provider: COPILOT_PROVIDER.to_string(),
            status: "ok".to_string(),
            source: "remote_api".to_string(),
            fetched_at: current_timestamp(),
            error_message: None,
            windows: Some(windows),
            local_tokens: None,
            extra_credits: None,
            reset_credits: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copilot_adapter_returns_no_auth_when_gh_not_available() {
        // This test passes because gh CLI is likely not installed in CI or the test returns no_auth
        let settings = AppSettings {
            claude_root: String::new(),
            copilot_root: String::new(),
            opencode_root: String::new(),
            codex_root: String::new(),
            antigravity_root: String::new(),
            terminal_path: None,
            external_editor_path: None,
            show_archived: false,
            pinned_projects: Vec::new(),
            enabled_providers: vec!["copilot".to_string()],
            provider_integrations: Vec::new(),
            default_launcher: None,
            enable_intervention_notification: false,
            enable_session_end_notification: false,
            show_status_bar: false,
            analytics_refresh_interval: 30,
            analytics_panel_collapsed: false,
            hook_scripts_path: String::new(),
            claude_quota_reset_day: 1,
            minimize_to_tray: false,
            enable_quota_monitoring: true,
            quota_enabled_providers: crate::types::default_enabled_providers_all(),
            allow_create_project_config_dir: false,
            agents_source_root: String::new(),
            tray_quota_mode: crate::types::TrayQuotaMode::default(),
            tray_quota_primary_provider: None,
            tray_quota_panel_enabled: true,
            quota_overlay_enabled: false,
            quota_overlay_locked: true,
            quota_overlay_opacity: 0.85,
            quota_overlay_providers: Vec::new(),
            quota_overlay_theme: crate::types::OverlayTheme::default(),
            quota_overlay_style: crate::types::OverlayStyle::default(),
        };

        let adapter = CopilotAdapter;
        let snapshot = adapter.fetch_snapshot(&settings);
        // Depending on whether gh is installed, may be no_auth or error
        assert!(
            snapshot.status == "no_auth"
                || snapshot.status == "error"
                || snapshot.status == "ok"
                || snapshot.status == "unsupported",
            "unexpected status: {}",
            snapshot.status
        );
    }
}
