use std::collections::{BTreeMap, BTreeSet, HashMap};
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
    if m.contains("fable-5-1")
        || m.contains("fable-5.1")
        || m.contains("mythos-5-1")
        || m.contains("mythos-5.1")
    {
        ClaudeModelPricing {
            input: 10.0,
            output: 50.0,
            cache_write_1h: 20.0,
            cache_write_5m: 12.5,
            cache_read: 0.25,
        }
    } else if m.contains("fable-5") || m.contains("mythos-5") {
        ClaudeModelPricing {
            input: 10.0,
            output: 50.0,
            cache_write_1h: 20.0,
            cache_write_5m: 12.5,
            cache_read: 1.0,
        }
    } else if m.contains("opus-5-5") || m.contains("opus-5.5") {
        ClaudeModelPricing {
            input: 4.0,
            output: 20.0,
            cache_write_1h: 8.0,
            cache_write_5m: 5.0,
            cache_read: 0.2,
        }
    } else if m.contains("opus-5") {
        ClaudeModelPricing {
            input: 5.0,
            output: 25.0,
            cache_write_1h: 10.0,
            cache_write_5m: 6.25,
            cache_read: 0.5,
        }
    } else if m.contains("sonnet-5") {
        ClaudeModelPricing {
            input: 2.0,
            output: 10.0,
            cache_write_1h: 4.0,
            cache_write_5m: 2.5,
            cache_read: 0.2,
        }
    } else if [
        "opus-4-8", "opus-4.8", "opus-4-7", "opus-4.7", "opus-4-6", "opus-4.6", "opus-4-5",
        "opus-4.5",
    ]
    .iter()
    .any(|version| m.contains(version))
    {
        ClaudeModelPricing {
            input: 5.0,
            output: 25.0,
            cache_write_1h: 10.0,
            cache_write_5m: 6.25,
            cache_read: 0.5,
        }
    } else if m.contains("opus-4") || m.contains("opus-3-5") || m.contains("opus-3.5") {
        ClaudeModelPricing {
            input: 15.0,
            output: 75.0,
            cache_write_1h: 30.0,
            cache_write_5m: 18.75,
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
            cache_write_5m: 3.75,
            cache_read: 0.3,
        }
    } else if m.contains("haiku-4-5") || m.contains("haiku-4.5") {
        ClaudeModelPricing {
            input: 1.0,
            output: 5.0,
            cache_write_1h: 2.0,
            cache_write_5m: 1.25,
            cache_read: 0.1,
        }
    } else if m.contains("haiku-3-5") || m.contains("haiku-3.5") {
        ClaudeModelPricing {
            input: 0.8,
            output: 4.0,
            cache_write_1h: 1.6,
            cache_write_5m: 1.0,
            cache_read: 0.08,
        }
    } else if m.contains("haiku") {
        ClaudeModelPricing {
            input: 0.25,
            output: 1.25,
            cache_write_1h: 0.5,
            cache_write_5m: 0.3125,
            cache_read: 0.025,
        }
    } else {
        // fallback: sonnet-level pricing
        ClaudeModelPricing {
            input: 3.0,
            output: 15.0,
            cache_write_1h: 6.0,
            cache_write_5m: 3.75,
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
            2.0
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

pub(crate) fn build_claude_usage_events(
    session_path: &Path,
    session_id: &str,
) -> Result<Vec<UsageEventRecord>, String> {
    const PRICE_VERSION: &str = "anthropic-api-pricing-2026-09-24";

    let file = fs::File::open(session_path)
        .map_err(|error| format!("failed to open Claude usage source: {error}"))?;
    let reader = BufReader::new(file);
    let mut messages = HashMap::<String, (String, serde_json::Value)>::new();

    for line in reader.lines().map_while(Result::ok) {
        let Ok(raw) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        if raw.get("type").and_then(|value| value.as_str()) != Some("assistant")
            || raw
                .get("isSidechain")
                .and_then(|value| value.as_bool())
                .unwrap_or(false)
        {
            continue;
        }
        let Some(message) = raw.get("message") else {
            continue;
        };
        let Some(message_id) = message.get("id").and_then(|value| value.as_str()) else {
            continue;
        };
        let Some(usage) = message.get("usage") else {
            continue;
        };
        if usage
            .get("input_tokens")
            .and_then(|value| value.as_u64())
            .is_none()
            || usage
                .get("output_tokens")
                .and_then(|value| value.as_u64())
                .is_none()
        {
            continue;
        }
        let Some(timestamp) = raw.get("timestamp").and_then(|value| value.as_str()) else {
            continue;
        };
        let Ok(parsed_timestamp) = chrono::DateTime::parse_from_rfc3339(timestamp) else {
            continue;
        };
        let timestamp = parsed_timestamp
            .with_timezone(&chrono::Utc)
            .to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);

        match messages.get_mut(message_id) {
            Some((_, latest)) => {
                latest["message"] = message.clone();
                if let Some(cwd) = raw.get("cwd") {
                    latest["cwd"] = cwd.clone();
                }
            }
            None => {
                messages.insert(message_id.to_string(), (timestamp, raw));
            }
        }
    }

    let mut events = Vec::with_capacity(messages.len());
    for (source_event_id, (occurred_at, raw)) in messages {
        let Some(message) = raw.get("message") else {
            continue;
        };
        let Some(usage) = message.get("usage") else {
            continue;
        };
        let input_tokens = usage
            .get("input_tokens")
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        let cache_creation_tokens = usage
            .get("cache_creation_input_tokens")
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        let cache_read_tokens = usage
            .get("cache_read_input_tokens")
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        let output_tokens = usage
            .get("output_tokens")
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        let normalized_input_tokens = input_tokens
            .saturating_add(cache_creation_tokens)
            .saturating_add(cache_read_tokens);
        let model = message
            .get("model")
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty() && *value != "<synthetic>")
            .unwrap_or("unknown")
            .to_string();
        let pricing_is_known = [
            "fable-5",
            "mythos-5",
            "opus-5",
            "opus-4",
            "opus-3-5",
            "opus-3.5",
            "sonnet-5",
            "sonnet-4",
            "sonnet-3-7",
            "sonnet-3.7",
            "sonnet-3-5",
            "sonnet-3.5",
            "haiku-4",
            "haiku-3-5",
            "haiku-3.5",
            "haiku",
        ]
        .iter()
        .any(|known| model.to_lowercase().contains(known));
        let (estimated_usd_micros, price_version) = if pricing_is_known {
            let pricing = claude_model_pricing(&model);
            let cache_creation = usage
                .get("cache_creation")
                .filter(|value| value.is_object());
            let cache_write_1h = cache_creation
                .and_then(|value| value.get("ephemeral_1h_input_tokens"))
                .and_then(|value| value.as_f64())
                .unwrap_or(0.0);
            let cache_write_5m = cache_creation
                .and_then(|value| value.get("ephemeral_5m_input_tokens"))
                .and_then(|value| value.as_f64())
                .unwrap_or(cache_creation_tokens as f64);
            let fast_multiplier =
                if usage.get("speed").and_then(|value| value.as_str()) == Some("fast") {
                    2.0
                } else {
                    1.0
                };
            let estimate = fast_multiplier
                * ((input_tokens as f64 / 1_000_000.0) * pricing.input
                    + (output_tokens as f64 / 1_000_000.0) * pricing.output
                    + (cache_write_1h / 1_000_000.0) * pricing.cache_write_1h
                    + (cache_write_5m / 1_000_000.0) * pricing.cache_write_5m
                    + (cache_read_tokens as f64 / 1_000_000.0) * pricing.cache_read);
            (usd_to_micros(estimate), Some(PRICE_VERSION.to_string()))
        } else {
            (None, None)
        };

        events.push(UsageEventRecord {
            provider: "claude".to_string(),
            model_provider_id: Some("claude".to_string()),
            service_tier: usage
                .get("speed")
                .and_then(|value| value.as_str())
                .or_else(|| usage.get("service_tier").and_then(|value| value.as_str()))
                .map(str::to_ascii_lowercase),
            session_id: session_id.to_string(),
            source_event_id,
            occurred_at,
            cwd: raw
                .get("cwd")
                .and_then(|value| value.as_str())
                .map(str::to_string),
            model: Some(model),
            input_tokens: Some(normalized_input_tokens),
            output_tokens: Some(output_tokens),
            cache_read_tokens: Some(cache_read_tokens),
            cache_write_tokens: Some(cache_creation_tokens),
            reasoning_tokens: usage
                .pointer("/output_tokens_details/thinking_tokens")
                .and_then(|value| value.as_i64()),
            estimated_usd_micros,
            estimated_usd_price_version: price_version,
            cost_points_micros: None,
            source_kind: "claude_message".to_string(),
            parser_version: 4,
        });
    }

    events.sort_by(|left, right| {
        left.occurred_at
            .cmp(&right.occurred_at)
            .then_with(|| left.source_event_id.cmp(&right.source_event_id))
    });
    Ok(events)
}

