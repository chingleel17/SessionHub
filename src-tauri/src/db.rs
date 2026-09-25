use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use chrono::Datelike;
use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::settings::{legacy_session_cache_path, metadata_db_path};
use crate::types::*;

/// 將 SQLite 讀出的 i64 轉為應用層 u64；負值視為 0
///
/// rusqlite 0.40 起移除了 u64 的 FromSql/ToSql 實作，需在 DB 邊界明確轉換
pub(crate) fn i64_to_u64(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

/// 將應用層 u64 轉為 SQLite 可寫入的 i64；超出範圍時取 i64::MAX
pub(crate) fn u64_to_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

pub(crate) fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create directory {}: {error}", parent.display()))?;
    }
    Ok(())
}

pub(crate) fn open_db_connection() -> Result<Connection, String> {
    let db_path = metadata_db_path()?;
    ensure_parent_dir(&db_path)?;
    let connection = Connection::open(db_path)
        .map_err(|error| format!("failed to open metadata db: {error}"))?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|error| format!("failed to configure metadata db busy timeout: {error}"))?;
    Ok(connection)
}

pub(crate) struct DbState {
    pub(crate) conn: Mutex<Connection>,
}

impl DbState {
    pub(crate) fn new() -> Result<Self, String> {
        let db_path = metadata_db_path()?;
        ensure_parent_dir(&db_path)?;
        let conn =
            Connection::open(&db_path).map_err(|e| format!("failed to open metadata db: {e}"))?;
        conn.busy_timeout(Duration::from_secs(5))
            .map_err(|error| format!("failed to configure metadata db busy timeout: {error}"))?;
        init_db(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

pub(crate) fn init_db(connection: &Connection) -> Result<(), String> {
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|error| format!("failed to configure metadata db busy timeout: {error}"))?;
    connection
        .execute_batch("PRAGMA journal_mode=WAL;")
        .map_err(|e| format!("failed to set WAL mode: {e}"))?;

    connection
        .execute(
            "
            CREATE TABLE IF NOT EXISTS session_meta (
                session_id TEXT PRIMARY KEY,
                notes TEXT,
                tags TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            ",
            [],
        )
        .map_err(|error| format!("failed to initialize metadata db: {error}"))?;

    connection
        .execute(
            "
            CREATE TABLE IF NOT EXISTS project_path_remap (
                old_path_key TEXT PRIMARY KEY,
                old_path TEXT NOT NULL,
                new_path TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            ",
            [],
        )
        .map_err(|error| format!("failed to initialize project path remap db: {error}"))?;

    connection
        .execute(
            "
            CREATE TABLE IF NOT EXISTS session_stats (
                session_id TEXT PRIMARY KEY,
                events_mtime INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                interaction_count INTEGER NOT NULL,
                tool_call_count INTEGER NOT NULL,
                duration_minutes INTEGER NOT NULL,
                models_used TEXT NOT NULL,
                reasoning_count INTEGER NOT NULL,
                tool_breakdown TEXT NOT NULL,
                model_metrics TEXT NOT NULL DEFAULT '{}'
            )
            ",
            [],
        )
        .map_err(|error| format!("failed to initialize session stats db: {error}"))?;

    connection
        .execute(
            "
            CREATE TABLE IF NOT EXISTS sessions_cache (
                session_id TEXT NOT NULL,
                provider TEXT NOT NULL DEFAULT 'copilot',
                cwd TEXT,
                repo_root TEXT,
                repo_name TEXT,
                git_branch TEXT,
                summary TEXT,
                summary_count INTEGER,
                created_at TEXT,
                updated_at TEXT,
                session_dir TEXT,
                parse_error INTEGER NOT NULL DEFAULT 0,
                is_archived INTEGER NOT NULL DEFAULT 0,
                has_plan INTEGER NOT NULL DEFAULT 0,
                has_events INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (session_id, provider)
            )
            ",
            [],
        )
        .map_err(|error| format!("failed to initialize sessions cache db: {error}"))?;

    connection
        .execute(
            "
            CREATE TABLE IF NOT EXISTS scan_state (
                provider TEXT NOT NULL PRIMARY KEY,
                last_full_scan_at INTEGER NOT NULL DEFAULT 0,
                last_cursor INTEGER NOT NULL DEFAULT 0
            )
            ",
            [],
        )
        .map_err(|error| format!("failed to initialize scan state db: {error}"))?;

    connection
        .execute(
            "
            CREATE TABLE IF NOT EXISTS session_mtimes (
                session_id TEXT NOT NULL PRIMARY KEY,
                provider TEXT NOT NULL DEFAULT 'copilot',
                mtime INTEGER NOT NULL DEFAULT 0
            )
            ",
            [],
        )
        .map_err(|error| format!("failed to initialize session mtimes db: {error}"))?;

    // Migration: 新增 input_tokens 欄位（舊資料庫相容）
    if let Err(error) = connection.execute(
        "ALTER TABLE session_stats ADD COLUMN input_tokens INTEGER NOT NULL DEFAULT 0",
        [],
    ) {
        let error_message = error.to_string();
        if !error_message.contains("duplicate column name") {
            eprintln!("Warning: failed to add input_tokens column: {error}");
        }
    }

    // Migration: 新增 model_metrics 欄位（舊資料庫相容）
    if let Err(error) = connection.execute(
        "ALTER TABLE session_stats ADD COLUMN model_metrics TEXT NOT NULL DEFAULT '{}'",
        [],
    ) {
        let error_message = error.to_string();
        if !error_message.contains("duplicate column name") {
            eprintln!("Warning: failed to add model_metrics column: {error}");
        }
    }

    for (column_name, sql_type) in [
        ("repo_root", "TEXT"),
        ("repo_name", "TEXT"),
        ("git_branch", "TEXT"),
    ] {
        if let Err(error) = connection.execute(
            &format!("ALTER TABLE sessions_cache ADD COLUMN {column_name} {sql_type}"),
            [],
        ) {
            let error_message = error.to_string();
            if !error_message.contains("duplicate column name") {
                eprintln!("Warning: failed to add {column_name} column: {error}");
            }
        }
    }

    // provider_quota: 各 provider 每個帳單週期的累計用量
    connection
        .execute(
            "
            CREATE TABLE IF NOT EXISTS provider_quota (
                provider TEXT NOT NULL,
                billing_period TEXT NOT NULL,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
                cache_read_tokens INTEGER NOT NULL DEFAULT 0,
                cost_usd REAL NOT NULL DEFAULT 0.0,
                PRIMARY KEY (provider, billing_period)
            )
            ",
            [],
        )
        .map_err(|error| format!("failed to initialize provider_quota table: {error}"))?;

    // provider_quota_settings: 各 provider 的方案上限設定
    connection
        .execute(
            "
            CREATE TABLE IF NOT EXISTS provider_quota_settings (
                provider TEXT NOT NULL PRIMARY KEY,
                monthly_limit_tokens INTEGER,
                monthly_limit_usd REAL,
                reset_day INTEGER NOT NULL DEFAULT 1
            )
            ",
            [],
        )
        .map_err(|error| format!("failed to initialize provider_quota_settings table: {error}"))?;

    // quota_snapshots: 從訂閱服務查回的剩餘額度快照
    connection
        .execute(
            "
            CREATE TABLE IF NOT EXISTS quota_snapshots (
                provider TEXT PRIMARY KEY,
                snapshot_json TEXT NOT NULL,
                fetched_at TEXT NOT NULL
            )
            ",
            [],
        )
        .map_err(|error| format!("failed to initialize quota_snapshots table: {error}"))?;

    migrate_usage_analytics_schema(connection)?;

    if let Err(error) = migrate_legacy_session_cache(connection) {
        eprintln!("Warning: failed to migrate legacy session cache: {error}");
    }

    Ok(())
}

const USAGE_SCHEMA_VERSION: i64 = 6;
const INITIAL_MODEL_VISIBILITY_VERSION: i64 = 5;
const OPENAI_MAX_HIDDEN_GPT5_MINOR: u32 = 2;
const ANTHROPIC_MAX_HIDDEN_MINOR: u32 = 1;

fn model_hidden_by_default(provider: &str, model: &str) -> bool {
    let provider = provider.to_ascii_lowercase();
    let model = model.to_ascii_lowercase();
    let versions = model
        .split(|character: char| !character.is_ascii_digit())
        .filter_map(|part| part.parse::<u32>().ok())
        .collect::<Vec<_>>();
    let Some(&major) = versions.first() else {
        return false;
    };
    let minor = versions
        .get(1)
        .copied()
        .filter(|value| *value < 10)
        .unwrap_or(0);

    match provider.as_str() {
        "openai" => {
            model.split('-').any(|part| part == "pro")
                || (model.starts_with('o')
                    && model
                        .as_bytes()
                        .get(1)
                        .is_some_and(|character| character.is_ascii_digit()))
                || (model.starts_with("gpt-")
                    && (major == 4 || (major == 5 && minor <= OPENAI_MAX_HIDDEN_GPT5_MINOR)))
        }
        "anthropic" => {
            model.starts_with("claude-mythos-")
                || (model.starts_with("claude-")
                    && (major < 4 || (major == 4 && minor <= ANTHROPIC_MAX_HIDDEN_MINOR)))
        }
        _ => false,
    }
}

fn model_newly_hidden_by_default(provider: &str, model: &str) -> bool {
    let provider = provider.to_ascii_lowercase();
    let model = model.to_ascii_lowercase();
    match provider.as_str() {
        "openai" => model.split('-').any(|part| part == "pro"),
        "anthropic" => model.starts_with("claude-mythos-"),
        _ => false,
    }
}

fn migrate_usage_analytics_schema(connection: &Connection) -> Result<(), String> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("failed to begin usage analytics migration: {error}"))?;
    transaction
        .execute_batch(
            "
            CREATE TABLE IF NOT EXISTS usage_events (
                provider TEXT NOT NULL,
                session_id TEXT NOT NULL,
                source_event_id TEXT NOT NULL,
                occurred_at TEXT NOT NULL,
                cwd TEXT,
                model TEXT,
                model_provider_id TEXT,
                service_tier TEXT,
                input_tokens INTEGER,
                output_tokens INTEGER,
                cache_read_tokens INTEGER,
                cache_write_tokens INTEGER,
                reasoning_tokens INTEGER,
                estimated_usd_micros INTEGER,
                estimated_usd_price_version TEXT,
                cost_points_micros INTEGER,
                source_kind TEXT NOT NULL,
                parser_version INTEGER NOT NULL CHECK (parser_version >= 0),
                PRIMARY KEY (provider, session_id, source_event_id)
            );
            CREATE INDEX IF NOT EXISTS idx_usage_events_time
                ON usage_events (occurred_at, provider, session_id);
            CREATE INDEX IF NOT EXISTS idx_usage_events_provider_time
                ON usage_events (provider, occurred_at, session_id, source_event_id);
            CREATE INDEX IF NOT EXISTS idx_usage_events_cwd_time
                ON usage_events (cwd, occurred_at, provider, session_id);
            CREATE INDEX IF NOT EXISTS idx_usage_events_model_time
                ON usage_events (model, occurred_at, provider, session_id);

            CREATE TABLE IF NOT EXISTS usage_session_summaries (
                provider TEXT NOT NULL,
                session_id TEXT NOT NULL,
                source_identity TEXT NOT NULL,
                cwd TEXT,
                model TEXT,
                active_from TEXT,
                active_until TEXT,
                input_tokens INTEGER,
                output_tokens INTEGER,
                estimated_usd_micros INTEGER,
                cost_points_micros INTEGER,
                source_kind TEXT NOT NULL,
                parser_version INTEGER NOT NULL CHECK (parser_version >= 0),
                integrity_status TEXT NOT NULL,
                PRIMARY KEY (provider, session_id, source_identity)
            );
            CREATE INDEX IF NOT EXISTS idx_usage_summary_scope
                ON usage_session_summaries (provider, cwd, session_id);

            CREATE TABLE IF NOT EXISTS usage_ingestion_state (
                provider TEXT NOT NULL,
                session_id TEXT NOT NULL,
                source_identity TEXT NOT NULL,
                source_fingerprint TEXT,
                parser_version INTEGER NOT NULL CHECK (parser_version >= 0),
                integrity_status TEXT NOT NULL,
                ingestion_status TEXT NOT NULL,
                session_revision INTEGER NOT NULL DEFAULT 0,
                error_message TEXT,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (provider, session_id, source_identity)
            );
            CREATE INDEX IF NOT EXISTS idx_usage_ingestion_status
                ON usage_ingestion_state (ingestion_status, provider, session_id);

            CREATE TABLE IF NOT EXISTS model_pricing_cache (
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                status TEXT NOT NULL,
                prompt_usd_per_token REAL,
                completion_usd_per_token REAL,
                cache_read_usd_per_token REAL,
                cache_write_usd_per_token REAL,
                source TEXT NOT NULL,
                fetched_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                error_kind TEXT,
                hidden INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (provider, model)
            );
            CREATE INDEX IF NOT EXISTS idx_model_pricing_cache_expiry
                ON model_pricing_cache (status, expires_at);

            CREATE TABLE IF NOT EXISTS analytics_revision (
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                revision INTEGER NOT NULL DEFAULT 0
            );
            INSERT OR IGNORE INTO analytics_revision (singleton, revision) VALUES (1, 0);

            CREATE TABLE IF NOT EXISTS usage_schema_version (
                singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
                version INTEGER NOT NULL
            );
            INSERT INTO usage_schema_version (singleton, version)
                VALUES (1, 4)
                ON CONFLICT(singleton) DO UPDATE SET version = MAX(version, excluded.version);
            ",
        )
        .map_err(|error| format!("failed to migrate usage analytics schema: {error}"))?;
    if let Err(error) = transaction.execute(
        "ALTER TABLE usage_events ADD COLUMN model_provider_id TEXT",
        [],
    ) {
        if !error.to_string().contains("duplicate column name") {
            return Err(format!(
                "failed to add model_provider_id to usage_events: {error}"
            ));
        }
    }
    if let Err(error) =
        transaction.execute("ALTER TABLE usage_events ADD COLUMN service_tier TEXT", [])
    {
        if !error.to_string().contains("duplicate column name") {
            return Err(format!(
                "failed to add service_tier to usage_events: {error}"
            ));
        }
    }
    for (column, definition) in [
        ("hidden", "INTEGER NOT NULL DEFAULT 0"),
        ("sort_order", "INTEGER NOT NULL DEFAULT 0"),
    ] {
        if let Err(error) = transaction.execute(
            &format!("ALTER TABLE model_pricing_cache ADD COLUMN {column} {definition}"),
            [],
        ) {
            if !error.to_string().contains("duplicate column name") {
                return Err(format!(
                    "failed to add {column} to model_pricing_cache: {error}"
                ));
            }
        }
    }
    transaction
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_usage_events_model_provider_time
             ON usage_events (model_provider_id, occurred_at, provider, session_id)",
            [],
        )
        .map_err(|error| format!("failed to create model provider usage index: {error}"))?;

    let schema_version: i64 = transaction
        .query_row(
            "SELECT version FROM usage_schema_version WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|error| format!("failed to read usage analytics schema version: {error}"))?;
    if schema_version < USAGE_SCHEMA_VERSION {
        let models = {
            let mut statement = transaction
                .prepare("SELECT provider, model FROM model_pricing_cache WHERE hidden = 0")
                .map_err(|error| {
                    format!("failed to prepare default model visibility migration: {error}")
                })?;
            let rows = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|error| {
                    format!("failed to read models for default visibility migration: {error}")
                })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|error| {
                format!("failed to collect models for default visibility migration: {error}")
            })?
        };
        for (provider, model) in models {
            let should_hide = if schema_version < INITIAL_MODEL_VISIBILITY_VERSION {
                model_hidden_by_default(&provider, &model)
            } else {
                model_newly_hidden_by_default(&provider, &model)
            };
            if should_hide {
                transaction
                    .execute(
                        "UPDATE model_pricing_cache SET hidden = 1 WHERE provider = ?1 AND model = ?2",
                        params![provider, model],
                    )
                    .map_err(|error| {
                        format!("failed to apply default model visibility: {error}")
                    })?;
            }
        }
        transaction
            .execute(
                "UPDATE usage_schema_version SET version = ?1 WHERE singleton = 1",
                [USAGE_SCHEMA_VERSION],
            )
            .map_err(|error| format!("failed to update usage analytics schema version: {error}"))?;
    }

