use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use crate::db::{
    delete_manual_model_pricing_record, distinct_unpriced_models, list_model_pricing_cache,
    model_pricing_cache, seed_builtin_model_pricing, set_model_pricing_hidden,
    unpriced_usage_events_for_model, upsert_model_pricing_cache, upsert_usage_events,
    usage_events_for_model,
};
use crate::quota::http::{self, ApiOutcome};
use crate::types::{
    ManualModelPricingInput, ModelPricingCacheRecord, ModelPricingEntry, UsageEventRecord,
};

const OPENROUTER_MODELS_URL: &str = "https://openrouter.ai/api/v1/models";
const OPENAI_PRICING_URL: &str = "https://developers.openai.com/api/docs/pricing.md";
const HTTP_TIMEOUT: Duration = Duration::from_secs(5);
const SUCCESS_TTL_SECONDS: i64 = 30 * 24 * 60 * 60;
const NOT_FOUND_TTL_SECONDS: i64 = 24 * 60 * 60;
const FAILED_TTL_SECONDS: i64 = 15 * 60;
const FETCHING_TTL_SECONDS: i64 = 5 * 60;
const EXCLUDED_OPENROUTER_MODELS: &[(&str, &str)] = &[("openai", "codex-auto-review")];

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PricingCandidate {
    event_provider: String,
    model_provider_id: Option<String>,
    author: String,
    event_model: String,
    model: String,
}

fn now_epoch_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn pricing_author(event_provider: &str, model_provider_id: Option<&str>) -> Option<&'static str> {
    match event_provider {
        "claude" => Some("anthropic"),
        "codex" => match model_provider_id.map(str::to_ascii_lowercase) {
            None => Some("openai"),
            Some(provider) if provider == "openai" || provider == "chatgpt" => Some("openai"),
            _ => None,
        },
        "opencode" => match model_provider_id.map(str::to_ascii_lowercase).as_deref() {
            Some("openai") => Some("openai"),
            Some("anthropic") | Some("claude") => Some("anthropic"),
            _ => None,
        },
        _ => None,
    }
}

fn is_excluded_openrouter_model(provider: &str, model: &str) -> bool {
    let provider = provider.to_ascii_lowercase();
    let model = model.to_ascii_lowercase();
    EXCLUDED_OPENROUTER_MODELS
        .iter()
        .any(|(excluded_provider, excluded_model)| {
            provider == *excluded_provider && model == *excluded_model
        })
}

fn catalog_keys(author: &str, model: &str) -> Vec<String> {
    let mut model = model.to_ascii_lowercase();
    let provider_prefix = format!("{author}/");
    if model.starts_with(&provider_prefix) {
        model = model[provider_prefix.len()..].to_string();
    }
    let mut keys = vec![format!("{author}/{model}")];
    let bytes = model.as_bytes();
    if bytes.len() > 9
        && bytes[bytes.len() - 9] == b'-'
        && bytes[bytes.len() - 8..]
            .iter()
            .all(|value| value.is_ascii_digit())
    {
        keys.push(format!("{author}/{}", &model[..model.len() - 9]));
    }
    keys
}

fn cached_pricing(
    connection: &Connection,
    author: &str,
    model: &str,
) -> Result<Option<ModelPricingCacheRecord>, String> {
    for key in catalog_keys(author, model) {
        let Some((provider, model)) = key.split_once('/') else {
            continue;
        };
        if let Some(record) = model_pricing_cache(connection, provider, model)? {
            return Ok(Some(record));
        }
    }
    Ok(None)
}

fn cache_record(
    candidate: &PricingCandidate,
    status: &str,
    now: i64,
    ttl: i64,
    error_kind: Option<String>,
) -> ModelPricingCacheRecord {
    ModelPricingCacheRecord {
        provider: candidate.author.clone(),
        model: candidate.model.clone(),
        status: status.to_string(),
        prompt_usd_per_token: None,
        completion_usd_per_token: None,
        cache_read_usd_per_token: None,
        cache_write_usd_per_token: None,
        source: "openrouter-catalog".to_string(),
        fetched_at: now,
        expires_at: now.saturating_add(ttl),
        error_kind,
        hidden: false,
        sort_order: 0,
    }
}

fn parse_non_negative_price(
    pricing: &serde_json::Value,
    field: &str,
) -> Result<Option<f64>, String> {
    let Some(value) = pricing.get(field) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let parsed = value
        .as_str()
        .ok_or_else(|| format!("invalid {field} price type"))?
        .parse::<f64>()
        .map_err(|_| format!("invalid {field} price"))?;
    if !parsed.is_finite() || parsed < 0.0 {
        return Err(format!("invalid {field} price"));
    }
    Ok(Some(parsed))
}

