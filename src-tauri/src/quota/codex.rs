use std::env;

use crate::types::{
    AppSettings, LocalTokenUsage, QuotaSnapshot, QuotaWindow, ResetCreditEntry, ResetCredits,
    CODEX_PROVIDER,
};

use super::QuotaAdapter;

pub(crate) struct CodexAdapter;

const HOUR_SECONDS: i64 = 60 * 60;
const DAY_SECONDS: i64 = 24 * HOUR_SECONDS;
const FIVE_HOUR_MIN_SECONDS: i64 = HOUR_SECONDS;
const FIVE_HOUR_MAX_SECONDS: i64 = DAY_SECONDS;
const SEVEN_DAY_MIN_SECONDS: i64 = 2 * DAY_SECONDS;
const SEVEN_DAY_MAX_SECONDS: i64 = 14 * DAY_SECONDS;
const RESET_CREDITS_URL: &str = "https://chatgpt.com/backend-api/wham/rate-limit-reset-credits";

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
        reset_credits: None,
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
        reset_credits: None,
    }
}

fn format_window_duration(window_seconds: i64) -> String {
    if window_seconds >= DAY_SECONDS {
        format!("{}d", (window_seconds + (DAY_SECONDS / 2)) / DAY_SECONDS)
    } else {
        format!("{}h", (window_seconds + (HOUR_SECONDS / 2)) / HOUR_SECONDS)
    }
}

fn classify_window(window_seconds: i64) -> (String, String) {
    if (FIVE_HOUR_MIN_SECONDS..FIVE_HOUR_MAX_SECONDS).contains(&window_seconds) {
        return ("five_hour".to_string(), "5h".to_string());
    }

    if (SEVEN_DAY_MIN_SECONDS..SEVEN_DAY_MAX_SECONDS).contains(&window_seconds) {
        return ("seven_day".to_string(), "7d".to_string());
    }

    (
        "codex_window".to_string(),
        format_window_duration(window_seconds),
    )
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
    let json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("failed to parse auth.json: {e}"))?;

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

fn resolve_window_seconds(obj: &serde_json::Value, now_timestamp: i64) -> Option<i64> {
    obj.get("limit_window_seconds")
        .and_then(|v| v.as_i64())
        .or_else(|| obj.get("reset_after_seconds").and_then(|v| v.as_i64()))
        .or_else(|| {
            obj.get("reset_at")
                .and_then(|v| v.as_i64())
                .map(|reset_at| reset_at - now_timestamp)
        })
        .filter(|seconds| *seconds > 0)
}

fn parse_window_with_now(obj: &serde_json::Value, now_timestamp: i64) -> Option<QuotaWindow> {
    let used_percent = obj.get("used_percent").and_then(|v| v.as_f64())?;
    let window_seconds = resolve_window_seconds(obj, now_timestamp)?;
    let (window_key, label) = classify_window(window_seconds);
    let resets_at = obj
        .get("reset_at")
        .and_then(|v| v.as_i64())
        .and_then(|secs| chrono::DateTime::from_timestamp(secs, 0))
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string());

    Some(QuotaWindow {
        window_key,
        label,
        utilization: used_percent / 100.0,
        resets_at,
        group: None,
    })
}

fn parse_window(obj: &serde_json::Value) -> Option<QuotaWindow> {
    parse_window_with_now(obj, chrono::Utc::now().timestamp())
}

fn normalize_timestamp_value(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Number(number) => number
            .as_i64()
            .and_then(|timestamp| chrono::DateTime::from_timestamp(timestamp, 0))
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        serde_json::Value::String(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return None;
            }

            if let Ok(timestamp) = trimmed.parse::<i64>() {
                return chrono::DateTime::from_timestamp(timestamp, 0)
                    .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string());
            }

            chrono::DateTime::parse_from_rfc3339(trimmed)
                .ok()
                .map(|dt| {
                    dt.with_timezone(&chrono::Utc)
                        .format("%Y-%m-%dT%H:%M:%SZ")
                        .to_string()
                })
        }
        _ => None,
    }
}

fn parse_reset_credit_entry(obj: &serde_json::Value) -> Option<ResetCreditEntry> {
    let status = obj
        .get("status")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    let granted_at = obj.get("granted_at").and_then(normalize_timestamp_value);
    let expires_at = obj.get("expires_at").and_then(normalize_timestamp_value);

    if status.is_empty() && granted_at.is_none() && expires_at.is_none() {
        return None;
    }

    Some(ResetCreditEntry {
        granted_at,
        expires_at,
        status,
    })
}

