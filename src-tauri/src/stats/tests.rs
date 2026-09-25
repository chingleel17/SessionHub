use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};

use crate::db::init_db;
use crate::types::SessionStats;

use super::{
    backfill_missing_stats_internal, build_claude_usage_events, build_opencode_usage_events,
    calculate_opencode_session_stats, claude_usage_coverage, compute_claude_stats,
    extract_codex_usage, extract_copilot_usage, get_session_stats_cache,
    parse_session_stats_internal, provider_usage_capabilities, session_events_mtime,
    upsert_session_stats_cache,
};

fn create_temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    let dir = env::temp_dir().join(format!("sessionhub-{name}-{suffix}"));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir
}

fn create_session_dir(base_dir: &Path, session_id: &str, with_events: bool) -> PathBuf {
    let session_dir = base_dir.join(session_id);
    fs::create_dir_all(&session_dir).expect("session dir should be created");
    if with_events {
        fs::write(session_dir.join("events.jsonl"), "").expect("events file should be created");
    }
    session_dir
}

fn insert_session_cache_row(
    connection: &Connection,
    session_id: &str,
    session_dir: &Path,
    updated_at: &str,
) {
    connection
        .execute(
            "
            INSERT INTO sessions_cache (
                session_id, provider, cwd, summary, summary_count, created_at, updated_at,
                session_dir, parse_error, is_archived, has_plan, has_events
            ) VALUES (?1, 'copilot', NULL, NULL, NULL, ?2, ?2, ?3, 0, 0, 0, 1)
            ",
            params![
                session_id,
                updated_at,
                session_dir.to_string_lossy().to_string()
            ],
        )
        .expect("session cache row should be inserted");
}