fn parse_catalog(
    body: serde_json::Value,
    now: i64,
) -> Result<BTreeMap<String, ModelPricingCacheRecord>, String> {
    let data = body
        .get("data")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "OpenRouter model catalog is missing data".to_string())?;
    let mut records = BTreeMap::new();
    for model in data {
        let Some(id) = model.get("id").and_then(serde_json::Value::as_str) else {
            continue;
        };
        if model
            .get("expiration_date")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .is_some_and(|value| value.timestamp() <= now)
        {
            continue;
        }
        let Some((provider, model_name)) = id.split_once('/') else {
            continue;
        };
        if is_excluded_openrouter_model(provider, model_name) {
            continue;
        }
        let Some(pricing) = model.get("pricing") else {
            continue;
        };
        if pricing
            .get("overrides")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|overrides| !overrides.is_empty())
        {
            records.insert(
                id.to_ascii_lowercase(),
                ModelPricingCacheRecord {
                    provider: provider.to_ascii_lowercase(),
                    model: model_name.to_ascii_lowercase(),
                    status: "failed".to_string(),
                    prompt_usd_per_token: None,
                    completion_usd_per_token: None,
                    cache_read_usd_per_token: None,
                    cache_write_usd_per_token: None,
                    source: "openrouter-catalog".to_string(),
                    fetched_at: now,
                    expires_at: now.saturating_add(NOT_FOUND_TTL_SECONDS),
                    error_kind: Some("conditional_pricing_unsupported".to_string()),
                    hidden: false,
                    sort_order: 0,
                },
            );
            continue;
        }
        let Ok(Some(prompt)) = parse_non_negative_price(pricing, "prompt") else {
            continue;
        };
        let Ok(Some(completion)) = parse_non_negative_price(pricing, "completion") else {
            continue;
        };
        let Ok(cache_read) = parse_non_negative_price(pricing, "input_cache_read") else {
            continue;
        };
        let Ok(cache_write) = parse_non_negative_price(pricing, "input_cache_write") else {
            continue;
        };
        let ttl = if cache_read.is_some() && cache_write.is_some() {
            SUCCESS_TTL_SECONDS
        } else {
            NOT_FOUND_TTL_SECONDS
        };
        records.insert(
            id.to_ascii_lowercase(),
            ModelPricingCacheRecord {
                provider: provider.to_ascii_lowercase(),
                model: model_name.to_ascii_lowercase(),
                status: "success".to_string(),
                prompt_usd_per_token: Some(prompt),
                completion_usd_per_token: Some(completion),
                cache_read_usd_per_token: cache_read,
                cache_write_usd_per_token: cache_write,
                source: "openrouter-catalog".to_string(),
                fetched_at: now,
                expires_at: now.saturating_add(ttl),
                error_kind: None,
                hidden: false,
                sort_order: 0,
            },
        );
    }
    Ok(records)
}

fn fetch_catalog(now: i64) -> Result<BTreeMap<String, ModelPricingCacheRecord>, String> {
    let response = http::get(OPENROUTER_MODELS_URL, Some(HTTP_TIMEOUT))
        .call()
        .map_err(|error| format!("OpenRouter catalog request failed: {error}"))?;
    match http::classify(response) {
        ApiOutcome::Success(mut response) => response
            .body_mut()
            .read_json::<serde_json::Value>()
            .map_err(|error| format!("failed to parse OpenRouter model catalog: {error}"))
            .and_then(|body| parse_catalog(body, now)),
        ApiOutcome::Unauthorized => Err("OpenRouter catalog rejected the request".to_string()),
        ApiOutcome::RateLimited { .. } => {
            Err("OpenRouter catalog request was rate limited".to_string())
        }
        ApiOutcome::UnexpectedStatus(status) => {
            Err(format!("OpenRouter catalog returned HTTP {status}"))
        }
    }
}

// 官方提供 Markdown 版本；只讀 Standard 的短上下文文字模型價格。
fn parse_openai_pricing(
    body: &str,
    now: i64,
) -> Result<BTreeMap<String, ModelPricingCacheRecord>, String> {
    let section = body
        .split_once("### Standard pricing data")
        .ok_or_else(|| "OpenAI standard pricing table is missing".to_string())?
        .1;
    let table = section.split("###").next().unwrap_or("");
    if !table.lines().any(|line| line.trim() == "| Model | Short context input | Short context cached input | Short context cache writes | Short context output | Long context input | Long context cached input | Long context cache writes | Long context output |") {
        return Err("OpenAI standard pricing columns are unsupported".to_string());
    }
    let mut records = BTreeMap::new();
    for line in table.lines().filter(|line| line.trim().starts_with('|')) {
        let cells = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if cells.len() != 9 || !cells[1].starts_with('$') {
            continue;
        }
        let model = cells[0].split_whitespace().next().unwrap_or("");
        if model.is_empty() {
            continue;
        }
        let price = |value: &str| -> Result<Option<f64>, String> {
            if value == "-" {
                return Ok(None);
            }
            let number = value
                .strip_prefix('$')
                .unwrap_or(value)
                .parse::<f64>()
                .map_err(|_| "invalid OpenAI price".to_string())?;
            if !number.is_finite() || number < 0.0 {
                return Err("invalid OpenAI price".to_string());
            }
            Ok(Some(number / 1_000_000.0))
        };
        let record = ModelPricingCacheRecord {
            provider: "openai".to_string(),
            model: model.to_ascii_lowercase(),
            status: "success".to_string(),
            prompt_usd_per_token: price(cells[1])?,
            cache_read_usd_per_token: price(cells[2])?,
            cache_write_usd_per_token: price(cells[3])?,
            completion_usd_per_token: price(cells[4])?,
            source: "openai-official".to_string(),
            fetched_at: now,
            expires_at: now.saturating_add(SUCCESS_TTL_SECONDS),
            error_kind: None,
            hidden: false,
            sort_order: now.saturating_sub(i64::try_from(records.len()).unwrap_or(i64::MAX)),
        };
        records.insert(format!("openai/{}", record.model), record);
    }
    if records.is_empty() {
        return Err("OpenAI standard pricing table is empty or unsupported".to_string());
    }
    Ok(records)
}

fn fetch_openai_pricing(now: i64) -> Result<BTreeMap<String, ModelPricingCacheRecord>, String> {
    let response = http::get(OPENAI_PRICING_URL, Some(HTTP_TIMEOUT))
        .call()
        .map_err(|error| format!("OpenAI pricing request failed: {error}"))?;
    match http::classify(response) {
        ApiOutcome::Success(mut response) => {
            let body = response
                .body_mut()
                .read_to_string()
                .map_err(|error| format!("failed to read OpenAI pricing: {error}"))?;
            parse_openai_pricing(&body, now)
        }
        _ => Err("OpenAI pricing request was rejected".to_string()),
    }
}

fn store_catalog(
    connection: &Connection,
    catalog: &BTreeMap<String, ModelPricingCacheRecord>,
) -> Result<(), String> {
    for record in catalog.values() {
        if let Some(existing) = model_pricing_cache(connection, &record.provider, &record.model)? {
            if existing.source == "user"
                || (record.source == "openrouter-catalog"
                    && matches!(
                        existing.source.as_str(),
                        "builtin-official" | "openai-official"
                    ))
            {
                continue;
            }
        }
        upsert_model_pricing_cache(connection, record)?;
        if record.status == "success" {
            apply_pricing_override(connection, record)?;
        }
    }
    Ok(())
}

