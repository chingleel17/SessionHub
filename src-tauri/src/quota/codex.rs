use crate::types::{AppSettings, LocalTokenUsage, QuotaSnapshot, CODEX_PROVIDER};

use super::QuotaAdapter;

pub(crate) struct CodexAdapter;

fn current_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn count_monthly_tokens_from_jsonl(codex_root: &str) -> (u64, u64) {
    let root = std::path::PathBuf::from(codex_root);
    if !root.exists() {
        return (0, 0);
    }

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

                // Filter by timestamp
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

                // Extract token usage from various known Codex JSONL formats
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
        let (input_tokens, output_tokens) =
            count_monthly_tokens_from_jsonl(&settings.codex_root);

        let period_label = chrono::Utc::now().format("本月（%Y-%m）").to_string();

        QuotaSnapshot {
            provider: CODEX_PROVIDER.to_string(),
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
