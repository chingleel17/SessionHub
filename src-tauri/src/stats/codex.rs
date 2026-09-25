use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::types::{AnalyticsCoverageStatus, ProviderUsageExtraction, UsageEventRecord};

const PRICE_VERSION: &str = "openai-api-pricing-2026-09-24";

#[derive(Clone, Copy, Default)]
struct TokenUsage {
    input: i64,
    cached_input: i64,
    cache_write_input: i64,
    output: i64,
    reasoning_output: i64,
    total: i64,
}

#[derive(Clone, Copy)]
struct OpenAiPricing {
    input: f64,
    cached_input: f64,
    output: f64,
}

fn openai_pricing(model: &str, fast: bool) -> Option<OpenAiPricing> {
    let pricing = match (model.to_ascii_lowercase().as_str(), fast) {
        ("gpt-6-astra", false) => (10.0, 1.0, 50.0),
        ("gpt-6-sol", false) => (2.0, 0.2, 10.0),
        ("gpt-6-luna", false) => (0.10, 0.01, 0.50),
        ("gpt-5.6-sol", false) => (4.00, 0.40, 20.00),
        ("gpt-5.6-terra", false) => (2.00, 0.20, 12.00),
        ("gpt-5.6-luna", false) => (0.20, 0.02, 1.20),
        ("gpt-5.5", false) => (5.00, 0.50, 30.00),
        ("gpt-5.4", false) => (2.50, 0.25, 15.00),
        ("gpt-5.4-mini", false) => (0.75, 0.075, 4.50),
        ("gpt-5.4-nano", false) => (0.20, 0.02, 1.25),
        ("gpt-5.3-codex", false) | ("gpt-5.2", false) => (1.75, 0.175, 14.00),
        ("gpt-5.1", false) | ("gpt-5", false) => (1.25, 0.125, 10.00),
        ("gpt-5-mini", false) => (0.25, 0.025, 2.00),
        ("gpt-5-nano", false) => (0.05, 0.005, 0.40),
        ("gpt-4.1", false) => (2.00, 0.50, 8.00),
        ("gpt-4.1-mini", false) => (0.40, 0.10, 1.60),
        ("gpt-4.1-nano", false) => (0.10, 0.025, 0.40),
        ("gpt-4o", false) => (2.50, 1.25, 10.00),
        ("gpt-4o-mini", false) => (0.15, 0.075, 0.60),
        ("o3", false) => (2.00, 0.50, 8.00),
        ("o4-mini", false) => (1.10, 0.275, 4.40),
        ("gpt-6-astra", true) => (20.0, 2.0, 100.0),
        ("gpt-6-sol", true) => (4.0, 0.4, 20.0),
        ("gpt-6-luna", true) => (0.2, 0.02, 1.0),
        ("gpt-5.6-sol", true) => (8.0, 0.8, 40.0),
        ("gpt-5.6-terra", true) => (4.0, 0.4, 24.0),
        ("gpt-5.6-luna", true) => (0.4, 0.04, 2.4),
        ("gpt-5.5", true) => (12.5, 1.25, 75.0),
        ("gpt-5.4", true) => (5.0, 0.5, 30.0),
        ("gpt-5.4-mini", true) => (1.5, 0.15, 9.0),
        ("gpt-5.3-codex", true) | ("gpt-5.2", true) => (3.5, 0.35, 28.0),
        ("gpt-5.1", true) | ("gpt-5", true) => (2.5, 0.25, 20.0),
        ("gpt-5-mini", true) => (0.45, 0.045, 3.6),
        ("gpt-4.1", true) => (3.5, 0.875, 14.0),
        ("gpt-4.1-mini", true) => (0.7, 0.175, 2.8),
        ("gpt-4.1-nano", true) => (0.2, 0.05, 0.8),
        ("gpt-4o", true) => (4.25, 2.125, 17.0),
        ("gpt-4o-mini", true) => (0.25, 0.125, 1.0),
        ("o3", true) => (3.5, 0.875, 14.0),
        ("o4-mini", true) => (2.0, 0.5, 8.0),
        _ => return None,
    };
    Some(OpenAiPricing {
        input: pricing.0,
        cached_input: pricing.1,
        output: pricing.2,
    })
}

fn parse_usage(value: &serde_json::Value) -> Option<TokenUsage> {
    let read = |name: &str| value.get(name).and_then(serde_json::Value::as_i64);
    Some(TokenUsage {
        input: read("input_tokens")?,
        cached_input: read("cached_input_tokens").unwrap_or(0),
        cache_write_input: read("cache_write_input_tokens").unwrap_or(0),
        output: read("output_tokens")?,
        reasoning_output: read("reasoning_output_tokens").unwrap_or(0),
        total: read("total_tokens")?,
    })
}

fn usage_delta(current: TokenUsage, previous: Option<TokenUsage>) -> TokenUsage {
    let Some(previous) = previous else {
        return current;
    };
    if current.total < previous.total {
        return current;
    }
    TokenUsage {
        input: current.input.saturating_sub(previous.input),
        cached_input: current.cached_input.saturating_sub(previous.cached_input),
        cache_write_input: current
            .cache_write_input
            .saturating_sub(previous.cache_write_input),
        output: current.output.saturating_sub(previous.output),
        reasoning_output: current
            .reasoning_output
            .saturating_sub(previous.reasoning_output),
        total: current.total.saturating_sub(previous.total),
    }
}