pub(crate) fn sync_model_pricing_internal(
    connection: &Connection,
) -> Result<Vec<ModelPricingEntry>, String> {
    let now = now_epoch_seconds();
    let official = fetch_openai_pricing(now)?;
    store_catalog(connection, &official)?;
    refresh_missing_model_prices(connection)?;
    for pricing in official.values() {
        if pricing.status == "success" {
            apply_pricing_override(connection, pricing)?;
        }
    }
    list_model_pricing_internal(connection)
}

fn fast_price_per_token(
    provider: &str,
    model: &str,
) -> Option<(f64, f64, Option<f64>, Option<f64>)> {
    let prices = match (provider, model) {
        ("anthropic", "claude-opus-5-5") => (8.0, 40.0, Some(0.4), Some(10.0)),
        ("anthropic", "claude-opus-5" | "claude-opus-4-8") => (10.0, 50.0, Some(1.0), Some(12.5)),
        ("openai", "gpt-6-astra") => (20.0, 100.0, Some(2.0), Some(25.0)),
        ("openai", "gpt-6-sol") => (4.0, 20.0, Some(0.4), Some(5.0)),
        ("openai", "gpt-6.1-sol") => (4.0, 20.0, Some(0.2), Some(5.0)),
        ("openai", "gpt-6-luna") => (0.2, 1.0, Some(0.02), Some(0.25)),
        ("openai", "gpt-5.6-sol") => (8.0, 40.0, Some(0.8), Some(10.0)),
        ("openai", "gpt-5.6-terra") => (4.0, 24.0, Some(0.4), Some(5.0)),
        ("openai", "gpt-5.6-luna") => (0.4, 2.4, Some(0.04), Some(0.5)),
        ("openai", "gpt-5.5") => (12.5, 75.0, Some(1.25), None),
        ("openai", "gpt-5.4") => (5.0, 30.0, Some(0.5), None),
        ("openai", "gpt-5.4-mini") => (1.5, 9.0, Some(0.15), None),
        ("openai", "gpt-5.3-codex" | "gpt-5.2") => (3.5, 28.0, Some(0.35), None),
        ("openai", "gpt-5.1" | "gpt-5") => (2.5, 20.0, Some(0.25), None),
        ("openai", "gpt-5-mini") => (0.45, 3.6, Some(0.045), None),
        ("openai", "gpt-4.1") => (3.5, 14.0, Some(0.875), None),
        ("openai", "gpt-4.1-mini") => (0.7, 2.8, Some(0.175), None),
        ("openai", "gpt-4.1-nano") => (0.2, 0.8, Some(0.05), None),
        ("openai", "gpt-4o") => (4.25, 17.0, Some(2.125), None),
        ("openai", "gpt-4o-mini") => (0.25, 1.0, Some(0.125), None),
        ("openai", "o3") => (3.5, 14.0, Some(0.875), None),
        ("openai", "o4-mini") => (2.0, 8.0, Some(0.5), None),
        _ => return None,
    };
    Some((
        prices.0 / 1_000_000.0,
        prices.1 / 1_000_000.0,
        prices.2.map(|value| value / 1_000_000.0),
        prices.3.map(|value| value / 1_000_000.0),
    ))
}

fn estimate_event(event: &UsageEventRecord, pricing: &ModelPricingCacheRecord) -> Option<i64> {
    let input = event.input_tokens?;
    let output = event.output_tokens?;
    let cache_read = event.cache_read_tokens.unwrap_or(0).max(0);
    let cache_write = event.cache_write_tokens.unwrap_or(0).max(0);
    let event_model = event.model.as_deref().map(str::to_ascii_lowercase);
    let model_fast_alias = event_model.as_deref().and_then(|model| {
        let base_model = model.strip_suffix("-fast")?;
        fast_price_per_token(&pricing.provider, base_model).map(|_| base_model)
    });
    let fast = matches!(event.service_tier.as_deref(), Some("fast" | "priority"))
        || model_fast_alias.is_some();
    let fast_prices = if fast
        && (model_fast_alias.is_some()
            || matches!(
                pricing.source.as_str(),
                "builtin-official" | "openai-official"
            )) {
        let model = model_fast_alias.unwrap_or(&pricing.model);
        fast_price_per_token(&pricing.provider, model)
    } else {
        None
    };
    let prompt_price = fast_prices
        .map(|prices| prices.0)
        .or(pricing.prompt_usd_per_token)?;
    let completion_price = fast_prices
        .map(|prices| prices.1)
        .or(pricing.completion_usd_per_token)?;
    let cache_read_cost = if cache_read == 0 {
        0.0
    } else {
        cache_read as f64
            * fast_prices
                .and_then(|prices| prices.2)
                .or(pricing.cache_read_usd_per_token)?
    };
    let cache_write_cost = if cache_write == 0 {
        0.0
    } else {
        cache_write as f64
            * fast_prices
                .and_then(|prices| prices.3)
                .or(pricing.cache_write_usd_per_token)?
    };
    let uncached_input = input
        .saturating_sub(cache_read)
        .saturating_sub(cache_write)
        .max(0);
    let usd = uncached_input as f64 * prompt_price
        + cache_read_cost
        + cache_write_cost
        + output.max(0) as f64 * completion_price;
    if !usd.is_finite() || usd < 0.0 {
        return None;
    }
    Some((usd.mul_add(1_000_000.0, 1e-9)).round() as i64)
}