#[test]
fn backfill_skips_cached_and_live_sessions() {
    let temp_root = create_temp_dir("stats-backfill");
    let database_path = temp_root.join("metadata.db");
    let connection = Connection::open(&database_path).expect("db should open");
    init_db(&connection).expect("db should initialize");

    let completed_dir = create_session_dir(&temp_root, "completed-session", true);
    let cached_dir = create_session_dir(&temp_root, "cached-session", true);
    let live_dir = create_session_dir(&temp_root, "live-session", true);
    fs::write(live_dir.join("inuse.invalid.lock"), "").expect("live lock should be created");

    insert_session_cache_row(
        &connection,
        "completed-session",
        &completed_dir,
        "2026-05-13T10:00:00Z",
    );
    insert_session_cache_row(
        &connection,
        "cached-session",
        &cached_dir,
        "2026-05-13T09:00:00Z",
    );
    insert_session_cache_row(
        &connection,
        "live-session",
        &live_dir,
        "2026-05-13T08:00:00Z",
    );

    let cached_mtime =
        session_events_mtime(&cached_dir.join("events.jsonl")).expect("cached mtime should read");
    upsert_session_stats_cache(
        &connection,
        "cached-session",
        cached_mtime,
        &SessionStats::default(),
    )
    .expect("cached stats should insert");

    let processed =
        backfill_missing_stats_internal(&connection, &temp_root).expect("backfill should succeed");

    assert_eq!(processed, 1);

    let cached_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM session_stats WHERE session_id = 'cached-session'",
            [],
            |row| row.get(0),
        )
        .expect("cached stats count should query");
    let completed_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM session_stats WHERE session_id = 'completed-session'",
            [],
            |row| row.get(0),
        )
        .expect("completed stats count should query");
    let live_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM session_stats WHERE session_id = 'live-session'",
            [],
            |row| row.get(0),
        )
        .expect("live stats count should query");

    assert_eq!(cached_count, 1);
    assert_eq!(completed_count, 1);
    assert_eq!(live_count, 0);

    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn session_stats_cache_roundtrip_preserves_large_u64_values() {
    let temp_root = create_temp_dir("stats-u64-roundtrip");
    let connection = Connection::open(temp_root.join("metadata.db")).expect("db should open");
    init_db(&connection).expect("db should initialize");

    // 使用明顯非零且超過 u32 範圍的值，確保 i64 與 u64 之間的轉換沒有截斷或錯位
    let stats = SessionStats {
        output_tokens: 9_876_543_210,
        input_tokens: 1_234_567_890_123,
        interaction_count: 42,
        tool_call_count: 17,
        duration_minutes: 5_000_000_000,
        models_used: vec!["claude-opus-5".to_string()],
        reasoning_count: 7,
        tool_breakdown: BTreeMap::from([("Read".to_string(), 3u32)]),
        model_metrics: BTreeMap::new(),
        is_live: false,
    };

    upsert_session_stats_cache(&connection, "u64-session", 1_700_000_000, &stats)
        .expect("stats should insert");

    let (events_mtime, restored) = get_session_stats_cache(&connection, "u64-session")
        .expect("stats should query")
        .expect("stats row should exist");

    assert_eq!(events_mtime, 1_700_000_000);
    assert_eq!(restored.output_tokens, stats.output_tokens);
    assert_eq!(restored.input_tokens, stats.input_tokens);
    assert_eq!(restored.duration_minutes, stats.duration_minutes);
    assert_eq!(restored.interaction_count, stats.interaction_count);
    assert_eq!(restored.tool_call_count, stats.tool_call_count);
    assert_eq!(restored.reasoning_count, stats.reasoning_count);
    assert_eq!(restored.models_used, stats.models_used);
    assert_eq!(restored.tool_breakdown, stats.tool_breakdown);

    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn provider_usage_fixtures_preserve_event_and_summary_semantics() {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/usage");
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("../../tests/fixtures/usage/expected.json"))
            .expect("fixture expectations should be valid JSON");

    let claude_path = create_temp_dir("claude-usage-fixture").join("session.jsonl");
    fs::write(
        &claude_path,
        include_str!("../../tests/fixtures/usage/claude.jsonl"),
    )
    .expect("Claude fixture should be written");
    let claude_stats = compute_claude_stats(&claude_path).expect("Claude fixture should parse");
    assert_eq!(
        claude_stats.input_tokens,
        expected["claude"]["rawInputTokens"]
            .as_u64()
            .expect("raw input")
    );
    assert_eq!(
        claude_stats.output_tokens,
        expected["claude"]["outputTokens"].as_u64().expect("output")
    );
    assert_eq!(claude_stats.model_metrics.len(), 1);
    assert_eq!(
        claude_stats.model_metrics["claude-sonnet-4-20250514"].requests_count,
        expected["claude"]["deduplicatedMessageCount"]
            .as_u64()
            .expect("messages") as f64
    );
    let expected_usd = expected["claude"]["estimatedUsd"]
        .as_f64()
        .expect("USD estimate");
    let actual_usd = claude_stats.model_metrics["claude-sonnet-4-20250514"].requests_cost;
    assert!((actual_usd - expected_usd).abs() < 0.000_000_1);
    assert_eq!(
        expected["claude"]["normalizedInputTokens"]
            .as_u64()
            .expect("normalized input"),
        expected["claude"]["rawInputTokens"]
            .as_u64()
            .expect("raw input")
            + expected["claude"]["cacheCreationInputTokens"]
                .as_u64()
                .expect("cache write")
            + expected["claude"]["cacheReadInputTokens"]
                .as_u64()
                .expect("cache read")
    );
    let claude_events = build_claude_usage_events(&claude_path, "claude-fixture-session")
        .expect("Claude usage events should parse");
    let (claude_coverage, claude_reason) =
        claude_usage_coverage(&claude_path).expect("Claude coverage should inspect");
    assert_eq!(
        claude_coverage,
        crate::types::AnalyticsCoverageStatus::Complete
    );
    assert_eq!(claude_reason, None);
    assert_eq!(claude_events.len(), 2);
    assert_eq!(claude_events[0].source_event_id, "msg_fixture_claude_001");
    assert_eq!(
        claude_events[0].model_provider_id.as_deref(),
        Some("claude")
    );
    assert_eq!(claude_events[0].parser_version, 4);
    assert_eq!(
        claude_events[0].occurred_at,
        "2026-04-10T09:00:00.000000000Z"
    );
    assert_eq!(claude_events[0].input_tokens, Some(180));
    assert_eq!(claude_events[0].output_tokens, Some(25));
    assert_eq!(claude_events[0].cache_write_tokens, Some(30));
    assert_eq!(claude_events[0].cache_read_tokens, Some(40));
    assert_eq!(claude_events[0].estimated_usd_micros, Some(830));
    assert!(claude_events[0].estimated_usd_price_version.is_some());
    assert_eq!(claude_events[0].cost_points_micros, None);

    let opencode_message_dir = fixture_root.join("opencode/message/ses_fixture");
    let opencode_stats = calculate_opencode_session_stats(&opencode_message_dir)
        .expect("OpenCode fixture should parse");
    assert_eq!(
        opencode_stats.input_tokens,
        expected["opencode"]["inputTokens"].as_u64().expect("input")
    );
    assert_eq!(
        opencode_stats.output_tokens,
        expected["opencode"]["outputTokens"]
            .as_u64()
            .expect("output")
    );
    assert_eq!(opencode_stats.reasoning_count, 2);
    let opencode_message: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/usage/opencode/message/ses_fixture/msg_fixture_opencode_001.json"
    ))
    .expect("OpenCode message fixture should be valid JSON");
    assert_eq!(
        opencode_message
            .pointer("/tokens/cache/read")
            .and_then(|value| value.as_u64()),
        Some(50)
    );
    assert_eq!(
        opencode_message
            .pointer("/tokens/reasoning")
            .and_then(|value| value.as_u64()),
        Some(20)
    );
    let opencode_events = build_opencode_usage_events(&opencode_message_dir, "ses_fixture")
        .expect("OpenCode usage events should parse");
    assert_eq!(opencode_events.len(), 2);
    assert_eq!(
        opencode_events[0].source_event_id,
        "msg_fixture_opencode_001"
    );
    assert_eq!(opencode_events[0].model.as_deref(), Some("gpt-5.4"));
    assert_eq!(
        opencode_events[0].model_provider_id.as_deref(),
        Some("openai")
    );
    assert_eq!(opencode_events[0].input_tokens, Some(300));
    assert_eq!(opencode_events[0].output_tokens, Some(60));
    assert_eq!(opencode_events[0].reasoning_tokens, Some(20));
    assert_eq!(opencode_events[0].cache_read_tokens, Some(50));
    assert_eq!(opencode_events[0].cache_write_tokens, Some(10));
    assert_eq!(opencode_events[0].estimated_usd_micros, Some(1_250));
    assert_eq!(
        opencode_events[0].estimated_usd_price_version.as_deref(),
        Some("opencode-persisted-cost-v1")
    );
    assert_eq!(opencode_events[0].cost_points_micros, None);
    assert_eq!(opencode_events[1].model.as_deref(), Some("claude-sonnet-4"));
    assert_eq!(
        opencode_events[1].model_provider_id.as_deref(),
        Some("ollama")
    );

    let opencode_db_root = create_temp_dir("opencode-db-usage-fixture");
    let opencode_storage_root = opencode_db_root.join("storage");
    fs::create_dir_all(opencode_storage_root.join("message"))
        .expect("OpenCode storage directory should be created");
    let opencode_db = Connection::open(opencode_db_root.join("opencode.db"))
        .expect("OpenCode db fixture should open");
    opencode_db
        .execute_batch(
            "CREATE TABLE message (id TEXT, session_id TEXT, data TEXT, time_created INTEGER);",
        )
        .expect("OpenCode db message table should be created");
    for fixture_name in [
        "msg_fixture_opencode_001.json",
        "msg_fixture_opencode_002.json",
    ] {
        let raw = fs::read_to_string(
            fixture_root
                .join("opencode/message/ses_fixture")
                .join(fixture_name),
        )
        .expect("OpenCode JSON message fixture should load");
        let message: serde_json::Value =
            serde_json::from_str(&raw).expect("OpenCode JSON message should parse");
        opencode_db
            .execute(
                "INSERT INTO message (id, session_id, data, time_created) VALUES (?1, ?2, ?3, ?4)",
                params![
                    message["id"].as_str().expect("message id"),
                    "ses_fixture",
                    raw,
                    message
                        .pointer("/time/created")
                        .and_then(|value| value.as_i64())
                        .expect("created time"),
                ],
            )
            .expect("OpenCode db message should insert");
    }
    let db_message_dir = opencode_storage_root.join("message/ses_fixture");
    let opencode_db_events = build_opencode_usage_events(&db_message_dir, "ses_fixture")
        .expect("OpenCode db usage events should parse");
    assert_eq!(opencode_db_events, opencode_events);

    let copilot_dir =
        create_session_dir(&create_temp_dir("copilot-usage-fixture"), "session", true);
    fs::write(
        copilot_dir.join("events.jsonl"),
        include_str!("../../tests/fixtures/usage/copilot.jsonl"),
    )
    .expect("Copilot fixture should be written");
    let copilot_stats =
        parse_session_stats_internal(&copilot_dir).expect("Copilot fixture should parse");
    assert_eq!(
        copilot_stats.output_tokens,
        expected["copilot"]["eventOutputTokens"]
            .as_u64()
            .expect("event output")
    );
    assert_eq!(copilot_stats.input_tokens, 0);
    assert_eq!(
        copilot_stats.model_metrics["fixture-model"].input_tokens,
        expected["copilot"]["shutdownSummaryInputTokens"]
            .as_u64()
            .expect("summary input")
    );
    assert_eq!(
        copilot_stats.model_metrics["fixture-model"].output_tokens,
        expected["copilot"]["shutdownSummaryOutputTokens"]
            .as_u64()
            .expect("summary output")
    );
    assert_eq!(
        copilot_stats.model_metrics["fixture-model"].requests_cost,
        expected["copilot"]["shutdownRequestCost"]
            .as_f64()
            .expect("source-native cost")
    );
    let (copilot_events, copilot_summaries) =
        extract_copilot_usage(&copilot_dir, "copilot-fixture-session")
            .expect("Copilot usage extraction should succeed");
    assert_eq!(copilot_events.len(), 3);
    assert_eq!(copilot_events[0].model.as_deref(), Some("fixture-model"));
    assert_eq!(copilot_events[1].model.as_deref(), Some("fixture-model-v2"));
    assert_ne!(copilot_events[0].occurred_at, copilot_events[1].occurred_at);
    assert_eq!(
        copilot_events
            .iter()
            .map(|event| event.output_tokens.unwrap_or(0))
            .sum::<i64>(),
        130
    );
    assert_eq!(copilot_events[0].input_tokens, None);
    assert_eq!(
        copilot_events[2].source_kind,
        "copilot_ai_credit_checkpoint"
    );
    assert_eq!(copilot_events[2].estimated_usd_micros, Some(250_000));
    assert_eq!(copilot_summaries.len(), 1);
    assert_eq!(copilot_summaries[0].input_tokens, Some(900));
    assert_eq!(copilot_summaries[0].output_tokens, Some(700));
    assert_eq!(copilot_summaries[0].cost_points_micros, Some(250_000));
    assert_eq!(copilot_summaries[0].estimated_usd_micros, Some(250_000));

    let capabilities = provider_usage_capabilities();
    let codex = capabilities
        .iter()
        .find(|capability| capability.provider == "codex")
        .expect("Codex capability should be reported");
    assert_eq!(
        codex.token_status,
        crate::types::AnalyticsCoverageStatus::Partial
    );
    let antigravity = capabilities
        .iter()
        .find(|capability| capability.provider == "antigravity")
        .expect("Antigravity capability should be reported");
    assert_eq!(
        antigravity.token_status,
        crate::types::AnalyticsCoverageStatus::Unsupported
    );
    let copilot_capability = capabilities
        .iter()
        .find(|capability| capability.provider == "copilot")
        .expect("Copilot capability should be reported");
    assert_eq!(
        copilot_capability.copilot_points_status,
        crate::types::AnalyticsCoverageStatus::SummaryOnly
    );
}

