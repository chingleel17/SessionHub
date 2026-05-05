use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rusqlite::{params, Connection};

use tauri::State;

use crate::db::DbState;
use crate::types::{AnalyticsDataPoint, ModelMetricsEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnalyticsGroupBy {
    Day,
    Week,
    Month,
}

impl AnalyticsGroupBy {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "day" => Ok(Self::Day),
            "week" => Ok(Self::Week),
            "month" => Ok(Self::Month),
            _ => Err("invalid groupBy value".to_string()),
        }
    }

    fn label_for_date(self, date: NaiveDate) -> String {
        match self {
            Self::Day => date.format("%Y-%m-%d").to_string(),
            Self::Week => {
                let iso_week = date.iso_week();
                format!("{}-W{:02}", iso_week.year(), iso_week.week())
            }
            Self::Month => date.format("%Y-%m").to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct AnalyticsRow {
    updated_at: String,
    output_tokens: Option<u64>,
    input_tokens: Option<u64>,
    interaction_count: Option<u32>,
    model_metrics: Option<String>,
}

fn parse_date_input(value: &str, field_name: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| format!("invalid date format: {field_name}"))
}

fn date_from_timestamp(value: &str) -> Option<NaiveDate> {
    let date_part = value.get(0..10)?;
    NaiveDate::parse_from_str(date_part, "%Y-%m-%d").ok()
}

fn sum_cost_points(model_metrics_json: Option<&str>) -> f64 {
    let Some(raw) = model_metrics_json else {
        return 0.0;
    };

    serde_json::from_str::<BTreeMap<String, ModelMetricsEntry>>(raw)
        .map(|metrics| metrics.values().map(|entry| entry.requests_cost).sum())
        .unwrap_or(0.0)
}

pub(crate) fn get_analytics_data_internal(
    connection: &Connection,
    cwd: Option<&str>,
    start_date: &str,
    end_date: &str,
    group_by: &str,
) -> Result<Vec<AnalyticsDataPoint>, String> {
    let start = parse_date_input(start_date, "startDate")?;
    let end = parse_date_input(end_date, "endDate")?;
    if start > end {
        return Err("startDate must be before endDate".to_string());
    }
    let group_by = AnalyticsGroupBy::parse(group_by)?;

    let query = "
        SELECT
            sc.updated_at,
            ss.output_tokens,
            ss.input_tokens,
            ss.interaction_count,
            ss.model_metrics
        FROM sessions_cache sc
        LEFT JOIN session_stats ss ON ss.session_id = sc.session_id
        WHERE sc.updated_at IS NOT NULL
          AND date(substr(sc.updated_at, 1, 10)) BETWEEN ?1 AND ?2
          AND (?3 IS NULL OR lower(sc.cwd) = lower(?3))
        ORDER BY sc.updated_at ASC
    ";

    let mut statement = connection
        .prepare(query)
        .map_err(|error| format!("failed to prepare analytics query: {error}"))?;

    let rows = statement
        .query_map(params![start_date, end_date, cwd], |row| {
            Ok(AnalyticsRow {
                updated_at: row.get(0)?,
                output_tokens: row.get(1)?,
                input_tokens: row.get(2)?,
                interaction_count: row.get(3)?,
                model_metrics: row.get(4)?,
            })
        })
        .map_err(|error| format!("failed to run analytics query: {error}"))?;

    let mut grouped = BTreeMap::<String, AnalyticsDataPoint>::new();

    for row in rows {
        let row = row.map_err(|error| format!("failed to read analytics row: {error}"))?;
        let Some(date) = date_from_timestamp(&row.updated_at) else {
            continue;
        };
        let label = group_by.label_for_date(date);
        let point = grouped.entry(label.clone()).or_insert_with(|| AnalyticsDataPoint {
            label,
            output_tokens: 0,
            input_tokens: 0,
            interaction_count: 0,
            cost_points: 0.0,
            session_count: 0,
            missing_count: 0,
        });

        point.session_count += 1;

        if let Some(output_tokens) = row.output_tokens {
            point.output_tokens += output_tokens;
            point.input_tokens += row.input_tokens.unwrap_or(0);
            point.interaction_count += row.interaction_count.unwrap_or(0);
            point.cost_points += sum_cost_points(row.model_metrics.as_deref());
        } else {
            point.missing_count += 1;
        }
    }

    Ok(grouped.into_values().collect())
}

#[tauri::command]
pub fn get_analytics_data(
    cwd: Option<String>,
    start_date: String,
    end_date: String,
    group_by: String,
    db: State<'_, DbState>,
) -> Result<Vec<AnalyticsDataPoint>, String> {
    let conn = db.conn.lock().map_err(|e| format!("db lock poisoned: {e}"))?;
    get_analytics_data_internal(
        &*conn,
        cwd.as_deref().map(str::trim).filter(|value| !value.is_empty()),
        &start_date,
        &end_date,
        &group_by,
    )
}
