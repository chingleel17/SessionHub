use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};

use crate::db::init_db;
use crate::types::SessionStats;

use super::{
    backfill_missing_stats_internal, session_events_mtime, upsert_session_stats_cache,
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

    let cached_mtime = session_events_mtime(&cached_dir.join("events.jsonl"))
        .expect("cached mtime should read");
    upsert_session_stats_cache(
        &connection,
        "cached-session",
        cached_mtime,
        &SessionStats::default(),
    )
    .expect("cached stats should insert");

    let processed = backfill_missing_stats_internal(&connection, &temp_root)
        .expect("backfill should succeed");

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