fn apply_pricing(
    connection: &Connection,
    candidate: &PricingCandidate,
    pricing: &ModelPricingCacheRecord,
) -> Result<(), String> {
    let mut events = unpriced_usage_events_for_model(
        connection,
        &candidate.event_provider,
        candidate.model_provider_id.as_deref(),
        &candidate.event_model,
    )?;
    let version = format!("{}:{}", pricing.source, pricing.fetched_at);
    for event in &mut events {
        if let Some(estimate) = estimate_event(event, pricing) {
            event.estimated_usd_micros = Some(estimate);
            event.estimated_usd_price_version = Some(version.clone());
        }
    }
    events.retain(|event| event.estimated_usd_micros.is_some());
    upsert_usage_events(connection, &events).map(|_| ())
}

fn apply_pricing_override(
    connection: &Connection,
    pricing: &ModelPricingCacheRecord,
) -> Result<(), String> {
    let version = format!("{}:{}", pricing.source, pricing.fetched_at);
    let mut models = vec![pricing.model.clone()];
    if fast_price_per_token(&pricing.provider, &pricing.model).is_some() {
        models.push(format!("{}-fast", pricing.model));
    }
    for model in models {
        let mut events = usage_events_for_model(connection, &model)?
            .into_iter()
            .filter(|event| {
                pricing_author(&event.provider, event.model_provider_id.as_deref())
                    == Some(pricing.provider.as_str())
            })
            .collect::<Vec<_>>();
        for event in &mut events {
            event.estimated_usd_micros = estimate_event(event, pricing);
            event.estimated_usd_price_version = event.estimated_usd_micros.map(|_| version.clone());
        }
        upsert_usage_events(connection, &events)?;
    }
    Ok(())
}

pub(crate) fn refresh_missing_model_prices(connection: &Connection) -> Result<(), String> {
    refresh_missing_model_prices_with_catalog(connection, fetch_openai_pricing, fetch_catalog)
}

fn refresh_missing_model_prices_with_catalog(
    connection: &Connection,
    fetch_official: impl FnOnce(i64) -> Result<BTreeMap<String, ModelPricingCacheRecord>, String>,
    fetch_remote: impl FnOnce(i64) -> Result<BTreeMap<String, ModelPricingCacheRecord>, String>,
) -> Result<(), String> {
    let now = now_epoch_seconds();
    let candidates = distinct_unpriced_models(connection, usize::MAX)?
        .into_iter()
        .filter_map(|(event_provider, model_provider_id, model)| {
            let author = pricing_author(&event_provider, model_provider_id.as_deref())?;
            if is_excluded_openrouter_model(author, &model) {
                return None;
            }
            let normalized_model = model.to_ascii_lowercase();
            let pricing_model = normalized_model
                .strip_suffix("-fast")
                .filter(|model| fast_price_per_token(author, model).is_some())
                .unwrap_or(&normalized_model)
                .to_string();
            Some(PricingCandidate {
                event_provider,
                model_provider_id,
                author: author.to_string(),
                event_model: model.clone(),
                model: pricing_model,
            })
        })
        .collect::<BTreeSet<_>>();
    let mut pending = Vec::new();
    for candidate in candidates {
        let cached = cached_pricing(connection, &candidate.author, &candidate.model)?;
        if let Some(record) = cached.as_ref() {
            if record.expires_at > now {
                if record.status == "success" {
                    if candidate.event_model != candidate.model {
                        apply_pricing_override(connection, record)?;
                    } else {
                        apply_pricing(connection, &candidate, record)?;
                    }
                }
                continue;
            }
        }
        upsert_model_pricing_cache(
            connection,
            &cache_record(&candidate, "fetching", now, FETCHING_TTL_SECONDS, None),
        )?;
        pending.push(candidate);
    }
    if pending.is_empty() {
        return Ok(());
    }

    if pending.iter().any(|candidate| candidate.author == "openai") {
        if let Ok(official) = fetch_official(now) {
            store_catalog(connection, &official)?;
            let mut unresolved = Vec::new();
            for candidate in pending {
                if let Some(record) = catalog_keys(&candidate.author, &candidate.model)
                    .iter()
                    .find_map(|key| official.get(key))
                {
                    let pricing = cached_pricing(connection, &candidate.author, &candidate.model)?
                        .unwrap_or_else(|| record.clone());
                    apply_pricing(connection, &candidate, &pricing)?;
                } else {
                    unresolved.push(candidate);
                }
            }
            pending = unresolved;
        }
    }
    if pending.is_empty() {
        return Ok(());
    }

    match fetch_remote(now) {
        Ok(catalog) => {
            for candidate in pending {
                let mut record = catalog_keys(&candidate.author, &candidate.model)
                    .into_iter()
                    .find_map(|key| catalog.get(&key).cloned())
                    .unwrap_or_else(|| {
                        cache_record(
                            &candidate,
                            "not_found",
                            now,
                            NOT_FOUND_TTL_SECONDS,
                            Some("model_not_in_catalog".to_string()),
                        )
                    });
                record.provider = candidate.author.clone();
                record.model = candidate.model.clone();
                upsert_model_pricing_cache(connection, &record)?;
                if record.status == "success" {
                    apply_pricing(connection, &candidate, &record)?;
                }
            }
        }
        Err(error) => {
            for candidate in pending {
                upsert_model_pricing_cache(
                    connection,
                    &cache_record(
                        &candidate,
                        "failed",
                        now,
                        FAILED_TTL_SECONDS,
                        Some(error.clone()),
                    ),
                )?;
            }
        }
    }
    Ok(())
}

fn to_entry(record: ModelPricingCacheRecord) -> ModelPricingEntry {
    let per_million = |value: Option<f64>| value.map(|price| price * 1_000_000.0);
    ModelPricingEntry {
        provider: record.provider,
        model: record.model,
        status: record.status,
        prompt_usd_per_million: per_million(record.prompt_usd_per_token),
        completion_usd_per_million: per_million(record.completion_usd_per_token),
        cache_read_usd_per_million: per_million(record.cache_read_usd_per_token),
        cache_write_usd_per_million: per_million(record.cache_write_usd_per_token),
        source: record.source,
        fetched_at: record.fetched_at,
        expires_at: record.expires_at,
        error_kind: record.error_kind,
        hidden: record.hidden,
    }
}

