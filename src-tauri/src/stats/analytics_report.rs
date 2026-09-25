use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Datelike, Days, NaiveDate, SecondsFormat, Utc};
use rusqlite::{params, Connection};

use crate::types::{
    AnalyticsComparison, AnalyticsComparisonStatus, AnalyticsCoverage, AnalyticsCoverageStatus,
    AnalyticsDetailStatus, AnalyticsMetric, AnalyticsMetricCoverage, AnalyticsMetricValue,
    AnalyticsProviderCoverage, AnalyticsQuery, AnalyticsRankingEntry, AnalyticsReport,
    AnalyticsSeriesPoint, AnalyticsSessionDetail, AnalyticsSessionPage, AnalyticsSessionSort,
    AnalyticsSessionSummaryOnly, AnalyticsValues, UsageSessionSummaryRecord,
};

use super::{
    analytics_bucket_label, local_date_start_utc, query_usage_events, resolve_analytics_date_range,
    AnalyticsEventRow,
};

pub(crate) fn get_analytics_report_internal(
    connection: &Connection,
    query: &AnalyticsQuery,
) -> Result<AnalyticsReport, String> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("failed to begin analytics report snapshot: {error}"))?;
    let range = resolve_analytics_date_range(&query.start_date, &query.end_date, &query.time_zone)?;
    let events = query_usage_events(&transaction, query)?;
    let provider_coverage = query_provider_coverage(&transaction, query)?;
    let is_complete_scope = query.providers.is_empty()
        || (!provider_coverage.is_empty()
            && provider_coverage
                .iter()
                .all(|provider| provider.status == AnalyticsCoverageStatus::Complete));
    let summary = aggregate_values(&events, is_complete_scope);
    let series = build_series(&events, query, range.time_zone, is_complete_scope)?;
    let project_ranking = build_ranking(
        &events,
        |event| Some(event.cwd.as_deref().unwrap_or("unknown")),
        query,
        is_complete_scope,
    );
    let model_ranking = build_ranking(
        &events,
        |event| {
            Some(
                event
                    .model
                    .as_deref()
                    .filter(|model| !model.trim().is_empty())
                    .unwrap_or("unknown"),
            )
        },
        query,
        is_complete_scope,
    );
    let platform_ranking = build_ranking(
        &events,
        |event| Some(event.provider.as_str()),
        query,
        is_complete_scope,
    );
    let supplier_ranking = build_supplier_ranking(&events, query, is_complete_scope);
    let coverage = build_coverage(&events, provider_coverage, is_complete_scope);
    let (_previous_query, previous_events, previous_provider_coverage, previous_complete_scope) =
        previous_period_data(&transaction, query)?;
    let previous_summary = aggregate_values(&previous_events, previous_complete_scope);
    let previous_coverage = build_coverage(
        &previous_events,
        previous_provider_coverage,
        previous_complete_scope,
    );
    let comparison = compare_periods(
        &summary,
        &coverage,
        &previous_summary,
        &previous_coverage,
        query,
    );
    let session_summary_only = query_session_summaries(&transaction, query, &range)?
        .into_iter()
        .map(to_session_summary_only)
        .collect();
    let report = AnalyticsReport {
        summary,
        previous_summary: Some(previous_summary),
        comparison,
        series,
        project_ranking,
        model_ranking,
        platform_ranking,
        supplier_ranking,
        coverage,
        session_summary_only,
        revision: transaction
            .query_row(
                "SELECT revision FROM analytics_revision WHERE singleton = 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|error| format!("failed to read analytics revision: {error}"))?
            .max(0) as u64,
        generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true),
        time_zone: range.time_zone.name().to_string(),
    };
    transaction
        .commit()
        .map_err(|error| format!("failed to commit analytics report snapshot: {error}"))?;
    Ok(report)
}

pub(crate) fn get_analytics_session_page_internal(
    connection: &Connection,
    query: &AnalyticsQuery,
    requested_revision: u64,
    page: u32,
    page_size: u32,
    sort: AnalyticsSessionSort,
) -> Result<AnalyticsSessionPage, String> {
    if page == 0 {
        return Err("page must be at least 1".to_string());
    }
    if !(1..=200).contains(&page_size) {
        return Err("pageSize must be between 1 and 200".to_string());
    }
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("failed to begin analytics detail snapshot: {error}"))?;
    let revision = transaction
        .query_row(
            "SELECT revision FROM analytics_revision WHERE singleton = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| format!("failed to read analytics revision: {error}"))?
        .max(0) as u64;
    if revision != requested_revision {
        let page = AnalyticsSessionPage {
            status: AnalyticsDetailStatus::StaleRevision,
            revision,
            page: 1,
            page_size,
            total_count: 0,
            items: Vec::new(),
        };
        transaction
            .commit()
            .map_err(|error| format!("failed to close stale detail snapshot: {error}"))?;
        return Ok(page);
    }

    let events = query_usage_events(&transaction, query)?;
    let range = resolve_analytics_date_range(&query.start_date, &query.end_date, &query.time_zone)?;
    let summaries = query_session_summaries(&transaction, query, &range)?;
    let mut summaries_by_session =
        BTreeMap::<(String, String), Vec<UsageSessionSummaryRecord>>::new();
    for summary in summaries {
        summaries_by_session
            .entry((summary.provider.clone(), summary.session_id.clone()))
            .or_default()
            .push(summary);
    }
    let mut sessions = BTreeMap::<(String, String), Vec<&AnalyticsEventRow>>::new();
    for event in &events {
        sessions
            .entry((event.provider.clone(), event.session_id.clone()))
            .or_default()
            .push(event);
    }
    let mut items = sessions
        .into_iter()
        .map(|((provider, session_id), session_events)| {
            let mut timestamps = session_events
                .iter()
                .map(|event| event.occurred_at.as_str())
                .collect::<Vec<_>>();
            timestamps.sort_unstable();
            let values = aggregate_values_ref(&session_events, false);
            let has_missing = [
                &values.total_tokens,
                &values.input_tokens,
                &values.output_tokens,
                &values.estimated_usd,
                &values.copilot_points,
            ]
            .iter()
            .any(|value| value.missing_field_event_count > 0);
            AnalyticsSessionDetail {
                provider: provider.clone(),
                model_provider_ids: session_events
                    .iter()
                    .filter_map(|event| event.model_provider_id.clone())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect(),
                session_id: session_id.clone(),
                cwd: session_events.first().and_then(|event| event.cwd.clone()),
                model: session_events
                    .first()
                    .and_then(|event| event.model.clone())
                    .or_else(|| Some("unknown".to_string())),
                first_event_at: timestamps.first().map(|value| (*value).to_string()),
                last_event_at: timestamps.last().map(|value| (*value).to_string()),
                values,
                session_summary_only: summaries_by_session
                    .remove(&(provider.clone(), session_id.clone()))
                    .map(|rows| aggregate_summary_values(&rows)),
                status: if has_missing {
                    AnalyticsCoverageStatus::Partial
                } else {
                    AnalyticsCoverageStatus::Complete
                },
            }
        })
        .collect::<Vec<_>>();
    match sort {
        AnalyticsSessionSort::EventTimeDesc => items.sort_by(|left, right| {
            right
                .last_event_at
                .cmp(&left.last_event_at)
                .then_with(|| left.provider.cmp(&right.provider))
                .then_with(|| left.session_id.cmp(&right.session_id))
        }),
    }
    let total_count = items.len() as u64;
    let start = (page as usize - 1).saturating_mul(page_size as usize);
    let items = items
        .into_iter()
        .skip(start)
        .take(page_size as usize)
        .collect();
    let page = AnalyticsSessionPage {
        status: AnalyticsDetailStatus::Ready,
        revision,
        page,
        page_size,
        total_count,
        items,
    };
    transaction
        .commit()
        .map_err(|error| format!("failed to close analytics detail snapshot: {error}"))?;
    Ok(page)
}

