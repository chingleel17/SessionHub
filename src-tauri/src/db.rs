use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use chrono::Datelike;
use rusqlite::{params, Connection};

use crate::settings::{legacy_session_cache_path, metadata_db_path};
use crate::types::*;

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
    Connection::open(db_path).map_err(|error| format!("failed to open metadata db: {error}"))
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
        init_db(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

pub(crate) fn init_db(connection: &Connection) -> Result<(), String> {
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

    if let Err(error) = migrate_legacy_session_cache(connection) {
        eprintln!("Warning: failed to migrate legacy session cache: {error}");
    }

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
    for provider in providers {
        connection
            .execute("DELETE FROM sessions_cache WHERE provider = ?1", [provider])
            .map_err(|error| format!("failed to clear sessions_cache: {error}"))?;
    }

    let mut statement = connection
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
        if !providers.iter().any(|p| p == &session.provider) {
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

    Ok(())
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
