use crate::types::{AppSettings, LocalTokenUsage, QuotaSnapshot, OPENCODE_PROVIDER};

use super::QuotaAdapter;

pub(crate) struct OpenCodeAdapter;

fn current_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn count_monthly_tokens_from_db(opencode_root: &str) -> Option<(u64, u64)> {
    let db_path = std::path::PathBuf::from(opencode_root).join("opencode.db");
    if !db_path.exists() {
        return None;
    }

    let conn = rusqlite::Connection::open(&db_path).ok()?;
    // Filter sessions updated in the current month (time_updated is ms since epoch)
    let month_start_ms = chrono::Utc::now()
        .with_day(1)?
        .date_naive()
        .and_hms_opt(0, 0, 0)?
        .and_utc()
        .timestamp_millis();

    let result = conn
        .query_row(
            "SELECT COALESCE(SUM(tokens_input), 0), COALESCE(SUM(tokens_output), 0)
         FROM session
         WHERE time_updated >= ?1",
            rusqlite::params![month_start_ms],
            |row| {
                let input: i64 = row.get(0)?;
                let output: i64 = row.get(1)?;
                Ok((input as u64, output as u64))
            },
        )
        .ok()?;

    Some(result)
}

fn count_monthly_tokens_from_json(opencode_root: &str) -> (u64, u64) {
    let storage_dir = std::path::PathBuf::from(opencode_root)
        .join("storage")
        .join("message");

    let now_month = chrono::Utc::now().format("%Y-%m").to_string();
    let mut input_total: u64 = 0;
    let mut output_total: u64 = 0;

    let session_parent = std::path::PathBuf::from(opencode_root)
        .join("storage")
        .join("session");

    if let Ok(project_entries) = std::fs::read_dir(&session_parent) {
        for project_entry in project_entries.flatten() {
            if let Ok(session_entries) = std::fs::read_dir(project_entry.path()) {
                for session_entry in session_entries.flatten() {
                    // Check session created_at month
                    let content = std::fs::read_to_string(session_entry.path()).unwrap_or_default();
                    let json: serde_json::Value =
                        serde_json::from_str(&content).unwrap_or_default();
                    let time_updated = json
                        .get("time")
                        .and_then(|t| t.get("updated"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);

                    if time_updated > 0 {
                        let dt = chrono::DateTime::from_timestamp_millis(time_updated);
                        if let Some(dt) = dt {
                            let session_month = dt.format("%Y-%m").to_string();
                            if session_month != now_month {
                                continue;
                            }
                        }
                    }

                    // Read messages for this session
                    let session_id = json.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                    let msg_dir = storage_dir.join(session_id);
                    if let Ok(msg_entries) = std::fs::read_dir(&msg_dir) {
                        for msg_entry in msg_entries.flatten() {
                            let msg_content =
                                std::fs::read_to_string(msg_entry.path()).unwrap_or_default();
                            let msg: serde_json::Value =
                                serde_json::from_str(&msg_content).unwrap_or_default();
                            if let Some(tokens) = msg.get("tokens") {
                                input_total +=
                                    tokens.get("input").and_then(|v| v.as_u64()).unwrap_or(0);
                                input_total += tokens
                                    .get("inputTokens")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0);
                                output_total +=
                                    tokens.get("output").and_then(|v| v.as_u64()).unwrap_or(0);
                                output_total += tokens
                                    .get("outputTokens")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0);
                            }
                        }
                    }
                }
            }
        }
    }

    (input_total, output_total)
}

impl QuotaAdapter for OpenCodeAdapter {
    fn provider_key(&self) -> &str {
        OPENCODE_PROVIDER
    }

    fn fetch_snapshot(&self, settings: &AppSettings) -> QuotaSnapshot {
        let opencode_root = &settings.opencode_root;

        let (input_tokens, output_tokens) = count_monthly_tokens_from_db(opencode_root)
            .unwrap_or_else(|| count_monthly_tokens_from_json(opencode_root));

        let period_label = chrono::Utc::now()
            .format("本月（%Y-%m，各 AI 供應商合計）")
            .to_string();

        QuotaSnapshot {
            provider: OPENCODE_PROVIDER.to_string(),
            status: "ok".to_string(),
            source: "local_scan".to_string(),
            fetched_at: current_timestamp(),
            error_message: None,
            windows: None,
            local_tokens: Some(LocalTokenUsage {
                input_tokens,
                output_tokens,
                period_label,
            }),
            extra_credits: None,
        }
    }
}

use chrono::Datelike;