fn aggregate_summary_values(summaries: &[UsageSessionSummaryRecord]) -> AnalyticsValues {
    let sum_optional = |values: Vec<Option<i64>>| {
        let eligible = values.iter().filter_map(|value| *value).collect::<Vec<_>>();
        AnalyticsMetricValue {
            known_value: if eligible.is_empty() {
                None
            } else {
                Some(
                    eligible
                        .iter()
                        .map(|value| i128::from(*value))
                        .sum::<i128>() as f64,
                )
            },
            eligible_event_count: eligible.len() as u64,
            missing_field_event_count: (values.len() - eligible.len()) as u64,
            price_versions: Vec::new(),
        }
    };
    let token_pair_values = summaries
        .iter()
        .map(|summary| {
            summary
                .input_tokens
                .zip(summary.output_tokens)
                .and_then(|(input, output)| input.checked_add(output))
        })
        .collect();
    AnalyticsValues {
        total_tokens: sum_optional(token_pair_values),
        input_tokens: sum_optional(
            summaries
                .iter()
                .map(|summary| summary.input_tokens)
                .collect(),
        ),
        output_tokens: sum_optional(
            summaries
                .iter()
                .map(|summary| summary.output_tokens)
                .collect(),
        ),
        cache_read_tokens: sum_optional(vec![None; summaries.len()]),
        cache_write_tokens: sum_optional(vec![None; summaries.len()]),
        reasoning_tokens: sum_optional(vec![None; summaries.len()]),
        estimated_usd: sum_optional(
            summaries
                .iter()
                .map(|summary| summary.estimated_usd_micros)
                .collect(),
        ),
        copilot_points: sum_optional(
            summaries
                .iter()
                .map(|summary| summary.cost_points_micros)
                .collect(),
        ),
    }
}

fn previous_period_data(
    connection: &Connection,
    query: &AnalyticsQuery,
) -> Result<
    (
        AnalyticsQuery,
        Vec<AnalyticsEventRow>,
        Vec<AnalyticsProviderCoverage>,
        bool,
    ),
    String,
> {
    let start = NaiveDate::parse_from_str(&query.start_date, "%Y-%m-%d")
        .map_err(|_| "invalid date format: startDate".to_string())?;
    let end = NaiveDate::parse_from_str(&query.end_date, "%Y-%m-%d")
        .map_err(|_| "invalid date format: endDate".to_string())?;
    let days = end
        .signed_duration_since(start)
        .num_days()
        .checked_add(1)
        .filter(|days| *days > 0)
        .ok_or_else(|| "invalid analytics date range".to_string())? as u64;
    let previous_end = start
        .checked_sub_days(Days::new(1))
        .ok_or_else(|| "previous analytics range is outside supported dates".to_string())?;
    let previous_start = previous_end
        .checked_sub_days(Days::new(days - 1))
        .ok_or_else(|| "previous analytics range is outside supported dates".to_string())?;
    let mut previous_query = query.clone();
    previous_query.start_date = previous_start.format("%Y-%m-%d").to_string();
    previous_query.end_date = previous_end.format("%Y-%m-%d").to_string();
    let events = query_usage_events(connection, &previous_query)?;
    let providers = query_provider_coverage(connection, &previous_query)?;
    let complete_scope = previous_query.providers.is_empty()
        || (!providers.is_empty()
            && providers
                .iter()
                .all(|provider| provider.status == AnalyticsCoverageStatus::Complete));
    Ok((previous_query, events, providers, complete_scope))
}

fn compare_periods(
    current: &AnalyticsValues,
    current_coverage: &AnalyticsCoverage,
    previous: &AnalyticsValues,
    previous_coverage: &AnalyticsCoverage,
    query: &AnalyticsQuery,
) -> AnalyticsComparison {
    let metric_key = match query.metric {
        AnalyticsMetric::Tokens => match query.token_field {
            crate::types::AnalyticsTokenField::Total => "totalTokens",
            crate::types::AnalyticsTokenField::Input => "inputTokens",
            crate::types::AnalyticsTokenField::Output => "outputTokens",
        },
        AnalyticsMetric::EstimatedUsd => "estimatedUsd",
        AnalyticsMetric::CopilotPoints => "copilotPoints",
    };
    let current_value = selected_value(current, query);
    let previous_value = selected_value(previous, query);
    let current_status = current_coverage
        .metrics
        .get(metric_key)
        .map(|coverage| &coverage.status);
    let previous_status = previous_coverage
        .metrics
        .get(metric_key)
        .map(|coverage| &coverage.status);
    if current_status != Some(&AnalyticsCoverageStatus::Complete)
        || previous_status != Some(&AnalyticsCoverageStatus::Complete)
    {
        return AnalyticsComparison {
            status: AnalyticsComparisonStatus::NotComparable,
            percent_change: None,
            reason: Some("One or both periods have incomplete metric coverage".to_string()),
        };
    }
    if query.metric == AnalyticsMetric::EstimatedUsd
        && current.estimated_usd.price_versions != previous.estimated_usd.price_versions
    {
        return AnalyticsComparison {
            status: AnalyticsComparisonStatus::NotComparable,
            percent_change: None,
            reason: Some("USD price versions differ between periods".to_string()),
        };
    }
    let (Some(current_value), Some(previous_value)) = (current_value, previous_value) else {
        return AnalyticsComparison {
            status: AnalyticsComparisonStatus::NotComparable,
            percent_change: None,
            reason: Some("The selected metric is unknown for one or both periods".to_string()),
        };
    };
    if previous_value == 0.0 && current_value == 0.0 {
        return AnalyticsComparison {
            status: AnalyticsComparisonStatus::NoChange,
            percent_change: None,
            reason: None,
        };
    }
    if previous_value == 0.0 {
        return AnalyticsComparison {
            status: AnalyticsComparisonStatus::NewUsage,
            percent_change: None,
            reason: None,
        };
    }
    AnalyticsComparison {
        status: AnalyticsComparisonStatus::Comparable,
        percent_change: Some((current_value - previous_value) / previous_value * 100.0),
        reason: None,
    }
}

fn selected_value(values: &AnalyticsValues, query: &AnalyticsQuery) -> Option<f64> {
    match query.metric {
        AnalyticsMetric::Tokens => match query.token_field {
            crate::types::AnalyticsTokenField::Total => values.total_tokens.known_value,
            crate::types::AnalyticsTokenField::Input => values.input_tokens.known_value,
            crate::types::AnalyticsTokenField::Output => values.output_tokens.known_value,
        },
        AnalyticsMetric::EstimatedUsd => values.estimated_usd.known_value,
        AnalyticsMetric::CopilotPoints => values.copilot_points.known_value,
    }
}

fn build_series(
    events: &[AnalyticsEventRow],
    query: &AnalyticsQuery,
    time_zone: chrono_tz::Tz,
    empty_scope_complete: bool,
) -> Result<Vec<AnalyticsSeriesPoint>, String> {
    let mut buckets = BTreeMap::<String, Vec<&AnalyticsEventRow>>::new();
    let bucket_bounds = build_bucket_bounds(query, time_zone)?;
    for event in events {
        let timestamp = DateTime::parse_from_rfc3339(&event.occurred_at)
            .map_err(|error| format!("invalid indexed event timestamp: {error}"))?
            .with_timezone(&Utc);
        let label =
            analytics_bucket_label(timestamp, time_zone, &group_by_string(&query.group_by))?;
        buckets.entry(label).or_default().push(event);
    }
    if empty_scope_complete {
        for label in bucket_bounds.keys() {
            buckets.entry(label.clone()).or_default();
        }
    }

    buckets
        .into_iter()
        .map(|(label, bucket_events)| {
            let (start_at, end_at) = bucket_bounds.get(&label).cloned().unwrap_or_default();
            let values = aggregate_values_ref(&bucket_events, empty_scope_complete);
            let interaction_count = bucket_events
                .iter()
                .filter(|event| event.source_kind != "copilot_ai_credit_checkpoint")
                .count() as u64;
            let session_count = distinct_session_count(&bucket_events);
            Ok(AnalyticsSeriesPoint {
                label,
                start_at,
                end_at,
                values,
                interaction_count,
                session_count,
            })
        })
        .collect()
}