pub(crate) fn list_model_pricing_internal(
    connection: &Connection,
) -> Result<Vec<ModelPricingEntry>, String> {
    list_model_pricing_cache(connection).map(|rows| rows.into_iter().map(to_entry).collect())
}

pub(crate) fn save_manual_model_pricing_internal(
    connection: &Connection,
    input: ManualModelPricingInput,
) -> Result<Vec<ModelPricingEntry>, String> {
    let provider = input.provider.trim().to_ascii_lowercase();
    let model = input.model.trim().to_ascii_lowercase();
    if provider.is_empty() || model.is_empty() {
        return Err("provider and model are required".to_string());
    }
    let validate = |name: &str, value: Option<f64>| -> Result<Option<f64>, String> {
        match value {
            Some(value) if value.is_finite() && value >= 0.0 => Ok(Some(value / 1_000_000.0)),
            Some(_) => Err(format!("{name} must be a non-negative number")),
            None => Ok(None),
        }
    };
    let now = now_epoch_seconds();
    let record = ModelPricingCacheRecord {
        provider,
        model,
        status: "success".to_string(),
        prompt_usd_per_token: validate("promptUsdPerMillion", Some(input.prompt_usd_per_million))?,
        completion_usd_per_token: validate(
            "completionUsdPerMillion",
            Some(input.completion_usd_per_million),
        )?,
        cache_read_usd_per_token: validate(
            "cacheReadUsdPerMillion",
            input.cache_read_usd_per_million,
        )?,
        cache_write_usd_per_token: validate(
            "cacheWriteUsdPerMillion",
            input.cache_write_usd_per_million,
        )?,
        source: "user".to_string(),
        fetched_at: now,
        expires_at: i64::MAX,
        error_kind: None,
        hidden: false,
        sort_order: 0,
    };
    upsert_model_pricing_cache(connection, &record)?;
    apply_pricing_override(connection, &record)?;
    refresh_missing_model_prices(connection)?;
    list_model_pricing_internal(connection)
}

pub(crate) fn set_model_pricing_hidden_internal(
    connection: &Connection,
    provider: &str,
    model: &str,
    hidden: bool,
) -> Result<Vec<ModelPricingEntry>, String> {
    let provider = provider.trim().to_ascii_lowercase();
    let model = model.trim().to_ascii_lowercase();
    if provider.is_empty() || model.is_empty() {
        return Err("provider and model are required".to_string());
    }
    if !set_model_pricing_hidden(connection, &provider, &model, hidden)? {
        return Err("model pricing entry was not found".to_string());
    }
    list_model_pricing_internal(connection)
}