pub(crate) fn claude_usage_coverage(
    session_path: &Path,
) -> Result<(AnalyticsCoverageStatus, Option<String>), String> {
    let file = fs::File::open(session_path)
        .map_err(|error| format!("failed to open Claude usage source: {error}"))?;
    let mut incomplete_records = 0_u64;
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        let Ok(raw) = serde_json::from_str::<serde_json::Value>(&line) else {
            incomplete_records += 1;
            continue;
        };
        if raw.get("type").and_then(|value| value.as_str()) != Some("assistant")
            || raw
                .get("isSidechain")
                .and_then(|value| value.as_bool())
                .unwrap_or(false)
        {
            continue;
        }
        let complete = raw
            .get("timestamp")
            .and_then(|value| value.as_str())
            .is_some()
            && raw
                .pointer("/message/id")
                .and_then(|value| value.as_str())
                .is_some()
            && raw
                .pointer("/message/usage/input_tokens")
                .and_then(|value| value.as_u64())
                .is_some()
            && raw
                .pointer("/message/usage/output_tokens")
                .and_then(|value| value.as_u64())
                .is_some();
        if !complete {
            incomplete_records += 1;
        }
    }
    if incomplete_records == 0 {
        Ok((AnalyticsCoverageStatus::Complete, None))
    } else {
        Ok((
            AnalyticsCoverageStatus::Partial,
            Some(format!(
                "{incomplete_records} Claude assistant records are missing usage identity, timestamp, input tokens, or output tokens"
            )),
        ))
    }
}

fn usd_to_micros(estimate_usd: f64) -> Option<i64> {
    if !estimate_usd.is_finite() || estimate_usd < 0.0 {
        return None;
    }
    Some((estimate_usd.mul_add(1_000_000.0, 1e-9)).round() as i64)
}

#[cfg(test)]
mod pricing_tests {
    use super::usd_to_micros;

    #[test]
    fn estimated_cost_rounds_half_up_to_six_decimal_places() {
        assert_eq!(usd_to_micros(0.000_000_5), Some(1));
        assert_eq!(usd_to_micros(0.000_001_5), Some(2));
        assert_eq!(usd_to_micros(0.000_001_4), Some(1));
        assert_eq!(usd_to_micros(f64::NAN), None);
    }
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