fn build_bucket_bounds(
    query: &AnalyticsQuery,
    time_zone: chrono_tz::Tz,
) -> Result<BTreeMap<String, (String, String)>, String> {
    let start_date = NaiveDate::parse_from_str(&query.start_date, "%Y-%m-%d")
        .map_err(|_| "invalid date format: startDate".to_string())?;
    let end_date = NaiveDate::parse_from_str(&query.end_date, "%Y-%m-%d")
        .map_err(|_| "invalid date format: endDate".to_string())?;
    let range = resolve_analytics_date_range(&query.start_date, &query.end_date, &query.time_zone)?;
    let mut output = BTreeMap::new();
    let mut cursor = start_date;
    while cursor <= end_date {
        let (bucket_start, bucket_end) = match query.group_by {
            crate::types::AnalyticsGroupBy::Day => (
                cursor,
                cursor
                    .checked_add_days(Days::new(1))
                    .ok_or_else(|| "date range is outside the supported range".to_string())?,
            ),
            crate::types::AnalyticsGroupBy::Week => {
                let monday = cursor
                    .checked_sub_days(Days::new(cursor.weekday().num_days_from_monday() as u64))
                    .ok_or_else(|| "week range is outside the supported range".to_string())?;
                (
                    monday,
                    monday
                        .checked_add_days(Days::new(7))
                        .ok_or_else(|| "week range is outside the supported range".to_string())?,
                )
            }
            crate::types::AnalyticsGroupBy::Month => {
                let month_start = cursor
                    .with_day(1)
                    .ok_or_else(|| "month range is outside the supported range".to_string())?;
                let next_month = if month_start.month() == 12 {
                    month_start
                        .with_year(month_start.year() + 1)
                        .and_then(|date| date.with_month(1))
                } else {
                    month_start.with_month(month_start.month() + 1)
                }
                .ok_or_else(|| "month range is outside the supported range".to_string())?;
                (month_start, next_month)
            }
        };
        let label = analytics_bucket_label(
            local_date_start_utc(bucket_start, time_zone)?,
            time_zone,
            &group_by_string(&query.group_by),
        )?;
        let start = local_date_start_utc(bucket_start, time_zone)?.max(range.start_utc);
        let end = local_date_start_utc(bucket_end, time_zone)?.min(range.end_utc);
        output.insert(
            label,
            (
                start.to_rfc3339_opts(SecondsFormat::Nanos, true),
                end.to_rfc3339_opts(SecondsFormat::Nanos, true),
            ),
        );
        cursor = bucket_end;
    }
    Ok(output)
}

fn build_ranking<'a, F>(
    events: &'a [AnalyticsEventRow],
    key_for: F,
    query: &AnalyticsQuery,
    empty_scope_complete: bool,
) -> Vec<AnalyticsRankingEntry>
where
    F: Fn(&'a AnalyticsEventRow) -> Option<&'a str>,
{
    let mut groups = BTreeMap::<&'a str, Vec<&'a AnalyticsEventRow>>::new();
    for event in events {
        if let Some(key) = key_for(event) {
            groups.entry(key).or_default().push(event);
        }
    }
    let mut ranking = groups
        .into_iter()
        .map(|(key, items)| {
            let value = aggregate_selected_metric(&items, query, empty_scope_complete);
            AnalyticsRankingEntry {
                label: key.to_string(),
                key: key.to_string(),
                value,
                session_count: distinct_session_count(&items),
                share_of_known_value: None,
            }
        })
        .collect::<Vec<_>>();
    let known_total = ranking
        .iter()
        .filter_map(|entry| entry.value.known_value)
        .sum::<f64>();
    if known_total > 0.0 {
        for entry in &mut ranking {
            entry.share_of_known_value = entry.value.known_value.map(|value| value / known_total);
        }
    }
    ranking.sort_by(|left, right| {
        let left_value = left.value.known_value.unwrap_or(f64::NEG_INFINITY);
        let right_value = right.value.known_value.unwrap_or(f64::NEG_INFINITY);
        right_value
            .partial_cmp(&left_value)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.key.cmp(&right.key))
    });
    ranking
}

fn build_supplier_ranking(
    events: &[AnalyticsEventRow],
    query: &AnalyticsQuery,
    empty_scope_complete: bool,
) -> Vec<AnalyticsRankingEntry> {
    build_ranking(
        events,
        |event| match event.provider.as_str() {
            "claude" => Some("claude"),
            "codex" => Some(event.model_provider_id.as_deref().unwrap_or("unknown")),
            "opencode" => Some(event.model_provider_id.as_deref().unwrap_or("unknown")),
            _ => None,
        },
        query,
        empty_scope_complete,
    )
}

fn aggregate_values(events: &[AnalyticsEventRow], empty_scope_complete: bool) -> AnalyticsValues {
    let refs = events.iter().collect::<Vec<_>>();
    aggregate_values_ref(&refs, empty_scope_complete)
}

fn aggregate_values_ref(
    events: &[&AnalyticsEventRow],
    empty_scope_complete: bool,
) -> AnalyticsValues {
    AnalyticsValues {
        total_tokens: metric_value(
            events,
            |event| Some(event.input_tokens? as f64 + event.output_tokens? as f64),
            empty_scope_complete,
        ),
        input_tokens: metric_value(
            events,
            |event| event.input_tokens.map(|value| value as f64),
            empty_scope_complete,
        ),
        output_tokens: metric_value(
            events,
            |event| event.output_tokens.map(|value| value as f64),
            empty_scope_complete,
        ),
        cache_read_tokens: metric_value(
            events,
            |event| event.cache_read_tokens.map(|value| value as f64),
            empty_scope_complete,
        ),
        cache_write_tokens: metric_value(
            events,
            |event| event.cache_write_tokens.map(|value| value as f64),
            empty_scope_complete,
        ),
        reasoning_tokens: metric_value(
            events,
            |event| event.reasoning_tokens.map(|value| value as f64),
            empty_scope_complete,
        ),
        estimated_usd: metric_micros_value(
            events,
            |event| event.estimated_usd_micros,
            empty_scope_complete,
            true,
        ),
        copilot_points: metric_micros_value(
            events,
            |event| event.cost_points_micros,
            empty_scope_complete,
            false,
        ),
    }
}

fn metric_value<F>(
    events: &[&AnalyticsEventRow],
    value_for: F,
    empty_scope_complete: bool,
) -> AnalyticsMetricValue
where
    F: Fn(&AnalyticsEventRow) -> Option<f64>,
{
    let mut known_value = 0.0;
    let mut eligible_event_count = 0_u64;
    for event in events {
        if let Some(value) = value_for(event) {
            known_value += value;
            eligible_event_count += 1;
        }
    }
    AnalyticsMetricValue {
        known_value: if eligible_event_count > 0 {
            Some(known_value)
        } else if events.is_empty() && empty_scope_complete {
            Some(0.0)
        } else {
            None
        },
        eligible_event_count,
        missing_field_event_count: (events.len() as u64).saturating_sub(eligible_event_count),
        price_versions: Vec::new(),
    }
}

fn metric_micros_value<F>(
    events: &[&AnalyticsEventRow],
    micros_for: F,
    empty_scope_complete: bool,
    include_price_versions: bool,
) -> AnalyticsMetricValue
where
    F: Fn(&AnalyticsEventRow) -> Option<i64>,
{
    let mut total_micros = 0_i128;
    let mut eligible_event_count = 0_u64;
    let mut price_versions = BTreeSet::new();
    for event in events {
        if let Some(micros) = micros_for(event) {
            total_micros += i128::from(micros);
            eligible_event_count += 1;
            if include_price_versions {
                if let Some(version) = event.estimated_usd_price_version.as_ref() {
                    price_versions.insert(version.clone());
                }
            }
        }
    }
    AnalyticsMetricValue {
        known_value: if eligible_event_count > 0 {
            Some(total_micros as f64 / 1_000_000.0)
        } else if events.is_empty() && empty_scope_complete {
            Some(0.0)
        } else {
            None
        },
        eligible_event_count,
        missing_field_event_count: (events.len() as u64).saturating_sub(eligible_event_count),
        price_versions: price_versions.into_iter().collect(),
    }
}