pub(crate) fn delete_manual_model_pricing_internal(
    connection: &Connection,
    provider: &str,
    model: &str,
) -> Result<Vec<ModelPricingEntry>, String> {
    if !delete_manual_model_pricing_record(
        connection,
        &provider.trim().to_ascii_lowercase(),
        &model.trim().to_ascii_lowercase(),
    )? {
        return Err("only user-defined model pricing can be deleted".to_string());
    }
    seed_builtin_model_pricing(connection)?;
    if let Some(record) = model_pricing_cache(
        connection,
        &provider.trim().to_ascii_lowercase(),
        &model.trim().to_ascii_lowercase(),
    )? {
        if record.status == "success" {
            apply_pricing_override(connection, &record)?;
        }
    }
    list_model_pricing_internal(connection)
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use serde_json::json;
    use std::collections::BTreeMap;

    use super::{
        catalog_keys, delete_manual_model_pricing_internal, estimate_event,
        is_excluded_openrouter_model, list_model_pricing_internal, parse_catalog,
        parse_openai_pricing, refresh_missing_model_prices, save_manual_model_pricing_internal,
        set_model_pricing_hidden_internal, store_catalog,
    };
    use crate::db::{
        init_db, model_pricing_cache, seed_builtin_model_pricing, upsert_model_pricing_cache,
        upsert_usage_events,
    };
    use crate::types::{ManualModelPricingInput, ModelPricingCacheRecord, UsageEventRecord};

    #[test]
    #[ignore = "需要連線至 OpenAI 官方價格頁"]
    fn live_official_pricing_sync() {
        let connection = Connection::open_in_memory().expect("open db");
        init_db(&connection).expect("init db");
        let entries =
            super::sync_model_pricing_internal(&connection).expect("sync live official prices");
        assert!(entries
            .iter()
            .any(|entry| entry.source == "openai-official" && entry.status == "success"));
    }

    #[test]
    fn official_sync_discovers_new_models_and_preserves_manual_prices_and_visibility() {
        let body = "### Standard pricing data\n| Model | Short context input | Short context cached input | Short context cache writes | Short context output | Long context input | Long context cached input | Long context cache writes | Long context output |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- |\n| gpt-6.1-sol | $2.00 | $0.10 | $2.50 | $10.00 | $4.00 | $0.20 | $5.00 | $15.00 |\n| gpt-5 | $1.25 | $0.125 | - | $10.00 | - | - | - | - |\n### Batch pricing data\n| gpt-6.1-sol | $1.00 | $0.05 | $1.25 | $5.00 | $2.00 | $0.10 | $2.50 | $7.50 |";
        let catalog = parse_openai_pricing(body, 100).expect("parse official prices");
        assert_eq!(
            catalog["openai/gpt-6.1-sol"].prompt_usd_per_token,
            Some(0.000002)
        );
        assert_eq!(
            catalog["openai/gpt-6.1-sol"].completion_usd_per_token,
            Some(0.00001)
        );
        assert_eq!(
            catalog["openai/gpt-6.1-sol"].cache_read_usd_per_token,
            Some(0.10 / 1_000_000.0)
        );
        assert!(
            parse_openai_pricing("### Standard pricing data\n| Model | Output | Input |", 100)
                .is_err()
        );
        let connection = Connection::open_in_memory().expect("open db");
        init_db(&connection).expect("init db");
        save_manual_model_pricing_internal(
            &connection,
            ManualModelPricingInput {
                provider: "openai".to_string(),
                model: "gpt-5".to_string(),
                prompt_usd_per_million: 9.0,
                completion_usd_per_million: 18.0,
                cache_read_usd_per_million: None,
                cache_write_usd_per_million: None,
            },
        )
        .expect("save manual override");
        store_catalog(&connection, &catalog).expect("sync catalog");
        set_model_pricing_hidden_internal(&connection, "openai", "gpt-6.1-sol", true)
            .expect("hide model");
        store_catalog(&connection, &catalog).expect("resync");
        seed_builtin_model_pricing(&connection).expect("reseed");
        let record = model_pricing_cache(&connection, "openai", "gpt-6.1-sol")
            .expect("read")
            .expect("new model exists");
        assert!(record.hidden);
        assert_eq!(record.source, "openai-official");
        assert_eq!(
            model_pricing_cache(&connection, "openai", "gpt-5")
                .expect("read")
                .expect("override exists")
                .prompt_usd_per_token,
            Some(0.000009)
        );
    }

    #[test]
    fn models_missing_from_catalog_are_discovered_even_when_usage_already_has_cost() {
        let connection = Connection::open_in_memory().expect("open db");
        init_db(&connection).expect("init db");
        upsert_usage_events(
            &connection,
            &[UsageEventRecord {
                provider: "opencode".to_string(),
                model_provider_id: Some("openai".to_string()),
                service_tier: None,
                session_id: "session".to_string(),
                source_event_id: "paid".to_string(),
                occurred_at: "2026-09-30T00:00:00Z".to_string(),
                cwd: None,
                model: Some("gpt-6.1-sol".to_string()),
                input_tokens: Some(100),
                output_tokens: Some(20),
                cache_read_tokens: None,
                cache_write_tokens: None,
                reasoning_tokens: None,
                estimated_usd_micros: Some(400),
                estimated_usd_price_version: Some("provider".to_string()),
                cost_points_micros: None,
                source_kind: "fixture".to_string(),
                parser_version: 1,
            }],
        )
        .expect("insert priced usage");
        let models = crate::db::distinct_unpriced_models(&connection, usize::MAX)
            .expect("find missing models");
        assert!(models.iter().any(|(_, _, model)| model == "gpt-6.1-sol"));
        super::refresh_missing_model_prices_with_catalog(
            &connection,
            |now| {
                Ok(BTreeMap::from([(
                    "openai/gpt-6.1-sol".to_string(),
                    ModelPricingCacheRecord {
                        provider: "openai".to_string(),
                        model: "gpt-6.1-sol".to_string(),
                        status: "success".to_string(),
                        prompt_usd_per_token: Some(0.000002),
                        completion_usd_per_token: Some(0.00001),
                        cache_read_usd_per_token: None,
                        cache_write_usd_per_token: None,
                        source: "openai-official".to_string(),
                        fetched_at: now,
                        expires_at: i64::MAX,
                        error_kind: None,
                        hidden: false,
                        sort_order: now,
                    },
                )]))
            },
            |_| panic!("official model should not require OpenRouter"),
        )
        .expect("automatically discover official model");
        assert!(list_model_pricing_internal(&connection)
            .expect("list prices")
            .iter()
            .any(|entry| entry.model == "gpt-6.1-sol" && entry.source == "openai-official"));
    }

    #[test]
    fn parses_openrouter_per_token_prices_and_rejects_conditional_models() {
        let catalog = parse_catalog(
            json!({"data": [
                {"id": "openai/gpt-test", "expiration_date": null, "pricing": {
                    "prompt": "0.000002", "completion": "0.00001",
                    "input_cache_read": "0.0000002", "input_cache_write": "0.0000025"
                }},
                {"id": "anthropic/conditional", "pricing": {
                    "prompt": "0.000003", "completion": "0.000015",
                    "overrides": [{"min_prompt_tokens": 200000, "prompt": "0.000006"}]
                }},
                {"id": "openai/codex-auto-review", "pricing": {
                    "prompt": "0.000001", "completion": "0.000002"
                }}
            ]}),
            100,
        )
        .expect("catalog should parse");
        let price = &catalog["openai/gpt-test"];
        assert_eq!(price.prompt_usd_per_token, Some(0.000002));
        assert_eq!(price.cache_read_usd_per_token, Some(0.0000002));
        assert_eq!(catalog["anthropic/conditional"].status, "failed");
        assert_eq!(
            catalog["anthropic/conditional"].error_kind.as_deref(),
            Some("conditional_pricing_unsupported")
        );
        assert!(!catalog.contains_key("openai/codex-auto-review"));
        assert!(is_excluded_openrouter_model("OpenAI", "Codex-Auto-Review"));
    }

    #[test]
    fn catalog_keys_support_provider_prefixes_and_anthropic_date_suffixes() {
        assert_eq!(catalog_keys("openai", "openai/gpt-5"), vec!["openai/gpt-5"]);
        assert_eq!(
            catalog_keys("anthropic", "claude-sonnet-4-20250514"),
            vec![
                "anthropic/claude-sonnet-4-20250514",
                "anthropic/claude-sonnet-4"
            ]
        );
    }

    #[test]
    fn estimates_cached_tokens_as_input_subsets() {
        let pricing = ModelPricingCacheRecord {
            provider: "openai".to_string(),
            model: "gpt-test".to_string(),
            status: "success".to_string(),
            prompt_usd_per_token: Some(0.000002),
            completion_usd_per_token: Some(0.00001),
            cache_read_usd_per_token: Some(0.0000002),
            cache_write_usd_per_token: Some(0.0000025),
            source: "fixture".to_string(),
            fetched_at: 1,
            expires_at: 2,
            error_kind: None,
            hidden: false,
            sort_order: 0,
        };
        let event = UsageEventRecord {
            provider: "codex".to_string(),
            model_provider_id: Some("openai".to_string()),
            service_tier: None,
            session_id: "session".to_string(),
            source_event_id: "event".to_string(),
            occurred_at: "2026-09-24T00:00:00Z".to_string(),
            cwd: None,
            model: Some("gpt-test".to_string()),
            input_tokens: Some(100),
            output_tokens: Some(20),
            cache_read_tokens: Some(20),
            cache_write_tokens: Some(10),
            reasoning_tokens: None,
            estimated_usd_micros: None,
            estimated_usd_price_version: None,
            cost_points_micros: None,
            source_kind: "fixture".to_string(),
            parser_version: 1,
        };
        assert_eq!(estimate_event(&event, &pricing), Some(369));
    }

    #[test]
    fn valid_cached_price_updates_unpriced_events_without_remote_lookup() {
        let connection = Connection::open_in_memory().expect("open pricing test db");
        init_db(&connection).expect("initialize pricing test db");
        connection
            .execute(
                "INSERT INTO sessions_cache (session_id, provider, cwd, is_archived)
                 VALUES ('session', 'codex', 'D:/fixture', 0)",
                [],
            )
            .expect("insert session scope");
        let event = UsageEventRecord {
            provider: "codex".to_string(),
            model_provider_id: Some("openai".to_string()),
            service_tier: None,
            session_id: "session".to_string(),
            source_event_id: "event".to_string(),
            occurred_at: "2026-09-24T00:00:00Z".to_string(),
            cwd: Some("D:/fixture".to_string()),
            model: Some("gpt-test".to_string()),
            input_tokens: Some(100),
            output_tokens: Some(20),
            cache_read_tokens: Some(20),
            cache_write_tokens: Some(0),
            reasoning_tokens: None,
            estimated_usd_micros: None,
            estimated_usd_price_version: None,
            cost_points_micros: None,
            source_kind: "fixture".to_string(),
            parser_version: 1,
        };
        upsert_usage_events(&connection, &[event]).expect("insert unpriced event");
        upsert_model_pricing_cache(
            &connection,
            &ModelPricingCacheRecord {
                provider: "openai".to_string(),
                model: "gpt-test".to_string(),
                status: "success".to_string(),
                prompt_usd_per_token: Some(0.000002),
                completion_usd_per_token: Some(0.00001),
                cache_read_usd_per_token: Some(0.0000002),
                cache_write_usd_per_token: None,
                source: "fixture-cache".to_string(),
                fetched_at: i64::MAX - 2,
                expires_at: i64::MAX - 1,
                error_kind: None,
                hidden: false,
                sort_order: 0,
            },
        )
        .expect("insert cached price");

        refresh_missing_model_prices(&connection).expect("apply cached price");

        let estimate: Option<i64> = connection
            .query_row(
                "SELECT estimated_usd_micros FROM usage_events WHERE source_event_id = 'event'",
                [],
                |row| row.get(0),
            )
            .expect("read estimated cost");
        assert_eq!(estimate, Some(364));
    }

    #[test]
    fn gpt_6_astra_codex_usage_with_cached_input_and_cache_write_gets_an_estimate() {
        let connection = Connection::open_in_memory().expect("open pricing test db");
        init_db(&connection).expect("initialize pricing test db");
        connection
            .execute(
                "INSERT INTO sessions_cache (session_id, provider, cwd, is_archived)
             VALUES ('astra-session', 'codex', 'D:/fixture', 0)",
                [],
            )
            .expect("insert Codex session scope");
        upsert_usage_events(
            &connection,
            &[UsageEventRecord {
                provider: "codex".to_string(),
                model_provider_id: Some("openai".to_string()),
                service_tier: None,
                session_id: "astra-session".to_string(),
                source_event_id: "astra-event".to_string(),
                occurred_at: "2026-09-30T00:00:00Z".to_string(),
                cwd: Some("D:/fixture".to_string()),
                model: Some("gpt-6-astra".to_string()),
                input_tokens: Some(100_000),
                output_tokens: Some(10_000),
                cache_read_tokens: Some(20_000),
                cache_write_tokens: Some(30_000),
                reasoning_tokens: None,
                estimated_usd_micros: None,
                estimated_usd_price_version: None,
                cost_points_micros: None,
                source_kind: "codex_token_count".to_string(),
                parser_version: 3,
            }],
        )
        .expect("insert unpriced Astra event");

        refresh_missing_model_prices(&connection).expect("apply seeded Astra price");

        let (estimate, version): (Option<i64>, Option<String>) = connection.query_row(
            "SELECT estimated_usd_micros, estimated_usd_price_version FROM usage_events WHERE source_event_id = 'astra-event'",
            [], |row| Ok((row.get(0)?, row.get(1)?)),
        ).expect("read estimated Astra cost");
        assert_eq!(estimate, Some(1_395_000));
        assert!(version.is_some());
    }

    #[test]
    fn gpt_6_astra_fast_alias_reestimates_previously_zero_cost_events() {
        let connection = Connection::open_in_memory().expect("open pricing test db");
        init_db(&connection).expect("initialize pricing test db");
        upsert_usage_events(
            &connection,
            &[UsageEventRecord {
                provider: "codex".to_string(),
                model_provider_id: Some("openai".to_string()),
                service_tier: None,
                session_id: "astra-fast-session".to_string(),
                source_event_id: "astra-fast-event".to_string(),
                occurred_at: "2026-09-30T00:00:00Z".to_string(),
                cwd: None,
                model: Some("gpt-6-astra-fast".to_string()),
                input_tokens: Some(100_000),
                output_tokens: Some(10_000),
                cache_read_tokens: None,
                cache_write_tokens: None,
                reasoning_tokens: None,
                estimated_usd_micros: Some(0),
                estimated_usd_price_version: Some("stale-zero".to_string()),
                cost_points_micros: None,
                source_kind: "codex_token_count".to_string(),
                parser_version: 3,
            }],
        )
        .expect("insert prior zero estimate");
        super::refresh_missing_model_prices_with_catalog(
            &connection,
            |_| panic!("cached built-in price should not fetch official catalog"),
            |_| panic!("cached built-in price should not fetch remote catalog"),
        )
        .expect("reestimate fast alias from cached Astra price");

        let (estimate, version): (Option<i64>, Option<String>) = connection.query_row(
            "SELECT estimated_usd_micros, estimated_usd_price_version FROM usage_events WHERE source_event_id = 'astra-fast-event'",
            [], |row| Ok((row.get(0)?, row.get(1)?)),
        ).expect("read updated estimate");
        assert_eq!(estimate, Some(3_000_000));
        assert!(version.is_some_and(|value| value.starts_with("builtin-official:")));
    }

    #[test]
    fn codex_auto_review_is_not_sent_to_openrouter_for_pricing() {
        let connection = Connection::open_in_memory().expect("open excluded model pricing db");
        init_db(&connection).expect("initialize excluded model pricing db");
        upsert_usage_events(
            &connection,
            &[UsageEventRecord {
                provider: "codex".to_string(),
                model_provider_id: Some("openai".to_string()),
                service_tier: None,
                session_id: "session".to_string(),
                source_event_id: "auto-review".to_string(),
                occurred_at: "2026-09-24T00:00:00Z".to_string(),
                cwd: None,
                model: Some("codex-auto-review".to_string()),
                input_tokens: Some(10),
                output_tokens: Some(1),
                cache_read_tokens: None,
                cache_write_tokens: None,
                reasoning_tokens: None,
                estimated_usd_micros: None,
                estimated_usd_price_version: None,
                cost_points_micros: None,
                source_kind: "fixture".to_string(),
                parser_version: 1,
            }],
        )
        .expect("insert Codex auto-review event");

        refresh_missing_model_prices(&connection).expect("skip excluded model lookup");

        assert!(
            model_pricing_cache(&connection, "openai", "codex-auto-review")
                .expect("read excluded model cache")
                .is_none()
        );
    }

    #[test]
    fn manual_override_can_be_saved_and_deleted_to_restore_builtin_price() {
        let connection = Connection::open_in_memory().expect("open manual pricing db");
        init_db(&connection).expect("initialize manual pricing db");
        let entries = save_manual_model_pricing_internal(
            &connection,
            ManualModelPricingInput {
                provider: "openai".to_string(),
                model: "gpt-5".to_string(),
                prompt_usd_per_million: 9.0,
                completion_usd_per_million: 18.0,
                cache_read_usd_per_million: None,
                cache_write_usd_per_million: None,
            },
        )
        .expect("save manual override");
        let overridden = entries
            .iter()
            .find(|entry| entry.provider == "openai" && entry.model == "gpt-5")
            .expect("manual price should be listed");
        assert_eq!(overridden.source, "user");
        assert_eq!(overridden.prompt_usd_per_million, Some(9.0));

        delete_manual_model_pricing_internal(&connection, "openai", "gpt-5")
            .expect("delete manual override");
        let restored = list_model_pricing_internal(&connection)
            .expect("list restored prices")
            .into_iter()
            .find(|entry| entry.provider == "openai" && entry.model == "gpt-5")
            .expect("built-in price should be restored");
        assert_eq!(restored.source, "builtin-official");
        assert_eq!(restored.prompt_usd_per_million, Some(1.25));
    }

    #[test]
    fn builtins_are_newest_first_and_visibility_is_persisted() {
        let connection = Connection::open_in_memory().expect("open pricing db");
        init_db(&connection).expect("initialize pricing db");

        let entries = list_model_pricing_internal(&connection).expect("list prices");
        let openai = entries
            .iter()
            .filter(|entry| entry.provider == "openai")
            .collect::<Vec<_>>();
        assert_eq!(
            openai.first().map(|entry| entry.model.as_str()),
            Some("gpt-6-astra")
        );
        assert!(openai.iter().any(|entry| entry.model == "gpt-6-sol"));

        let entries = set_model_pricing_hidden_internal(&connection, "openai", "gpt-5", true)
            .expect("hide model");
        assert!(entries
            .iter()
            .find(|entry| entry.provider == "openai" && entry.model == "gpt-5")
            .is_some_and(|entry| entry.hidden));

        seed_builtin_model_pricing(&connection).expect("reseed prices");
        assert!(list_model_pricing_internal(&connection)
            .expect("list reseeded prices")
            .into_iter()
            .find(|entry| entry.provider == "openai" && entry.model == "gpt-5")
            .is_some_and(|entry| entry.hidden));
    }

    #[test]
    fn builtin_price_replaces_stale_remote_negative_cache() {
        let connection = Connection::open_in_memory().expect("open pricing db");
        init_db(&connection).expect("initialize pricing db");
        connection
            .execute(
                "UPDATE model_pricing_cache SET
                    status = 'not_found',
                    prompt_usd_per_token = NULL,
                    completion_usd_per_token = NULL,
                    cache_read_usd_per_token = NULL,
                    cache_write_usd_per_token = NULL,
                    source = 'openrouter-catalog',
                    error_kind = 'model_not_in_catalog'
                 WHERE provider = 'openai' AND model = 'gpt-6-astra'",
                [],
            )
            .expect("replace builtin with stale cache fixture");

        seed_builtin_model_pricing(&connection).expect("restore official prices");
        let price = model_pricing_cache(&connection, "openai", "gpt-6-astra")
            .expect("read restored price")
            .expect("gpt-6-astra should exist");
        assert_eq!(price.status, "success");
        assert_eq!(price.source, "builtin-official");
        assert_eq!(price.prompt_usd_per_token, Some(10.0 / 1_000_000.0));
        assert_eq!(price.completion_usd_per_token, Some(50.0 / 1_000_000.0));
        assert_eq!(price.cache_read_usd_per_token, Some(1.0 / 1_000_000.0));
        assert_eq!(price.cache_write_usd_per_token, Some(12.5 / 1_000_000.0));
    }
}
