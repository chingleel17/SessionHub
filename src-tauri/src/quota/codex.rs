use std::env;

use crate::types::{AppSettings, LocalTokenUsage, QuotaSnapshot, QuotaWindow, CODEX_PROVIDER};

use super::QuotaAdapter;

pub(crate) struct CodexAdapter;

fn current_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn no_auth_snapshot(error_message: impl Into<String>) -> QuotaSnapshot {
    QuotaSnapshot {
        provider: CODEX_PROVIDER.to_string(),
        status: "no_auth".to_string(),
        source: "remote_api".to_string(),
        fetched_at: current_timestamp(),
        error_message: Some(error_message.into()),
        windows: None,
        local_tokens: None,
        extra_credits: None,
    }
}

fn error_snapshot(error_message: impl Into<String>) -> QuotaSnapshot {
    QuotaSnapshot {
        provider: CODEX_PROVIDER.to_string(),
        status: "error".to_string(),
        source: "remote_api".to_string(),
        fetched_at: current_timestamp(),
        error_message: Some(error_message.into()),
        windows: None,
        local_tokens: None,
        extra_credits: None,
    }
}

struct CodexCredentials {
    access_token: String,
    account_id: Option<String>,
}

/// 讀取 Codex CLI 的 auth.json（`$CODEX_HOME/auth.json` 或 `~/.codex/auth.json`）
/// 參考 costats 的 CodexOAuthUsageFetcher 實作
fn read_codex_credentials(codex_root: &str) -> Result<CodexCredentials, String> {
    let auth_path = if !codex_root.trim().is_empty() {
        std::path::PathBuf::from(codex_root.trim()).join("auth.json")
    } else if let Ok(codex_home) = env::var("CODEX_HOME") {
        std::path::PathBuf::from(codex_home.trim()).join("auth.json")
    } else {
        let user_profile =
            env::var("USERPROFILE").map_err(|_| "USERPROFILE not set".to_string())?;
        std::path::PathBuf::from(user_profile)
            .join(".codex")
            .join("auth.json")
    };

    if !auth_path.exists() {
        return Err(format!("auth.json not found at {}", auth_path.display()));
    }

    let content = std::fs::read_to_string(&auth_path)
        .map_err(|e| format!("failed to read {}: {e}", auth_path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse auth.json: {e}"))?;

    // 新格式：tokens 物件包含 access_token / account_id
    if let Some(tokens) = json.get("tokens") {
        let access_token = tokens
            .get("access_token")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let account_id = tokens
            .get("account_id")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        if let Some(access_token) = access_token {
            return Ok(CodexCredentials {
                access_token,
                account_id,
            });
        }
    }

    // 舊格式：直接存放 OPENAI_API_KEY
    if let Some(api_key) = json.get("OPENAI_API_KEY").and_then(|v| v.as_str()) {
        return Ok(CodexCredentials {
            access_token: api_key.to_string(),
            account_id: None,
        });
    }

    Err("no access_token found in auth.json".to_string())
}

fn parse_window(key: &str, label: &str, obj: &serde_json::Value) -> Option<QuotaWindow> {
    let used_percent = obj.get("used_percent").and_then(|v| v.as_f64())?;
    let resets_at = obj
        .get("reset_at")
        .and_then(|v| v.as_i64())
        .and_then(|secs| chrono::DateTime::from_timestamp(secs, 0))
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string());

    Some(QuotaWindow {
        window_key: key.to_string(),
        label: label.to_string(),
        utilization: used_percent / 100.0,
        resets_at,
    })
}

fn count_monthly_tokens_from_jsonl(codex_root: &str) -> (u64, u64) {
    let root = std::path::PathBuf::from(codex_root);
    if !root.exists() {
        return (0, 0);
    }

    use chrono::Datelike;

    let now = chrono::Utc::now();
    let current_year = now.year();
    let current_month = now.month();
    let mut input_total: u64 = 0;
    let mut output_total: u64 = 0;

    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for line in content.lines() {
                let json: serde_json::Value = match serde_json::from_str(line) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                let ts = json
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc));

                if let Some(dt) = ts {
                    if dt.year() != current_year || dt.month() != current_month {
                        continue;
                    }
                }

                if let Some(usage) = json.get("usage") {
                    input_total += usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                    input_total += usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                    output_total += usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                    output_total += usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                }
            }
        }
    }

    (input_total, output_total)
}

impl QuotaAdapter for CodexAdapter {
    fn provider_key(&self) -> &str {
        CODEX_PROVIDER
    }

    fn fetch_snapshot(&self, settings: &AppSettings) -> QuotaSnapshot {
        let creds = match read_codex_credentials(&settings.codex_root) {
            Ok(c) => c,
            Err(e) => return no_auth_snapshot(format!("無法讀取 Codex 憑證: {e}")),
        };

        let mut request = ureq::get("https://chatgpt.com/backend-api/wham/usage")
            .set("Authorization", &format!("Bearer {}", creds.access_token))
            .set("Accept", "application/json");

        if let Some(account_id) = &creds.account_id {
            request = request.set("ChatGPT-Account-Id", account_id);
        }

        let response = match request.call() {
            Ok(r) => r,
            Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => {
                return no_auth_snapshot("Codex token 被拒絕，請重新登入 Codex CLI (codex login)");
            }
            Err(e) => {
                return error_snapshot(format!("Codex usage API 呼叫失敗: {e}"));
            }
        };

        let body: serde_json::Value = match response.into_json() {
            Ok(v) => v,
            Err(e) => return error_snapshot(format!("failed to parse Codex API response: {e}")),
        };

        let mut windows = Vec::new();
        if let Some(rate_limit) = body.get("rate_limit") {
            if let Some(primary) = rate_limit.get("primary_window") {
                if let Some(window) = parse_window("primary", "5h", primary) {
                    windows.push(window);
                }
            }
            if let Some(secondary) = rate_limit.get("secondary_window") {
                if let Some(window) = parse_window("secondary", "7d", secondary) {
                    windows.push(window);
                }
            }
        }

        let (input_tokens, output_tokens) = count_monthly_tokens_from_jsonl(&settings.codex_root);
        let period_label = chrono::Utc::now().format("本月（%Y-%m）").to_string();

        QuotaSnapshot {
            provider: CODEX_PROVIDER.to_string(),
            status: "ok".to_string(),
            source: "remote_api".to_string(),
            fetched_at: current_timestamp(),
            error_message: None,
            windows: if windows.is_empty() { None } else { Some(windows) },
            local_tokens: Some(LocalTokenUsage {
                input_tokens,
                output_tokens,
                period_label,
            }),
            extra_credits: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_adapter_returns_no_auth_when_credentials_missing() {
        let settings = AppSettings {
            claude_root: String::new(),
            copilot_root: String::new(),
            opencode_root: String::new(),
            codex_root: "/nonexistent/path/.codex".to_string(),
            terminal_path: None,
            external_editor_path: None,
            show_archived: false,
            pinned_projects: Vec::new(),
            enabled_providers: vec!["codex".to_string()],
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
        };

        let adapter = CodexAdapter;
        let snapshot = adapter.fetch_snapshot(&settings);
        assert_eq!(snapshot.status, "no_auth");
        assert_eq!(snapshot.provider, "codex");
    }
}