fn aggregate_selected_metric(
    events: &[&AnalyticsEventRow],
    query: &AnalyticsQuery,
    empty_scope_complete: bool,
) -> AnalyticsMetricValue {
    match query.metric {
        AnalyticsMetric::Tokens => metric_value(
            events,
            |event| match query.token_field {
                crate::types::AnalyticsTokenField::Total => {
                    Some(event.input_tokens? as f64 + event.output_tokens? as f64)
                }
                crate::types::AnalyticsTokenField::Input => {
                    event.input_tokens.map(|value| value as f64)
                }
                crate::types::AnalyticsTokenField::Output => {
                    event.output_tokens.map(|value| value as f64)
                }
            },
            empty_scope_complete,
        ),
        AnalyticsMetric::EstimatedUsd => metric_micros_value(
            events,
            |event| event.estimated_usd_micros,
            empty_scope_complete,
            true,
        ),
        AnalyticsMetric::CopilotPoints => metric_micros_value(
            events,
            |event| event.cost_points_micros,
            empty_scope_complete,
            false,
        ),
    }
}

fn distinct_session_count(events: &[&AnalyticsEventRow]) -> u64 {
    events
        .iter()
        .map(|event| (event.provider.as_str(), event.session_id.as_str()))
        .collect::<BTreeSet<_>>()
        .len() as u64
}

fn build_coverage(
    events: &[AnalyticsEventRow],
    providers: Vec<AnalyticsProviderCoverage>,
    empty_scope_complete: bool,
) -> AnalyticsCoverage {
    let event_refs = events.iter().collect::<Vec<_>>();
    let values = aggregate_values_ref(&event_refs, empty_scope_complete);
    let mut metrics = BTreeMap::new();
    for (name, value) in [
        ("totalTokens", values.total_tokens),
        ("inputTokens", values.input_tokens),
        ("outputTokens", values.output_tokens),
        ("cacheReadTokens", values.cache_read_tokens),
        ("cacheWriteTokens", values.cache_write_tokens),
        ("reasoningTokens", values.reasoning_tokens),
        ("estimatedUsd", values.estimated_usd),
        ("copilotPoints", values.copilot_points),
    ] {
        let status = if value.missing_field_event_count > 0 {
            AnalyticsCoverageStatus::Partial
        } else if providers.iter().any(|provider| {
            matches!(
                provider.status,
                AnalyticsCoverageStatus::Pending
                    | AnalyticsCoverageStatus::Unsupported
                    | AnalyticsCoverageStatus::Error
                    | AnalyticsCoverageStatus::SummaryOnly
            )
        }) {
            AnalyticsCoverageStatus::Partial
        } else {
            AnalyticsCoverageStatus::Complete
        };
        metrics.insert(
            name.to_string(),
            AnalyticsMetricCoverage {
                eligible_event_count: value.eligible_event_count,
                missing_field_event_count: value.missing_field_event_count,
                status,
            },
        );
    }
    AnalyticsCoverage { metrics, providers }
}

fn query_provider_coverage(
    connection: &Connection,
    query: &AnalyticsQuery,
) -> Result<Vec<AnalyticsProviderCoverage>, String> {
    let providers_json = serde_json::to_string(&query.providers)
        .map_err(|error| format!("failed to serialize analytics providers: {error}"))?;
    let cwds_json = serde_json::to_string(&query.cwds)
        .map_err(|error| format!("failed to serialize analytics cwd filters: {error}"))?;
    let models_json = serde_json::to_string(&query.models)
        .map_err(|error| format!("failed to serialize analytics model filters: {error}"))?;
    let mut statement = connection
        .prepare(
            "
            SELECT sc.provider, COALESCE(state.ingestion_status, 'pending'), COUNT(*),
                   SUM(CASE
                       WHEN COALESCE(state.ingestion_status, 'pending') IN ('pending', 'error', 'unsupported') THEN 1
                       WHEN state.ingestion_status = 'summary_only' AND NOT EXISTS (
                           SELECT 1 FROM usage_session_summaries summary
                           WHERE summary.provider = sc.provider
                             AND summary.session_id = sc.session_id
                             AND summary.active_from IS NOT NULL
                             AND summary.active_until IS NOT NULL
                       ) THEN 1
                       ELSE 0
                   END)
            FROM sessions_cache sc
            LEFT JOIN usage_ingestion_state state
              ON state.provider = sc.provider AND state.session_id = sc.session_id
            WHERE EXISTS (
                SELECT 1 FROM json_each(?1) selected_provider
                WHERE selected_provider.value = sc.provider
            )
              AND (?2 IS NULL OR lower(COALESCE(sc.repo_root, sc.cwd)) = lower(?2)
                   OR (?2 = 'unknown' AND COALESCE(sc.repo_root, sc.cwd) IS NULL)
                   OR lower(sc.cwd) = lower(?2) OR lower(sc.repo_root) = lower(?2))
               AND (
                   json_array_length(?3) = 0
                   OR EXISTS (
                       SELECT 1 FROM usage_events e, json_each(?3) selected_cwd
                       WHERE e.provider = sc.provider AND e.session_id = sc.session_id
                         AND (
                           (selected_cwd.value = 'unknown' AND COALESCE(e.cwd, sc.repo_root, sc.cwd) IS NULL)
                           OR lower(COALESCE(e.cwd, sc.repo_root, sc.cwd)) = lower(selected_cwd.value)
                           OR lower(sc.cwd) = lower(selected_cwd.value)
                           OR lower(sc.repo_root) = lower(selected_cwd.value)
                         )
                   )
               )
               AND (
                   json_array_length(?4) = 0
                   OR EXISTS (
                       SELECT 1 FROM usage_events e, json_each(?4) selected_model
                       WHERE e.provider = sc.provider AND e.session_id = sc.session_id
                         AND COALESCE(e.model, 'unknown') = selected_model.value
                   )
               )
               AND (?5 = 1 OR sc.is_archived = 0)
            GROUP BY sc.provider, COALESCE(state.ingestion_status, 'pending')
            ",
        )
        .map_err(|error| format!("failed to prepare provider coverage query: {error}"))?;
    let rows = statement
        .query_map(
            params![
                providers_json,
                query.cwd,
                cwds_json,
                models_json,
                if query.include_archived { 1 } else { 0 }
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?.max(0) as u64,
                    row.get::<_, Option<i64>>(3)?.unwrap_or(0).max(0) as u64,
                ))
            },
        )
        .map_err(|error| format!("failed to query provider coverage: {error}"))?;
    let mut by_provider = BTreeMap::<String, AnalyticsProviderCoverage>::new();
    for row in rows {
        let (provider, state, count, period_unknown_count) =
            row.map_err(|error| format!("failed to read provider coverage row: {error}"))?;
        let coverage =
            by_provider
                .entry(provider.clone())
                .or_insert_with(|| AnalyticsProviderCoverage {
                    provider,
                    status: AnalyticsCoverageStatus::Complete,
                    complete_session_count: 0,
                    partial_session_count: 0,
                    pending_session_count: 0,
                    unsupported_session_count: 0,
                    error_session_count: 0,
                    summary_only_session_count: 0,
                    period_unknown_session_count: 0,
                });
        match state.as_str() {
            "complete" => coverage.complete_session_count += count,
            "partial" => coverage.partial_session_count += count,
            "summary_only" => coverage.summary_only_session_count += count,
            "unsupported" => coverage.unsupported_session_count += count,
            "error" => coverage.error_session_count += count,
            _ => coverage.pending_session_count += count,
        }
        coverage.period_unknown_session_count += period_unknown_count;
    }
    for provider in by_provider.values_mut() {
        provider.status = if provider.error_session_count > 0 {
            AnalyticsCoverageStatus::Error
        } else if provider.pending_session_count > 0 {
            AnalyticsCoverageStatus::Pending
        } else if provider.unsupported_session_count > 0
            && provider.complete_session_count == 0
            && provider.partial_session_count == 0
            && provider.summary_only_session_count == 0
        {
            AnalyticsCoverageStatus::Unsupported
        } else if provider.partial_session_count > 0 || provider.unsupported_session_count > 0 {
            AnalyticsCoverageStatus::Partial
        } else if provider.summary_only_session_count > 0 && provider.complete_session_count == 0 {
            AnalyticsCoverageStatus::SummaryOnly
        } else {
            AnalyticsCoverageStatus::Complete
        };
    }
    Ok(by_provider.into_values().collect())
}