    transaction
        .execute(
            "DELETE FROM model_pricing_cache
             WHERE provider = 'openai' AND model = 'codex-auto-review'
               AND source = 'openrouter-catalog'",
            [],
        )
        .map_err(|error| format!("failed to remove excluded OpenRouter model: {error}"))?;

    seed_builtin_model_pricing(&transaction)?;
    transaction
        .commit()
        .map_err(|error| format!("failed to commit usage analytics migration: {error}"))
}

pub(crate) fn seed_builtin_model_pricing(connection: &Connection) -> Result<(), String> {
    let prices = [
        (
            "openai",
            "gpt-6-astra",
            10.0,
            50.0,
            Some(1.0),
            Some(12.5),
            630,
        ),
        ("openai", "gpt-6-sol", 2.0, 10.0, Some(0.2), Some(2.5), 620),
        (
            "openai",
            "gpt-6-luna",
            0.1,
            0.5,
            Some(0.01),
            Some(0.125),
            610,
        ),
        (
            "openai",
            "gpt-5.6-sol",
            4.0,
            20.0,
            Some(0.4),
            Some(5.0),
            560,
        ),
        (
            "openai",
            "gpt-5.6-terra",
            2.0,
            12.0,
            Some(0.2),
            Some(2.5),
            550,
        ),
        (
            "openai",
            "gpt-5.6-luna",
            0.2,
            1.2,
            Some(0.02),
            Some(0.25),
            540,
        ),
        ("openai", "gpt-5.5", 5.0, 30.0, Some(0.5), None, 530),
        ("openai", "gpt-5.5-pro", 30.0, 180.0, None, None, 525),
        ("openai", "gpt-5.4", 2.5, 15.0, Some(0.25), None, 520),
        ("openai", "gpt-5.4-mini", 0.75, 4.5, Some(0.075), None, 515),
        ("openai", "gpt-5.4-nano", 0.2, 1.25, Some(0.02), None, 510),
        ("openai", "gpt-5.4-pro", 30.0, 180.0, None, None, 505),
        (
            "openai",
            "gpt-5.3-codex",
            1.75,
            14.0,
            Some(0.175),
            None,
            500,
        ),
        ("openai", "gpt-5.2", 1.75, 14.0, Some(0.175), None, 490),
        ("openai", "gpt-5.2-pro", 21.0, 168.0, None, None, 485),
        ("openai", "gpt-5.1", 1.25, 10.0, Some(0.125), None, 480),
        ("openai", "gpt-5", 1.25, 10.0, Some(0.125), None, 470),
        ("openai", "gpt-5-mini", 0.25, 2.0, Some(0.025), None, 460),
        ("openai", "gpt-5-nano", 0.05, 0.4, Some(0.005), None, 450),
        ("openai", "gpt-5-pro", 15.0, 120.0, None, None, 440),
        ("openai", "gpt-4.1", 2.0, 8.0, Some(0.5), None, 410),
        ("openai", "gpt-4.1-mini", 0.4, 1.6, Some(0.1), None, 400),
        ("openai", "gpt-4.1-nano", 0.1, 0.4, Some(0.025), None, 390),
        ("openai", "gpt-4o", 2.5, 10.0, Some(1.25), None, 380),
        ("openai", "gpt-4o-mini", 0.15, 0.6, Some(0.075), None, 370),
        ("openai", "o4-mini", 1.1, 4.4, Some(0.275), None, 360),
        ("openai", "o3", 2.0, 8.0, Some(0.5), None, 350),
        ("openai", "o3-pro", 20.0, 80.0, None, None, 340),
        ("openai", "o3-mini", 1.1, 4.4, Some(0.55), None, 330),
        ("openai", "o1", 15.0, 60.0, Some(7.5), None, 320),
        ("openai", "o1-pro", 150.0, 600.0, None, None, 310),
        (
            "anthropic",
            "claude-fable-5-1",
            10.0,
            50.0,
            Some(0.25),
            Some(12.5),
            610,
        ),
        (
            "anthropic",
            "claude-mythos-5-1",
            10.0,
            50.0,
            Some(0.25),
            Some(12.5),
            605,
        ),
        (
            "anthropic",
            "claude-opus-5-5",
            4.0,
            20.0,
            Some(0.2),
            Some(5.0),
            600,
        ),
        (
            "anthropic",
            "claude-fable-5",
            10.0,
            50.0,
            Some(1.0),
            Some(12.5),
            590,
        ),
        (
            "anthropic",
            "claude-mythos-5",
            10.0,
            50.0,
            Some(1.0),
            Some(12.5),
            585,
        ),
        (
            "anthropic",
            "claude-opus-5",
            5.0,
            25.0,
            Some(0.5),
            Some(6.25),
            580,
        ),
        (
            "anthropic",
            "claude-sonnet-5",
            2.0,
            10.0,
            Some(0.2),
            Some(2.5),
            570,
        ),
        (
            "anthropic",
            "claude-opus-4-8",
            5.0,
            25.0,
            Some(0.5),
            Some(6.25),
            550,
        ),
        (
            "anthropic",
            "claude-opus-4-7",
            5.0,
            25.0,
            Some(0.5),
            Some(6.25),
            540,
        ),
        (
            "anthropic",
            "claude-opus-4-6",
            5.0,
            25.0,
            Some(0.5),
            Some(6.25),
            530,
        ),
        (
            "anthropic",
            "claude-sonnet-4-6",
            3.0,
            15.0,
            Some(0.3),
            Some(3.75),
            520,
        ),
        (
            "anthropic",
            "claude-opus-4-5",
            5.0,
            25.0,
            Some(0.5),
            Some(6.25),
            510,
        ),
        (
            "anthropic",
            "claude-sonnet-4-5",
            3.0,
            15.0,
            Some(0.3),
            Some(3.75),
            500,
        ),
        (
            "anthropic",
            "claude-haiku-4-5",
            1.0,
            5.0,
            Some(0.1),
            Some(1.25),
            490,
        ),
        (
            "anthropic",
            "claude-opus-4-1",
            15.0,
            75.0,
            Some(1.5),
            Some(18.75),
            410,
        ),
        (
            "anthropic",
            "claude-opus-4",
            15.0,
            75.0,
            Some(1.5),
            Some(18.75),
            400,
        ),
        (
            "anthropic",
            "claude-sonnet-4",
            3.0,
            15.0,
            Some(0.3),
            Some(3.75),
            390,
        ),
        (
            "anthropic",
            "claude-haiku-3-5",
            0.8,
            4.0,
            Some(0.08),
            Some(1.0),
            350,
        ),
    ];
    for (provider, model, prompt, completion, cache_read, cache_write, sort_order) in prices {
        connection
            .execute(
                "INSERT OR IGNORE INTO model_pricing_cache (
                    provider, model, status, prompt_usd_per_token,
                    completion_usd_per_token, cache_read_usd_per_token,
                    cache_write_usd_per_token, source, fetched_at, expires_at, error_kind,
                    hidden, sort_order
                 ) VALUES (?1, ?2, 'success', ?3, ?4, ?5, ?6,
                    'builtin-official', 0, ?7, NULL, ?8, ?9)
                 ON CONFLICT(provider, model) DO UPDATE SET
                    status = excluded.status,
                    prompt_usd_per_token = excluded.prompt_usd_per_token,
                    completion_usd_per_token = excluded.completion_usd_per_token,
                    cache_read_usd_per_token = excluded.cache_read_usd_per_token,
                    cache_write_usd_per_token = excluded.cache_write_usd_per_token,
                    source = excluded.source,
                    fetched_at = excluded.fetched_at,
                    expires_at = excluded.expires_at,
                    error_kind = excluded.error_kind,
                    sort_order = excluded.sort_order
                 WHERE model_pricing_cache.source <> 'user'",
                params![
                    provider,
                    model,
                    prompt / 1_000_000.0,
                    completion / 1_000_000.0,
                    cache_read.map(|value| value / 1_000_000.0),
                    cache_write.map(|value| value / 1_000_000.0),
                    i64::MAX,
                    model_hidden_by_default(provider, model),
                    sort_order,
                ],
            )
            .map_err(|error| format!("failed to seed built-in model pricing: {error}"))?;
    }
    Ok(())
}