#[test]
fn codex_usage_fixtures_reject_unsafe_counter_interpretations() {
    let fixture = include_str!("../../tests/fixtures/usage/codex_token_counts.jsonl");
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/usage/codex_expected.json"
    ))
    .expect("Codex fixture expectations should be valid JSON");
    let records = fixture
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("token event JSON"))
        .collect::<Vec<_>>();
    let counters = records
        .iter()
        .map(|record| {
            record
                .pointer("/info/total_token_usage/total_tokens")
                .and_then(serde_json::Value::as_i64)
                .expect("cumulative token count")
        })
        .collect::<Vec<_>>();

    assert_eq!(records.len(), 5);
    assert_eq!(counters[0], counters[1]);
    assert!(counters[2] > counters[1]);
    assert!(counters[3] < counters[2]);
    assert!(counters[4] > counters[3]);
    assert_eq!(counters[2] - counters[1], 55);
    assert_eq!(counters[4] - counters[3], 20);
    assert!(expected["persistenceWrapperVerified"].as_bool() == Some(false));
    assert_eq!(
        expected["outcomes"][0]["deltaTokens"],
        serde_json::Value::Null
    );
    assert_eq!(expected["outcomes"][2]["status"], "partial");
    assert_eq!(expected["outcomes"][5]["status"], "unsupported");

    let unknown = include_str!("../../tests/fixtures/usage/codex_unknown.jsonl");
    let unknown: serde_json::Value =
        serde_json::from_str(unknown.trim()).expect("unknown-format fixture should be valid JSON");
    assert!(unknown.pointer("/info/total_token_usage").is_none());

    let unknown_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/usage/codex_unknown.jsonl");
    let extraction = extract_codex_usage(&unknown_path, "session-id")
        .expect("unknown Codex persistence format should produce capability status");
    assert_eq!(
        extraction.status,
        crate::types::AnalyticsCoverageStatus::Unsupported
    );
    assert!(extraction.events.is_empty());
    assert!(extraction.session_summaries.is_empty());
    assert!(extraction.reason.is_some());
}