fn query_session_summaries(
    connection: &Connection,
    query: &AnalyticsQuery,
    range: &super::AnalyticsDateRange,
) -> Result<Vec<UsageSessionSummaryRecord>, String> {
    let providers_json = serde_json::to_string(&query.providers)
        .map_err(|error| format!("failed to serialize analytics providers: {error}"))?;
    let cwds_json = serde_json::to_string(&query.cwds)
        .map_err(|error| format!("failed to serialize analytics cwd filters: {error}"))?;
    let models_json = serde_json::to_string(&query.models)
        .map_err(|error| format!("failed to serialize analytics model filters: {error}"))?;
    let start_utc = range.start_utc.to_rfc3339_opts(SecondsFormat::Nanos, true);
    let end_utc = range.end_utc.to_rfc3339_opts(SecondsFormat::Nanos, true);
    let mut statement = connection
        .prepare(
            "
            SELECT summary.provider, summary.session_id, summary.source_identity,
                   COALESCE(summary.cwd, sc.repo_root, sc.cwd), summary.model,
                   summary.active_from, summary.active_until, summary.input_tokens,
                   summary.output_tokens, summary.estimated_usd_micros,
                   summary.cost_points_micros, summary.source_kind, summary.parser_version,
                   summary.integrity_status
            FROM usage_session_summaries summary
            JOIN sessions_cache sc
              ON sc.provider = summary.provider AND sc.session_id = summary.session_id
            WHERE EXISTS (
                SELECT 1 FROM json_each(?1) selected_provider
                WHERE selected_provider.value = summary.provider
            )
              AND (?2 IS NULL OR lower(COALESCE(summary.cwd, sc.repo_root, sc.cwd)) = lower(?2)
                   OR (?2 = 'unknown' AND COALESCE(summary.cwd, sc.repo_root, sc.cwd) IS NULL)
                   OR lower(sc.cwd) = lower(?2) OR lower(sc.repo_root) = lower(?2))
              AND (?3 IS NULL OR COALESCE(summary.model, 'unknown') = ?3)
               AND (
                   json_array_length(?4) = 0
                   OR EXISTS (
                       SELECT 1 FROM json_each(?4) selected_cwd
                       WHERE (selected_cwd.value = 'unknown' AND COALESCE(summary.cwd, sc.repo_root, sc.cwd) IS NULL)
                          OR lower(COALESCE(summary.cwd, sc.repo_root, sc.cwd)) = lower(selected_cwd.value)
                   )
               )
               AND (
                   json_array_length(?5) = 0
                   OR EXISTS (
                       SELECT 1 FROM json_each(?5) selected_model
                       WHERE COALESCE(summary.model, 'unknown') = selected_model.value
                   )
               )
               AND (?6 = 1 OR sc.is_archived = 0)
               AND (summary.active_from IS NULL OR summary.active_until IS NULL
                    OR (summary.active_until >= ?7 AND summary.active_from < ?8))
            ORDER BY summary.provider, summary.session_id, summary.source_identity
            ",
        )
        .map_err(|error| format!("failed to prepare summary-only query: {error}"))?;
    let rows = statement
        .query_map(
            params![
                providers_json,
                query.cwd,
                query.model,
                cwds_json,
                models_json,
                if query.include_archived { 1 } else { 0 },
                start_utc,
                end_utc,
            ],
            |row| {
                Ok(UsageSessionSummaryRecord {
                    provider: row.get(0)?,
                    session_id: row.get(1)?,
                    source_identity: row.get(2)?,
                    cwd: row.get(3)?,
                    model: row.get(4)?,
                    active_from: row.get(5)?,
                    active_until: row.get(6)?,
                    input_tokens: row.get(7)?,
                    output_tokens: row.get(8)?,
                    estimated_usd_micros: row.get(9)?,
                    cost_points_micros: row.get(10)?,
                    source_kind: row.get(11)?,
                    parser_version: row.get(12)?,
                    integrity_status: row.get(13)?,
                })
            },
        )
        .map_err(|error| format!("failed to query session summaries: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read session summary row: {error}"))?;
    Ok(rows)
}

fn to_session_summary_only(summary: UsageSessionSummaryRecord) -> AnalyticsSessionSummaryOnly {
    let input = summary.input_tokens.map(|value| value as f64);
    let output = summary.output_tokens.map(|value| value as f64);
    let metric = |value: Option<f64>, price_versions: Vec<String>| AnalyticsMetricValue {
        known_value: value,
        eligible_event_count: u64::from(value.is_some()),
        missing_field_event_count: u64::from(value.is_none()),
        price_versions,
    };
    let values = AnalyticsValues {
        total_tokens: metric(
            input.zip(output).map(|(input, output)| input + output),
            Vec::new(),
        ),
        input_tokens: metric(input, Vec::new()),
        output_tokens: metric(output, Vec::new()),
        cache_read_tokens: metric(None, Vec::new()),
        cache_write_tokens: metric(None, Vec::new()),
        reasoning_tokens: metric(None, Vec::new()),
        estimated_usd: metric(
            summary
                .estimated_usd_micros
                .map(|value| value as f64 / 1_000_000.0),
            Vec::new(),
        ),
        copilot_points: metric(
            summary
                .cost_points_micros
                .map(|value| value as f64 / 1_000_000.0),
            Vec::new(),
        ),
    };
    AnalyticsSessionSummaryOnly {
        provider: summary.provider,
        session_id: summary.session_id,
        cwd: summary.cwd,
        model: summary.model,
        active_from: summary.active_from,
        active_until: summary.active_until,
        values,
        status: match summary.integrity_status.as_str() {
            "unsupported" => AnalyticsCoverageStatus::Unsupported,
            "error" => AnalyticsCoverageStatus::Error,
            "pending" => AnalyticsCoverageStatus::Pending,
            _ => AnalyticsCoverageStatus::SummaryOnly,
        },
    }
}

