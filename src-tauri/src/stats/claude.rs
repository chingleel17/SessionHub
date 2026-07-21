use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::types::*;

// ── Claude JSONL stats 解析 ───────────────────────────────────────────────────

/// Claude 模型定價表（每 1M tokens 美元）
struct ClaudeModelPricing {
    input: f64,
    output: f64,
    cache_write_1h: f64,
    cache_write_5m: f64,
    cache_read: f64,
}

fn claude_model_pricing(model: &str) -> ClaudeModelPricing {
    let m = model.to_lowercase();
    if m.contains("opus-4") || m.contains("opus-3-5") || m.contains("opus-3.5") {
        ClaudeModelPricing {
            input: 15.0,
            output: 75.0,
            cache_write_1h: 30.0,
            cache_write_5m: 15.0,
            cache_read: 1.5,
        }
    } else if m.contains("sonnet-4")
        || m.contains("sonnet-3-7")
        || m.contains("sonnet-3.7")
        || m.contains("sonnet-3-5")
        || m.contains("sonnet-3.5")
    {
        ClaudeModelPricing {
            input: 3.0,
            output: 15.0,
            cache_write_1h: 6.0,
            cache_write_5m: 3.0,
            cache_read: 0.3,
        }
    } else if m.contains("haiku-4") || m.contains("haiku-3-5") || m.contains("haiku-3.5") {
        ClaudeModelPricing {
            input: 0.8,
            output: 4.0,
            cache_write_1h: 1.6,
            cache_write_5m: 0.8,
            cache_read: 0.08,
        }
    } else if m.contains("haiku") {
        ClaudeModelPricing {
            input: 0.25,
            output: 1.25,
            cache_write_1h: 0.3,
            cache_write_5m: 0.25,
            cache_read: 0.03,
        }
    } else {
        // fallback: sonnet-level pricing
        ClaudeModelPricing {
            input: 3.0,
            output: 15.0,
            cache_write_1h: 6.0,
            cache_write_5m: 3.0,
            cache_read: 0.3,
        }
    }
}

pub(crate) fn is_claude_session_file(path: &Path) -> bool {
    if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
        return false;
    }
    let path_str = path.to_string_lossy();
    path_str.contains(".claude") && path_str.contains("projects")
}

