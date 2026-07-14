use std::env;

use crate::types::{AppSettings, ExtraCredits, QuotaSnapshot, QuotaWindow, CLAUDE_PROVIDER};

use super::QuotaAdapter;

pub(crate) struct ClaudeAdapter;

fn current_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn no_auth_snapshot(error_message: impl Into<String>) -> QuotaSnapshot {
    QuotaSnapshot {
        provider: CLAUDE_PROVIDER.to_string(),
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
        provider: CLAUDE_PROVIDER.to_string(),
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

fn rate_limit_message(resp: &ureq::Response) -> String {
    match resp
        .header("Retry-After")
        .and_then(|v| v.parse::<u64>().ok())
    {
        Some(secs) => format!("Anthropic usage API 請求過於頻繁，請於 {secs} 秒後再試"),
        None => "Anthropic usage API 請求過於頻繁，請稍後再試".to_string(),
    }
}

/// 429 限流時使用的特殊狀態，呼叫端應保留快取中的舊 snapshot 不覆蓋
fn rate_limited_snapshot(error_message: impl Into<String>) -> QuotaSnapshot {
    QuotaSnapshot {
        provider: CLAUDE_PROVIDER.to_string(),
        status: "rate_limited".to_string(),
        source: "remote_api".to_string(),
        fetched_at: current_timestamp(),
        error_message: Some(error_message.into()),
        windows: None,
        local_tokens: None,
        extra_credits: None,
        reset_credits: None,
    }
}

/// Claude Code OAuth client ID (public PKCE client, same as Claude Code CLI)
const CLAUDE_OAUTH_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const CLAUDE_OAUTH_TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";

struct ClaudeCredentials {
    access_token: Option<String>,
    refresh_token: Option<String>,
    /// Unix ms timestamp
    expires_at_ms: Option<i64>,
}

fn read_claude_credentials(claude_root: &str) -> Result<ClaudeCredentials, String> {
    let root = if claude_root.trim().is_empty() {
        let user_profile =
            env::var("USERPROFILE").map_err(|_| "USERPROFILE not set".to_string())?;
        std::path::PathBuf::from(user_profile).join(".claude")
    } else {
        std::path::PathBuf::from(claude_root)
    };

    // Try both .credentials.json and credentials.json
    let candidate_paths = [
        root.join(".credentials.json"),
        root.join("credentials.json"),
    ];
    let creds_path = candidate_paths.iter().find(|p| p.exists()).ok_or_else(|| {
        format!(
            "credentials file not found — checked: {}",
            candidate_paths
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    let content = std::fs::read_to_string(creds_path)
        .map_err(|e| format!("failed to read {}: {e}", creds_path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse credentials JSON: {e}"))?;

    // costats / Claude Code use "claudeAiOauth" as the top-level key
    // fallback to older naming conventions just in case
    let oauth = json
        .get("claudeAiOauth")
        .or_else(|| json.get("oauth"))
        .or_else(|| json.get("claudeAiOauthGitHubIntegration"));

    let access_token = oauth
        .and_then(|o| o.get("accessToken").or_else(|| o.get("oauthToken")))
        .and_then(|t| t.as_str())
        .or_else(|| json.get("accessToken").and_then(|t| t.as_str()))
        .map(str::to_string);

    let refresh_token = oauth
        .and_then(|o| o.get("refreshToken"))
        .and_then(|t| t.as_str())
        .map(str::to_string);

    let expires_at_ms = oauth
        .and_then(|o| o.get("expiresAt"))
        .and_then(|v| v.as_i64());

    if access_token.is_none() && refresh_token.is_none() {
        let keys: Vec<String> = json
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        return Err(format!(
            "No OAuth token found in {} (top-level keys: {})",
            creds_path.display(),
            keys.join(", ")
        ));
    }

    Ok(ClaudeCredentials {
        access_token,
        refresh_token,
        expires_at_ms,
    })
}

fn is_token_expired(expires_at_ms: Option<i64>) -> bool {
    let Some(exp_ms) = expires_at_ms else {
        return false;
    };
    let now_ms = chrono::Utc::now().timestamp_millis();
    // Consider expired if within 5 minutes of expiry
    now_ms >= exp_ms - 5 * 60 * 1000
}

fn refresh_oauth_token(refresh_token: &str) -> Result<String, String> {
    let payload = serde_json::json!({
        "grant_type": "refresh_token",
        "refresh_token": refresh_token,
        "client_id": CLAUDE_OAUTH_CLIENT_ID,
    });
    let body = serde_json::to_string(&payload)
        .map_err(|e| format!("failed to serialize refresh request: {e}"))?;

    let response = ureq::post(CLAUDE_OAUTH_TOKEN_URL)
        .set("Content-Type", "application/json")
        .send_string(&body)
        .map_err(|e| format!("OAuth token refresh request failed: {e}"))?;

    let json: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("failed to parse token refresh response: {e}"))?;

    json.get("access_token")
        .and_then(|t| t.as_str())
        .map(str::to_string)
        .ok_or_else(|| format!("access_token missing from token response: {json}"))
}

/// 解析 usage API 回應 `limits` 陣列中，依模型範圍（scope.model.display_name）的每週視窗。
/// Fable 等 scoped 額度僅出現在此處（頂層對應 key 為 null），故資料驅動地各產生一個視窗。
/// `existing_keys` 為頂層解析已產生的 window_key，用於去重避免重複計入。
fn parse_scoped_weekly_windows(
    body: &serde_json::Value,
    existing_keys: &[String],
) -> Vec<QuotaWindow> {
    let mut windows = Vec::new();
    let Some(limits) = body.get("limits").and_then(|v| v.as_array()) else {
        return windows;
    };

    for item in limits {
        let is_weekly = item.get("group").and_then(|v| v.as_str()) == Some("weekly");
        if !is_weekly {
            continue;
        }
        let Some(display_name) = item
            .get("scope")
            .and_then(|s| s.get("model"))
            .and_then(|m| m.get("display_name"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
        else {
            continue;
        };

        let window_key = format!("seven_day_{}", display_name.to_lowercase());
        if existing_keys.iter().any(|k| k == &window_key)
            || windows.iter().any(|w: &QuotaWindow| w.window_key == window_key)
        {
            continue;
        }

        let Some(percent) = item.get("percent").and_then(|v| v.as_f64()) else {
            continue;
        };
        let utilization = if percent > 1.0 { percent / 100.0 } else { percent };
        let resets_at = item
            .get("resets_at")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        windows.push(QuotaWindow {
            window_key,
            label: display_name.to_string(),
            utilization,
            resets_at,
            group: None,
        });
    }

    windows
}

fn parse_window(key: &str, label: &str, obj: &serde_json::Value) -> Option<QuotaWindow> {
    // "utilization" is the correct field name (costats-confirmed, 0–100 range)
    // fall back to "used_percentage" for older API versions
    let utilization = obj
        .get("utilization")
        .or_else(|| obj.get("used_percentage"))
        .and_then(|v| v.as_f64())
        .map(|pct| {
            // Normalise 0–100 → 0.0–1.0
            if pct > 1.0 {
                pct / 100.0
            } else {
                pct
            }
        })
        .or_else(|| {
            let used = obj.get("tokens")?.as_f64()?;
            let max = obj.get("max_tokens")?.as_f64()?;
            if max > 0.0 {
                Some(used / max)
            } else {
                None
            }
        })?;

    let resets_at = obj
        .get("resets_at")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    Some(QuotaWindow {
        window_key: key.to_string(),
        label: label.to_string(),
        utilization,
        resets_at,
        group: None,
    })
}

impl QuotaAdapter for ClaudeAdapter {
    fn provider_key(&self) -> &str {
        CLAUDE_PROVIDER
    }

    fn fetch_snapshot(&self, settings: &AppSettings) -> QuotaSnapshot {
        let creds = match read_claude_credentials(&settings.claude_root) {
            Ok(c) => c,
            Err(e) => return no_auth_snapshot(format!("無法讀取 Claude 憑證: {e}")),
        };

        // Determine the access token to use (refresh if needed)
        let token = if let Some(ref at) = creds.access_token {
            if !is_token_expired(creds.expires_at_ms) {
                at.clone()
            } else if let Some(ref rt) = creds.refresh_token {
                match refresh_oauth_token(rt) {
                    Ok(new_at) => new_at,
                    Err(e) => return no_auth_snapshot(format!("Token 已過期且刷新失敗: {e}")),
                }
            } else {
                at.clone() // use expired token, will 401 below
            }
        } else if let Some(ref rt) = creds.refresh_token {
            match refresh_oauth_token(rt) {
                Ok(new_at) => new_at,
                Err(e) => return no_auth_snapshot(format!("無法取得 access token: {e}")),
            }
        } else {
            return no_auth_snapshot("credentials 檔案中沒有 accessToken 或 refreshToken");
        };

        let call_usage_api = |access_token: &str| {
            ureq::get("https://api.anthropic.com/api/oauth/usage")
                .set("Authorization", &format!("Bearer {access_token}"))
                .set("anthropic-beta", "oauth-2025-04-20")
                .call()
        };

        let response = match call_usage_api(&token) {
            Ok(r) => r,
            Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => {
                // access token rejected — try refresh once more if we have a refresh token
                if let Some(ref rt) = creds.refresh_token {
                    match refresh_oauth_token(rt) {
                        Ok(new_at) => match call_usage_api(&new_at) {
                            Ok(r) => r,
                            Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => {
                                return no_auth_snapshot(
                                    "Token 刷新後仍被拒絕，請重新登入 Claude Code",
                                );
                            }
                            Err(ureq::Error::Status(429, resp)) => {
                                return rate_limited_snapshot(rate_limit_message(&resp));
                            }
                            Err(e) => return error_snapshot(format!("usage API 錯誤: {e}")),
                        },
                        Err(e) => return no_auth_snapshot(format!("Token 刷新失敗: {e}")),
                    }
                } else {
                    return no_auth_snapshot(
                        "Claude OAuth token 被拒絕，請重新登入 Claude Code (claude /login)",
                    );
                }
            }
            Err(ureq::Error::Status(429, resp)) => {
                return rate_limited_snapshot(rate_limit_message(&resp));
            }
            Err(e) => {
                return error_snapshot(format!("Anthropic usage API 呼叫失敗: {e}"));
            }
        };

        let body: serde_json::Value = match response.into_json() {
            Ok(v) => v,
            Err(e) => {
                return error_snapshot(format!("failed to parse Anthropic API response: {e}"))
            }
        };

        // Anthropic OAuth usage API: windows are at the TOP LEVEL of the response
        // (confirmed by costats source code — not nested under "rate_limits")
        let window_defs = [
            ("five_hour", "5h"),
            ("seven_day", "7d"),
            ("seven_day_sonnet", "7d Sonnet"),
            ("seven_day_opus", "7d Opus"),
        ];

        let mut windows = Vec::new();
        for (key, label) in &window_defs {
            if let Some(obj) = body.get(key) {
                if let Some(window) = parse_window(key, label, obj) {
                    windows.push(window);
                }
            }
        }

        // 追加 limits[] 中依模型範圍的每週視窗（如 Fable），頂層 key 為 null 故只在此出現
        let existing_keys: Vec<String> =
            windows.iter().map(|w| w.window_key.clone()).collect();
        windows.extend(parse_scoped_weekly_windows(&body, &existing_keys));

        // Parse extra_usage
        let extra_usage_raw = body
            .get("extra_usage")
            .or_else(|| body.get("data").and_then(|d| d.get("extra_usage")));

        // extra_usage: used_credits and monthly_limit are in CENTS (÷100 → USD)
        let extra_credits = extra_usage_raw.and_then(|eu| {
            let is_enabled = eu.get("is_enabled")?.as_bool()?;
            let used_cents = eu.get("used_credits")?.as_f64().unwrap_or(0.0);
            let used_credits = used_cents / 100.0; // cents → dollars
            let monthly_limit_cents = eu.get("monthly_limit").and_then(|v| v.as_u64());
            let monthly_limit = monthly_limit_cents.map(|c| c); // store as cents for now
            let utilization = monthly_limit_cents.map(|limit| {
                if limit > 0 {
                    used_cents / limit as f64
                } else {
                    0.0
                }
            });
            Some(ExtraCredits {
                is_enabled,
                monthly_limit,
                used_credits,
                utilization,
            })
        });

        QuotaSnapshot {
            provider: CLAUDE_PROVIDER.to_string(),
            status: "ok".to_string(),
            source: "remote_api".to_string(),
            fetched_at: current_timestamp(),
            error_message: None,
            windows: if windows.is_empty() {
                None
            } else {
                Some(windows)
            },
            local_tokens: None,
            extra_credits,
            reset_credits: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_scoped_weekly_windows_extracts_fable() {
        // 取自實際 Anthropic OAuth usage API 回應的 limits 結構
        let body = json!({
            "limits": [
                { "kind": "session", "group": "session", "percent": 20,
                  "resets_at": "2026-07-14T18:10:00.245240+00:00", "scope": null },
                { "kind": "weekly_all", "group": "weekly", "percent": 93,
                  "resets_at": "2026-07-14T16:00:00.245266+00:00", "scope": null },
                { "kind": "weekly_scoped", "group": "weekly", "percent": 100,
                  "resets_at": "2026-07-14T16:00:00.245603+00:00",
                  "scope": { "model": { "id": null, "display_name": "Fable" } },
                  "is_active": true }
            ]
        });

        let windows = parse_scoped_weekly_windows(&body, &[]);
        assert_eq!(windows.len(), 1);
        let w = &windows[0];
        assert_eq!(w.window_key, "seven_day_fable");
        assert_eq!(w.label, "Fable");
        assert_eq!(w.utilization, 1.0);
        assert_eq!(
            w.resets_at.as_deref(),
            Some("2026-07-14T16:00:00.245603+00:00")
        );
    }

    #[test]
    fn parse_scoped_weekly_windows_handles_unknown_model_and_skips_session() {
        let body = json!({
            "limits": [
                { "group": "session", "percent": 10, "scope": { "model": { "display_name": "X" } } },
                { "group": "weekly", "percent": 50,
                  "resets_at": "2026-07-20T00:00:00Z",
                  "scope": { "model": { "display_name": "Newmodel" } } }
            ]
        });

        let windows = parse_scoped_weekly_windows(&body, &[]);
        assert_eq!(windows.len(), 1, "session-group 項目不應產生視窗");
        assert_eq!(windows[0].window_key, "seven_day_newmodel");
        assert_eq!(windows[0].label, "Newmodel");
        assert_eq!(windows[0].utilization, 0.5);
    }

    #[test]
    fn parse_scoped_weekly_windows_dedups_existing_key() {
        let body = json!({
            "limits": [
                { "group": "weekly", "percent": 30, "resets_at": "2026-07-20T00:00:00Z",
                  "scope": { "model": { "display_name": "Fable" } } }
            ]
        });

        let existing = vec!["seven_day_fable".to_string()];
        let windows = parse_scoped_weekly_windows(&body, &existing);
        assert!(windows.is_empty(), "已存在的 window_key 應被去重");
    }

    #[test]
    fn parse_scoped_weekly_windows_empty_when_no_scoped_limits() {
        assert!(parse_scoped_weekly_windows(&json!({}), &[]).is_empty());
        let body = json!({
            "limits": [
                { "group": "weekly", "percent": 93, "scope": null },
                { "group": "weekly", "percent": 20, "scope": { "model": { "display_name": "" } } }
            ]
        });
        assert!(parse_scoped_weekly_windows(&body, &[]).is_empty());
    }

    #[test]
    fn claude_adapter_returns_no_auth_when_credentials_missing() {
        let settings = AppSettings {
            claude_root: "/nonexistent/path/.claude".to_string(),
            copilot_root: String::new(),
            opencode_root: String::new(),
            codex_root: String::new(),
            antigravity_root: String::new(),
            terminal_path: None,
            external_editor_path: None,
            show_archived: false,
            pinned_projects: Vec::new(),
            enabled_providers: vec!["claude".to_string()],
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
        };

        let adapter = ClaudeAdapter;
        let snapshot = adapter.fetch_snapshot(&settings);
        assert_eq!(snapshot.status, "no_auth");
        assert_eq!(snapshot.provider, "claude");
    }
}