fn group_by_string(group_by: &crate::types::AnalyticsGroupBy) -> String {
    match group_by {
        crate::types::AnalyticsGroupBy::Day => "day",
        crate::types::AnalyticsGroupBy::Week => "week",
        crate::types::AnalyticsGroupBy::Month => "month",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::db::init_db;
    use crate::types::{
        AnalyticsBreakdown, AnalyticsComparisonStatus, AnalyticsDetailStatus, AnalyticsMetric,
        AnalyticsQuery, AnalyticsSessionSort, AnalyticsTokenField, UsageEventRecord,
    };

    use super::{
        build_supplier_ranking, get_analytics_report_internal, get_analytics_session_page_internal,
    };

    fn supplier_ranking_event(
        provider: &str,
        model_provider_id: Option<&str>,
        output_tokens: i64,
    ) -> super::AnalyticsEventRow {
        super::AnalyticsEventRow {
            provider: provider.to_string(),
            model_provider_id: model_provider_id.map(str::to_string),
            session_id: format!("{provider}-{model_provider_id:?}"),
            occurred_at: "2026-04-10T01:00:00Z".to_string(),
            cwd: None,
            model: Some("fixture-model".to_string()),
            input_tokens: Some(10),
            output_tokens: Some(output_tokens),
            cache_read_tokens: None,
            cache_write_tokens: None,
            reasoning_tokens: None,
            estimated_usd_micros: None,
            estimated_usd_price_version: None,
            cost_points_micros: None,
            source_kind: "fixture".to_string(),
        }
    }

    #[test]
    fn report_uses_one_revision_and_keeps_summary_only_usage_separate() {
        let connection = rusqlite::Connection::open_in_memory().expect("open test db");
        init_db(&connection).expect("initialize test db");
        connection
            .execute(
                "INSERT INTO sessions_cache (session_id, provider, cwd, is_archived)
                 VALUES ('copilot-session', 'copilot', 'D:/repo', 0)",
                [],
            )
            .expect("insert session scope");
        connection
            .execute(
                "INSERT INTO usage_ingestion_state (
                    provider, session_id, source_identity, parser_version,
                    integrity_status, ingestion_status
                 ) VALUES ('copilot', 'copilot-session', 'root/session', 1, 'complete', 'complete')",
                [],
            )
            .expect("insert provider coverage");
        let event = UsageEventRecord {
            provider: "copilot".to_string(),
            model_provider_id: None,
            service_tier: None,
            session_id: "copilot-session".to_string(),
            source_event_id: "output-1".to_string(),
            occurred_at: "2026-04-10T01:00:00.000000000Z".to_string(),
            cwd: Some("D:/repo".to_string()),
            model: Some("fixture-model".to_string()),
            input_tokens: None,
            output_tokens: Some(80),
            cache_read_tokens: None,
            cache_write_tokens: None,
            reasoning_tokens: None,
            estimated_usd_micros: None,
            estimated_usd_price_version: None,
            cost_points_micros: None,
            source_kind: "copilot_assistant_output".to_string(),
            parser_version: 1,
        };
        let mut unknown_model_event = event.clone();
        unknown_model_event.source_event_id = "output-2".to_string();
        unknown_model_event.model = None;
        unknown_model_event.output_tokens = Some(20);
        unknown_model_event.model_provider_id = Some("ollama".to_string());
        let mut event = event;
        event.model_provider_id = Some("openai".to_string());
        let mut previous_event = event.clone();
        previous_event.source_event_id = "previous-output".to_string();
        previous_event.occurred_at = "2026-04-09T01:00:00.000000000Z".to_string();
        previous_event.output_tokens = Some(50);
        crate::db::upsert_usage_events(&connection, &[event, unknown_model_event, previous_event])
            .expect("insert events");
        connection
            .execute(
                "INSERT INTO usage_session_summaries (
                    provider, session_id, source_identity, cwd, model, active_from, active_until,
                    input_tokens, output_tokens, estimated_usd_micros, cost_points_micros,
                    source_kind, parser_version, integrity_status
                 ) VALUES ('copilot', 'copilot-session', 'shutdown:fixture-model', 'D:/repo',
                    'fixture-model', '2026-04-10T00:00:00.000000000Z',
                    '2026-04-10T02:00:00.000000000Z', 900, 700, NULL, 250000,
                    'copilot_shutdown_summary', 1, 'summary_only')",
                [],
            )
            .expect("insert summary-only usage");
        let query = AnalyticsQuery {
            start_date: "2026-04-10".to_string(),
            end_date: "2026-04-10".to_string(),
            time_zone: "Asia/Taipei".to_string(),
            group_by: crate::types::AnalyticsGroupBy::Day,
            providers: vec!["copilot".to_string()],
            cwd: None,
            model: None,
            cwds: Vec::new(),
            models: Vec::new(),
            include_archived: false,
            metric: AnalyticsMetric::Tokens,
            token_field: AnalyticsTokenField::Output,
            breakdown: AnalyticsBreakdown::Total,
        };

        let report = get_analytics_report_internal(&connection, &query).expect("build report");

        assert_eq!(report.revision, 1);
        assert_eq!(report.summary.output_tokens.known_value, Some(100.0));
        assert_eq!(report.summary.input_tokens.known_value, None);
        assert_eq!(report.summary.total_tokens.known_value, None);
        assert_eq!(
            report
                .previous_summary
                .as_ref()
                .unwrap()
                .output_tokens
                .known_value,
            Some(50.0)
        );
        assert_eq!(
            report.comparison.status,
            AnalyticsComparisonStatus::Comparable
        );
        assert_eq!(report.comparison.percent_change, Some(100.0));
        assert_eq!(report.series.len(), 1);
        assert_eq!(report.series[0].session_count, 1);
        assert_eq!(report.series[0].interaction_count, 2);
        assert_eq!(
            report.coverage.metrics["totalTokens"].eligible_event_count,
            0
        );
        assert_eq!(
            report.coverage.metrics["totalTokens"].missing_field_event_count,
            2
        );
        assert_eq!(
            report.coverage.metrics["outputTokens"].eligible_event_count,
            2
        );
        assert_eq!(report.coverage.providers[0].period_unknown_session_count, 0);
        assert_eq!(report.model_ranking.len(), 2);
        assert_eq!(report.model_ranking[0].label, "fixture-model");
        assert_eq!(report.platform_ranking[0].key, "copilot");
        assert!(report.supplier_ranking.is_empty());
        assert_eq!(report.model_ranking[0].share_of_known_value, Some(0.8));
        assert_eq!(report.model_ranking[1].share_of_known_value, Some(0.2));
        assert_eq!(report.session_summary_only.len(), 1);
        assert_eq!(
            report.session_summary_only[0]
                .values
                .total_tokens
                .known_value,
            Some(1600.0)
        );
        assert_eq!(
            report.session_summary_only[0]
                .values
                .copilot_points
                .known_value,
            Some(0.25)
        );
        assert_eq!(
            report.session_summary_only[0]
                .values
                .estimated_usd
                .known_value,
            None
        );
        assert!(report.generated_at <= Utc::now().to_rfc3339());

        connection
            .execute(
                "DELETE FROM usage_events WHERE source_event_id = 'previous-output'",
                [],
            )
            .expect("remove previous-period usage");
        let zero_baseline =
            get_analytics_report_internal(&connection, &query).expect("build zero-baseline report");
        assert_eq!(
            zero_baseline.comparison.status,
            AnalyticsComparisonStatus::NewUsage
        );
        assert_eq!(zero_baseline.comparison.percent_change, None);

        connection
            .execute("DELETE FROM usage_events", [])
            .expect("remove all event usage");
        let all_zero =
            get_analytics_report_internal(&connection, &query).expect("build all-zero report");
        assert_eq!(all_zero.summary.output_tokens.known_value, Some(0.0));
        assert_eq!(
            all_zero.previous_summary.unwrap().output_tokens.known_value,
            Some(0.0)
        );
        assert_eq!(
            all_zero.comparison.status,
            AnalyticsComparisonStatus::NoChange
        );
    }

    #[test]
    fn supplier_ranking_labels_claude_and_preserves_opencode_unknowns() {
        let query = AnalyticsQuery {
            start_date: "2026-04-10".to_string(),
            end_date: "2026-04-10".to_string(),
            time_zone: "UTC".to_string(),
            group_by: crate::types::AnalyticsGroupBy::Day,
            providers: Vec::new(),
            cwd: None,
            model: None,
            cwds: Vec::new(),
            models: Vec::new(),
            include_archived: false,
            metric: AnalyticsMetric::Tokens,
            token_field: AnalyticsTokenField::Output,
            breakdown: AnalyticsBreakdown::Total,
        };
        let events = vec![
            supplier_ranking_event("opencode", Some("openai"), 100),
            supplier_ranking_event("opencode", Some("ollama"), 40),
            supplier_ranking_event("opencode", None, 60),
            supplier_ranking_event("codex", Some("openai"), 20),
            supplier_ranking_event("claude", None, 30),
            supplier_ranking_event("copilot", None, 1000),
        ];

        let ranking = build_supplier_ranking(&events, &query, false);

        assert_eq!(ranking.len(), 4);
        assert_eq!(ranking[0].key, "openai");
        assert_eq!(ranking[0].value.known_value, Some(120.0));
        assert_eq!(ranking[0].session_count, 2);
        assert_eq!(ranking[1].key, "unknown");
        assert_eq!(ranking[1].value.known_value, Some(60.0));
        assert_eq!(ranking[2].key, "ollama");
        assert_eq!(ranking[2].value.known_value, Some(40.0));
        assert_eq!(ranking[3].key, "claude");
        assert_eq!(ranking[3].value.known_value, Some(30.0));
    }

    #[test]
    fn complete_empty_day_bucket_is_zero_but_unindexed_scope_is_unknown() {
        let connection = rusqlite::Connection::open_in_memory().expect("open test db");
        init_db(&connection).expect("initialize test db");
        connection
            .execute(
                "INSERT INTO sessions_cache (session_id, provider, cwd, is_archived)
                 VALUES ('empty-session', 'claude', 'D:/repo', 0)",
                [],
            )
            .expect("insert empty session scope");
        connection
            .execute(
                "INSERT INTO usage_ingestion_state (
                    provider, session_id, source_identity, parser_version,
                    integrity_status, ingestion_status
                 ) VALUES ('claude', 'empty-session', 'root/session.jsonl', 1, 'complete', 'complete')",
                [],
            )
            .expect("mark scope complete");
        let query = AnalyticsQuery {
            start_date: "2026-04-10".to_string(),
            end_date: "2026-04-10".to_string(),
            time_zone: "Asia/Taipei".to_string(),
            group_by: crate::types::AnalyticsGroupBy::Day,
            providers: vec!["claude".to_string()],
            cwd: None,
            model: None,
            cwds: Vec::new(),
            models: Vec::new(),
            include_archived: false,
            metric: AnalyticsMetric::Tokens,
            token_field: AnalyticsTokenField::Total,
            breakdown: AnalyticsBreakdown::Total,
        };
        let complete_report = get_analytics_report_internal(&connection, &query)
            .expect("build complete empty report");
        assert_eq!(complete_report.summary.total_tokens.known_value, Some(0.0));
        assert_eq!(complete_report.series.len(), 1);
        assert_eq!(complete_report.series[0].label, "2026-04-10");
        assert_eq!(
            complete_report.series[0].values.total_tokens.known_value,
            Some(0.0)
        );

        connection
            .execute(
                "DELETE FROM usage_ingestion_state WHERE provider = 'claude' AND session_id = 'empty-session'",
                [],
            )
            .expect("remove ingestion coverage");
        let unindexed_report =
            get_analytics_report_internal(&connection, &query).expect("build unindexed report");
        assert_eq!(unindexed_report.summary.total_tokens.known_value, None);
        assert!(unindexed_report.series.is_empty());
    }

    #[test]
    fn comparison_rejects_mixed_historical_price_versions() {
        let connection = rusqlite::Connection::open_in_memory().expect("open test db");
        init_db(&connection).expect("initialize test db");
        connection
            .execute(
                "INSERT INTO sessions_cache (session_id, provider, cwd, is_archived)
                 VALUES ('claude-session', 'claude', 'D:/repo', 0)",
                [],
            )
            .expect("insert session scope");
        connection
            .execute(
                "INSERT INTO usage_ingestion_state (
                    provider, session_id, source_identity, parser_version,
                    integrity_status, ingestion_status
                 ) VALUES ('claude', 'claude-session', 'root/session.jsonl', 1, 'complete', 'complete')",
                [],
            )
            .expect("insert complete provider state");
        let current = UsageEventRecord {
            provider: "claude".to_string(),
            model_provider_id: None,
            service_tier: None,
            session_id: "claude-session".to_string(),
            source_event_id: "current-usd".to_string(),
            occurred_at: "2026-04-10T01:00:00.000000000Z".to_string(),
            cwd: Some("D:/repo".to_string()),
            model: Some("claude-sonnet-4".to_string()),
            input_tokens: Some(0),
            output_tokens: Some(0),
            cache_read_tokens: None,
            cache_write_tokens: None,
            reasoning_tokens: None,
            estimated_usd_micros: Some(1_234_567),
            estimated_usd_price_version: Some("price-v2".to_string()),
            cost_points_micros: None,
            source_kind: "fixture".to_string(),
            parser_version: 1,
        };
        let mut previous = current.clone();
        previous.source_event_id = "previous-usd".to_string();
        previous.occurred_at = "2026-04-09T01:00:00.000000000Z".to_string();
        previous.estimated_usd_micros = Some(1_000_001);
        previous.estimated_usd_price_version = Some("price-v1".to_string());
        crate::db::upsert_usage_events(&connection, &[current, previous])
            .expect("insert cost events");
        let query = AnalyticsQuery {
            start_date: "2026-04-10".to_string(),
            end_date: "2026-04-10".to_string(),
            time_zone: "Asia/Taipei".to_string(),
            group_by: crate::types::AnalyticsGroupBy::Day,
            providers: vec!["claude".to_string()],
            cwd: None,
            model: None,
            cwds: Vec::new(),
            models: Vec::new(),
            include_archived: false,
            metric: AnalyticsMetric::EstimatedUsd,
            token_field: AnalyticsTokenField::Total,
            breakdown: AnalyticsBreakdown::Total,
        };

        let report = get_analytics_report_internal(&connection, &query)
            .expect("build mixed price version report");
        assert_eq!(report.summary.estimated_usd.known_value, Some(1.234567));
        assert_eq!(
            report.comparison.status,
            AnalyticsComparisonStatus::NotComparable
        );
        assert_eq!(report.comparison.percent_change, None);
    }

    #[test]
    fn session_detail_pages_are_stable_and_reject_stale_revisions() {
        let connection = rusqlite::Connection::open_in_memory().expect("open test db");
        init_db(&connection).expect("initialize test db");
        connection
            .execute_batch(
                "INSERT INTO sessions_cache (session_id, provider, cwd, is_archived) VALUES
                    ('same-id', 'claude', 'D:/repo', 0),
                    ('same-id', 'codex', 'D:/repo', 0),
                    ('older-session', 'claude', 'D:/repo', 0);",
            )
            .expect("insert sessions");
        let make_event = |provider: &str, session_id: &str, event_id: &str, occurred_at: &str| {
            UsageEventRecord {
                provider: provider.to_string(),
                model_provider_id: None,
                service_tier: None,
                session_id: session_id.to_string(),
                source_event_id: event_id.to_string(),
                occurred_at: occurred_at.to_string(),
                cwd: Some("D:/repo".to_string()),
                model: Some("fixture-model".to_string()),
                input_tokens: Some(10),
                output_tokens: Some(5),
                cache_read_tokens: None,
                cache_write_tokens: None,
                reasoning_tokens: None,
                estimated_usd_micros: None,
                estimated_usd_price_version: None,
                cost_points_micros: None,
                source_kind: "fixture".to_string(),
                parser_version: 1,
            }
        };
        crate::db::upsert_usage_events(
            &connection,
            &[
                make_event(
                    "claude",
                    "same-id",
                    "claude-event",
                    "2026-04-10T01:00:00.000000000Z",
                ),
                make_event(
                    "codex",
                    "same-id",
                    "codex-event",
                    "2026-04-10T01:00:00.000000000Z",
                ),
                make_event(
                    "claude",
                    "older-session",
                    "older-event",
                    "2026-04-10T00:00:00.000000000Z",
                ),
            ],
        )
        .expect("insert detail events");
        let query = AnalyticsQuery {
            start_date: "2026-04-10".to_string(),
            end_date: "2026-04-10".to_string(),
            time_zone: "Asia/Taipei".to_string(),
            group_by: crate::types::AnalyticsGroupBy::Day,
            providers: vec!["claude".to_string(), "codex".to_string()],
            cwd: None,
            model: None,
            cwds: Vec::new(),
            models: Vec::new(),
            include_archived: false,
            metric: AnalyticsMetric::Tokens,
            token_field: AnalyticsTokenField::Total,
            breakdown: AnalyticsBreakdown::Total,
        };
        let report =
            get_analytics_report_internal(&connection, &query).expect("build report snapshot");
        let page_one = get_analytics_session_page_internal(
            &connection,
            &query,
            report.revision,
            1,
            1,
            AnalyticsSessionSort::EventTimeDesc,
        )
        .expect("query first page");
        let page_two = get_analytics_session_page_internal(
            &connection,
            &query,
            report.revision,
            2,
            1,
            AnalyticsSessionSort::EventTimeDesc,
        )
        .expect("query second page");
        let page_three = get_analytics_session_page_internal(
            &connection,
            &query,
            report.revision,
            3,
            1,
            AnalyticsSessionSort::EventTimeDesc,
        )
        .expect("query third page");
        assert_eq!(page_one.total_count, 3);
        assert_eq!(page_one.items[0].provider, "claude");
        assert_eq!(page_two.items[0].provider, "codex");
        assert_eq!(page_three.items[0].session_id, "older-session");
        assert_eq!(page_one.status, AnalyticsDetailStatus::Ready);

        let stale = get_analytics_session_page_internal(
            &connection,
            &query,
            report.revision.saturating_sub(1),
            3,
            1,
            AnalyticsSessionSort::EventTimeDesc,
        )
        .expect("stale detail request should return status");
        assert_eq!(stale.status, AnalyticsDetailStatus::StaleRevision);
        assert_eq!(stale.page, 1);
        assert!(stale.items.is_empty());
        assert!(get_analytics_session_page_internal(
            &connection,
            &query,
            report.revision,
            1,
            201,
            AnalyticsSessionSort::EventTimeDesc,
        )
        .is_err());
    }

    #[test]
    #[ignore = "benchmark workload: run explicitly with --ignored --nocapture"]
    fn performance_100k_events_10k_sessions_warm_report_p95_under_500ms() {
        use std::fs;
        use std::time::Instant;
        use std::time::{SystemTime, UNIX_EPOCH};

        use rusqlite::{params, TransactionBehavior};

        use crate::types::{AnalyticsBreakdown, AnalyticsMetric, AnalyticsTokenField, SessionInfo};

        const SESSION_COUNT: usize = 10_000;
        const EVENTS_PER_SESSION: usize = 10;

        let benchmark_root = std::env::temp_dir().join(format!(
            "session-hub-analytics-perf-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be valid")
                .as_nanos()
        ));
        fs::create_dir_all(&benchmark_root).expect("create benchmark root");
        let mut connection =
            rusqlite::Connection::open_in_memory().expect("open benchmark database");
        init_db(&connection).expect("initialize benchmark db");
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .expect("begin benchmark transaction");
        {
            let mut session_insert = transaction
                .prepare(
                    "INSERT INTO sessions_cache (
                        session_id, provider, cwd, repo_root, is_archived, session_dir
                    ) VALUES (?1, 'copilot', ?2, ?2, 0, ?3)",
                )
                .expect("prepare benchmark sessions");
            let mut state_insert = transaction
                .prepare(
                    "INSERT INTO usage_ingestion_state (
                        provider, session_id, source_identity, parser_version,
                        integrity_status, ingestion_status
                    ) VALUES ('copilot', ?1, ?2, 1, 'complete', 'complete')",
                )
                .expect("prepare benchmark ingestion state");
            for session_index in 0..SESSION_COUNT {
                let session_id = format!("session-{session_index:05}");
                let cwd = format!("D:/projects/project-{session_index:05}");
                session_insert
                    .execute(params![
                        session_id,
                        cwd,
                        format!("D:/sessions/{session_id}")
                    ])
                    .expect("insert benchmark session");
                state_insert
                    .execute(params![session_id, format!("D:/sessions/{session_id}")])
                    .expect("insert complete state");
            }
        }
        {
            let mut event_insert = transaction
                .prepare(
                    "INSERT INTO usage_events (
                        provider, session_id, source_event_id, occurred_at, cwd, model,
                        input_tokens, output_tokens, cache_read_tokens, cache_write_tokens,
                        reasoning_tokens, estimated_usd_micros, estimated_usd_price_version,
                        cost_points_micros, source_kind, parser_version
                    ) VALUES ('copilot', ?1, ?2, ?3, ?4, 'fixture-model', 100, 50, NULL, NULL,
                        NULL, NULL, NULL, NULL, 'benchmark', 1)",
                )
                .expect("prepare benchmark events");
            for event_index in 0..(SESSION_COUNT * EVENTS_PER_SESSION) {
                let session_index = event_index / EVENTS_PER_SESSION;
                let day = event_index % 30 + 1;
                let session_id = format!("session-{session_index:05}");
                let cwd = format!("D:/projects/project-{session_index:05}");
                let occurred_at = format!("2026-04-{day:02}T12:00:00.000000000Z");
                event_insert
                    .execute(params![
                        session_id,
                        format!("event-{event_index:06}"),
                        occurred_at,
                        cwd
                    ])
                    .expect("insert benchmark event");
            }
        }
        transaction
            .execute(
                "UPDATE analytics_revision SET revision = 1 WHERE singleton = 1",
                [],
            )
            .expect("set benchmark revision");
        transaction.commit().expect("commit benchmark data");

        let query = AnalyticsQuery {
            start_date: "2026-04-01".to_string(),
            end_date: "2026-04-30".to_string(),
            time_zone: "UTC".to_string(),
            group_by: crate::types::AnalyticsGroupBy::Day,
            providers: vec!["copilot".to_string()],
            cwd: None,
            model: None,
            cwds: Vec::new(),
            models: Vec::new(),
            include_archived: false,
            metric: AnalyticsMetric::Tokens,
            token_field: AnalyticsTokenField::Total,
            breakdown: AnalyticsBreakdown::Total,
        };
        let mut query_plan = connection
            .prepare(
                "EXPLAIN QUERY PLAN
                 SELECT e.provider, e.session_id, e.source_event_id
                 FROM usage_events e JOIN sessions_cache sc
                   ON sc.provider = e.provider AND sc.session_id = e.session_id
                 WHERE e.occurred_at >= ?1 AND e.occurred_at < ?2
                   AND e.provider = 'copilot' AND sc.is_archived = 0",
            )
            .expect("prepare analytics query plan");
        let query_plan_rows = query_plan
            .query_map(
                [
                    "2026-04-01T00:00:00.000000000Z",
                    "2026-05-01T00:00:00.000000000Z",
                ],
                |row| row.get::<_, String>(3),
            )
            .expect("run query plan")
            .collect::<Result<Vec<_>, _>>()
            .expect("read query plan");
        drop(query_plan);

        let mut baseline_durations = Vec::with_capacity(20);
        for _ in 0..20 {
            let started = Instant::now();
            get_analytics_report_internal(&connection, &query).expect("run baseline report query");
            baseline_durations.push(started.elapsed());
        }
        baseline_durations.sort_unstable();
        let baseline_p95 = baseline_durations[18];

        let fixture_path = benchmark_root.join("copilot");
        let mut backfill_sessions = Vec::with_capacity(50);
        for session_index in 0..50 {
            let session_id = format!("session-{session_index:05}");
            let session_dir = fixture_path.join(&session_id);
            fs::create_dir_all(&session_dir).expect("create backfill session directory");
            fs::write(
                session_dir.join("events.jsonl"),
                "{\"type\":\"assistant.message\",\"timestamp\":\"2026-04-30T12:00:00Z\",\"data\":{\"outputTokens\":50}}\n",
            )
            .expect("write backfill event fixture");
            backfill_sessions.push(SessionInfo {
                id: session_id,
                provider: "copilot".to_string(),
                cwd: Some(format!("D:/projects/project-{session_index:05}")),
                repo_root: None,
                repo_name: None,
                git_branch: None,
                summary: None,
                summary_count: None,
                created_at: None,
                updated_at: None,
                session_dir: session_dir.to_string_lossy().to_string(),
                parse_error: false,
                is_archived: false,
                notes: None,
                tags: Vec::new(),
                has_plan: false,
                has_events: true,
            });
        }
        let backfill_handle = std::thread::spawn(move || {
            let connection = rusqlite::Connection::open_in_memory()
                .expect("open separate background indexing database");
            init_db(&connection).expect("initialize background indexing database");
            crate::stats::index_usage_batch_internal(&connection, &backfill_sessions)
                .expect("run background index batch")
        });
        let mut warm_query_durations = Vec::with_capacity(20);
        for _ in 0..20 {
            let started = Instant::now();
            get_analytics_report_internal(&connection, &query).expect("run warm report query");
            warm_query_durations.push(started.elapsed());
        }
        let backfilled_sessions = backfill_handle.join().expect("join backfill worker");
        warm_query_durations.sort_unstable();
        let p95 = warm_query_durations[18];
        eprintln!(
            "analytics benchmark: storage=in_memory, os={}, arch={}, debug_assertions={}, sessions={}, events={}, warm_runs=20, baseline_p95={:?}, concurrent_backfill_p95={:?}, backfilled_sessions={}, query_plan={:?}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            cfg!(debug_assertions),
            SESSION_COUNT,
            SESSION_COUNT * EVENTS_PER_SESSION,
            baseline_p95,
            p95,
            backfilled_sessions,
            query_plan_rows,
        );
        assert!(
            baseline_p95.as_millis() < 500,
            "warm analytics baseline p95 exceeded 500ms: {baseline_p95:?}"
        );
        assert_eq!(backfilled_sessions, 50);
        drop(connection);
        fs::remove_dir_all(benchmark_root).ok();
    }
}