fn parse_reset_credits_response(body: &serde_json::Value) -> ResetCredits {
    let available_count = body
        .get("available_count")
        .and_then(|value| value.as_u64())
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(0);
    let credits = body
        .get("credits")
        .and_then(|value| value.as_array())
        .map(|entries| {
            entries
                .iter()
                .filter_map(parse_reset_credit_entry)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    ResetCredits {
        available_count,
        credits,
    }
}

fn fetch_reset_credits(creds: &CodexCredentials) -> Result<Option<ResetCredits>, String> {
    let mut request = ureq::get(RESET_CREDITS_URL)
        .set("Authorization", &format!("Bearer {}", creds.access_token))
        .set("Accept", "application/json");

    if let Some(account_id) = &creds.account_id {
        request = request.set("ChatGPT-Account-Id", account_id);
    }

    let response = match request.call() {
        Ok(response) => response,
        Err(ureq::Error::Status(404, _)) => return Ok(None),
        Err(error) => return Err(format!("Codex reset credits API 呼叫失敗: {error}")),
    };

    let body: serde_json::Value = response
        .into_json()
        .map_err(|error| format!("failed to parse Codex reset credits response: {error}"))?;
    let reset_credits = parse_reset_credits_response(&body);

    if reset_credits.credits.is_empty() {
        return Ok(None);
    }

    Ok(Some(reset_credits))
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
                    input_total += usage
                        .get("input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    input_total += usage
                        .get("prompt_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    output_total += usage
                        .get("output_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    output_total += usage
                        .get("completion_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
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
            if let Some(window) = rate_limit.get("primary_window").and_then(parse_window) {
                windows.push(window);
            }
            if let Some(window) = rate_limit.get("secondary_window").and_then(parse_window) {
                windows.push(window);
            }
        }

        let (input_tokens, output_tokens) = count_monthly_tokens_from_jsonl(&settings.codex_root);
        let period_label = chrono::Utc::now().format("本月（%Y-%m）").to_string();
        let reset_credits = match fetch_reset_credits(&creds) {
            Ok(reset_credits) => reset_credits,
            Err(error) => {
                eprintln!("[quota][codex] reset credits fetch failed: {error}");
                None
            }
        };

        QuotaSnapshot {
            provider: CODEX_PROVIDER.to_string(),
            status: "ok".to_string(),
            source: "remote_api".to_string(),
            fetched_at: current_timestamp(),
            error_message: None,
            windows: if windows.is_empty() {
                None
            } else {
                Some(windows)
            },
            local_tokens: Some(LocalTokenUsage {
                input_tokens,
                output_tokens,
                period_label,
            }),
            extra_credits: None,
            reset_credits,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn classify_window_maps_known_and_dynamic_windows() {
        assert_eq!(
            classify_window(18_000),
            ("five_hour".to_string(), "5h".to_string())
        );
        assert_eq!(
            classify_window(604_800),
            ("seven_day".to_string(), "7d".to_string())
        );
        assert_eq!(
            classify_window(2_592_000),
            ("codex_window".to_string(), "30d".to_string())
        );
    }

    #[test]
    fn classify_window_honors_tolerance_boundaries() {
        assert_eq!(
            classify_window(FIVE_HOUR_MIN_SECONDS),
            ("five_hour".to_string(), "5h".to_string())
        );
        assert_eq!(
            classify_window(FIVE_HOUR_MAX_SECONDS - 1),
            ("five_hour".to_string(), "5h".to_string())
        );
        assert_eq!(
            classify_window(FIVE_HOUR_MIN_SECONDS - 1),
            ("codex_window".to_string(), "1h".to_string())
        );
        assert_eq!(
            classify_window(SEVEN_DAY_MIN_SECONDS),
            ("seven_day".to_string(), "7d".to_string())
        );
        assert_eq!(
            classify_window(SEVEN_DAY_MAX_SECONDS - 1),
            ("seven_day".to_string(), "7d".to_string())
        );
        assert_eq!(
            classify_window(SEVEN_DAY_MAX_SECONDS),
            ("codex_window".to_string(), "14d".to_string())
        );
    }

    #[test]
    fn parse_window_uses_reset_after_seconds_when_limit_window_missing() {
        let window = parse_window_with_now(
            &json!({
                "used_percent": 50.0,
                "reset_after_seconds": 604800,
                "reset_at": 1_800_000_000,
            }),
            1_700_000_000,
        )
        .expect("window should parse");

        assert_eq!(window.window_key, "seven_day");
        assert_eq!(window.label, "7d");
    }

    #[test]
    fn parse_window_uses_reset_at_when_other_duration_fields_missing() {
        let now_timestamp = 1_700_000_000;
        let window = parse_window_with_now(
            &json!({
                "used_percent": 25.0,
                "reset_at": now_timestamp + (30 * DAY_SECONDS),
            }),
            now_timestamp,
        )
        .expect("window should parse");

        assert_eq!(window.window_key, "codex_window");
        assert_eq!(window.label, "30d");
    }

    #[test]
    fn parse_reset_credits_response_supports_epoch_timestamps() {
        let parsed = parse_reset_credits_response(&json!({
            "available_count": 2,
            "credits": [
                {
                    "granted_at": 1_720_000_000,
                    "expires_at": 1_720_086_400,
                    "status": "active"
                }
            ]
        }));

        assert_eq!(parsed.available_count, 2);
        assert_eq!(parsed.credits.len(), 1);
        assert_eq!(parsed.credits[0].status, "active");
        assert_eq!(
            parsed.credits[0].granted_at.as_deref(),
            Some("2024-07-03T09:46:40Z")
        );
        assert_eq!(
            parsed.credits[0].expires_at.as_deref(),
            Some("2024-07-04T09:46:40Z")
        );
    }

    #[test]
    fn parse_reset_credits_response_supports_iso_timestamps() {
        let parsed = parse_reset_credits_response(&json!({
            "available_count": 1,
            "credits": [
                {
                    "granted_at": "2026-07-14T08:00:00Z",
                    "expires_at": "2026-07-21T23:59:00+08:00",
                    "status": "active"
                }
            ]
        }));

        assert_eq!(parsed.available_count, 1);
        assert_eq!(
            parsed.credits[0].granted_at.as_deref(),
            Some("2026-07-14T08:00:00Z")
        );
        assert_eq!(
            parsed.credits[0].expires_at.as_deref(),
            Some("2026-07-21T15:59:00Z")
        );
    }

    #[test]
    fn parse_reset_credits_response_is_lenient_for_missing_fields() {
        let parsed = parse_reset_credits_response(&json!({
            "credits": [
                { "status": "expired" },
                { "expires_at": 1_720_086_400 }
            ]
        }));

        assert_eq!(parsed.available_count, 0);
        assert_eq!(parsed.credits.len(), 2);
        assert_eq!(parsed.credits[0].status, "expired");
        assert_eq!(parsed.credits[0].granted_at, None);
        assert_eq!(parsed.credits[1].status, "");
        assert_eq!(
            parsed.credits[1].expires_at.as_deref(),
            Some("2024-07-04T09:46:40Z")
        );
    }

    #[test]
    fn parse_reset_credits_response_handles_empty_list() {
        let parsed = parse_reset_credits_response(&json!({
            "available_count": 0,
            "credits": []
        }));

        assert_eq!(parsed.available_count, 0);
        assert!(parsed.credits.is_empty());
    }

    #[test]
    fn quota_snapshot_deserializes_without_reset_credits_field() {
        let snapshot = serde_json::from_value::<QuotaSnapshot>(json!({
            "provider": "codex",
            "status": "ok",
            "source": "remote_api",
            "fetchedAt": "2026-07-14T08:00:00Z",
            "errorMessage": null,
            "windows": [],
            "localTokens": null,
            "extraCredits": null
        }))
        .expect("snapshot should deserialize");

        assert_eq!(snapshot.reset_credits, None);
    }

    #[test]
    fn codex_adapter_returns_no_auth_when_credentials_missing() {
        let settings = AppSettings {
            claude_root: String::new(),
            copilot_root: String::new(),
            opencode_root: String::new(),
            codex_root: "/nonexistent/path/.codex".to_string(),
            antigravity_root: String::new(),
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

        let adapter = CodexAdapter;
        let snapshot = adapter.fetch_snapshot(&settings);
        assert_eq!(snapshot.status, "no_auth");
        assert_eq!(snapshot.provider, "codex");
    }
}