pub(crate) fn compute_claude_stats(session_path: &Path) -> Result<SessionStats, String> {
    use std::collections::HashMap as StdHashMap;

    let file =
        fs::File::open(session_path).map_err(|e| format!("failed to open Claude session: {e}"))?;
    let reader = BufReader::new(file);

    struct DedupEntry {
        usage: crate::types::ClaudeUsage,
        model: String,
        tool_names: Vec<String>,
    }

    let mut dedup: StdHashMap<String, DedupEntry> = StdHashMap::new();
    let mut models_used: BTreeSet<String> = BTreeSet::new();
    let mut interaction_count: u32 = 0;
    let mut first_ts: Option<i64> = None;
    let mut last_ts: Option<i64> = None;

    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let Ok(raw) = serde_json::from_str::<serde_json::Value>(trimmed) else {
            continue;
        };

        let entry_type = raw.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let is_sidechain = raw
            .get("isSidechain")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let is_meta = raw.get("isMeta").and_then(|v| v.as_bool()).unwrap_or(false);

        // 只統計非 meta 的 user entry 作為互動次數
        if entry_type == "user" && !is_meta && !is_sidechain {
            if raw.pointer("/message/role").and_then(|v| v.as_str()) == Some("user") {
                interaction_count += 1;
            }
            continue;
        }

        if entry_type != "assistant" || is_sidechain {
            continue;
        }

        let msg = match raw.get("message") {
            Some(m) => m,
            None => continue,
        };
        let usage = match msg.get("usage") {
            Some(u) => u,
            None => continue,
        };
        let msg_id = match msg.get("id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => continue,
        };

        if let Some(ts_str) = raw.get("timestamp").and_then(|v| v.as_str()) {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts_str) {
                let secs = dt.timestamp();
                if first_ts.is_none() || secs < first_ts.unwrap_or(i64::MAX) {
                    first_ts = Some(secs);
                }
                if last_ts.is_none() || secs > last_ts.unwrap_or(i64::MIN) {
                    last_ts = Some(secs);
                }
            }
        }

        let model = msg
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if !model.is_empty() && model != "<synthetic>" {
            models_used.insert(model.clone());
        }

        // 從 message.content 陣列解析 tool_use
        let tool_names = extract_tool_names_from_content(msg);

        let input_tokens = usage
            .get("input_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let output_tokens = usage
            .get("output_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let cache_creation_input_tokens = usage
            .get("cache_creation_input_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let cache_read_input_tokens = usage
            .get("cache_read_input_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let speed = usage
            .get("speed")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let service_tier = usage
            .get("service_tier")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let cache_creation =
            usage
                .get("cache_creation")
                .map(|cc| crate::types::ClaudeCacheCreation {
                    ephemeral_1h_input_tokens: cc
                        .get("ephemeral_1h_input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    ephemeral_5m_input_tokens: cc
                        .get("ephemeral_5m_input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                });

        let total = input_tokens + output_tokens;
        let existing_total = dedup
            .get(msg_id.as_str())
            .map(|e| e.usage.input_tokens + e.usage.output_tokens)
            .unwrap_or(0);

        if total >= existing_total {
            dedup.insert(
                msg_id,
                DedupEntry {
                    usage: crate::types::ClaudeUsage {
                        input_tokens,
                        output_tokens,
                        cache_creation_input_tokens,
                        cache_read_input_tokens,
                        speed,
                        service_tier,
                        cache_creation,
                    },
                    model,
                    tool_names,
                },
            );
        }
    }

    let mut total_input: u64 = 0;
    let mut total_output: u64 = 0;
    let mut tool_call_count: u32 = 0;
    let mut tool_breakdown: BTreeMap<String, u32> = BTreeMap::new();
    let mut model_metrics: BTreeMap<String, ModelMetricsEntry> = BTreeMap::new();

    for entry in dedup.values() {
        total_input += entry.usage.input_tokens;
        total_output += entry.usage.output_tokens;
        tool_call_count += entry.tool_names.len() as u32;
        for name in &entry.tool_names {
            *tool_breakdown.entry(name.clone()).or_insert(0) += 1;
        }

        let pricing = claude_model_pricing(&entry.model);
        let fast_multiplier = if entry.usage.speed.as_deref() == Some("fast") {
            1.3
        } else {
            1.0
        };

        let cc = entry.usage.cache_creation.as_ref();
        let tokens_1h = cc.map(|c| c.ephemeral_1h_input_tokens).unwrap_or(0);
        let tokens_5m = cc
            .map(|c| c.ephemeral_5m_input_tokens)
            .unwrap_or(entry.usage.cache_creation_input_tokens);

        let cost = fast_multiplier
            * ((entry.usage.input_tokens as f64 / 1_000_000.0) * pricing.input
                + (entry.usage.output_tokens as f64 / 1_000_000.0) * pricing.output
                + (tokens_1h as f64 / 1_000_000.0) * pricing.cache_write_1h
                + (tokens_5m as f64 / 1_000_000.0) * pricing.cache_write_5m
                + (entry.usage.cache_read_input_tokens as f64 / 1_000_000.0) * pricing.cache_read);

        let model_entry = model_metrics
            .entry(entry.model.clone())
            .or_insert(ModelMetricsEntry {
                requests_count: 0.0,
                requests_cost: 0.0,
                input_tokens: 0,
                output_tokens: 0,
            });
        model_entry.requests_count += 1.0;
        model_entry.requests_cost += cost;
        model_entry.input_tokens += entry.usage.input_tokens;
        model_entry.output_tokens += entry.usage.output_tokens;
    }

    let duration_minutes = match (first_ts, last_ts) {
        (Some(start), Some(end)) if end > start => ((end - start) as u64) / 60,
        _ => 0,
    };

    Ok(SessionStats {
        input_tokens: total_input,
        output_tokens: total_output,
        interaction_count,
        tool_call_count,
        duration_minutes,
        models_used: models_used.into_iter().collect(),
        reasoning_count: 0,
        tool_breakdown,
        model_metrics,
        is_live: false,
    })
}

/// 從 message JSON 的 content 陣列中提取 tool_use 的工具名稱清單
fn extract_tool_names_from_content(msg: &serde_json::Value) -> Vec<String> {
    let Some(content) = msg.get("content").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    content
        .iter()
        .filter(|item| item.get("type").and_then(|v| v.as_str()) == Some("tool_use"))
        .filter_map(|item| {
            item.get("name")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect()
}

pub(crate) fn build_claude_usage_blocks(
    session_path: &Path,
) -> Result<Vec<ClaudeUsageBlock>, String> {
    use std::collections::HashMap as StdHashMap;

    let file = fs::File::open(session_path)
        .map_err(|e| format!("failed to open Claude session for blocks: {e}"))?;
    let reader = BufReader::new(file);

    struct Entry {
        ts: i64,
        usage: crate::types::ClaudeUsage,
        model: String,
        is_error: bool,
        error_text: String,
    }

    let mut entries: Vec<Entry> = Vec::new();
    let mut dedup_ids: StdHashMap<String, ()> = StdHashMap::new();

    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let Ok(entry) = serde_json::from_str::<crate::types::ClaudeEntry>(trimmed) else {
            continue;
        };

        if entry.entry_type == "assistant" && !entry.is_sidechain {
            if let Some(msg) = &entry.message {
                if let (Some(msg_id), Some(usage), Some(ts_str)) =
                    (&msg.id, &msg.usage, &entry.timestamp)
                {
                    if dedup_ids.contains_key(msg_id.as_str()) {
                        continue;
                    }
                    let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts_str) else {
                        continue;
                    };
                    dedup_ids.insert(msg_id.clone(), ());
                    let is_error = entry.is_api_error_message.unwrap_or(false);
                    // Try to extract reset time from content if error
                    let error_text = if is_error {
                        // Content is opaque here; we'll parse from raw line
                        trimmed.to_string()
                    } else {
                        String::new()
                    };
                    entries.push(Entry {
                        ts: dt.timestamp(),
                        usage: crate::types::ClaudeUsage {
                            input_tokens: usage.input_tokens,
                            output_tokens: usage.output_tokens,
                            cache_creation_input_tokens: usage.cache_creation_input_tokens,
                            cache_read_input_tokens: usage.cache_read_input_tokens,
                            speed: usage.speed.clone(),
                            service_tier: usage.service_tier.clone(),
                            cache_creation: usage.cache_creation.as_ref().map(|cc| {
                                crate::types::ClaudeCacheCreation {
                                    ephemeral_1h_input_tokens: cc.ephemeral_1h_input_tokens,
                                    ephemeral_5m_input_tokens: cc.ephemeral_5m_input_tokens,
                                }
                            }),
                        },
                        model: msg.model.clone().unwrap_or_default(),
                        is_error,
                        error_text,
                    });
                }
            }
        }
    }

    entries.sort_by_key(|e| e.ts);

    const FIVE_HOURS_SECS: i64 = 5 * 3600;
    let mut blocks: Vec<ClaudeUsageBlock> = Vec::new();

    for entry in &entries {
        let needs_new_block = blocks.last().map_or(true, |b: &ClaudeUsageBlock| {
            let block_end_ts = chrono::DateTime::parse_from_rfc3339(&b.end_time)
                .map(|dt| dt.timestamp())
                .unwrap_or(0);
            entry.ts > block_end_ts
        });

        if needs_new_block {
            let start = chrono::DateTime::<chrono::Utc>::from_timestamp(entry.ts, 0)
                .unwrap_or_default()
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            let end =
                chrono::DateTime::<chrono::Utc>::from_timestamp(entry.ts + FIVE_HOURS_SECS, 0)
                    .unwrap_or_default()
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            blocks.push(ClaudeUsageBlock {
                start_time: start,
                end_time: end,
                is_active: false,
                input_tokens: 0,
                output_tokens: 0,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
                cost_usd: 0.0,
                usage_limit_reset_time: None,
            });
        }

        if let Some(block) = blocks.last_mut() {
            block.input_tokens += entry.usage.input_tokens;
            block.output_tokens += entry.usage.output_tokens;
            let cc = entry.usage.cache_creation.as_ref();
            let cc_tokens = cc
                .map(|c| c.ephemeral_1h_input_tokens + c.ephemeral_5m_input_tokens)
                .unwrap_or(entry.usage.cache_creation_input_tokens);
            block.cache_creation_tokens += cc_tokens;
            block.cache_read_tokens += entry.usage.cache_read_input_tokens;

            let pricing = claude_model_pricing(&entry.model);
            let fast_mul = if entry.usage.speed.as_deref() == Some("fast") {
                1.3
            } else {
                1.0
            };
            let tokens_1h = cc.map(|c| c.ephemeral_1h_input_tokens).unwrap_or(0);
            let tokens_5m = cc
                .map(|c| c.ephemeral_5m_input_tokens)
                .unwrap_or(entry.usage.cache_creation_input_tokens);
            block.cost_usd += fast_mul
                * ((entry.usage.input_tokens as f64 / 1_000_000.0) * pricing.input
                    + (entry.usage.output_tokens as f64 / 1_000_000.0) * pricing.output
                    + (tokens_1h as f64 / 1_000_000.0) * pricing.cache_write_1h
                    + (tokens_5m as f64 / 1_000_000.0) * pricing.cache_write_5m
                    + (entry.usage.cache_read_input_tokens as f64 / 1_000_000.0)
                        * pricing.cache_read);

            // Parse reset time from error messages
            if entry.is_error && block.usage_limit_reset_time.is_none() {
                // Look for |<unix_seconds> pattern
                if let Some(pos) = entry.error_text.rfind('|') {
                    let candidate = entry.error_text[pos + 1..]
                        .split('"')
                        .next()
                        .unwrap_or("")
                        .trim();
                    if let Ok(reset_secs) = candidate.parse::<i64>() {
                        block.usage_limit_reset_time =
                            chrono::DateTime::<chrono::Utc>::from_timestamp(reset_secs, 0)
                                .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
                    }
                }
            }
        }
    }

    // Mark last block as active if within 5 hours
    let now = chrono::Utc::now().timestamp();
    if let Some(last) = blocks.last_mut() {
        let block_end = chrono::DateTime::parse_from_rfc3339(&last.end_time)
            .map(|dt| dt.timestamp())
            .unwrap_or(0);
        last.is_active = now < block_end;
    }

    Ok(blocks)
}
