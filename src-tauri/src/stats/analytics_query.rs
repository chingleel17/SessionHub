use chrono::SecondsFormat;
use rusqlite::{params_from_iter, types::Value, Connection};

use crate::types::AnalyticsQuery;

use super::resolve_analytics_date_range;

#[derive(Debug, Clone)]
pub(crate) struct AnalyticsEventRow {
    pub(crate) provider: String,
    pub(crate) model_provider_id: Option<String>,
    pub(crate) session_id: String,
    pub(crate) occurred_at: String,
    pub(crate) cwd: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) input_tokens: Option<i64>,
    pub(crate) output_tokens: Option<i64>,
    pub(crate) cache_read_tokens: Option<i64>,
    pub(crate) cache_write_tokens: Option<i64>,
    pub(crate) reasoning_tokens: Option<i64>,
    pub(crate) estimated_usd_micros: Option<i64>,
    pub(crate) estimated_usd_price_version: Option<String>,
    pub(crate) cost_points_micros: Option<i64>,
    pub(crate) source_kind: String,
}

pub(crate) fn query_usage_events(
    connection: &Connection,
    query: &AnalyticsQuery,
) -> Result<Vec<AnalyticsEventRow>, String> {
    let range = resolve_analytics_date_range(&query.start_date, &query.end_date, &query.time_zone)?;
    let start_utc = range.start_utc.to_rfc3339_opts(SecondsFormat::Nanos, true);
    let end_utc = range.end_utc.to_rfc3339_opts(SecondsFormat::Nanos, true);
    let mut sql_values = vec![Value::Text(start_utc), Value::Text(end_utc)];
    let provider_filter = if query.providers.is_empty() {
        "0".to_string()
    } else {
        let placeholders = query
            .providers
            .iter()
            .map(|provider| {
                sql_values.push(Value::Text(provider.clone()));
                format!("?{}", sql_values.len())
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("e.provider IN ({placeholders})")
    };
    let cwd_parameter = sql_values.len() + 1;
    sql_values.push(query.cwd.clone().map(Value::Text).unwrap_or(Value::Null));
    let model_parameter = sql_values.len() + 1;
    sql_values.push(query.model.clone().map(Value::Text).unwrap_or(Value::Null));
    let cwd_list_filter = if query.cwds.is_empty() {
        "1".to_string()
    } else {
        let filters = query
            .cwds
            .iter()
            .map(|cwd| {
                sql_values.push(Value::Text(cwd.clone()));
                let parameter = sql_values.len();
                format!(
                    "((?{parameter} = 'unknown' AND COALESCE(e.cwd, sc.repo_root, sc.cwd) IS NULL)
                      OR lower(COALESCE(e.cwd, sc.repo_root, sc.cwd)) = lower(?{parameter})
                      OR lower(sc.cwd) = lower(?{parameter})
                      OR lower(sc.repo_root) = lower(?{parameter}))"
                )
            })
            .collect::<Vec<_>>()
            .join(" OR ");
        format!("({filters})")
    };
    let model_list_filter = if query.models.is_empty() {
        "1".to_string()
    } else {
        let filters = query
            .models
            .iter()
            .map(|model| {
                sql_values.push(Value::Text(model.clone()));
                format!("COALESCE(e.model, 'unknown') = ?{}", sql_values.len())
            })
            .collect::<Vec<_>>()
            .join(" OR ");
        format!("({filters})")
    };
    let archived_parameter = sql_values.len() + 1;
    sql_values.push(Value::Integer(if query.include_archived { 1 } else { 0 }));

    let sql = format!(
        "
            SELECT e.provider, e.model_provider_id, e.session_id, e.occurred_at,
                   COALESCE(e.cwd, sc.repo_root, sc.cwd), COALESCE(e.model, 'unknown'),
                   e.input_tokens, e.output_tokens, e.cache_read_tokens,
                   e.cache_write_tokens, e.reasoning_tokens, e.estimated_usd_micros,
                   e.estimated_usd_price_version, e.cost_points_micros, e.source_kind
            FROM usage_events e
            JOIN sessions_cache sc
              ON sc.provider = e.provider AND sc.session_id = e.session_id
            WHERE e.occurred_at >= ?1 AND e.occurred_at < ?2
              AND ({provider_filter})
              AND (
                  ?{cwd_parameter} IS NULL
                  OR (?{cwd_parameter} = 'unknown' AND COALESCE(e.cwd, sc.repo_root, sc.cwd) IS NULL)
                  OR lower(COALESCE(e.cwd, sc.repo_root, sc.cwd)) = lower(?{cwd_parameter})
                  OR lower(sc.cwd) = lower(?{cwd_parameter})
                  OR lower(sc.repo_root) = lower(?{cwd_parameter})
              )
              AND (?{model_parameter} IS NULL OR COALESCE(e.model, 'unknown') = ?{model_parameter})
              AND {cwd_list_filter}
              AND {model_list_filter}
              AND (?{archived_parameter} = 1 OR sc.is_archived = 0)
            ORDER BY e.occurred_at ASC, e.provider ASC, e.session_id ASC
            "
    );
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| format!("failed to prepare usage analytics query: {error}"))?;
    let events = statement
        .query_map(params_from_iter(sql_values.iter()), |row| {
            Ok(AnalyticsEventRow {
                provider: row.get(0)?,
                model_provider_id: row.get(1)?,
                session_id: row.get(2)?,
                occurred_at: row.get(3)?,
                cwd: row.get(4)?,
                model: row.get(5)?,
                input_tokens: row.get(6)?,
                output_tokens: row.get(7)?,
                cache_read_tokens: row.get(8)?,
                cache_write_tokens: row.get(9)?,
                reasoning_tokens: row.get(10)?,
                estimated_usd_micros: row.get(11)?,
                estimated_usd_price_version: row.get(12)?,
                cost_points_micros: row.get(13)?,
                source_kind: row.get(14)?,
            })
        })
        .map_err(|error| format!("failed to run usage analytics query: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read usage analytics row: {error}"))?;
    Ok(events)
}