#[test]
fn copilot_checkpoint_supplies_session_level_ai_credit_estimate() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/usage/copilot_checkpoint.jsonl");
    let temp_root = create_temp_dir("copilot-checkpoint");
    let events_path = temp_root.join("events.jsonl");
    fs::copy(&fixture, &events_path).expect("copy Copilot checkpoint fixture");

    let (_, summaries) = extract_copilot_usage(&temp_root, "copilot-checkpoint-session")
        .expect("Copilot checkpoint should parse");

    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].model, None);
    assert_eq!(summaries[0].estimated_usd_micros, Some(125_000));
    assert_eq!(summaries[0].cost_points_micros, None);
    assert_eq!(summaries[0].source_kind, "copilot_ai_credit_checkpoint");

    fs::remove_dir_all(temp_root).expect("remove Copilot checkpoint fixture directory");
}

#[test]
fn codex_rollout_fixture_indexes_timestamped_usage_and_known_model_cost() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/usage/codex_rollout.jsonl");

    let extraction = extract_codex_usage(&fixture, "codex-fixture-session")
        .expect("verified Codex rollout should parse");

    assert_eq!(
        extraction.status,
        crate::types::AnalyticsCoverageStatus::Partial
    );
    assert_eq!(extraction.events.len(), 2);
    assert_eq!(extraction.events[0].model.as_deref(), Some("gpt-6-luna"));
    assert_eq!(
        extraction.events[0].model_provider_id.as_deref(),
        Some("openai")
    );
    assert_eq!(extraction.events[0].input_tokens, Some(100));
    assert_eq!(extraction.events[0].cache_read_tokens, Some(20));
    assert_eq!(extraction.events[0].output_tokens, Some(20));
    assert_eq!(extraction.events[0].reasoning_tokens, Some(5));
    assert_eq!(extraction.events[0].estimated_usd_micros, Some(18));
    assert_eq!(extraction.events[1].input_tokens, Some(40));
    assert_eq!(extraction.events[1].estimated_usd_micros, Some(11));
}

