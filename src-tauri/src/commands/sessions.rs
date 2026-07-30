use std::collections::HashMap;
use std::sync::Arc;

use tauri::State;

use crate::activity::get_session_activity_statuses_internal;
use crate::db::{
    delete_session_meta_internal, load_sessions_cache_from_db, open_db_connection,
    upsert_session_meta_internal, DbState,
};
use crate::sessions::{
    archive_session_internal, delete_empty_sessions_internal, delete_session_internal,
    directory_exists, extract_session_texts, find_session_by_cwd_internal, get_sessions_internal,
    open_terminal_internal, unarchive_session_internal,
};
use crate::settings::resolve_copilot_root;
use crate::stats::{backfill_missing_stats_internal, get_session_stats_internal};
use crate::types::*;

pub(crate) fn get_sessions_cached_internal(
    connection: &rusqlite::Connection,
    show_archived: Option<bool>,
    enabled_providers: Option<Vec<String>>,
) -> Result<Vec<SessionInfo>, String> {
    let include_archived = show_archived.unwrap_or(false);
    let enabled_providers = enabled_providers.unwrap_or_else(default_enabled_providers);
    let mut sessions = load_sessions_cache_from_db(connection, None)?;
    sessions.retain(|session| {
        enabled_providers
            .iter()
            .any(|provider| provider == &session.provider)
            && (include_archived || !session.is_archived)
    });
    sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    Ok(sessions)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSearchTarget {
    id: String,
    provider: String,
    session_dir: String,
}

pub(crate) fn search_session_content_internal(
    query: &str,
    sessions: &[SessionSearchTarget],
) -> Vec<String> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    sessions
        .iter()
        .filter_map(|session| {
            let texts = extract_session_texts(
                &session.provider,
                std::path::Path::new(&session.session_dir),
            );
            texts
                .into_iter()
                .any(|text| text.to_lowercase().contains(&query))
                .then(|| session.id.clone())
        })
        .collect()
}

#[tauri::command]
pub async fn search_session_content(
    query: String,
    sessions: Vec<SessionSearchTarget>,
) -> Result<Vec<String>, String> {
    Ok(tauri::async_runtime::spawn_blocking(move || {
        search_session_content_internal(&query, &sessions)
    })
    .await
    .map_err(|error| format!("failed to join content search task: {error}"))?)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use rusqlite::Connection;

    use super::{search_session_content_internal, SessionSearchTarget};

    fn unique_test_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        std::env::temp_dir().join(format!("session-hub-content-search-{name}-{suffix}"))
    }

    fn copilot_target(root: &PathBuf, id: &str, content: &str) -> SessionSearchTarget {
        let session_dir = root.join(id);
        fs::create_dir_all(&session_dir).expect("create session directory");
        fs::write(
            session_dir.join("events.jsonl"),
            format!(
                "{{\"type\":\"user.message\",\"data\":{{\"content\":\"{content}\"}}}}\n{{\"type\":\"assistant.message\",\"data\":{{\"content\":\"tool arguments must not match\"}}}}"
            ),
        )
        .expect("write events");
        SessionSearchTarget {
            id: id.to_string(),
            provider: "copilot".to_string(),
            session_dir: session_dir.to_string_lossy().to_string(),
        }
    }

    #[test]
    fn content_search_matches_case_insensitively_and_skips_non_matches() {
        let root = unique_test_dir("match");
        let matching = copilot_target(&root, "matching", "Discussing Rust ownership");
        let missing = copilot_target(&root, "missing", "Discussing TypeScript");

        let matches = search_session_content_internal("OWNERSHIP", &[matching, missing]);

        assert_eq!(matches, vec!["matching"]);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn content_search_returns_empty_without_reading_targets_for_blank_query() {
        let root = unique_test_dir("blank");
        let target = SessionSearchTarget {
            id: "missing".to_string(),
            provider: "copilot".to_string(),
            session_dir: root.join("does-not-exist").to_string_lossy().to_string(),
        };

        assert!(search_session_content_internal("   ", &[target]).is_empty());
    }

    #[test]
    fn content_search_skips_missing_session_files() {
        let root = unique_test_dir("missing");
        let target = SessionSearchTarget {
            id: "missing".to_string(),
            provider: "copilot".to_string(),
            session_dir: root.join("does-not-exist").to_string_lossy().to_string(),
        };

        assert!(search_session_content_internal("anything", &[target]).is_empty());
    }

    #[test]
    fn content_search_does_not_change_metadata_database() {
        let root = unique_test_dir("metadata");
        fs::create_dir_all(&root).expect("create test directory");
        let database_path = root.join("metadata.db");
        let connection = Connection::open(&database_path).expect("open metadata database");
        crate::db::init_db(&connection).expect("initialize metadata database");
        connection
            .execute(
                "INSERT INTO session_meta (session_id, notes, tags) VALUES (?1, ?2, ?3)",
                ["session-001", "note", "[]"],
            )
            .expect("seed metadata database");
        drop(connection);

        let row_count_before = Connection::open(&database_path)
            .expect("reopen metadata database")
            .query_row("SELECT COUNT(*) FROM session_meta", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("count metadata rows");
        let size_before = fs::metadata(&database_path)
            .expect("read database metadata")
            .len();
        let target = copilot_target(&root, "session-001", "Find this conversation");

        assert_eq!(
            search_session_content_internal("conversation", &[target]),
            vec!["session-001"]
        );

        let row_count_after = Connection::open(&database_path)
            .expect("reopen metadata database")
            .query_row("SELECT COUNT(*) FROM session_meta", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("count metadata rows");
        let size_after = fs::metadata(&database_path)
            .expect("read database metadata")
            .len();
        assert_eq!(row_count_after, row_count_before);
        assert_eq!(size_after, size_before);
        fs::remove_dir_all(root).ok();
    }
}

pub(crate) fn get_all_session_stats_internal(
    connection: &rusqlite::Connection,
    session_dirs: &[String],
) -> HashMap<String, SessionStats> {
    let mut stats_map = HashMap::with_capacity(session_dirs.len());
    for session_dir in session_dirs {
        if let Ok(stats) = get_session_stats_internal(connection, session_dir) {
            stats_map.insert(session_dir.clone(), stats);
        }
    }
    stats_map
}

#[tauri::command]
pub async fn get_sessions(
    root_dir: Option<String>,
    opencode_root: Option<String>,
    codex_root: Option<String>,
    claude_root: Option<String>,
    antigravity_root: Option<String>,
    show_archived: Option<bool>,
    enabled_providers: Option<Vec<String>>,
    force_full: Option<bool>,
    scan_cache: State<'_, Arc<ScanCache>>,
    _db: State<'_, DbState>,
) -> Result<Vec<SessionInfo>, String> {
    // 將整個掃描（磁碟 I/O + git 子程序）移至背景執行緒，避免阻塞 Tauri 主執行緒導致 UI 白屏無回應。
    // ScanCache 以 Arc 共享，可安全移入 spawn_blocking 閉包；DB 則於背景執行緒另開連線，
    // 與 trigger_stats_backfill 採相同模式。
    let scan_cache = Arc::clone(scan_cache.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let conn = open_db_connection()?;
        get_sessions_internal(
            root_dir,
            opencode_root,
            codex_root,
            claude_root,
            antigravity_root,
            show_archived,
            enabled_providers,
            force_full,
            &scan_cache,
            &conn,
        )
    })
    .await
    .map_err(|error| format!("failed to join sessions scan task: {error}"))?
}

#[tauri::command]
pub fn get_sessions_cached(
    show_archived: Option<bool>,
    enabled_providers: Option<Vec<String>>,
    db: State<'_, DbState>,
) -> Result<Vec<SessionInfo>, String> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    get_sessions_cached_internal(&*conn, show_archived, enabled_providers)
}