#[cfg(test)]
mod tests {
    use chrono::{SecondsFormat, TimeZone, Utc};
    use rusqlite::Connection;

    use crate::db::init_db;
    use crate::types::{
        AnalyticsBreakdown, AnalyticsMetric, AnalyticsQuery, AnalyticsTokenField, UsageEventRecord,
    };

    use super::query_usage_events;

    fn query(
        providers: Vec<&str>,
        cwd: Option<&str>,
        model: Option<&str>,
        include_archived: bool,
    ) -> AnalyticsQuery {
        AnalyticsQuery {
            start_date: "2026-04-10".to_string(),
            end_date: "2026-04-10".to_string(),
            time_zone: "Asia/Taipei".to_string(),
            group_by: crate::types::AnalyticsGroupBy::Day,
            providers: providers.into_iter().map(str::to_string).collect(),
            cwd: cwd.map(str::to_string),
            model: model.map(str::to_string),
            cwds: Vec::new(),
            models: Vec::new(),
            include_archived,
            metric: AnalyticsMetric::Tokens,
            token_field: AnalyticsTokenField::Total,
            breakdown: AnalyticsBreakdown::Total,
        }
    }

    fn event(
        provider: &str,
        session_id: &str,
        event_id: &str,
        model: Option<&str>,
        occurred_at: &str,
    ) -> UsageEventRecord {
        UsageEventRecord {
            provider: provider.to_string(),
            model_provider_id: None,
            service_tier: None,
            session_id: session_id.to_string(),
            source_event_id: event_id.to_string(),
            occurred_at: occurred_at.to_string(),
            cwd: None,
            model: model.map(str::to_string),
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
    }

    #[test]
    fn applies_provider_cwd_model_archive_and_half_open_date_filters() {
        let connection = Connection::open_in_memory().expect("open test database");
        init_db(&connection).expect("initialize test database");
        connection
            .execute_batch(
                "INSERT INTO sessions_cache (session_id, provider, cwd, repo_root, git_branch, is_archived)
                 VALUES
                    ('same-id', 'claude', 'D:/repo', 'D:/repo', 'main', 0),
                    ('archived-id', 'claude', 'D:/repo', 'D:/repo', 'feature', 1),
                    ('same-id', 'codex', 'D:/repo', 'D:/repo', 'main', 0),
                    ('other-id', 'claude', 'D:/other', 'D:/other', 'main', 0);",
            )
            .expect("seed scoped sessions");
        let inside_start = Utc
            .with_ymd_and_hms(2026, 4, 9, 16, 0, 0)
            .single()
            .expect("range start")
            .to_rfc3339_opts(SecondsFormat::Nanos, true);
        let inside_end = Utc
            .with_ymd_and_hms(2026, 4, 10, 15, 59, 59)
            .single()
            .expect("last second in range")
            .to_rfc3339_opts(SecondsFormat::Nanos, true);
        let at_end = Utc
            .with_ymd_and_hms(2026, 4, 10, 16, 0, 0)
            .single()
            .expect("exclusive end")
            .to_rfc3339_opts(SecondsFormat::Nanos, true);
        let events = [
            event(
                "claude",
                "same-id",
                "unknown-model-event",
                None,
                &inside_start,
            ),
            event(
                "claude",
                "same-id",
                "outside-event",
                Some("sonnet"),
                &at_end,
            ),
            event(
                "claude",
                "archived-id",
                "archived-event",
                Some("sonnet"),
                &inside_end,
            ),
            event("codex", "same-id", "codex-event", Some("gpt"), &inside_end),
            event(
                "claude",
                "other-id",
                "other-model-event",
                Some("sonnet"),
                &inside_end,
            ),
        ];
        crate::db::upsert_usage_events(&connection, &events).expect("insert usage fixtures");

        assert!(
            query_usage_events(&connection, &query(Vec::new(), None, None, true))
                .expect("empty provider selection")
                .is_empty()
        );
        let active_claude = query_usage_events(
            &connection,
            &query(vec!["claude"], Some("D:/repo"), Some("unknown"), false),
        )
        .expect("query unknown model in active Claude sessions");
        assert_eq!(active_claude.len(), 1);
        assert_eq!(active_claude[0].session_id, "same-id");
        assert_eq!(active_claude[0].model.as_deref(), Some("unknown"));

        let active_codex = query_usage_events(
            &connection,
            &query(vec!["codex"], Some("D:/repo"), None, false),
        )
        .expect("query Codex provider independently");
        assert_eq!(active_codex.len(), 1);
        assert_eq!(active_codex[0].provider, "codex");

        let mut multi_select_query = query(vec!["claude"], None, None, false);
        multi_select_query.cwds = vec!["D:/repo".to_string(), "D:/other".to_string()];
        multi_select_query.models = vec!["unknown".to_string(), "sonnet".to_string()];
        let multi_selected = query_usage_events(&connection, &multi_select_query)
            .expect("query multiple cwd and model selections");
        assert_eq!(multi_selected.len(), 2);
        assert_eq!(
            multi_selected
                .iter()
                .map(|event| event.session_id.as_str())
                .collect::<std::collections::BTreeSet<_>>(),
            std::collections::BTreeSet::from(["other-id", "same-id"])
        );

        let with_archived = query_usage_events(
            &connection,
            &query(vec!["claude"], Some("D:/repo"), None, true),
        )
        .expect("query archived sessions");
        assert_eq!(with_archived.len(), 2);
    }
}