pub(crate) fn model_pricing_cache(
    connection: &Connection,
    provider: &str,
    model: &str,
) -> Result<Option<ModelPricingCacheRecord>, String> {
    connection
        .query_row(
            "SELECT provider, model, status, prompt_usd_per_token,
                    completion_usd_per_token, cache_read_usd_per_token,
                    cache_write_usd_per_token, source, fetched_at, expires_at, error_kind,
                    hidden, sort_order
             FROM model_pricing_cache
             WHERE provider = ?1 AND model = ?2",
            params![provider, model],
            |row| {
                Ok(ModelPricingCacheRecord {
                    provider: row.get(0)?,
                    model: row.get(1)?,
                    status: row.get(2)?,
                    prompt_usd_per_token: row.get(3)?,
                    completion_usd_per_token: row.get(4)?,
                    cache_read_usd_per_token: row.get(5)?,
                    cache_write_usd_per_token: row.get(6)?,
                    source: row.get(7)?,
                    fetched_at: row.get(8)?,
                    expires_at: row.get(9)?,
                    error_kind: row.get(10)?,
                    hidden: row.get(11)?,
                    sort_order: row.get(12)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("failed to read model pricing cache: {error}"))
}

pub(crate) fn upsert_model_pricing_cache(
    connection: &Connection,
    record: &ModelPricingCacheRecord,
) -> Result<(), String> {
    let hidden = record.hidden || model_hidden_by_default(&record.provider, &record.model);
    connection
        .execute(
            "INSERT INTO model_pricing_cache (
                provider, model, status, prompt_usd_per_token,
                completion_usd_per_token, cache_read_usd_per_token,
                cache_write_usd_per_token, source, fetched_at, expires_at, error_kind,
                hidden, sort_order
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(provider, model) DO UPDATE SET
                status = excluded.status,
                prompt_usd_per_token = excluded.prompt_usd_per_token,
                completion_usd_per_token = excluded.completion_usd_per_token,
                cache_read_usd_per_token = excluded.cache_read_usd_per_token,
                cache_write_usd_per_token = excluded.cache_write_usd_per_token,
                source = excluded.source,
                fetched_at = excluded.fetched_at,
                expires_at = excluded.expires_at,
                error_kind = excluded.error_kind,
                sort_order = MAX(model_pricing_cache.sort_order, excluded.sort_order)",
            params![
                record.provider,
                record.model,
                record.status,
                record.prompt_usd_per_token,
                record.completion_usd_per_token,
                record.cache_read_usd_per_token,
                record.cache_write_usd_per_token,
                record.source,
                record.fetched_at,
                record.expires_at,
                record.error_kind,
                hidden,
                record.sort_order,
            ],
        )
        .map(|_| ())
        .map_err(|error| format!("failed to write model pricing cache: {error}"))
}

pub(crate) fn list_model_pricing_cache(
    connection: &Connection,
) -> Result<Vec<ModelPricingCacheRecord>, String> {
    let mut statement = connection
        .prepare(
            "SELECT provider, model, status, prompt_usd_per_token,
                    completion_usd_per_token, cache_read_usd_per_token,
                    cache_write_usd_per_token, source, fetched_at, expires_at, error_kind,
                    hidden, sort_order
             FROM model_pricing_cache
             ORDER BY provider, sort_order DESC, model DESC",
        )
        .map_err(|error| format!("failed to prepare model pricing list: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok(ModelPricingCacheRecord {
                provider: row.get(0)?,
                model: row.get(1)?,
                status: row.get(2)?,
                prompt_usd_per_token: row.get(3)?,
                completion_usd_per_token: row.get(4)?,
                cache_read_usd_per_token: row.get(5)?,
                cache_write_usd_per_token: row.get(6)?,
                source: row.get(7)?,
                fetched_at: row.get(8)?,
                expires_at: row.get(9)?,
                error_kind: row.get(10)?,
                hidden: row.get(11)?,
                sort_order: row.get(12)?,
            })
        })
        .map_err(|error| format!("failed to query model pricing list: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read model pricing list: {error}"))?;
    Ok(rows)
}

pub(crate) fn set_model_pricing_hidden(
    connection: &Connection,
    provider: &str,
    model: &str,
    hidden: bool,
) -> Result<bool, String> {
    connection
        .execute(
            "UPDATE model_pricing_cache SET hidden = ?3 WHERE provider = ?1 AND model = ?2",
            params![provider, model, hidden],
        )
        .map(|affected| affected > 0)
        .map_err(|error| format!("failed to update model pricing visibility: {error}"))
}

pub(crate) fn delete_manual_model_pricing_record(
    connection: &Connection,
    provider: &str,
    model: &str,
) -> Result<bool, String> {
    connection
        .execute(
            "DELETE FROM model_pricing_cache
             WHERE provider = ?1 AND model = ?2 AND source = 'user'",
            params![provider, model],
        )
        .map(|affected| affected > 0)
        .map_err(|error| format!("failed to delete manual model pricing: {error}"))
}

pub(crate) fn distinct_unpriced_models(
    connection: &Connection,
    limit: usize,
) -> Result<Vec<(String, Option<String>, String)>, String> {
    let mut statement = connection
        .prepare(
            "SELECT DISTINCT provider, model_provider_id, model
             FROM usage_events
             WHERE estimated_usd_micros IS NULL
               AND model IS NOT NULL AND model != '' AND model != 'unknown'
             ORDER BY provider, model_provider_id, model
             LIMIT ?1",
        )
        .map_err(|error| format!("failed to prepare unpriced model query: {error}"))?;
    let rows = statement
        .query_map([i64::try_from(limit).unwrap_or(i64::MAX)], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|error| format!("failed to query unpriced models: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read unpriced model: {error}"))?;
    Ok(rows)
}

pub(crate) fn unpriced_usage_events_for_model(
    connection: &Connection,
    provider: &str,
    model_provider_id: Option<&str>,
    model: &str,
) -> Result<Vec<UsageEventRecord>, String> {
    let mut statement = connection
        .prepare(
            "SELECT provider, model_provider_id, session_id, source_event_id, occurred_at,
                    cwd, model, input_tokens, output_tokens, cache_read_tokens,
                    cache_write_tokens, reasoning_tokens, estimated_usd_micros,
                    estimated_usd_price_version, cost_points_micros, source_kind, parser_version,
                    service_tier
             FROM usage_events
             WHERE provider = ?1 AND model_provider_id IS ?2 AND model = ?3
               AND estimated_usd_micros IS NULL",
        )
        .map_err(|error| format!("failed to prepare unpriced event query: {error}"))?;
    let events = statement
        .query_map(params![provider, model_provider_id, model], |row| {
            Ok(UsageEventRecord {
                provider: row.get(0)?,
                model_provider_id: row.get(1)?,
                session_id: row.get(2)?,
                source_event_id: row.get(3)?,
                occurred_at: row.get(4)?,
                cwd: row.get(5)?,
                model: row.get(6)?,
                input_tokens: row.get(7)?,
                output_tokens: row.get(8)?,
                cache_read_tokens: row.get(9)?,
                cache_write_tokens: row.get(10)?,
                reasoning_tokens: row.get(11)?,
                estimated_usd_micros: row.get(12)?,
                estimated_usd_price_version: row.get(13)?,
                cost_points_micros: row.get(14)?,
                source_kind: row.get(15)?,
                parser_version: row.get(16)?,
                service_tier: row.get(17)?,
            })
        })
        .map_err(|error| format!("failed to query unpriced events: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read unpriced event: {error}"))?;
    Ok(events)
}

pub(crate) fn usage_events_for_model(
    connection: &Connection,
    model: &str,
) -> Result<Vec<UsageEventRecord>, String> {
    let mut statement = connection
        .prepare(
            "SELECT provider, model_provider_id, session_id, source_event_id, occurred_at,
                    cwd, model, input_tokens, output_tokens, cache_read_tokens,
                    cache_write_tokens, reasoning_tokens, estimated_usd_micros,
                    estimated_usd_price_version, cost_points_micros, source_kind, parser_version,
                    service_tier
             FROM usage_events
             WHERE lower(model) = lower(?1)",
        )
        .map_err(|error| format!("failed to prepare model event query: {error}"))?;
    let events = statement
        .query_map([model], |row| {
            Ok(UsageEventRecord {
                provider: row.get(0)?,
                model_provider_id: row.get(1)?,
                session_id: row.get(2)?,
                source_event_id: row.get(3)?,
                occurred_at: row.get(4)?,
                cwd: row.get(5)?,
                model: row.get(6)?,
                input_tokens: row.get(7)?,
                output_tokens: row.get(8)?,
                cache_read_tokens: row.get(9)?,
                cache_write_tokens: row.get(10)?,
                reasoning_tokens: row.get(11)?,
                estimated_usd_micros: row.get(12)?,
                estimated_usd_price_version: row.get(13)?,
                cost_points_micros: row.get(14)?,
                source_kind: row.get(15)?,
                parser_version: row.get(16)?,
                service_tier: row.get(17)?,
            })
        })
        .map_err(|error| format!("failed to query model events: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read model event: {error}"))?;
    Ok(events)
}

pub(crate) fn upsert_usage_events(
    connection: &Connection,
    events: &[UsageEventRecord],
) -> Result<u64, String> {
    if events.is_empty() {
        return Ok(0);
    }
    validate_usage_events(events)?;

    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("failed to begin usage event upsert: {error}"))?;
    let changed = insert_usage_events(&transaction, events)?;
    if changed > 0 {
        bump_analytics_revision(&transaction)?;
    }
    transaction
        .commit()
        .map_err(|error| format!("failed to commit usage event upsert: {error}"))?;
    Ok(changed)
}

#[cfg(test)]
pub(crate) fn replace_usage_events_for_session(
    connection: &Connection,
    provider: &str,
    session_id: &str,
    events: &[UsageEventRecord],
) -> Result<(), String> {
    if provider.trim().is_empty() || session_id.trim().is_empty() {
        return Err("provider and sessionId are required".to_string());
    }
    if events
        .iter()
        .any(|event| event.provider != provider || event.session_id != session_id)
    {
        return Err("usage event does not match replacement provider/session".to_string());
    }
    validate_usage_events(events)?;

    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("failed to begin usage session replacement: {error}"))?;
    let removed = transaction
        .execute(
            "DELETE FROM usage_events WHERE provider = ?1 AND session_id = ?2",
            params![provider, session_id],
        )
        .map_err(|error| format!("failed to clear usage session events: {error}"))?;
    let inserted = insert_usage_events(&transaction, events)?;
    if removed > 0 || inserted > 0 {
        bump_analytics_revision(&transaction)?;
    }
    transaction
        .commit()
        .map_err(|error| format!("failed to commit usage session replacement: {error}"))
}

pub(crate) fn usage_ingestion_state(
    connection: &Connection,
    provider: &str,
    session_id: &str,
    source_identity: &str,
) -> Result<Option<(Option<String>, i64, String)>, String> {
    connection
        .query_row(
            "SELECT source_fingerprint, parser_version, ingestion_status
             FROM usage_ingestion_state
             WHERE provider = ?1 AND session_id = ?2 AND source_identity = ?3",
            params![provider, session_id, source_identity],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|error| format!("failed to query usage ingestion state: {error}"))
}

#[cfg(test)]
pub(crate) fn replace_usage_session_data(
    connection: &Connection,
    state: &UsageIngestionStateRecord,
    events: &[UsageEventRecord],
    summaries: &[UsageSessionSummaryRecord],
) -> Result<(), String> {
    replace_usage_sessions_batch(
        connection,
        &[(state.clone(), events.to_vec(), summaries.to_vec())],
    )
}

pub(crate) fn replace_usage_sessions_batch(
    connection: &Connection,
    sessions: &[(
        UsageIngestionStateRecord,
        Vec<UsageEventRecord>,
        Vec<UsageSessionSummaryRecord>,
    )],
) -> Result<(), String> {
    if sessions.is_empty() {
        return Ok(());
    }
    for (state, events, summaries) in sessions {
        validate_usage_session_data(state, events, summaries)?;
    }
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("failed to begin usage batch commit: {error}"))?;
    for (state, events, summaries) in sessions {
        replace_usage_session_data_in_transaction(&transaction, state, events, summaries)?;
    }
    bump_analytics_revision(&transaction)?;
    transaction
        .commit()
        .map_err(|error| format!("failed to commit usage session batch: {error}"))
}

fn validate_usage_session_data(
    state: &UsageIngestionStateRecord,
    events: &[UsageEventRecord],
    summaries: &[UsageSessionSummaryRecord],
) -> Result<(), String> {
    if events
        .iter()
        .any(|event| event.provider != state.provider || event.session_id != state.session_id)
        || summaries.iter().any(|summary| {
            summary.provider != state.provider || summary.session_id != state.session_id
        })
    {
        return Err("usage session data does not match ingestion identity".to_string());
    }
    validate_usage_events(events)?;
    Ok(())
}