#[test]
fn analytics_contract_serializes_camel_case_nulls_revisions_and_units() {
    use crate::types::{
        AnalyticsBreakdown, AnalyticsComparison, AnalyticsComparisonStatus, AnalyticsCoverage,
        AnalyticsDetailStatus, AnalyticsMetric, AnalyticsMetricValue, AnalyticsQuery,
        AnalyticsReport, AnalyticsSessionDetail, AnalyticsSessionPage, AnalyticsTokenField,
        AnalyticsValues,
    };

    let query = AnalyticsQuery {
        start_date: "2026-04-01".to_string(),
        end_date: "2026-04-30".to_string(),
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
    let query_json = serde_json::to_value(query).expect("analytics query should serialize");
    assert_eq!(query_json["timeZone"], "Asia/Taipei");
    assert_eq!(query_json["groupBy"], "day");
    assert_eq!(query_json["metric"], "estimated_usd");
    assert_eq!(query_json["cwd"], serde_json::Value::Null);

    let metric = |known_value| AnalyticsMetricValue {
        known_value,
        eligible_event_count: 2,
        missing_field_event_count: 1,
        price_versions: vec!["fixture-price-v1".to_string()],
    };
    let values = AnalyticsValues {
        total_tokens: metric(None),
        input_tokens: metric(Some(100.0)),
        output_tokens: metric(Some(40.0)),
        cache_read_tokens: metric(None),
        cache_write_tokens: metric(None),
        reasoning_tokens: metric(None),
        estimated_usd: metric(Some(0.000_125)),
        copilot_points: metric(None),
    };
    let report = AnalyticsReport {
        summary: values.clone(),
        previous_summary: None,
        comparison: AnalyticsComparison {
            status: AnalyticsComparisonStatus::NotComparable,
            percent_change: None,
            reason: None,
        },
        series: Vec::new(),
        project_ranking: Vec::new(),
        model_ranking: Vec::new(),
        platform_ranking: Vec::new(),
        supplier_ranking: Vec::new(),
        coverage: AnalyticsCoverage {
            metrics: Default::default(),
            providers: Vec::new(),
        },
        session_summary_only: Vec::new(),
        revision: 7,
        generated_at: "2026-04-30T12:00:00Z".to_string(),
        time_zone: "Asia/Taipei".to_string(),
    };
    let report_json = serde_json::to_value(report).expect("analytics report should serialize");
    assert_eq!(report_json["revision"], 7);
    assert_eq!(report_json["previousSummary"], serde_json::Value::Null);
    assert_eq!(report_json["comparison"]["status"], "not_comparable");
    assert_eq!(
        report_json["summary"]["totalTokens"]["knownValue"],
        serde_json::Value::Null
    );
    assert_eq!(
        report_json["summary"]["estimatedUsd"]["knownValue"],
        0.000_125
    );
    assert_eq!(
        report_json["summary"]["copilotPoints"]["knownValue"],
        serde_json::Value::Null
    );

    let detail = |provider: &str| AnalyticsSessionDetail {
        provider: provider.to_string(),
        model_provider_ids: Vec::new(),
        session_id: "shared-session-id".to_string(),
        cwd: None,
        model: None,
        first_event_at: None,
        last_event_at: None,
        values: values.clone(),
        session_summary_only: None,
        status: crate::types::AnalyticsCoverageStatus::Partial,
    };
    let page = AnalyticsSessionPage {
        status: AnalyticsDetailStatus::StaleRevision,
        revision: 8,
        page: 1,
        page_size: 50,
        total_count: 2,
        items: vec![detail("claude"), detail("codex")],
    };
    let page_json = serde_json::to_value(page).expect("analytics detail page should serialize");
    assert_eq!(page_json["status"], "stale_revision");
    assert_eq!(page_json["items"][0]["sessionId"], "shared-session-id");
    assert_eq!(page_json["items"][0]["provider"], "claude");
    assert_eq!(page_json["items"][1]["provider"], "codex");
}