#[tauri::command]
pub fn archive_session(root_dir: Option<String>, session_id: String) -> Result<(), String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    archive_session_internal(&resolved_root, &session_id)
}

#[tauri::command]
pub fn unarchive_session(root_dir: Option<String>, session_id: String) -> Result<(), String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    unarchive_session_internal(&resolved_root, &session_id)
}

#[tauri::command]
pub fn delete_session(
    root_dir: Option<String>,
    session_id: String,
    db: State<'_, DbState>,
) -> Result<(), String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    delete_session_internal(&resolved_root, &session_id, &*conn)
}

#[tauri::command]
pub fn delete_empty_sessions(
    root_dir: Option<String>,
    db: State<'_, DbState>,
) -> Result<usize, String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    delete_empty_sessions_internal(&resolved_root.to_string_lossy(), &*conn)
}

#[tauri::command]
pub fn open_terminal(
    terminal_path: String,
    cwd: String,
    _session_id: String,
) -> Result<(), String> {
    open_terminal_internal(&terminal_path, &cwd)
}

#[tauri::command]
pub fn check_directory_exists(path: String) -> bool {
    directory_exists(&path)
}

#[tauri::command]
pub fn get_session_activity_statuses(
    sessions: Vec<serde_json::Value>,
    opencode_root: Option<String>,
    scan_cache: State<'_, Arc<ScanCache>>,
) -> Vec<SessionActivityStatus> {
    get_session_activity_statuses_internal(
        &sessions,
        opencode_root.as_deref(),
        &scan_cache.activity,
    )
}

#[tauri::command]
pub fn get_session_stats(
    session_dir: String,
    db: State<'_, DbState>,
) -> Result<SessionStats, String> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    get_session_stats_internal(&*conn, &session_dir)
}

#[tauri::command]
pub fn get_all_session_stats(
    session_dirs: Vec<String>,
    db: State<'_, DbState>,
) -> Result<HashMap<String, SessionStats>, String> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    Ok(get_all_session_stats_internal(&*conn, &session_dirs))
}

#[tauri::command]
pub async fn trigger_stats_backfill(
    root_dir: Option<String>,
    _db: State<'_, DbState>,
) -> Result<usize, String> {
    let copilot_root = resolve_copilot_root(root_dir.as_deref())?;
    tauri::async_runtime::spawn_blocking(move || {
        let connection = open_db_connection()?;
        backfill_missing_stats_internal(&connection, &copilot_root)
    })
    .await
    .map_err(|error| format!("failed to join stats backfill task: {error}"))?
}

#[tauri::command]
pub fn upsert_session_meta(
    session_id: String,
    notes: Option<String>,
    tags: Vec<String>,
    db: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    upsert_session_meta_internal(&*conn, &session_id, notes, tags)
}

#[tauri::command]
pub fn delete_session_meta(session_id: String, db: State<'_, DbState>) -> Result<(), String> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    delete_session_meta_internal(&*conn, &session_id)
}

#[tauri::command]
pub fn get_session_by_cwd(
    cwd: String,
    root_dir: Option<String>,
    db: State<'_, DbState>,
) -> Result<Option<SessionInfo>, String> {
    let copilot_root = resolve_copilot_root(root_dir.as_deref())?;
    let conn = db
        .conn
        .lock()
        .map_err(|e| format!("db lock poisoned: {e}"))?;
    find_session_by_cwd_internal(&copilot_root, &cwd, &*conn)
}