fn replace_usage_session_data_in_transaction(
    transaction: &Transaction<'_>,
    state: &UsageIngestionStateRecord,
    events: &[UsageEventRecord],
    summaries: &[UsageSessionSummaryRecord],
) -> Result<(), String> {
    transaction
        .execute(
            "DELETE FROM usage_events WHERE provider = ?1 AND session_id = ?2",
            params![state.provider, state.session_id],
        )
        .map_err(|error| format!("failed to replace usage events: {error}"))?;
    transaction
        .execute(
            "DELETE FROM usage_session_summaries WHERE provider = ?1 AND session_id = ?2",
            params![state.provider, state.session_id],
        )
        .map_err(|error| format!("failed to replace usage summaries: {error}"))?;
    transaction
        .execute(
            "DELETE FROM usage_ingestion_state
             WHERE provider = ?1 AND session_id = ?2 AND source_identity != ?3",
            params![state.provider, state.session_id, state.source_identity],
        )
        .map_err(|error| format!("failed to replace usage source state: {error}"))?;
    insert_usage_events(&transaction, events)?;
    for summary in summaries {
        transaction
            .execute(
                "INSERT INTO usage_session_summaries (
                    provider, session_id, source_identity, cwd, model, active_from, active_until,
                    input_tokens, output_tokens, estimated_usd_micros, cost_points_micros,
                    source_kind, parser_version, integrity_status
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    summary.provider,
                    summary.session_id,
                    summary.source_identity,
                    summary.cwd,
                    summary.model,
                    summary.active_from,
                    summary.active_until,
                    summary.input_tokens,
                    summary.output_tokens,
                    summary.estimated_usd_micros,
                    summary.cost_points_micros,
                    summary.source_kind,
                    summary.parser_version,
                    summary.integrity_status,
                ],
            )
            .map_err(|error| format!("failed to persist usage session summary: {error}"))?;
    }
    transaction
        .execute(
            "INSERT INTO usage_ingestion_state (
                provider, session_id, source_identity, source_fingerprint, parser_version,
                integrity_status, ingestion_status, session_revision, error_message
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8)
            ON CONFLICT(provider, session_id, source_identity) DO UPDATE SET
                source_fingerprint = excluded.source_fingerprint,
                parser_version = excluded.parser_version,
                integrity_status = excluded.integrity_status,
                ingestion_status = excluded.ingestion_status,
                session_revision = usage_ingestion_state.session_revision + 1,
                error_message = excluded.error_message,
                updated_at = CURRENT_TIMESTAMP",
            params![
                state.provider,
                state.session_id,
                state.source_identity,
                state.source_fingerprint,
                state.parser_version,
                state.integrity_status,
                state.ingestion_status,
                state.error_message,
            ],
        )
        .map_err(|error| format!("failed to persist usage ingestion state: {error}"))?;
    Ok(())
}

pub(crate) fn cleanup_usage_for_missing_sessions(
    connection: &Connection,
    providers: &[String],
) -> Result<usize, String> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("failed to begin usage cleanup: {error}"))?;
    let mut removed = 0usize;
    for provider in providers {
        for table in [
            "usage_events",
            "usage_session_summaries",
            "usage_ingestion_state",
        ] {
            let query = format!(
                "DELETE FROM {table} WHERE provider = ?1
                 AND NOT EXISTS (
                    SELECT 1 FROM sessions_cache sc
                    WHERE sc.provider = ?1 AND sc.session_id = {table}.session_id
                 )"
            );
            let count = transaction.execute(&query, [provider]).map_err(|error| {
                format!("failed to clean stale usage rows from {table}: {error}")
            })?;
            removed += count;
        }
    }
    if removed > 0 {
        bump_analytics_revision(&transaction)?;
    }
    transaction
        .commit()
        .map_err(|error| format!("failed to commit usage cleanup: {error}"))?;
    Ok(removed)
}

#[cfg(test)]
pub(crate) fn usage_index_progress(connection: &Connection) -> Result<UsageIndexProgress, String> {
    connection
        .query_row(
            "
            SELECT
                COUNT(*),
                SUM(CASE WHEN state.ingestion_status IN ('complete', 'partial', 'summary_only') THEN 1 ELSE 0 END),
                SUM(CASE WHEN state.ingestion_status = 'pending' THEN 1 ELSE 0 END),
                SUM(CASE WHEN state.ingestion_status = 'unsupported' THEN 1 ELSE 0 END),
                SUM(CASE WHEN state.ingestion_status = 'error' THEN 1 ELSE 0 END),
                SUM(CASE WHEN state.provider IS NULL THEN 1 ELSE 0 END)
            FROM sessions_cache sc
            LEFT JOIN (
                SELECT provider, session_id, MAX(updated_at) AS latest_update,
                       ingestion_status
                FROM usage_ingestion_state
                GROUP BY provider, session_id
            ) state ON state.provider = sc.provider AND state.session_id = sc.session_id
            ",
            [],
            |row| {
                Ok(UsageIndexProgress {
                    total_sessions: i64_to_u64(row.get::<_, Option<i64>>(0)?.unwrap_or(0)),
                    indexed_sessions: i64_to_u64(row.get::<_, Option<i64>>(1)?.unwrap_or(0)),
                    pending_sessions: i64_to_u64(row.get::<_, Option<i64>>(2)?.unwrap_or(0)),
                    unsupported_sessions: i64_to_u64(row.get::<_, Option<i64>>(3)?.unwrap_or(0)),
                    error_sessions: i64_to_u64(row.get::<_, Option<i64>>(4)?.unwrap_or(0)),
                    unindexed_sessions: i64_to_u64(row.get::<_, Option<i64>>(5)?.unwrap_or(0)),
                })
            },
        )
        .map_err(|error| format!("failed to query usage index progress: {error}"))
}

#[cfg(test)]
pub(crate) fn rebuild_usage_derived_data(connection: &Connection) -> Result<(), String> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("failed to begin usage index rebuild: {error}"))?;
    let mut changed = 0usize;
    for table in [
        "usage_events",
        "usage_session_summaries",
        "usage_ingestion_state",
    ] {
        changed += transaction
            .execute(&format!("DELETE FROM {table}"), [])
            .map_err(|error| format!("failed to clear derived usage table {table}: {error}"))?;
    }
    if changed > 0 {
        bump_analytics_revision(&transaction)?;
    }
    transaction
        .commit()
        .map_err(|error| format!("failed to commit usage index rebuild: {error}"))
}

pub(crate) fn read_analytics_revision_internal(connection: &Connection) -> Result<u64, String> {
    connection
        .query_row(
            "SELECT revision FROM analytics_revision WHERE singleton = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|revision| i64_to_u64(revision))
        .map_err(|error| format!("failed to read analytics revision: {error}"))
}

pub(crate) fn update_usage_ingestion_status(
    connection: &Connection,
    state: &UsageIngestionStateRecord,
) -> Result<(), String> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("failed to begin usage state update: {error}"))?;
    transaction
        .execute(
            "INSERT INTO usage_ingestion_state (
                provider, session_id, source_identity, source_fingerprint, parser_version,
                integrity_status, ingestion_status, session_revision, error_message
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8)
            ON CONFLICT(provider, session_id, source_identity) DO UPDATE SET
                source_fingerprint = excluded.source_fingerprint,
                parser_version = excluded.parser_version,
                integrity_status = excluded.integrity_status,
                ingestion_status = excluded.ingestion_status,
                session_revision = usage_ingestion_state.session_revision + 1,
                error_message = excluded.error_message,
                updated_at = CURRENT_TIMESTAMP",
            params![
                state.provider,
                state.session_id,
                state.source_identity,
                state.source_fingerprint,
                state.parser_version,
                state.integrity_status,
                state.ingestion_status,
                state.error_message,
            ],
        )
        .map_err(|error| format!("failed to update usage ingestion state: {error}"))?;
    bump_analytics_revision(&transaction)?;
    transaction
        .commit()
        .map_err(|error| format!("failed to commit usage state update: {error}"))
}

fn validate_usage_events(events: &[UsageEventRecord]) -> Result<(), String> {
    for event in events {
        if event.provider.trim().is_empty()
            || event.session_id.trim().is_empty()
            || event.source_event_id.trim().is_empty()
            || event.occurred_at.trim().is_empty()
            || event.source_kind.trim().is_empty()
        {
            return Err("usage event identity, timestamp and sourceKind are required".to_string());
        }
    }
    Ok(())
}

fn insert_usage_events(
    connection: &Connection,
    events: &[UsageEventRecord],
) -> Result<u64, String> {
    let mut changed = 0_u64;
    for event in events {
        let affected = connection
            .execute(
                "
                INSERT INTO usage_events (
                    provider, session_id, source_event_id, occurred_at, cwd, model, model_provider_id,
                    input_tokens, output_tokens, cache_read_tokens, cache_write_tokens,
                    reasoning_tokens, estimated_usd_micros, estimated_usd_price_version,
                    cost_points_micros, source_kind, parser_version, service_tier
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18
                )
                ON CONFLICT(provider, session_id, source_event_id) DO UPDATE SET
                    occurred_at = excluded.occurred_at,
                    cwd = excluded.cwd,
                    model = excluded.model,
                    model_provider_id = excluded.model_provider_id,
                    input_tokens = excluded.input_tokens,
                    output_tokens = excluded.output_tokens,
                    cache_read_tokens = excluded.cache_read_tokens,
                    cache_write_tokens = excluded.cache_write_tokens,
                    reasoning_tokens = excluded.reasoning_tokens,
                    estimated_usd_micros = excluded.estimated_usd_micros,
                    estimated_usd_price_version = excluded.estimated_usd_price_version,
                    cost_points_micros = excluded.cost_points_micros,
                    source_kind = excluded.source_kind,
                    parser_version = excluded.parser_version,
                    service_tier = excluded.service_tier
                WHERE usage_events.occurred_at IS NOT excluded.occurred_at
                   OR usage_events.cwd IS NOT excluded.cwd
                   OR usage_events.model IS NOT excluded.model
                   OR usage_events.model_provider_id IS NOT excluded.model_provider_id
                   OR usage_events.input_tokens IS NOT excluded.input_tokens
                   OR usage_events.output_tokens IS NOT excluded.output_tokens
                   OR usage_events.cache_read_tokens IS NOT excluded.cache_read_tokens
                   OR usage_events.cache_write_tokens IS NOT excluded.cache_write_tokens
                   OR usage_events.reasoning_tokens IS NOT excluded.reasoning_tokens
                   OR usage_events.estimated_usd_micros IS NOT excluded.estimated_usd_micros
                   OR usage_events.estimated_usd_price_version IS NOT excluded.estimated_usd_price_version
                   OR usage_events.cost_points_micros IS NOT excluded.cost_points_micros
                   OR usage_events.source_kind IS NOT excluded.source_kind
                   OR usage_events.parser_version IS NOT excluded.parser_version
                   OR usage_events.service_tier IS NOT excluded.service_tier
                ",
                params![
                    event.provider,
                    event.session_id,
                    event.source_event_id,
                    event.occurred_at,
                    event.cwd,
                    event.model,
                    event.model_provider_id,
                    event.input_tokens,
                    event.output_tokens,
                    event.cache_read_tokens,
                    event.cache_write_tokens,
                    event.reasoning_tokens,
                    event.estimated_usd_micros,
                    event.estimated_usd_price_version,
                    event.cost_points_micros,
                    event.source_kind,
                    event.parser_version,
                    event.service_tier,
                ],
            )
            .map_err(|error| format!("failed to upsert usage event: {error}"))?;
        changed += u64::try_from(affected).unwrap_or(0);
    }
    Ok(changed)
}

fn bump_analytics_revision(connection: &Connection) -> Result<(), String> {
    connection
        .execute(
            "UPDATE analytics_revision SET revision = revision + 1 WHERE singleton = 1",
            [],
        )
        .map_err(|error| format!("failed to increment analytics revision: {error}"))?;
    Ok(())
}

pub(crate) fn normalize_remap_path(path: &str) -> String {
    let normalized = path.trim().replace('/', "\\");
    let trimmed = normalized.trim_end_matches('\\');
    if trimmed.len() == 2 && trimmed.ends_with(':') {
        normalized.to_lowercase()
    } else {
        trimmed.to_lowercase()
    }
}