fn estimate_usd_micros(model: &str, usage: TokenUsage, service_tier: Option<&str>) -> Option<i64> {
    if usage.cache_write_input > 0 {
        return None;
    }
    let fast = matches!(service_tier, Some("fast" | "priority"));
    let pricing = openai_pricing(model, fast)?;
    let uncached_input = usage.input.saturating_sub(usage.cached_input);
    let usd = (uncached_input as f64 * pricing.input
        + usage.cached_input as f64 * pricing.cached_input
        + usage.output as f64 * pricing.output)
        / 1_000_000.0;
    Some((usd * 1_000_000.0).round() as i64)
}

pub(crate) fn extract_codex_usage(
    session_path: &Path,
    session_id: &str,
) -> Result<ProviderUsageExtraction, String> {
    let file = fs::File::open(session_path)
        .map_err(|error| format!("failed to open Codex usage source: {error}"))?;
    let mut events = Vec::new();
    let mut model: Option<String> = None;
    let mut model_provider_id: Option<String> = None;
    let mut service_tier: Option<String> = None;
    let mut previous_usage = None;

    for (line_index, line) in BufReader::new(file).lines().enumerate() {
        let line = line.map_err(|error| format!("failed to read Codex usage line: {error}"))?;
        let Ok(record) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        let record_type = record.get("type").and_then(serde_json::Value::as_str);
        let payload = record.get("payload").unwrap_or(&record);
        match record_type {
            Some("session_meta") => {
                model_provider_id = payload
                    .get("model_provider")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .map(str::to_string);
            }
            Some("turn_context") => {
                model = payload
                    .get("model")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .map(str::to_string);
                service_tier = payload
                    .get("service_tier")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .map(str::to_ascii_lowercase);
            }
            _ => {}
        }

        let token_info = if record_type == Some("event_msg")
            && payload.get("type").and_then(serde_json::Value::as_str) == Some("token_count")
        {
            payload.get("info")
        } else if record_type == Some("token_count") {
            record.get("info")
        } else {
            None
        };
        let Some(current_usage) = token_info
            .and_then(|info| info.get("total_token_usage"))
            .and_then(parse_usage)
        else {
            continue;
        };
        let delta = usage_delta(current_usage, previous_usage);
        previous_usage = Some(current_usage);
        if delta.total == 0 {
            continue;
        }
        let Some(occurred_at) = record
            .get("timestamp")
            .and_then(serde_json::Value::as_str)
            .and_then(super::parse_rfc3339_utc)
        else {
            continue;
        };
        let model_name = model.clone().unwrap_or_else(|| "unknown".to_string());
        let event_service_tier = token_info
            .and_then(|info| info.get("service_tier"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_ascii_lowercase)
            .or_else(|| service_tier.clone());
        let estimated_usd_micros =
            estimate_usd_micros(&model_name, delta, event_service_tier.as_deref());
        events.push(UsageEventRecord {
            provider: "codex".to_string(),
            model_provider_id: model_provider_id.clone(),
            service_tier: event_service_tier,
            session_id: session_id.to_string(),
            source_event_id: format!("token-count-{line_index}"),
            occurred_at,
            cwd: None,
            model: Some(model_name),
            input_tokens: Some(delta.input),
            output_tokens: Some(delta.output),
            cache_read_tokens: Some(delta.cached_input),
            cache_write_tokens: Some(delta.cache_write_input),
            reasoning_tokens: Some(delta.reasoning_output),
            estimated_usd_micros,
            estimated_usd_price_version: estimated_usd_micros.map(|_| PRICE_VERSION.to_string()),
            cost_points_micros: None,
            source_kind: "codex_token_count".to_string(),
            parser_version: 3,
        });
    }

    Ok(ProviderUsageExtraction {
        status: if events.is_empty() {
            AnalyticsCoverageStatus::Unsupported
        } else {
            AnalyticsCoverageStatus::Partial
        },
        reason: if events.is_empty() {
            Some("No supported persisted Codex token usage records were found".to_string())
        } else if events
            .iter()
            .any(|event| event.estimated_usd_micros.is_none())
        {
            Some("Codex tokens were indexed; API-equivalent cost is unavailable for unknown models or cache-write pricing".to_string())
        } else {
            Some(
                "Codex persisted token counters were indexed with API-equivalent estimates"
                    .to_string(),
            )
        },
        events,
        session_summaries: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::{estimate_usd_micros, usage_delta, TokenUsage};

    #[test]
    fn cumulative_usage_handles_duplicates_and_resets() {
        let first = TokenUsage {
            input: 100,
            cached_input: 20,
            output: 20,
            reasoning_output: 5,
            total: 120,
            ..TokenUsage::default()
        };
        assert_eq!(usage_delta(first, None).total, 120);
        assert_eq!(usage_delta(first, Some(first)).total, 0);
        let reset = TokenUsage {
            input: 25,
            output: 5,
            total: 30,
            ..TokenUsage::default()
        };
        assert_eq!(usage_delta(reset, Some(first)).total, 30);
    }

    #[test]
    fn known_openai_model_uses_cached_input_without_double_counting() {
        let usage = TokenUsage {
            input: 100,
            cached_input: 20,
            output: 20,
            total: 120,
            ..TokenUsage::default()
        };
        assert_eq!(estimate_usd_micros("gpt-6-luna", usage, None), Some(18));
        assert_eq!(
            estimate_usd_micros("gpt-6-luna", usage, Some("fast")),
            Some(36)
        );
        assert_eq!(estimate_usd_micros("unknown", usage, None), None);
    }
}