pub(crate) fn list_path_remaps(connection: &Connection) -> Result<Vec<ProjectPathRemap>, String> {
    let mut statement = connection
        .prepare("SELECT old_path, new_path, created_at FROM project_path_remap ORDER BY created_at DESC")
        .map_err(|error| format!("failed to prepare path remap query: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok(ProjectPathRemap {
                old_path: row.get(0)?,
                new_path: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|error| format!("failed to query path remaps: {error}"))?;

    rows.map(|row| row.map_err(|error| format!("failed to read path remap row: {error}")))
        .collect()
}

pub(crate) fn upsert_path_remap(
    connection: &Connection,
    old_path: &str,
    new_path: &str,
) -> Result<(), String> {
    let old_path_key = normalize_remap_path(old_path);
    if old_path_key.is_empty() {
        return Err("old path must not be empty".to_string());
    }

    connection
        .execute(
            "
            INSERT INTO project_path_remap (old_path_key, old_path, new_path)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(old_path_key) DO UPDATE SET
                old_path = excluded.old_path,
                new_path = excluded.new_path
            ",
            params![old_path_key, old_path.trim(), new_path.trim()],
        )
        .map_err(|error| format!("failed to upsert path remap: {error}"))?;
    Ok(())
}

pub(crate) fn delete_path_remap(connection: &Connection, old_path: &str) -> Result<(), String> {
    connection
        .execute(
            "DELETE FROM project_path_remap WHERE old_path_key = ?1",
            params![normalize_remap_path(old_path)],
        )
        .map_err(|error| format!("failed to delete path remap: {error}"))?;
    Ok(())
}

pub(crate) fn remap_path(path: &str, remaps: &[ProjectPathRemap]) -> String {
    let normalized_path = normalize_remap_path(path);
    for remap in remaps {
        let old_path_key = normalize_remap_path(&remap.old_path);
        if normalized_path == old_path_key {
            return remap.new_path.clone();
        }
        if let Some(suffix) = path.get(old_path_key.len()..) {
            if normalized_path.starts_with(&(old_path_key + "\\")) {
                return format!("{}{}", remap.new_path.trim_end_matches(['\\', '/']), suffix);
            }
        }
    }
    path.to_string()
}

pub(crate) fn load_quota_snapshots_from_db(
    connection: &Connection,
) -> Result<Vec<crate::types::QuotaSnapshot>, String> {
    let mut statement = connection
        .prepare("SELECT snapshot_json FROM quota_snapshots")
        .map_err(|e| format!("failed to prepare quota_snapshots query: {e}"))?;

    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("failed to query quota_snapshots: {e}"))?;

    let mut snapshots = Vec::new();
    for row_result in rows {
        let json = row_result.map_err(|e| format!("failed to read quota_snapshots row: {e}"))?;
        if let Ok(snapshot) = serde_json::from_str::<crate::types::QuotaSnapshot>(&json) {
            snapshots.push(snapshot);
        }
    }
    Ok(snapshots)
}

pub(crate) fn save_quota_snapshot_to_db(
    connection: &Connection,
    snapshot: &crate::types::QuotaSnapshot,
) -> Result<(), String> {
    let json = serde_json::to_string(snapshot)
        .map_err(|e| format!("failed to serialize quota snapshot: {e}"))?;
    connection
        .execute(
            "INSERT INTO quota_snapshots (provider, snapshot_json, fetched_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(provider) DO UPDATE SET snapshot_json = excluded.snapshot_json, fetched_at = excluded.fetched_at",
            params![snapshot.provider, json, snapshot.fetched_at],
        )
        .map_err(|e| format!("failed to save quota snapshot: {e}"))?;
    Ok(())
}

pub(crate) fn delete_quota_snapshots_for_provider(
    connection: &Connection,
    provider: &str,
) -> Result<(), String> {
    connection
        .execute(
            "DELETE FROM quota_snapshots WHERE provider = ?1",
            params![provider],
        )
        .map_err(|e| format!("failed to delete quota snapshot: {e}"))?;
    Ok(())
}

pub(crate) fn billing_period_for(reset_day: u8, now: &chrono::NaiveDate) -> String {
    let day = reset_day.max(1).min(28) as u32;
    if now.day() >= day {
        format!("{}-{:02}", now.format("%Y-%m"), day)
    } else {
        let prev = if now.month() == 1 {
            chrono::NaiveDate::from_ymd_opt(now.year() - 1, 12, day).unwrap_or(*now)
        } else {
            chrono::NaiveDate::from_ymd_opt(now.year(), now.month() - 1, day).unwrap_or(*now)
        };
        format!("{}-{:02}", prev.format("%Y-%m"), day)
    }
}

pub(crate) fn next_reset_date_for(reset_day: u8, now: &chrono::NaiveDate) -> String {
    let day = reset_day.max(1).min(28) as u32;
    if now.day() < day {
        chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), day)
            .map(|d| d.to_string())
            .unwrap_or_default()
    } else {
        let (year, month) = if now.month() == 12 {
            (now.year() + 1, 1)
        } else {
            (now.year(), now.month() + 1)
        };
        chrono::NaiveDate::from_ymd_opt(year, month, day)
            .map(|d| d.to_string())
            .unwrap_or_default()
    }
}

pub(crate) fn upsert_provider_quota(
    connection: &Connection,
    provider: &str,
    billing_period: &str,
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_tokens: u64,
    cache_read_tokens: u64,
    cost_usd: f64,
) -> Result<(), String> {
    connection
        .execute(
            "
            INSERT INTO provider_quota (provider, billing_period, input_tokens, output_tokens, cache_creation_tokens, cache_read_tokens, cost_usd)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(provider, billing_period) DO UPDATE SET
                input_tokens = excluded.input_tokens,
                output_tokens = excluded.output_tokens,
                cache_creation_tokens = excluded.cache_creation_tokens,
                cache_read_tokens = excluded.cache_read_tokens,
                cost_usd = excluded.cost_usd
            ",
            rusqlite::params![provider, billing_period, input_tokens as i64, output_tokens as i64, cache_creation_tokens as i64, cache_read_tokens as i64, cost_usd],
        )
        .map_err(|e| format!("failed to upsert provider quota: {e}"))?;
    Ok(())
}

pub(crate) fn get_provider_quota_from_db(
    connection: &Connection,
    provider: &str,
    billing_period: &str,
) -> Result<Option<(u64, u64, u64, u64, f64)>, String> {
    let mut stmt = connection
        .prepare("SELECT input_tokens, output_tokens, cache_creation_tokens, cache_read_tokens, cost_usd FROM provider_quota WHERE provider = ?1 AND billing_period = ?2")
        .map_err(|e| format!("failed to prepare quota query: {e}"))?;
    let mut rows = stmt
        .query(rusqlite::params![provider, billing_period])
        .map_err(|e| format!("failed to query provider quota: {e}"))?;
    if let Some(row) = rows
        .next()
        .map_err(|e| format!("failed to read quota row: {e}"))?
    {
        Ok(Some((
            row.get::<_, i64>(0).unwrap_or(0) as u64,
            row.get::<_, i64>(1).unwrap_or(0) as u64,
            row.get::<_, i64>(2).unwrap_or(0) as u64,
            row.get::<_, i64>(3).unwrap_or(0) as u64,
            row.get::<_, f64>(4).unwrap_or(0.0),
        )))
    } else {
        Ok(None)
    }
}

pub(crate) fn get_provider_quota_settings_from_db(
    connection: &Connection,
    provider: &str,
) -> Result<(Option<u64>, Option<f64>, u8), String> {
    let mut stmt = connection
        .prepare("SELECT monthly_limit_tokens, monthly_limit_usd, reset_day FROM provider_quota_settings WHERE provider = ?1")
        .map_err(|e| format!("failed to prepare quota settings query: {e}"))?;
    let mut rows = stmt
        .query(rusqlite::params![provider])
        .map_err(|e| format!("failed to query provider quota settings: {e}"))?;
    if let Some(row) = rows
        .next()
        .map_err(|e| format!("failed to read quota settings row: {e}"))?
    {
        let limit_tokens: Option<i64> = row.get(0).ok();
        let limit_usd: Option<f64> = row.get(1).ok();
        let reset_day: i64 = row.get(2).unwrap_or(1);
        Ok((
            limit_tokens.map(|v| v as u64),
            limit_usd,
            (reset_day.max(1).min(28)) as u8,
        ))
    } else {
        Ok((None, None, 1))
    }
}

pub(crate) fn set_provider_quota_settings_in_db(
    connection: &Connection,
    provider: &str,
    monthly_limit_tokens: Option<u64>,
    monthly_limit_usd: Option<f64>,
    reset_day: u8,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO provider_quota_settings (provider, monthly_limit_tokens, monthly_limit_usd, reset_day) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(provider) DO UPDATE SET monthly_limit_tokens=excluded.monthly_limit_tokens, monthly_limit_usd=excluded.monthly_limit_usd, reset_day=excluded.reset_day",
            rusqlite::params![provider, monthly_limit_tokens.map(|v| v as i64), monthly_limit_usd, reset_day as i64],
        )
        .map_err(|e| format!("failed to set provider quota settings: {e}"))?;
    Ok(())
}

pub(crate) fn migrate_legacy_session_cache(connection: &Connection) -> Result<(), String> {
    let cache_path = legacy_session_cache_path()?;
    if !cache_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&cache_path)
        .map_err(|error| format!("failed to read legacy session cache: {error}"))?;
    let sessions = serde_json::from_str::<Vec<SessionInfo>>(&content)
        .map_err(|error| format!("failed to parse legacy session cache: {error}"))?;
    let providers: Vec<String> = sessions
        .iter()
        .map(|session| session.provider.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    if !providers.is_empty() {
        save_sessions_cache_to_db(connection, &providers, &sessions)?;
    }

    fs::remove_file(&cache_path)
        .map_err(|error| format!("failed to remove legacy session cache: {error}"))?;
    Ok(())
}

pub(crate) fn load_sessions_cache_from_db(
    connection: &Connection,
    provider: Option<&str>,
) -> Result<Vec<SessionInfo>, String> {
    struct CachedSessionRow {
        session_id: String,
        provider: String,
        cwd: Option<String>,
        repo_root: Option<String>,
        repo_name: Option<String>,
        git_branch: Option<String>,
        summary: Option<String>,
        summary_count: Option<u32>,
        created_at: Option<String>,
        updated_at: Option<String>,
        session_dir: String,
        parse_error: i64,
        is_archived: i64,
        has_plan: i64,
        has_events: i64,
    }

    let mut sessions = Vec::new();
    let query = if provider.is_some() {
        "SELECT session_id, provider, cwd, repo_root, repo_name, git_branch, summary, summary_count, created_at, updated_at, \
                session_dir, parse_error, is_archived, has_plan, has_events \
         FROM sessions_cache \
         WHERE provider = ?1"
    } else {
        "SELECT session_id, provider, cwd, repo_root, repo_name, git_branch, summary, summary_count, created_at, updated_at, \
                session_dir, parse_error, is_archived, has_plan, has_events \
         FROM sessions_cache"
    };

    let mut statement = connection
        .prepare(query)
        .map_err(|error| format!("failed to prepare sessions_cache query: {error}"))?;

    let mut cached_rows = Vec::new();
    if let Some(provider) = provider {
        let rows = statement
            .query_map([provider], |row| {
                Ok(CachedSessionRow {
                    session_id: row.get(0)?,
                    provider: row.get(1)?,
                    cwd: row.get(2)?,
                    repo_root: row.get(3)?,
                    repo_name: row.get(4)?,
                    git_branch: row.get(5)?,
                    summary: row.get(6)?,
                    summary_count: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    session_dir: row.get(10)?,
                    parse_error: row.get(11)?,
                    is_archived: row.get(12)?,
                    has_plan: row.get(13)?,
                    has_events: row.get(14)?,
                })
            })
            .map_err(|error| format!("failed to query sessions_cache: {error}"))?;
        for row_result in rows {
            cached_rows.push(
                row_result
                    .map_err(|error| format!("failed to read sessions_cache row: {error}"))?,
            );
        }
    } else {
        let rows = statement
            .query_map([], |row| {
                Ok(CachedSessionRow {
                    session_id: row.get(0)?,
                    provider: row.get(1)?,
                    cwd: row.get(2)?,
                    repo_root: row.get(3)?,
                    repo_name: row.get(4)?,
                    git_branch: row.get(5)?,
                    summary: row.get(6)?,
                    summary_count: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    session_dir: row.get(10)?,
                    parse_error: row.get(11)?,
                    is_archived: row.get(12)?,
                    has_plan: row.get(13)?,
                    has_events: row.get(14)?,
                })
            })
            .map_err(|error| format!("failed to query sessions_cache: {error}"))?;
        for row_result in rows {
            cached_rows.push(
                row_result
                    .map_err(|error| format!("failed to read sessions_cache row: {error}"))?,
            );
        }
    }

    for row in cached_rows {
        let meta = read_session_meta(connection, &row.session_id).unwrap_or(SessionMeta {
            notes: None,
            tags: Vec::new(),
        });

        sessions.push(SessionInfo {
            id: row.session_id,
            provider: row.provider,
            cwd: row.cwd,
            repo_root: row.repo_root,
            repo_name: row.repo_name,
            git_branch: row.git_branch,
            summary: row.summary,
            summary_count: row.summary_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
            session_dir: row.session_dir,
            parse_error: row.parse_error != 0,
            is_archived: row.is_archived != 0,
            notes: meta.notes,
            tags: meta.tags,
            has_plan: row.has_plan != 0,
            has_events: row.has_events != 0,
        });
    }

    Ok(sessions)
}

pub(crate) fn save_sessions_cache_to_db(
    connection: &Connection,
    providers: &[String],
    sessions: &[SessionInfo],
) -> Result<(), String> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(|error| format!("failed to begin sessions_cache update: {error}"))?;
    let mut previous_scope =
        Vec::<(String, String, Option<String>, Option<String>, i64, String)>::new();
    for provider in providers {
        let mut statement = transaction
            .prepare(
                "SELECT session_id, provider, cwd, repo_root, is_archived, session_dir
                 FROM sessions_cache WHERE provider = ?1",
            )
            .map_err(|error| format!("failed to prepare sessions scope query: {error}"))?;
        let rows = statement
            .query_map([provider], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })
            .map_err(|error| format!("failed to query previous analytics scope: {error}"))?;
        previous_scope.extend(
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|error| format!("failed to read previous analytics scope: {error}"))?,
        );
    }
    let mut next_scope = sessions
        .iter()
        .filter(|session| {
            providers
                .iter()
                .any(|provider| provider == &session.provider)
        })
        .map(|session| {
            (
                session.id.clone(),
                session.provider.clone(),
                session.cwd.clone(),
                session.repo_root.clone(),
                if session.is_archived { 1 } else { 0 },
                session.session_dir.clone(),
            )
        })
        .collect::<Vec<_>>();
    previous_scope.sort();
    next_scope.sort();

    for provider in providers {
        transaction
            .execute("DELETE FROM sessions_cache WHERE provider = ?1", [provider])
            .map_err(|error| format!("failed to clear sessions_cache: {error}"))?;
    }

    {
        let mut statement = transaction
            .prepare(
                "
                INSERT INTO sessions_cache (
                    session_id, provider, cwd, repo_root, repo_name, git_branch, summary, summary_count, created_at, updated_at,
                    session_dir, parse_error, is_archived, has_plan, has_events
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                ",
            )
            .map_err(|error| format!("failed to prepare sessions_cache insert: {error}"))?;

        for session in sessions {
            if !providers
                .iter()
                .any(|provider| provider == &session.provider)
            {
                continue;
            }
            statement
                .execute(params![
                    session.id,
                    session.provider,
                    session.cwd,
                    session.repo_root,
                    session.repo_name,
                    session.git_branch,
                    session.summary,
                    session.summary_count,
                    session.created_at,
                    session.updated_at,
                    session.session_dir,
                    if session.parse_error { 1 } else { 0 },
                    if session.is_archived { 1 } else { 0 },
                    if session.has_plan { 1 } else { 0 },
                    if session.has_events { 1 } else { 0 }
                ])
                .map_err(|error| format!("failed to insert sessions_cache row: {error}"))?;
        }
    }

    if previous_scope != next_scope {
        bump_analytics_revision(&transaction)?;
    }
    transaction
        .commit()
        .map_err(|error| format!("failed to commit sessions_cache update: {error}"))
}

pub(crate) fn load_scan_state_from_db(
    connection: &Connection,
    provider: &str,
) -> Result<(i64, i64), String> {
    let mut statement = connection
        .prepare("SELECT last_full_scan_at, last_cursor FROM scan_state WHERE provider = ?1")
        .map_err(|error| format!("failed to prepare scan_state query: {error}"))?;

    let mut rows = statement
        .query(params![provider])
        .map_err(|error| format!("failed to query scan_state: {error}"))?;

    match rows
        .next()
        .map_err(|error| format!("failed to read scan_state row: {error}"))?
    {
        Some(row) => {
            let last_full_scan_at: i64 = row
                .get(0)
                .map_err(|error| format!("failed to read scan_state last_full_scan_at: {error}"))?;
            let last_cursor: i64 = row
                .get(1)
                .map_err(|error| format!("failed to read scan_state last_cursor: {error}"))?;
            Ok((last_full_scan_at, last_cursor))
        }
        None => Ok((0, 0)),
    }
}

pub(crate) fn save_scan_state_to_db(
    connection: &Connection,
    provider: &str,
    last_full_scan_at: i64,
    last_cursor: i64,
) -> Result<(), String> {
    connection
        .execute(
            "
            INSERT INTO scan_state (provider, last_full_scan_at, last_cursor)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(provider) DO UPDATE SET
                last_full_scan_at = excluded.last_full_scan_at,
                last_cursor = excluded.last_cursor
            ",
            params![provider, last_full_scan_at, last_cursor],
        )
        .map_err(|error| format!("failed to upsert scan_state: {error}"))?;
    Ok(())
}

pub(crate) fn load_session_mtimes_from_db(
    connection: &Connection,
    provider: &str,
) -> Result<HashMap<String, i64>, String> {
    let mut statement = connection
        .prepare("SELECT session_id, mtime FROM session_mtimes WHERE provider = ?1")
        .map_err(|error| format!("failed to prepare session_mtimes query: {error}"))?;

    let rows = statement
        .query_map([provider], |row| {
            let session_id: String = row.get(0)?;
            let mtime: i64 = row.get(1)?;
            Ok((session_id, mtime))
        })
        .map_err(|error| format!("failed to query session_mtimes: {error}"))?;

    let mut mtimes = HashMap::new();
    for row_result in rows {
        let (session_id, mtime) =
            row_result.map_err(|error| format!("failed to read session_mtimes row: {error}"))?;
        mtimes.insert(session_id, mtime);
    }

    Ok(mtimes)
}

pub(crate) fn save_session_mtimes_to_db(
    connection: &Connection,
    provider: &str,
    mtimes: &HashMap<String, i64>,
) -> Result<(), String> {
    connection
        .execute(
            "DELETE FROM session_mtimes WHERE provider = ?1",
            params![provider],
        )
        .map_err(|error| format!("failed to clear session_mtimes: {error}"))?;

    let mut statement = connection
        .prepare("INSERT INTO session_mtimes (session_id, provider, mtime) VALUES (?1, ?2, ?3)")
        .map_err(|error| format!("failed to prepare session_mtimes insert: {error}"))?;

    for (session_id, mtime) in mtimes {
        statement
            .execute(params![session_id, provider, mtime])
            .map_err(|error| format!("failed to insert session_mtimes row: {error}"))?;
    }

    Ok(())
}

pub(crate) fn instant_from_unix_secs(stored: i64) -> Instant {
    let now = Instant::now();
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    if stored <= 0 {
        return now
            .checked_sub(Duration::from_secs(FULL_SCAN_THRESHOLD_SECS + 1))
            .unwrap_or(now);
    }
    let stored_secs = stored as u64;
    if stored_secs >= now_secs {
        return now;
    }
    let elapsed = now_secs - stored_secs;
    now.checked_sub(Duration::from_secs(elapsed)).unwrap_or(now)
}

pub(crate) fn persist_provider_cache(
    connection: &Connection,
    provider: &str,
    cache: &ProviderCache,
) -> Result<(), String> {
    let provider_sessions: Vec<SessionInfo> = cache
        .sessions
        .iter()
        .filter(|session| session.provider == provider)
        .cloned()
        .collect();
    let providers = vec![provider.to_string()];
    save_sessions_cache_to_db(connection, &providers, &provider_sessions)?;

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let last_full_scan_at = now_secs
        .saturating_sub(cache.last_full_scan_at.elapsed().as_secs())
        .try_into()
        .unwrap_or(0);
    save_scan_state_to_db(connection, provider, last_full_scan_at, cache.last_cursor)?;
    save_session_mtimes_to_db(connection, provider, &cache.session_mtimes)?;

    Ok(())
}

pub(crate) fn read_session_meta(
    connection: &Connection,
    session_id: &str,
) -> Result<SessionMeta, String> {
    let mut statement = connection
        .prepare("SELECT notes, tags FROM session_meta WHERE session_id = ?1")
        .map_err(|error| format!("failed to prepare metadata query: {error}"))?;

    read_session_meta_with_statement(&mut statement, session_id)
}

fn read_session_meta_with_statement(
    statement: &mut rusqlite::Statement<'_>,
    session_id: &str,
) -> Result<SessionMeta, String> {
    let mut rows = statement
        .query(params![session_id])
        .map_err(|error| format!("failed to query metadata: {error}"))?;

    match rows
        .next()
        .map_err(|error| format!("failed to read metadata row: {error}"))?
    {
        Some(row) => {
            let notes: Option<String> = row
                .get(0)
                .map_err(|error| format!("failed to read notes column: {error}"))?;
            let tags_json: Option<String> = row
                .get(1)
                .map_err(|error| format!("failed to read tags column: {error}"))?;

            let tags = tags_json
                .and_then(|value| serde_json::from_str::<Vec<String>>(&value).ok())
                .unwrap_or_default();

            Ok(SessionMeta { notes, tags })
        }
        None => Ok(SessionMeta {
            notes: None,
            tags: Vec::new(),
        }),
    }
}

pub(crate) fn apply_session_metadata(
    connection: &Connection,
    sessions: &mut [SessionInfo],
) -> Result<(), String> {
    let mut statement = connection
        .prepare("SELECT notes, tags FROM session_meta WHERE session_id = ?1")
        .map_err(|error| format!("failed to prepare metadata query: {error}"))?;

    for session in sessions {
        let meta = read_session_meta_with_statement(&mut statement, &session.id)?;
        session.notes = meta.notes;
        session.tags = meta.tags;
    }
    Ok(())
}

pub(crate) fn upsert_session_meta_internal(
    connection: &Connection,
    session_id: &str,
    notes: Option<String>,
    tags: Vec<String>,
) -> Result<(), String> {
    let tags_json = serde_json::to_string(&tags)
        .map_err(|error| format!("failed to serialize tags: {error}"))?;

    connection
        .execute(
            "
            INSERT INTO session_meta (session_id, notes, tags)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(session_id) DO UPDATE SET
                notes = excluded.notes,
                tags = excluded.tags
            ",
            params![session_id, notes, tags_json],
        )
        .map_err(|error| format!("failed to upsert metadata: {error}"))?;

    Ok(())
}

pub(crate) fn delete_session_meta_internal(
    connection: &Connection,
    session_id: &str,
) -> Result<(), String> {
    connection
        .execute(
            "DELETE FROM session_meta WHERE session_id = ?1",
            params![session_id],
        )
        .map_err(|error| format!("failed to delete metadata: {error}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cached_session(id: &str) -> SessionInfo {
        SessionInfo {
            id: id.to_string(),
            provider: "copilot".to_string(),
            cwd: None,
            repo_root: None,
            repo_name: None,
            git_branch: None,
            summary: Some("summary".to_string()),
            summary_count: Some(1),
            created_at: None,
            updated_at: None,
            session_dir: format!("C:/sessions/{id}"),
            parse_error: false,
            is_archived: false,
            notes: Some("stale note".to_string()),
            tags: vec!["stale".to_string()],
            has_plan: false,
            has_events: true,
        }
    }

    fn remap(old_path: &str, new_path: &str) -> ProjectPathRemap {
        ProjectPathRemap {
            old_path: old_path.to_string(),
            new_path: new_path.to_string(),
            created_at: String::new(),
        }
    }

    #[test]
    fn normalize_remap_path_unifies_separators_case_and_trailing_separator() {
        assert_eq!(normalize_remap_path("D:/Old/Project/"), "d:\\old\\project");
        assert_eq!(normalize_remap_path("D:\\"), "d:\\");
    }

    #[test]
    fn upsert_path_remap_replaces_case_insensitive_old_path() {
        let connection = Connection::open_in_memory().expect("open test db");
        init_db(&connection).expect("initialize test db");
        upsert_path_remap(&connection, "D:\\Old\\Project", "D:\\New\\Project")
            .expect("insert remap");
        upsert_path_remap(&connection, "d:\\old\\project", "D:\\Renamed\\Project")
            .expect("update remap");

        let remaps = list_path_remaps(&connection).expect("list remaps");
        assert_eq!(remaps.len(), 1);
        assert_eq!(remaps[0].new_path, "D:\\Renamed\\Project");
    }

    #[test]
    fn remap_path_matches_directory_boundaries_only() {
        let remaps = vec![remap("D:\\old\\project", "D:\\new\\project")];
        assert_eq!(remap_path("D:\\old\\project", &remaps), "D:\\new\\project");
        assert_eq!(
            remap_path("D:\\old\\project\\Src\\App", &remaps),
            "D:\\new\\project\\Src\\App"
        );
        assert_eq!(
            remap_path("D:\\old\\project2", &remaps),
            "D:\\old\\project2"
        );
        assert_eq!(remap_path("D:\\other", &remaps), "D:\\other");
        assert_eq!(remap_path("D:\\old\\project", &[]), "D:\\old\\project");
    }

    #[test]
    fn session_metadata_updates_and_clears_cached_values() {
        let connection = Connection::open_in_memory().expect("open test db");
        init_db(&connection).expect("initialize test db");
        let sessions = vec![cached_session("session-001"), cached_session("session-002")];
        save_sessions_cache_to_db(&connection, &["copilot".to_string()], &sessions)
            .expect("save sessions cache");

        upsert_session_meta_internal(
            &connection,
            "session-001",
            Some("fresh note".to_string()),
            vec!["fresh".to_string()],
        )
        .expect("insert metadata");
        let loaded = load_sessions_cache_from_db(&connection, Some("copilot"))
            .expect("load sessions after insert");
        assert_eq!(loaded[0].notes.as_deref(), Some("fresh note"));
        assert_eq!(loaded[0].tags, vec!["fresh"]);
        assert_eq!(loaded[1].notes, None);

        upsert_session_meta_internal(&connection, "session-001", None, Vec::new())
            .expect("clear metadata");
        let loaded = load_sessions_cache_from_db(&connection, Some("copilot"))
            .expect("load sessions after clear");
        assert_eq!(loaded[0].notes, None);
        assert!(loaded[0].tags.is_empty());
        assert!(loaded[1].tags.is_empty());
    }

    #[test]
    fn apply_session_metadata_overwrites_stale_provider_values_only_by_id() {
        let connection = Connection::open_in_memory().expect("open test db");
        init_db(&connection).expect("initialize test db");
        upsert_session_meta_internal(
            &connection,
            "session-001",
            Some("database note".to_string()),
            vec!["database".to_string()],
        )
        .expect("insert metadata");
        let mut sessions = vec![cached_session("session-001"), cached_session("session-002")];

        apply_session_metadata(&connection, &mut sessions).expect("apply metadata");

        assert_eq!(sessions[0].notes.as_deref(), Some("database note"));
        assert_eq!(sessions[0].tags, vec!["database"]);
        assert_eq!(sessions[1].notes, None);
        assert!(sessions[1].tags.is_empty());
    }

    fn usage_event(provider: &str, session_id: &str, event_id: &str) -> UsageEventRecord {
        UsageEventRecord {
            provider: provider.to_string(),
            model_provider_id: None,
            service_tier: None,
            session_id: session_id.to_string(),
            source_event_id: event_id.to_string(),
            occurred_at: "2026-04-10T09:00:00Z".to_string(),
            cwd: Some("D:\\workspace".to_string()),
            model: Some("fixture-model".to_string()),
            input_tokens: Some(100),
            output_tokens: Some(20),
            cache_read_tokens: Some(5),
            cache_write_tokens: Some(10),
            reasoning_tokens: Some(2),
            estimated_usd_micros: Some(125),
            estimated_usd_price_version: Some("fixture-v1".to_string()),
            cost_points_micros: None,
            source_kind: "message".to_string(),
            parser_version: 1,
        }
    }

    #[test]
    fn usage_event_upsert_is_idempotent_and_provider_scoped() {
        let connection = Connection::open_in_memory().expect("open usage db");
        init_db(&connection).expect("initialize usage db");
        let claude_event = usage_event("claude", "shared-session", "shared-event");

        assert_eq!(
            upsert_usage_events(&connection, &[claude_event.clone()]).expect("insert"),
            1
        );
        assert_eq!(
            upsert_usage_events(&connection, &[claude_event]).expect("repeat"),
            0
        );
        assert_eq!(
            upsert_usage_events(
                &connection,
                &[usage_event("codex", "shared-session", "shared-event")],
            )
            .expect("same id in another provider"),
            1
        );

        let rows: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM usage_events WHERE session_id = 'shared-session' AND source_event_id = 'shared-event'",
                [],
                |row| row.get(0),
            )
            .expect("count provider-scoped events");
        assert_eq!(rows, 2);
        let revision: i64 = connection
            .query_row(
                "SELECT revision FROM analytics_revision WHERE singleton = 1",
                [],
                |row| row.get(0),
            )
            .expect("read revision");
        assert_eq!(revision, 2);
    }

    #[test]
    fn usage_session_replacement_rolls_back_partial_failure() {
        let connection = Connection::open_in_memory().expect("open usage db");
        init_db(&connection).expect("initialize usage db");
        let original = usage_event("claude", "session-atomic", "original");
        upsert_usage_events(&connection, &[original.clone()]).expect("insert original event");
        let revision_before = connection
            .query_row::<i64, _, _>(
                "SELECT revision FROM analytics_revision WHERE singleton = 1",
                [],
                |row| row.get(0),
            )
            .expect("read revision before replacement");

        let replacement = usage_event("claude", "session-atomic", "replacement");
        let mut invalid = usage_event("claude", "session-atomic", "invalid");
        invalid.parser_version = -1;
        assert!(replace_usage_events_for_session(
            &connection,
            "claude",
            "session-atomic",
            &[replacement, invalid]
        )
        .is_err());

        let event_ids = connection
            .prepare("SELECT source_event_id FROM usage_events WHERE provider = 'claude' AND session_id = 'session-atomic'")
            .expect("prepare event id query")
            .query_map([], |row| row.get::<_, String>(0))
            .expect("query event ids")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect event ids");
        assert_eq!(event_ids, vec!["original"]);
        let revision_after = connection
            .query_row::<i64, _, _>(
                "SELECT revision FROM analytics_revision WHERE singleton = 1",
                [],
                |row| row.get(0),
            )
            .expect("read revision after replacement");
        assert_eq!(revision_after, revision_before);

        replace_usage_events_for_session(
            &connection,
            "claude",
            "session-atomic",
            &[usage_event("claude", "session-atomic", "replacement")],
        )
        .expect("replace session events");
        let replaced_id: String = connection
            .query_row(
                "SELECT source_event_id FROM usage_events WHERE provider = 'claude' AND session_id = 'session-atomic'",
                [],
                |row| row.get(0),
            )
            .expect("read replaced event id");
        assert_eq!(replaced_id, "replacement");
    }

    #[test]
    fn usage_cleanup_waits_for_session_removal_from_a_successful_scan() {
        let connection = Connection::open_in_memory().expect("open usage db");
        init_db(&connection).expect("initialize usage db");
        connection
            .execute_batch(
                "INSERT INTO sessions_cache (session_id, provider, session_dir) VALUES
                    ('retained-session', 'copilot', 'C:/old-root/retained'),
                    ('removed-session', 'copilot', 'C:/old-root/removed'),
                    ('other-provider-session', 'codex', 'C:/codex/session');",
            )
            .expect("seed session cache");
        let events = [
            usage_event("copilot", "retained-session", "event-1"),
            usage_event("copilot", "removed-session", "event-2"),
            usage_event("codex", "other-provider-session", "event-3"),
        ];
        upsert_usage_events(&connection, &events).expect("insert usage events");

        assert_eq!(
            cleanup_usage_for_missing_sessions(&connection, &["copilot".to_string()])
                .expect("cleanup while sources remain cached"),
            0
        );
        connection
            .execute(
                "DELETE FROM sessions_cache WHERE provider = 'copilot' AND session_id = 'removed-session'",
                [],
            )
            .expect("simulate completed scan confirming deletion");
        assert_eq!(
            cleanup_usage_for_missing_sessions(&connection, &["copilot".to_string()])
                .expect("cleanup confirmed deleted session"),
            1
        );
        let remaining: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM usage_events WHERE provider = 'copilot'",
                [],
                |row| row.get(0),
            )
            .expect("count retained Copilot usage");
        assert_eq!(remaining, 1);
        let codex_remaining: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM usage_events WHERE provider = 'codex'",
                [],
                |row| row.get(0),
            )
            .expect("count unrelated provider usage");
        assert_eq!(codex_remaining, 1);
    }

    #[test]
    fn usage_root_switch_replaces_the_same_provider_session_generation() {
        let connection = Connection::open_in_memory().expect("open usage db");
        init_db(&connection).expect("initialize usage db");
        let make_state =
            |source_identity: &str, source_fingerprint: &str| UsageIngestionStateRecord {
                provider: "claude".to_string(),
                session_id: "same-session-id".to_string(),
                source_identity: source_identity.to_string(),
                source_fingerprint: Some(source_fingerprint.to_string()),
                parser_version: 1,
                integrity_status: "partial".to_string(),
                ingestion_status: "partial".to_string(),
                error_message: None,
            };
        let old_event = usage_event("claude", "same-session-id", "message-1");
        replace_usage_session_data(
            &connection,
            &make_state("C:/old-root/session.jsonl", "old-fingerprint"),
            &[old_event],
            &[],
        )
        .expect("store old root generation");
        let mut new_event = usage_event("claude", "same-session-id", "message-1");
        new_event.input_tokens = Some(500);
        new_event.cwd = Some("C:\\new-root".to_string());
        replace_usage_session_data(
            &connection,
            &make_state("C:/new-root/session.jsonl", "new-fingerprint"),
            &[new_event],
            &[],
        )
        .expect("replace with new root generation");

        let source_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM usage_ingestion_state WHERE provider = 'claude' AND session_id = 'same-session-id'",
                [],
                |row| row.get(0),
            )
            .expect("count active root state");
        let (input_tokens, cwd): (i64, String) = connection
            .query_row(
                "SELECT input_tokens, cwd FROM usage_events WHERE provider = 'claude' AND session_id = 'same-session-id'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read active root usage");
        assert_eq!(source_count, 1);
        assert_eq!(input_tokens, 500);
        assert_eq!(cwd, "C:\\new-root");
    }

    #[test]
    fn usage_progress_and_rebuild_only_touch_derived_usage_data() {
        let connection = Connection::open_in_memory().expect("open usage db");
        init_db(&connection).expect("initialize usage db");
        connection
            .execute_batch(
                "INSERT INTO sessions_cache (session_id, provider) VALUES
                    ('complete', 'copilot'), ('pending', 'copilot'),
                    ('unsupported', 'copilot'), ('error', 'copilot'), ('unindexed', 'copilot');
                INSERT INTO usage_ingestion_state (
                    provider, session_id, source_identity, parser_version, integrity_status, ingestion_status
                ) VALUES
                    ('copilot', 'complete', 'root/complete', 1, 'partial', 'partial'),
                    ('copilot', 'pending', 'root/pending', 1, 'pending', 'pending'),
                    ('copilot', 'unsupported', 'root/unsupported', 1, 'unsupported', 'unsupported'),
                    ('copilot', 'error', 'root/error', 1, 'error', 'error');
                INSERT INTO session_meta (session_id, notes, tags) VALUES ('complete', 'keep', '[\"tag\"]');
                INSERT INTO session_stats (
                    session_id, events_mtime, output_tokens, interaction_count, tool_call_count,
                    duration_minutes, models_used, reasoning_count, tool_breakdown
                ) VALUES ('complete', 1, 5, 1, 0, 1, '[]', 0, '{}');",
            )
            .expect("seed progress and metadata");
        let progress = usage_index_progress(&connection).expect("read usage progress");
        assert_eq!(progress.total_sessions, 5);
        assert_eq!(progress.indexed_sessions, 1);
        assert_eq!(progress.pending_sessions, 1);
        assert_eq!(progress.unsupported_sessions, 1);
        assert_eq!(progress.error_sessions, 1);
        assert_eq!(progress.unindexed_sessions, 1);
        upsert_usage_events(
            &connection,
            &[usage_event("copilot", "complete", "derived-event")],
        )
        .expect("insert derived event");

        rebuild_usage_derived_data(&connection).expect("rebuild usage tables");

        let (note, tags): (String, String) = connection
            .query_row(
                "SELECT notes, tags FROM session_meta WHERE session_id = 'complete'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read preserved session metadata");
        assert_eq!(note, "keep");
        assert_eq!(tags, "[\"tag\"]");
        let old_stats: i64 = connection
            .query_row(
                "SELECT output_tokens FROM session_stats WHERE session_id = 'complete'",
                [],
                |row| row.get(0),
            )
            .expect("read preserved session stats");
        assert_eq!(old_stats, 5);
        let derived_events: i64 = connection
            .query_row("SELECT COUNT(*) FROM usage_events", [], |row| row.get(0))
            .expect("count cleared events");
        let ingestion_rows: i64 = connection
            .query_row("SELECT COUNT(*) FROM usage_ingestion_state", [], |row| {
                row.get(0)
            })
            .expect("count cleared ingestion state");
        assert_eq!(derived_events, 0);
        assert_eq!(ingestion_rows, 0);
    }

    #[test]
    fn usage_analytics_schema_migration_is_idempotent_and_atomic() {
        let fresh = Connection::open_in_memory().expect("open fresh db");
        init_db(&fresh).expect("migrate fresh db");
        let event_table_count: i64 = fresh
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'usage_events'",
                [],
                |row| row.get(0),
            )
            .expect("query usage table");
        assert_eq!(event_table_count, 1);

        let existing = Connection::open_in_memory().expect("open existing db");
        existing
            .execute_batch(
                "CREATE TABLE session_meta (
                    session_id TEXT PRIMARY KEY,
                    notes TEXT,
                    tags TEXT
                );
                CREATE TABLE usage_events (
                    provider TEXT NOT NULL,
                    session_id TEXT NOT NULL,
                    source_event_id TEXT NOT NULL,
                    occurred_at TEXT NOT NULL,
                    cwd TEXT,
                    model TEXT,
                    input_tokens INTEGER,
                    output_tokens INTEGER,
                    cache_read_tokens INTEGER,
                    cache_write_tokens INTEGER,
                    reasoning_tokens INTEGER,
                    estimated_usd_micros INTEGER,
                    estimated_usd_price_version TEXT,
                    cost_points_micros INTEGER,
                    source_kind TEXT NOT NULL,
                    parser_version INTEGER NOT NULL,
                    PRIMARY KEY (provider, session_id, source_event_id)
                );
                INSERT INTO session_meta (session_id, notes, tags)
                VALUES ('existing-session', 'keep this note', '[\"retain\"]');",
            )
            .expect("seed legacy metadata");
        init_db(&existing).expect("migrate existing db");
        init_db(&existing).expect("rerun migration");
        let (note, tags): (String, String) = existing
            .query_row(
                "SELECT notes, tags FROM session_meta WHERE session_id = 'existing-session'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read preserved metadata");
        assert_eq!(note, "keep this note");
        assert_eq!(tags, "[\"retain\"]");
        let version: i64 = existing
            .query_row(
                "SELECT version FROM usage_schema_version WHERE singleton = 1",
                [],
                |row| row.get(0),
            )
            .expect("read schema version");
        assert_eq!(version, USAGE_SCHEMA_VERSION);
        let model_provider_column_count: i64 = existing
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('usage_events') WHERE name = 'model_provider_id'",
                [],
                |row| row.get(0),
            )
            .expect("read migrated model provider column");
        assert_eq!(model_provider_column_count, 1);
        let revision: i64 = existing
            .query_row(
                "SELECT revision FROM analytics_revision WHERE singleton = 1",
                [],
                |row| row.get(0),
            )
            .expect("read initial analytics revision");
        assert_eq!(revision, 0);

        let failed = Connection::open_in_memory().expect("open failure db");
        failed
            .execute_batch(
                "CREATE TABLE session_meta (
                    session_id TEXT PRIMARY KEY,
                    notes TEXT,
                    tags TEXT
                );
                INSERT INTO session_meta (session_id, notes, tags)
                VALUES ('failure-session', 'still here', '[]');
                CREATE VIEW usage_events AS SELECT 1 AS incompatible;",
            )
            .expect("seed migration conflict");
        assert!(init_db(&failed).is_err());
        let failure_note: String = failed
            .query_row(
                "SELECT notes FROM session_meta WHERE session_id = 'failure-session'",
                [],
                |row| row.get(0),
            )
            .expect("metadata should survive failed migration");
        assert_eq!(failure_note, "still here");
        let rolled_back_table_count: i64 = failed
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'usage_session_summaries'",
                [],
                |row| row.get(0),
            )
            .expect("query rolled back table");
        assert_eq!(rolled_back_table_count, 0);
    }

    #[test]
    fn model_pricing_cache_preserves_success_and_negative_lookup_states() {
        let connection = Connection::open_in_memory().expect("open pricing cache db");
        init_db(&connection).expect("initialize pricing cache db");
        let success = ModelPricingCacheRecord {
            provider: "openai".to_string(),
            model: "gpt-fixture".to_string(),
            status: "success".to_string(),
            prompt_usd_per_token: Some(0.000002),
            completion_usd_per_token: Some(0.00001),
            cache_read_usd_per_token: Some(0.0000002),
            cache_write_usd_per_token: None,
            source: "openrouter-catalog".to_string(),
            fetched_at: 100,
            expires_at: 200,
            error_kind: None,
            hidden: false,
            sort_order: 0,
        };
        upsert_model_pricing_cache(&connection, &success).expect("cache successful price");
        assert_eq!(
            model_pricing_cache(&connection, "openai", "gpt-fixture")
                .expect("read successful price"),
            Some(success)
        );

        let not_found = ModelPricingCacheRecord {
            provider: "openai".to_string(),
            model: "retired-fixture".to_string(),
            status: "not_found".to_string(),
            prompt_usd_per_token: None,
            completion_usd_per_token: None,
            cache_read_usd_per_token: None,
            cache_write_usd_per_token: None,
            source: "openrouter-catalog".to_string(),
            fetched_at: 100,
            expires_at: 150,
            error_kind: Some("model_not_in_catalog".to_string()),
            hidden: false,
            sort_order: 0,
        };
        upsert_model_pricing_cache(&connection, &not_found).expect("cache missing model");
        assert_eq!(
            model_pricing_cache(&connection, "openai", "retired-fixture")
                .expect("read missing model"),
            Some(not_found)
        );
    }

    #[test]
    fn requested_model_families_are_hidden_by_default() {
        let connection = Connection::open_in_memory().expect("open pricing visibility db");
        init_db(&connection).expect("initialize pricing visibility db");

        for (provider, model) in [
            ("openai", "o3"),
            ("openai", "o1-pro"),
            ("openai", "gpt-4.1"),
            ("openai", "gpt-4o-mini"),
            ("openai", "gpt-5.2"),
            ("openai", "gpt-5.5-pro"),
            ("anthropic", "claude-opus-4-1"),
            ("anthropic", "claude-sonnet-4"),
            ("anthropic", "claude-haiku-3-5"),
            ("anthropic", "claude-mythos-5-1"),
        ] {
            assert!(
                model_pricing_cache(&connection, provider, model)
                    .expect("read default-hidden model")
                    .is_some_and(|record| record.hidden),
                "{provider}/{model} should be hidden by default"
            );
        }

        for (provider, model) in [
            ("openai", "gpt-5.3-codex"),
            ("openai", "gpt-5.5"),
            ("openai", "gpt-6-astra"),
            ("anthropic", "claude-opus-4-5"),
            ("anthropic", "claude-opus-5"),
        ] {
            assert!(
                !model_hidden_by_default(provider, model),
                "{provider}/{model} should remain visible by default"
            );
        }

        let remote_model = ModelPricingCacheRecord {
            provider: "openai".to_string(),
            model: "gpt-5.2-codex".to_string(),
            status: "success".to_string(),
            prompt_usd_per_token: Some(0.000001),
            completion_usd_per_token: Some(0.000002),
            cache_read_usd_per_token: None,
            cache_write_usd_per_token: None,
            source: "openrouter-catalog".to_string(),
            fetched_at: 100,
            expires_at: 200,
            error_kind: None,
            hidden: false,
            sort_order: 0,
        };
        upsert_model_pricing_cache(&connection, &remote_model)
            .expect("insert default-hidden remote model");
        assert!(model_pricing_cache(&connection, "openai", "gpt-5.2-codex")
            .expect("read remote model")
            .is_some_and(|record| record.hidden));
    }

    #[test]
    fn default_model_visibility_migration_runs_once() {
        let connection = Connection::open_in_memory().expect("open legacy pricing db");
        init_db(&connection).expect("initialize legacy pricing db");
        connection
            .execute(
                "UPDATE usage_schema_version SET version = 4 WHERE singleton = 1",
                [],
            )
            .expect("set legacy schema version");
        connection
            .execute(
                "UPDATE model_pricing_cache SET hidden = 0
                 WHERE (provider = 'openai' AND model = 'gpt-5.2')
                    OR (provider = 'anthropic' AND model = 'claude-opus-4-1')",
                [],
            )
            .expect("make legacy rows visible");

        migrate_usage_analytics_schema(&connection).expect("migrate legacy visibility defaults");
        assert!(model_pricing_cache(&connection, "openai", "gpt-5.2")
            .expect("read migrated GPT row")
            .is_some_and(|record| record.hidden));
        assert!(
            model_pricing_cache(&connection, "anthropic", "claude-opus-4-1")
                .expect("read migrated Claude row")
                .is_some_and(|record| record.hidden)
        );

        set_model_pricing_hidden(&connection, "openai", "gpt-5.2", false)
            .expect("preserve user's visibility choice");
        migrate_usage_analytics_schema(&connection).expect("rerun visibility migration");
        assert!(model_pricing_cache(&connection, "openai", "gpt-5.2")
            .expect("read user-visible GPT row")
            .is_some_and(|record| !record.hidden));
    }

    #[test]
    fn extended_default_visibility_migration_preserves_prior_user_choices() {
        let connection = Connection::open_in_memory().expect("open extended visibility db");
        init_db(&connection).expect("initialize extended visibility db");
        connection
            .execute(
                "UPDATE usage_schema_version SET version = 5 WHERE singleton = 1",
                [],
            )
            .expect("set previous visibility schema version");
        connection
            .execute(
                "UPDATE model_pricing_cache SET hidden = 0
                 WHERE (provider = 'openai' AND model = 'gpt-5.5-pro')
                    OR (provider = 'openai' AND model = 'gpt-5.2')
                    OR (provider = 'anthropic' AND model = 'claude-mythos-5-1')",
                [],
            )
            .expect("prepare previous visibility choices");

        migrate_usage_analytics_schema(&connection).expect("migrate extended visibility defaults");

        for (provider, model) in [
            ("openai", "gpt-5.5-pro"),
            ("anthropic", "claude-mythos-5-1"),
        ] {
            assert!(model_pricing_cache(&connection, provider, model)
                .expect("read newly hidden model")
                .is_some_and(|record| record.hidden));
        }
        assert!(model_pricing_cache(&connection, "openai", "gpt-5.2")
            .expect("read prior user choice")
            .is_some_and(|record| !record.hidden));
    }

    #[test]
    fn migration_removes_codex_auto_review_openrouter_cache() {
        let connection = Connection::open_in_memory().expect("open excluded model db");
        init_db(&connection).expect("initialize excluded model db");
        upsert_model_pricing_cache(
            &connection,
            &ModelPricingCacheRecord {
                provider: "openai".to_string(),
                model: "codex-auto-review".to_string(),
                status: "not_found".to_string(),
                prompt_usd_per_token: None,
                completion_usd_per_token: None,
                cache_read_usd_per_token: None,
                cache_write_usd_per_token: None,
                source: "openrouter-catalog".to_string(),
                fetched_at: 100,
                expires_at: 200,
                error_kind: Some("model_not_in_catalog".to_string()),
                hidden: false,
                sort_order: 0,
            },
        )
        .expect("cache excluded OpenRouter model");

        migrate_usage_analytics_schema(&connection).expect("remove excluded OpenRouter cache");

        assert!(
            model_pricing_cache(&connection, "openai", "codex-auto-review")
                .expect("read excluded model cache")
                .is_none()
        );
    }
}
