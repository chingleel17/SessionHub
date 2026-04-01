use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NEW_CONSOLE: u32 = 0x00000010;

use notify::{recommended_watcher, RecommendedWatcher, RecursiveMode, Watcher};
use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};

#[derive(Debug, Deserialize)]
struct WorkspaceYaml {
    id: String,
    cwd: Option<String>,
    summary: Option<String>,
    summary_count: Option<u32>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

fn default_provider() -> String {
    "copilot".to_string()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionInfo {
    id: String,
    #[serde(default = "default_provider")]
    provider: String,
    cwd: Option<String>,
    summary: Option<String>,
    summary_count: Option<u32>,
    created_at: Option<String>,
    updated_at: Option<String>,
    session_dir: String,
    parse_error: bool,
    is_archived: bool,
    notes: Option<String>,
    tags: Vec<String>,
    has_plan: bool,
    has_events: bool,
}

fn default_enabled_providers() -> Vec<String> {
    vec!["copilot".to_string(), "opencode".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    copilot_root: String,
    #[serde(default)]
    opencode_root: String,
    terminal_path: Option<String>,
    external_editor_path: Option<String>,
    show_archived: bool,
    #[serde(default)]
    pinned_projects: Vec<String>,
    #[serde(default = "default_enabled_providers")]
    enabled_providers: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionMeta {
    notes: Option<String>,
    tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct SessionStats {
    output_tokens: u64,
    interaction_count: u32,
    tool_call_count: u32,
    duration_minutes: u64,
    models_used: Vec<String>,
    reasoning_count: u32,
    tool_breakdown: BTreeMap<String, u32>,
    is_live: bool,
}

impl Default for SessionStats {
    fn default() -> Self {
        Self {
            output_tokens: 0,
            interaction_count: 0,
            tool_call_count: 0,
            duration_minutes: 0,
            models_used: Vec::new(),
            reasoning_count: 0,
            tool_breakdown: BTreeMap::new(),
            is_live: false,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionStartData {
    #[serde(default)]
    start_time: Option<String>,
    #[serde(default)]
    selected_model: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionModelChangeData {
    #[serde(default)]
    new_model: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TopLevelFilterData {
    #[serde(default)]
    parent_tool_call_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolExecutionStartData {
    #[serde(default)]
    parent_tool_call_id: Option<String>,
    #[serde(default)]
    tool_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssistantMessageData {
    #[serde(default)]
    parent_tool_call_id: Option<String>,
    #[serde(default)]
    output_tokens: Option<u64>,
    #[serde(default)]
    reasoning_opaque: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SessionEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    data: serde_json::Value,
}

#[derive(Default)]
struct WatcherState {
    sessions: Mutex<Option<RecommendedWatcher>>,
    plan: Mutex<Option<RecommendedWatcher>>,
    opencode: Mutex<Option<RecommendedWatcher>>,
}

fn default_copilot_root() -> Result<PathBuf, String> {
    let user_profile = env::var("USERPROFILE")
        .map_err(|_| "USERPROFILE environment variable is not set".to_string())?;

    Ok(PathBuf::from(user_profile).join(".copilot"))
}

fn default_opencode_root() -> Result<PathBuf, String> {
    let user_profile = env::var("USERPROFILE")
        .map_err(|_| "USERPROFILE environment variable is not set".to_string())?;

    Ok(PathBuf::from(user_profile)
        .join(".local")
        .join("share")
        .join("opencode"))
}

fn default_app_data_dir() -> Result<PathBuf, String> {
    if let Ok(override_dir) = env::var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE") {
        return Ok(PathBuf::from(override_dir).join("SessionHub"));
    }

    let app_data =
        env::var("APPDATA").map_err(|_| "APPDATA environment variable is not set".to_string())?;

    Ok(PathBuf::from(app_data).join("SessionHub"))
}

fn resolve_copilot_root(root_dir: Option<&str>) -> Result<PathBuf, String> {
    match root_dir {
        Some(path) if !path.trim().is_empty() => Ok(PathBuf::from(path)),
        _ => default_copilot_root(),
    }
}

fn resolve_opencode_root(root_dir: Option<&str>) -> Result<PathBuf, String> {
    match root_dir {
        Some(path) if !path.trim().is_empty() => Ok(PathBuf::from(path)),
        _ => default_opencode_root(),
    }
}

/// 以唯讀模式開啟 OpenCode 的 SQLite 資料庫
fn open_opencode_db_readonly(opencode_root: &Path) -> Result<Connection, String> {
    let db_path = opencode_root.join("opencode.db");

    if !db_path.exists() {
        return Err(format!(
            "opencode.db does not exist at {}",
            db_path.display()
        ));
    }

    Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|error| {
        format!(
            "failed to open opencode db at {}: {error}",
            db_path.display()
        )
    })
}

/// 將 unix timestamp（毫秒）轉換為 ISO 8601 字串
fn unix_ms_to_iso8601(timestamp_ms: i64) -> Option<String> {
    let timestamp_secs = timestamp_ms / 1000;
    let nanos = ((timestamp_ms % 1000) * 1_000_000) as u32;

    chrono::DateTime::from_timestamp(timestamp_secs, nanos)
        .map(|datetime| datetime.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}

/// 掃描 OpenCode 資料庫中的 session，映射為 Vec<SessionInfo>
fn scan_opencode_sessions_internal(
    opencode_root: &Path,
    show_archived: bool,
    metadata_conn: &Connection,
) -> Result<Vec<SessionInfo>, String> {
    let oc_conn = match open_opencode_db_readonly(opencode_root) {
        Ok(conn) => conn,
        Err(error) => {
            eprintln!("opencode provider: {error}");
            return Ok(Vec::new());
        }
    };

    let query = if show_archived {
        "SELECT s.id, s.title, s.time_created, s.time_updated, s.time_archived, \
                p.worktree, s.summary_additions, s.summary_deletions, s.summary_files \
         FROM session s \
         LEFT JOIN project p ON s.project_id = p.id"
    } else {
        "SELECT s.id, s.title, s.time_created, s.time_updated, s.time_archived, \
                p.worktree, s.summary_additions, s.summary_deletions, s.summary_files \
         FROM session s \
         LEFT JOIN project p ON s.project_id = p.id \
         WHERE s.time_archived IS NULL"
    };

    let mut statement = oc_conn.prepare(query).map_err(|error| {
        eprintln!("opencode provider: failed to prepare query: {error}");
        format!("opencode db query error: {error}")
    })?;

    let rows = statement
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let title: Option<String> = row.get(1)?;
            let time_created: Option<i64> = row.get(2)?;
            let time_updated: Option<i64> = row.get(3)?;
            let time_archived: Option<i64> = row.get(4)?;
            let worktree: Option<String> = row.get(5)?;
            let summary_additions: Option<i64> = row.get(6)?;
            let summary_deletions: Option<i64> = row.get(7)?;
            let summary_files: Option<i64> = row.get(8)?;

            Ok((
                id,
                title,
                time_created,
                time_updated,
                time_archived,
                worktree,
                summary_additions,
                summary_deletions,
                summary_files,
            ))
        })
        .map_err(|error| {
            eprintln!("opencode provider: failed to query sessions: {error}");
            format!("opencode db query error: {error}")
        })?;

    let mut sessions = Vec::new();

    for row_result in rows {
        let (
            id,
            title,
            time_created,
            time_updated,
            time_archived,
            worktree,
            summary_additions,
            summary_deletions,
            summary_files,
        ) = match row_result {
            Ok(data) => data,
            Err(error) => {
                eprintln!("opencode provider: failed to read row: {error}");
                continue;
            }
        };

        let is_archived = time_archived.is_some();
        let created_at = time_created.and_then(unix_ms_to_iso8601);
        let updated_at = time_updated.and_then(unix_ms_to_iso8601);

        // 建構 summary，包含 additions/deletions/files 統計
        let summary = title;
        let summary_count = {
            let total = summary_additions.unwrap_or(0)
                + summary_deletions.unwrap_or(0)
                + summary_files.unwrap_or(0);
            if total > 0 {
                Some(total as u32)
            } else {
                None
            }
        };

        let meta = read_session_meta(metadata_conn, &id).unwrap_or(SessionMeta {
            notes: None,
            tags: Vec::new(),
        });

        sessions.push(SessionInfo {
            id,
            provider: "opencode".to_string(),
            cwd: worktree,
            summary,
            summary_count,
            created_at,
            updated_at,
            session_dir: String::new(),
            parse_error: false,
            is_archived,
            notes: meta.notes,
            tags: meta.tags,
            has_plan: false,
            has_events: false,
        });
    }

    Ok(sessions)
}

impl AppSettings {
    fn default() -> Result<Self, String> {
        let terminal_path = detect_terminal_path()?;
        let external_editor_path = detect_vscode_path()?;

        Ok(Self {
            copilot_root: default_copilot_root()?.to_string_lossy().to_string(),
            opencode_root: default_opencode_root()?.to_string_lossy().to_string(),
            terminal_path,
            external_editor_path,
            show_archived: false,
            pinned_projects: Vec::new(),
            enabled_providers: default_enabled_providers(),
        })
    }
}

fn settings_file_path() -> Result<PathBuf, String> {
    Ok(default_app_data_dir()?.join("settings.json"))
}

fn metadata_db_path() -> Result<PathBuf, String> {
    Ok(default_app_data_dir()?.join("metadata.db"))
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create directory {}: {error}", parent.display()))?;
    }

    Ok(())
}

fn open_db_connection() -> Result<Connection, String> {
    let db_path = metadata_db_path()?;
    ensure_parent_dir(&db_path)?;

    Connection::open(db_path).map_err(|error| format!("failed to open metadata db: {error}"))
}

fn create_sessions_watcher(
    app: &tauri::AppHandle,
    root: &Path,
) -> Result<RecommendedWatcher, String> {
    let app_handle = app.clone();
    let mut watcher = recommended_watcher(move |result: notify::Result<notify::Event>| {
        if result.is_ok() {
            let _ = app_handle.emit("sessions-updated", ());
        }
    })
    .map_err(|error| format!("failed to create session watcher: {error}"))?;

    let session_state_dir = root.join("session-state");
    if session_state_dir.exists() {
        watcher
            .watch(&session_state_dir, RecursiveMode::Recursive)
            .map_err(|error| format!("failed to watch {}: {error}", session_state_dir.display()))?;
    }

    let archive_dir = root.join("session-state-archive");
    if archive_dir.exists() {
        watcher
            .watch(&archive_dir, RecursiveMode::Recursive)
            .map_err(|error| format!("failed to watch {}: {error}", archive_dir.display()))?;
    }

    Ok(watcher)
}

/// 建立 OpenCode DB 的 WAL 檔案監聽器
fn create_opencode_watcher(
    app: &tauri::AppHandle,
    opencode_root: &Path,
) -> Result<RecommendedWatcher, String> {
    let wal_path = opencode_root.join("opencode.db-wal");
    let db_path = opencode_root.join("opencode.db");

    // WAL 或主 DB 至少一個必須存在
    if !wal_path.exists() && !db_path.exists() {
        return Err(format!(
            "opencode db does not exist at {}",
            opencode_root.display()
        ));
    }

    let app_handle = app.clone();
    let mut watcher = recommended_watcher(move |result: notify::Result<notify::Event>| {
        if result.is_ok() {
            let _ = app_handle.emit("sessions-updated", ());
        }
    })
    .map_err(|error| format!("failed to create opencode watcher: {error}"))?;

    // 監聽整個 opencode 目錄（包含 .db、.db-wal、.db-shm）
    watcher
        .watch(opencode_root, RecursiveMode::NonRecursive)
        .map_err(|error| format!("failed to watch {}: {error}", opencode_root.display()))?;

    Ok(watcher)
}

fn restart_session_watcher_internal(
    app: &tauri::AppHandle,
    watcher_state: &WatcherState,
    copilot_root: Option<&str>,
    opencode_root: Option<&str>,
    enabled_providers: &[String],
) -> Result<(), String> {
    // Copilot watcher
    if enabled_providers.iter().any(|p| p == "copilot") {
        let root = resolve_copilot_root(copilot_root)?;
        match create_sessions_watcher(app, &root) {
            Ok(watcher) => {
                let mut session_watcher = watcher_state
                    .sessions
                    .lock()
                    .map_err(|_| "failed to lock session watcher state".to_string())?;
                *session_watcher = Some(watcher);
            }
            Err(error) => {
                eprintln!("failed to start copilot session watcher: {error}");
            }
        }
    } else {
        // 停用時釋放 watcher
        let mut session_watcher = watcher_state
            .sessions
            .lock()
            .map_err(|_| "failed to lock session watcher state".to_string())?;
        *session_watcher = None;
    }

    // OpenCode watcher
    if enabled_providers.iter().any(|p| p == "opencode") {
        if let Ok(oc_root) = resolve_opencode_root(opencode_root) {
            match create_opencode_watcher(app, &oc_root) {
                Ok(watcher) => {
                    let mut oc_watcher = watcher_state
                        .opencode
                        .lock()
                        .map_err(|_| "failed to lock opencode watcher state".to_string())?;
                    *oc_watcher = Some(watcher);
                }
                Err(error) => {
                    eprintln!("failed to start opencode watcher: {error}");
                }
            }
        }
    } else {
        // 停用時釋放 watcher
        let mut oc_watcher = watcher_state
            .opencode
            .lock()
            .map_err(|_| "failed to lock opencode watcher state".to_string())?;
        *oc_watcher = None;
    }

    Ok(())
}

fn watch_plan_file_internal(
    app: &tauri::AppHandle,
    watcher_state: &WatcherState,
    session_dir: &str,
) -> Result<(), String> {
    let plan_path = PathBuf::from(session_dir).join("plan.md");

    if !plan_path.exists() {
        return Err("plan.md does not exist".to_string());
    }

    let app_handle = app.clone();
    let watched_session_dir = session_dir.to_string();
    let mut watcher = recommended_watcher(move |result: notify::Result<notify::Event>| {
        if result.is_ok() {
            let _ = app_handle.emit("plan-file-changed", watched_session_dir.clone());
        }
    })
    .map_err(|error| format!("failed to create plan watcher: {error}"))?;

    watcher
        .watch(&plan_path, RecursiveMode::NonRecursive)
        .map_err(|error| format!("failed to watch {}: {error}", plan_path.display()))?;

    let mut plan_watcher = watcher_state
        .plan
        .lock()
        .map_err(|_| "failed to lock plan watcher state".to_string())?;
    *plan_watcher = Some(watcher);
    Ok(())
}

fn init_db(connection: &Connection) -> Result<(), String> {
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
                tool_breakdown TEXT NOT NULL
            )
            ",
            [],
        )
        .map_err(|error| format!("failed to initialize session stats db: {error}"))?;

    Ok(())
}

fn get_session_stats_cache(
    connection: &Connection,
    session_id: &str,
) -> Result<Option<(i64, SessionStats)>, String> {
    let mut statement = connection
        .prepare(
            "
            SELECT events_mtime, output_tokens, interaction_count, tool_call_count,
                   duration_minutes, models_used, reasoning_count, tool_breakdown
            FROM session_stats
            WHERE session_id = ?1
            ",
        )
        .map_err(|error| format!("failed to prepare session stats cache query: {error}"))?;

    let mut rows = statement
        .query(params![session_id])
        .map_err(|error| format!("failed to query session stats cache: {error}"))?;

    match rows
        .next()
        .map_err(|error| format!("failed to read session stats cache row: {error}"))?
    {
        Some(row) => {
            let events_mtime: i64 = row
                .get(0)
                .map_err(|error| format!("failed to read events_mtime column: {error}"))?;
            let models_used_json: String = row
                .get(5)
                .map_err(|error| format!("failed to read models_used column: {error}"))?;
            let tool_breakdown_json: String = row
                .get(7)
                .map_err(|error| format!("failed to read tool_breakdown column: {error}"))?;

            let models_used = serde_json::from_str::<Vec<String>>(&models_used_json)
                .map_err(|error| format!("failed to deserialize cached models_used: {error}"))?;
            let tool_breakdown = serde_json::from_str::<BTreeMap<String, u32>>(
                &tool_breakdown_json,
            )
            .map_err(|error| format!("failed to deserialize cached tool_breakdown: {error}"))?;

            Ok(Some((
                events_mtime,
                SessionStats {
                    output_tokens: row
                        .get(1)
                        .map_err(|error| format!("failed to read output_tokens column: {error}"))?,
                    interaction_count: row.get(2).map_err(|error| {
                        format!("failed to read interaction_count column: {error}")
                    })?,
                    tool_call_count: row.get(3).map_err(|error| {
                        format!("failed to read tool_call_count column: {error}")
                    })?,
                    duration_minutes: row.get(4).map_err(|error| {
                        format!("failed to read duration_minutes column: {error}")
                    })?,
                    models_used,
                    reasoning_count: row.get(6).map_err(|error| {
                        format!("failed to read reasoning_count column: {error}")
                    })?,
                    tool_breakdown,
                    is_live: false,
                },
            )))
        }
        None => Ok(None),
    }
}

fn upsert_session_stats_cache(
    connection: &Connection,
    session_id: &str,
    events_mtime: i64,
    stats: &SessionStats,
) -> Result<(), String> {
    let models_used_json = serde_json::to_string(&stats.models_used)
        .map_err(|error| format!("failed to serialize models_used: {error}"))?;
    let tool_breakdown_json = serde_json::to_string(&stats.tool_breakdown)
        .map_err(|error| format!("failed to serialize tool_breakdown: {error}"))?;

    connection
        .execute(
            "
            INSERT INTO session_stats (
                session_id, events_mtime, output_tokens, interaction_count, tool_call_count,
                duration_minutes, models_used, reasoning_count, tool_breakdown
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(session_id) DO UPDATE SET
                events_mtime = excluded.events_mtime,
                output_tokens = excluded.output_tokens,
                interaction_count = excluded.interaction_count,
                tool_call_count = excluded.tool_call_count,
                duration_minutes = excluded.duration_minutes,
                models_used = excluded.models_used,
                reasoning_count = excluded.reasoning_count,
                tool_breakdown = excluded.tool_breakdown
            ",
            params![
                session_id,
                events_mtime,
                stats.output_tokens,
                stats.interaction_count,
                stats.tool_call_count,
                stats.duration_minutes,
                models_used_json,
                stats.reasoning_count,
                tool_breakdown_json,
            ],
        )
        .map_err(|error| format!("failed to upsert session stats cache: {error}"))?;

    Ok(())
}

fn session_events_mtime(events_path: &Path) -> Result<i64, String> {
    let modified = fs::metadata(events_path)
        .map_err(|error| {
            format!(
                "failed to read metadata for {}: {error}",
                events_path.display()
            )
        })?
        .modified()
        .map_err(|error| {
            format!(
                "failed to read modified time for {}: {error}",
                events_path.display()
            )
        })?;

    let seconds = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to convert modified time to unix epoch: {error}"))?
        .as_secs();

    i64::try_from(seconds).map_err(|error| format!("failed to convert modified time: {error}"))
}

fn session_id_from_dir(session_dir: &Path) -> String {
    session_dir
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn is_live_session(session_dir: &Path) -> Result<bool, String> {
    let entries = fs::read_dir(session_dir).map_err(|error| {
        format!(
            "failed to read session dir {}: {error}",
            session_dir.display()
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read session dir entry: {error}"))?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.starts_with("inuse.") && file_name.ends_with(".lock") {
            return Ok(true);
        }
    }

    Ok(false)
}

fn parse_session_stats_internal(session_dir: &Path) -> Result<SessionStats, String> {
    let events_path = session_dir.join("events.jsonl");
    let mut stats = SessionStats {
        is_live: is_live_session(session_dir)?,
        ..SessionStats::default()
    };

    if !events_path.exists() {
        return Ok(stats);
    }

    let file = fs::File::open(&events_path)
        .map_err(|error| format!("failed to open {}: {error}", events_path.display()))?;
    let reader = BufReader::new(file);
    let mut start_time: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut last_timestamp: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut models_used = BTreeSet::new();

    for line in reader.lines() {
        let line = match line {
            Ok(value) => value,
            Err(_) => continue,
        };

        let event = match serde_json::from_str::<SessionEvent>(&line) {
            Ok(value) => value,
            Err(_) => continue,
        };

        if let Some(timestamp) = event.timestamp.as_deref() {
            if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(timestamp) {
                last_timestamp = Some(parsed.with_timezone(&chrono::Utc));
            }
        }

        match event.event_type.as_str() {
            "session.start" => {
                if let Ok(data) = serde_json::from_value::<SessionStartData>(event.data) {
                    if let Some(model) =
                        data.selected_model.filter(|value| !value.trim().is_empty())
                    {
                        models_used.insert(model);
                    }
                    if let Some(raw_time) = data.start_time {
                        if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&raw_time) {
                            start_time = Some(parsed.with_timezone(&chrono::Utc));
                        }
                    }
                }
            }
            "session.model_change" => {
                if let Ok(data) = serde_json::from_value::<SessionModelChangeData>(event.data) {
                    if let Some(model) = data.new_model.filter(|value| !value.trim().is_empty()) {
                        models_used.insert(model);
                    }
                }
            }
            "user.message" => {
                if let Ok(data) = serde_json::from_value::<TopLevelFilterData>(event.data) {
                    if data.parent_tool_call_id.is_none() {
                        stats.interaction_count += 1;
                    }
                }
            }
            "tool.execution_start" => {
                if let Ok(data) = serde_json::from_value::<ToolExecutionStartData>(event.data) {
                    if data.parent_tool_call_id.is_none() {
                        stats.tool_call_count += 1;
                        let tool_name = data.tool_name.unwrap_or_else(|| "unknown".to_string());
                        *stats.tool_breakdown.entry(tool_name).or_insert(0) += 1;
                    }
                }
            }
            "assistant.message" => {
                if let Ok(data) = serde_json::from_value::<AssistantMessageData>(event.data) {
                    if data.parent_tool_call_id.is_none() {
                        stats.output_tokens += data.output_tokens.unwrap_or(0);
                        if data.reasoning_opaque.is_some() {
                            stats.reasoning_count += 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    stats.models_used = models_used.into_iter().collect();

    if let (Some(start), Some(end)) = (start_time, last_timestamp) {
        let seconds = (end - start).num_seconds();
        if seconds > 0 {
            stats.duration_minutes = (seconds as u64) / 60;
        }
    }

    Ok(stats)
}

fn get_session_stats_internal(
    connection: &Connection,
    session_dir: &str,
) -> Result<SessionStats, String> {
    let session_path = PathBuf::from(session_dir);
    let session_id = session_id_from_dir(&session_path);
    let events_path = session_path.join("events.jsonl");
    let is_live = is_live_session(&session_path)?;

    if !events_path.exists() {
        return Ok(SessionStats {
            is_live,
            ..SessionStats::default()
        });
    }

    let current_mtime = session_events_mtime(&events_path)?;
    if !is_live {
        if let Some((cached_mtime, mut cached_stats)) =
            get_session_stats_cache(connection, &session_id)?
        {
            if cached_mtime == current_mtime {
                cached_stats.is_live = false;
                return Ok(cached_stats);
            }
        }
    }

    let stats = parse_session_stats_internal(&session_path)?;
    if !stats.is_live {
        upsert_session_stats_cache(connection, &session_id, current_mtime, &stats)?;
    }

    Ok(stats)
}

fn read_session_meta(connection: &Connection, session_id: &str) -> Result<SessionMeta, String> {
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

fn upsert_session_meta_internal(
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

fn delete_session_meta_internal(connection: &Connection, session_id: &str) -> Result<(), String> {
    connection
        .execute(
            "DELETE FROM session_meta WHERE session_id = ?1",
            params![session_id],
        )
        .map_err(|error| format!("failed to delete metadata: {error}"))?;

    Ok(())
}

fn parse_workspace_file(
    session_dir: &Path,
    workspace_path: &Path,
    is_archived: bool,
    meta: SessionMeta,
) -> SessionInfo {
    let fallback_id = session_dir
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown-session".to_string());
    let has_plan = session_dir.join("plan.md").exists();
    let has_events = session_dir
        .join("events.jsonl")
        .metadata()
        .map(|meta| meta.len() > 0)
        .unwrap_or(false);
    let fallback_session = || SessionInfo {
        id: fallback_id.clone(),
        provider: default_provider(),
        cwd: None,
        summary: None,
        summary_count: None,
        created_at: None,
        updated_at: None,
        session_dir: session_dir.to_string_lossy().to_string(),
        parse_error: true,
        is_archived,
        notes: meta.notes.clone(),
        tags: meta.tags.clone(),
        has_plan,
        has_events,
    };

    match fs::read_to_string(workspace_path) {
        Ok(content) => match serde_yaml::from_str::<WorkspaceYaml>(&content) {
            Ok(workspace) => SessionInfo {
                id: workspace.id,
                provider: default_provider(),
                cwd: workspace.cwd,
                summary: workspace.summary,
                summary_count: workspace.summary_count,
                created_at: workspace.created_at,
                updated_at: workspace.updated_at,
                session_dir: session_dir.to_string_lossy().to_string(),
                parse_error: false,
                is_archived,
                notes: meta.notes,
                tags: meta.tags,
                has_plan,
                has_events,
            },
            Err(_) => fallback_session(),
        },
        Err(_) => fallback_session(),
    }
}

fn scan_session_dir(
    session_state_dir: &Path,
    is_archived: bool,
    connection: &Connection,
) -> Result<Vec<SessionInfo>, String> {
    if !session_state_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(session_state_dir)
        .map_err(|error| format!("failed to read {}: {error}", session_state_dir.display()))?;
    let mut sessions = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read session entry: {error}"))?;
        let session_dir = entry.path();

        if !session_dir.is_dir() {
            continue;
        }

        let workspace_path = session_dir.join("workspace.yaml");

        if !workspace_path.exists() {
            continue;
        }

        let session_id = session_dir
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_default();
        let meta = read_session_meta(connection, &session_id)?;

        sessions.push(parse_workspace_file(
            &session_dir,
            &workspace_path,
            is_archived,
            meta,
        ));
    }

    Ok(sessions)
}

fn scan_sessions(
    copilot_root: &Path,
    opencode_root: Option<&Path>,
    show_archived: bool,
    enabled_providers: &[String],
) -> Result<Vec<SessionInfo>, String> {
    let connection = open_db_connection()?;
    init_db(&connection)?;

    let mut sessions = Vec::new();

    // Copilot sessions
    if enabled_providers.iter().any(|p| p == "copilot") {
        let mut copilot_sessions =
            scan_session_dir(&copilot_root.join("session-state"), false, &connection)?;

        if show_archived {
            copilot_sessions.extend(scan_session_dir(
                &copilot_root.join("session-state-archive"),
                true,
                &connection,
            )?);
        }

        sessions.extend(copilot_sessions);
    }

    // OpenCode sessions
    if enabled_providers.iter().any(|p| p == "opencode") {
        if let Some(oc_root) = opencode_root {
            match scan_opencode_sessions_internal(oc_root, show_archived, &connection) {
                Ok(oc_sessions) => sessions.extend(oc_sessions),
                Err(error) => {
                    eprintln!("opencode provider error (ignored): {error}");
                }
            }
        }
    }

    sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

    Ok(sessions)
}

fn detect_terminal_path() -> Result<Option<String>, String> {
    for terminal_name in ["pwsh", "powershell"] {
        let output = Command::new("where")
            .arg(terminal_name)
            .output()
            .map_err(|error| format!("failed to execute where command: {error}"))?;

        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .map(|line| line.trim().to_string());

            if value.is_some() {
                return Ok(value);
            }
        }
    }

    Ok(None)
}

fn detect_vscode_path() -> Result<Option<String>, String> {
    let output = Command::new("where")
        .arg("code")
        .output()
        .map_err(|error| format!("failed to execute where command: {error}"))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .map(|line| line.trim().to_string()));
    }

    Ok(None)
}

fn load_settings_internal() -> Result<AppSettings, String> {
    let settings_path = settings_file_path()?;

    if !settings_path.exists() {
        return AppSettings::default();
    }

    let content = fs::read_to_string(&settings_path)
        .map_err(|error| format!("failed to read settings file: {error}"))?;

    serde_json::from_str::<AppSettings>(&content)
        .map_err(|error| format!("failed to parse settings file: {error}"))
}

fn save_settings_internal(settings: &AppSettings) -> Result<(), String> {
    let settings_path = settings_file_path()?;
    ensure_parent_dir(&settings_path)?;

    let content = serde_json::to_string_pretty(settings)
        .map_err(|error| format!("failed to serialize settings: {error}"))?;

    fs::write(&settings_path, content)
        .map_err(|error| format!("failed to write settings file: {error}"))?;

    Ok(())
}

fn validate_terminal_path_internal(path: &str) -> bool {
    let candidate = PathBuf::from(path);

    candidate.exists() && candidate.is_file()
}

fn archive_session_internal(root_dir: &Path, session_id: &str) -> Result<(), String> {
    let source_dir = root_dir.join("session-state").join(session_id);
    let target_dir = root_dir.join("session-state-archive").join(session_id);

    if !source_dir.exists() {
        return Err(format!("session {} does not exist", session_id));
    }

    if target_dir.exists() {
        return Err(format!("archived session {} already exists", session_id));
    }

    ensure_parent_dir(&target_dir)?;
    fs::rename(&source_dir, &target_dir)
        .map_err(|error| format!("failed to archive session {}: {error}", session_id))?;

    Ok(())
}

fn unarchive_session_internal(root_dir: &Path, session_id: &str) -> Result<(), String> {
    let source_dir = root_dir.join("session-state-archive").join(session_id);
    let target_dir = root_dir.join("session-state").join(session_id);

    if !source_dir.exists() {
        return Err(format!("archived session {} does not exist", session_id));
    }

    if target_dir.exists() {
        return Err(format!(
            "session {} already exists in session-state",
            session_id
        ));
    }

    ensure_parent_dir(&target_dir)?;
    fs::rename(&source_dir, &target_dir)
        .map_err(|error| format!("failed to unarchive session {}: {error}", session_id))?;

    Ok(())
}

fn delete_session_internal(root_dir: &Path, session_id: &str) -> Result<(), String> {
    for candidate in [
        root_dir.join("session-state").join(session_id),
        root_dir.join("session-state-archive").join(session_id),
    ] {
        if candidate.exists() {
            fs::remove_dir_all(&candidate)
                .map_err(|error| format!("failed to delete session {}: {error}", session_id))?;

            let connection = open_db_connection()?;
            init_db(&connection)?;
            delete_session_meta_internal(&connection, session_id)?;

            return Ok(());
        }
    }

    Err(format!("session {} does not exist", session_id))
}

fn delete_empty_sessions_internal(copilot_root: &str) -> Result<usize, String> {
    let root = PathBuf::from(copilot_root);
    let session_state_dir = root.join("session-state");

    if !session_state_dir.exists() {
        return Ok(0);
    }

    let entries = fs::read_dir(&session_state_dir)
        .map_err(|error| format!("failed to read session-state directory: {error}"))?;

    let mut deleted_count: usize = 0;

    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read directory entry: {error}"))?;
        let session_dir = entry.path();

        if !session_dir.is_dir() {
            continue;
        }

        let workspace_path = session_dir.join("workspace.yaml");
        if !workspace_path.exists() {
            continue;
        }

        let events_path = session_dir.join("events.jsonl");
        let has_events = events_path
            .metadata()
            .map(|meta| meta.len() > 0)
            .unwrap_or(false);
        if has_events {
            continue;
        }

        let session_id = session_dir
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_default();

        match fs::remove_dir_all(&session_dir) {
            Ok(_) => {
                if let Ok(connection) = open_db_connection() {
                    let _ = init_db(&connection);
                    let _ = delete_session_meta_internal(&connection, &session_id);
                }
                deleted_count += 1;
            }
            Err(error) => {
                eprintln!("failed to delete empty session {}: {error}", session_id);
            }
        }
    }

    Ok(deleted_count)
}

fn open_terminal_internal(terminal_path: &str, cwd: &str, session_id: &str) -> Result<(), String> {
    let mut cmd = Command::new(terminal_path);
    cmd.args([
        "-NoExit",
        "-Command",
        &format!("copilot --resume={}", session_id),
    ])
    .current_dir(cwd);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NEW_CONSOLE);

    cmd.spawn()
        .map_err(|error| format!("failed to open terminal: {error}"))?;

    Ok(())
}

fn directory_exists(path: &str) -> bool {
    PathBuf::from(path).is_dir()
}

fn read_plan_internal(session_dir: &str) -> Result<Option<String>, String> {
    let plan_path = PathBuf::from(session_dir).join("plan.md");

    if !plan_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&plan_path)
        .map_err(|error| format!("failed to read plan file {}: {error}", plan_path.display()))?;

    Ok(Some(content))
}

fn write_plan_internal(session_dir: &str, content: &str) -> Result<(), String> {
    let plan_path = PathBuf::from(session_dir).join("plan.md");
    fs::write(&plan_path, content)
        .map_err(|error| format!("failed to write plan file {}: {error}", plan_path.display()))?;

    Ok(())
}

fn open_plan_external_internal(session_dir: &str, editor_cmd: Option<&str>) -> Result<(), String> {
    let plan_path = PathBuf::from(session_dir).join("plan.md");

    if !plan_path.exists() {
        write_plan_internal(session_dir, "")?;
    }

    if let Some(editor_path) = editor_cmd.filter(|value| !value.trim().is_empty()) {
        Command::new(editor_path)
            .arg(&plan_path)
            .spawn()
            .map_err(|error| format!("failed to open external editor: {error}"))?;

        return Ok(());
    }

    if let Some(vscode_path) = detect_vscode_path()? {
        Command::new(vscode_path)
            .arg(&plan_path)
            .spawn()
            .map_err(|error| format!("failed to open VSCode: {error}"))?;

        return Ok(());
    }

    Command::new("cmd")
        .args(["/C", "start", "", &plan_path.to_string_lossy()])
        .spawn()
        .map_err(|error| format!("failed to open plan with default app: {error}"))?;

    Ok(())
}

#[tauri::command]
fn get_sessions(
    root_dir: Option<String>,
    opencode_root: Option<String>,
    show_archived: Option<bool>,
    enabled_providers: Option<Vec<String>>,
) -> Result<Vec<SessionInfo>, String> {
    let resolved_copilot = resolve_copilot_root(root_dir.as_deref())?;
    let resolved_opencode = resolve_opencode_root(opencode_root.as_deref()).ok();
    let providers = enabled_providers.unwrap_or_else(default_enabled_providers);

    scan_sessions(
        &resolved_copilot,
        resolved_opencode.as_deref(),
        show_archived.unwrap_or(false),
        &providers,
    )
}

#[tauri::command]
fn get_settings() -> Result<AppSettings, String> {
    load_settings_internal()
}

#[tauri::command]
fn save_settings(settings: AppSettings) -> Result<(), String> {
    save_settings_internal(&settings)
}

#[tauri::command]
fn detect_terminal() -> Result<Option<String>, String> {
    detect_terminal_path()
}

#[tauri::command]
fn detect_vscode() -> Result<Option<String>, String> {
    detect_vscode_path()
}

#[tauri::command]
fn restart_session_watcher(
    app: tauri::AppHandle,
    watcher_state: State<'_, WatcherState>,
    copilot_root: Option<String>,
    opencode_root: Option<String>,
    enabled_providers: Option<Vec<String>>,
) -> Result<(), String> {
    let providers = enabled_providers.unwrap_or_else(default_enabled_providers);
    restart_session_watcher_internal(
        &app,
        &watcher_state,
        copilot_root.as_deref(),
        opencode_root.as_deref(),
        &providers,
    )
}

#[tauri::command]
fn watch_plan_file(
    app: tauri::AppHandle,
    watcher_state: State<'_, WatcherState>,
    session_dir: String,
) -> Result<(), String> {
    watch_plan_file_internal(&app, &watcher_state, &session_dir)
}

#[tauri::command]
fn stop_plan_watch(watcher_state: State<'_, WatcherState>) -> Result<(), String> {
    let mut plan_watcher = watcher_state
        .plan
        .lock()
        .map_err(|_| "failed to lock plan watcher state".to_string())?;
    *plan_watcher = None;
    Ok(())
}

#[tauri::command]
fn validate_terminal_path(path: String) -> bool {
    validate_terminal_path_internal(&path)
}

#[tauri::command]
fn upsert_session_meta(
    session_id: String,
    notes: Option<String>,
    tags: Vec<String>,
) -> Result<(), String> {
    let connection = open_db_connection()?;
    init_db(&connection)?;
    upsert_session_meta_internal(&connection, &session_id, notes, tags)
}

#[tauri::command]
fn delete_session_meta(session_id: String) -> Result<(), String> {
    let connection = open_db_connection()?;
    init_db(&connection)?;
    delete_session_meta_internal(&connection, &session_id)
}

#[tauri::command]
fn archive_session(root_dir: Option<String>, session_id: String) -> Result<(), String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    archive_session_internal(&resolved_root, &session_id)
}

#[tauri::command]
fn unarchive_session(root_dir: Option<String>, session_id: String) -> Result<(), String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    unarchive_session_internal(&resolved_root, &session_id)
}

#[tauri::command]
fn delete_session(root_dir: Option<String>, session_id: String) -> Result<(), String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    delete_session_internal(&resolved_root, &session_id)
}

#[tauri::command]
fn delete_empty_sessions(root_dir: Option<String>) -> Result<usize, String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    delete_empty_sessions_internal(&resolved_root.to_string_lossy())
}

#[tauri::command]
fn get_session_stats(session_dir: String) -> Result<SessionStats, String> {
    let connection = open_db_connection()?;
    init_db(&connection)?;
    get_session_stats_internal(&connection, &session_dir)
}

#[tauri::command]
fn open_terminal(terminal_path: String, cwd: String, session_id: String) -> Result<(), String> {
    open_terminal_internal(&terminal_path, &cwd, &session_id)
}

#[tauri::command]
fn check_directory_exists(path: String) -> bool {
    directory_exists(&path)
}

#[tauri::command]
fn read_plan(session_dir: String) -> Result<Option<String>, String> {
    read_plan_internal(&session_dir)
}

#[tauri::command]
fn write_plan(session_dir: String, content: String) -> Result<(), String> {
    write_plan_internal(&session_dir, &content)
}

#[tauri::command]
fn open_plan_external(session_dir: String, editor_cmd: Option<String>) -> Result<(), String> {
    open_plan_external_internal(&session_dir, editor_cmd.as_deref())
}

// ========================
// Sisyphus (.sisyphus) Reader
// ========================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SisyphusBoulder {
    active_plan: Option<String>,
    plan_name: Option<String>,
    agent: Option<String>,
    session_ids: Vec<String>,
    started_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SisyphusPlan {
    name: String,
    path: String,
    title: Option<String>,
    tldr: Option<String>,
    is_active: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SisyphusNotepad {
    name: String,
    has_issues: bool,
    has_learnings: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SisyphusData {
    active_plan: Option<SisyphusBoulder>,
    plans: Vec<SisyphusPlan>,
    notepads: Vec<SisyphusNotepad>,
    evidence_files: Vec<String>,
    draft_files: Vec<String>,
}

/// 從 Markdown 內容取得第一個 `# heading`
fn extract_md_heading(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            let heading = heading.trim();
            if !heading.is_empty() {
                return Some(heading.to_string());
            }
        }
    }
    None
}

/// 從 Markdown 內容取得 `## TL;DR` section 的前幾行
fn extract_md_tldr(content: &str) -> Option<String> {
    let mut in_tldr = false;
    let mut lines: Vec<&str> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if in_tldr {
            // 遇到下一個 heading 就結束
            if trimmed.starts_with("## ") || trimmed.starts_with("# ") {
                break;
            }
            if !trimmed.is_empty() {
                lines.push(trimmed);
                if lines.len() >= 5 {
                    break;
                }
            }
        } else if trimmed.starts_with("## TL;DR") || trimmed.starts_with("## tl;dr") {
            in_tldr = true;
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// 掃描 `.sisyphus/` 目錄，回傳完整 SisyphusData
fn scan_sisyphus_internal(project_dir: &Path) -> SisyphusData {
    let sisyphus_dir = project_dir.join(".sisyphus");

    if !sisyphus_dir.is_dir() {
        return SisyphusData {
            active_plan: None,
            plans: Vec::new(),
            notepads: Vec::new(),
            evidence_files: Vec::new(),
            draft_files: Vec::new(),
        };
    }

    // 9.3: 解析 boulder.json
    let boulder = {
        let boulder_path = sisyphus_dir.join("boulder.json");
        if boulder_path.is_file() {
            fs::read_to_string(&boulder_path)
                .ok()
                .and_then(|content| serde_json::from_str::<SisyphusBoulder>(&content).ok())
        } else {
            None
        }
    };

    let active_plan_name = boulder
        .as_ref()
        .and_then(|b| b.active_plan.as_deref())
        .map(|p| {
            // active_plan 可能是路徑或名稱，取出檔名去除副檔名
            Path::new(p)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(p)
                .to_string()
        });

    // 9.4: 掃描 plans/*.md
    let plans = {
        let plans_dir = sisyphus_dir.join("plans");
        if plans_dir.is_dir() {
            let mut result = Vec::new();
            if let Ok(entries) = fs::read_dir(&plans_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("md") {
                        let name = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let content = fs::read_to_string(&path).unwrap_or_default();
                        let title = extract_md_heading(&content);
                        let tldr = extract_md_tldr(&content);
                        let is_active = active_plan_name
                            .as_deref()
                            .map(|ap| ap == name)
                            .unwrap_or(false);

                        result.push(SisyphusPlan {
                            name,
                            path: path.to_string_lossy().to_string(),
                            title,
                            tldr,
                            is_active,
                        });
                    }
                }
            }
            result.sort_by(|a, b| a.name.cmp(&b.name));
            result
        } else {
            Vec::new()
        }
    };

    // 9.5: 掃描 notepads/*/
    let notepads = {
        let notepads_dir = sisyphus_dir.join("notepads");
        if notepads_dir.is_dir() {
            let mut result = Vec::new();
            if let Ok(entries) = fs::read_dir(&notepads_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let has_issues = path.join("issues.md").is_file();
                        let has_learnings = path.join("learnings.md").is_file();
                        result.push(SisyphusNotepad {
                            name,
                            has_issues,
                            has_learnings,
                        });
                    }
                }
            }
            result.sort_by(|a, b| a.name.cmp(&b.name));
            result
        } else {
            Vec::new()
        }
    };

    // 9.6: 掃描 evidence/*.txt 與 drafts/*.md
    let evidence_files = list_files_with_ext(&sisyphus_dir.join("evidence"), "txt");
    let draft_files = list_files_with_ext(&sisyphus_dir.join("drafts"), "md");

    SisyphusData {
        active_plan: boulder,
        plans,
        notepads,
        evidence_files,
        draft_files,
    }
}

/// 列舉指定目錄下指定副檔名的檔案名稱清單
fn list_files_with_ext(dir: &Path, ext: &str) -> Vec<String> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some(ext) {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    result.push(name.to_string());
                }
            }
        }
    }
    result.sort();
    result
}

// ========================
// OpenSpec Reader
// ========================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenSpecChange {
    name: String,
    has_proposal: bool,
    has_design: bool,
    has_tasks: bool,
    specs_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenSpecSpec {
    name: String,
    path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenSpecData {
    schema: Option<String>,
    active_changes: Vec<OpenSpecChange>,
    archived_changes: Vec<OpenSpecChange>,
    specs: Vec<OpenSpecSpec>,
}

/// 掃描單一 change 目錄，回傳 OpenSpecChange
fn scan_openspec_change(change_dir: &Path) -> OpenSpecChange {
    let name = change_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let has_proposal = change_dir.join("proposal.md").is_file();
    let has_design = change_dir.join("design.md").is_file();
    let has_tasks = change_dir.join("tasks.md").is_file();

    let specs_dir = change_dir.join("specs");
    let specs_count = if specs_dir.is_dir() {
        fs::read_dir(&specs_dir)
            .map(|entries| entries.flatten().filter(|e| e.path().is_dir()).count())
            .unwrap_or(0)
    } else {
        0
    };

    OpenSpecChange {
        name,
        has_proposal,
        has_design,
        has_tasks,
        specs_count,
    }
}

/// 掃描 `openspec/` 目錄，回傳完整 OpenSpecData
fn scan_openspec_internal(project_dir: &Path) -> OpenSpecData {
    let openspec_dir = project_dir.join("openspec");

    if !openspec_dir.is_dir() {
        return OpenSpecData {
            schema: None,
            active_changes: Vec::new(),
            archived_changes: Vec::new(),
            specs: Vec::new(),
        };
    }

    // 10.3: 解析 config.yaml
    let schema = {
        let config_path = openspec_dir.join("config.yaml");
        if config_path.is_file() {
            fs::read_to_string(&config_path)
                .ok()
                .and_then(|content| serde_yaml::from_str::<serde_yaml::Value>(&content).ok())
                .and_then(|value| {
                    value
                        .get("schema")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
        } else {
            None
        }
    };

    // 10.4: 掃描 changes/（排除 archive/）
    let active_changes = {
        let changes_dir = openspec_dir.join("changes");
        if changes_dir.is_dir() {
            let mut result = Vec::new();
            if let Ok(entries) = fs::read_dir(&changes_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                        // 排除 archive 目錄
                        if dir_name != "archive" {
                            result.push(scan_openspec_change(&path));
                        }
                    }
                }
            }
            result.sort_by(|a, b| a.name.cmp(&b.name));
            result
        } else {
            Vec::new()
        }
    };

    // 10.5: 掃描 changes/archive/
    let archived_changes = {
        let archive_dir = openspec_dir.join("changes").join("archive");
        if archive_dir.is_dir() {
            let mut result = Vec::new();
            if let Ok(entries) = fs::read_dir(&archive_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        result.push(scan_openspec_change(&path));
                    }
                }
            }
            result.sort_by(|a, b| a.name.cmp(&b.name));
            result
        } else {
            Vec::new()
        }
    };

    // 10.6: 掃描 specs/
    let specs = {
        let specs_dir = openspec_dir.join("specs");
        if specs_dir.is_dir() {
            let mut result = Vec::new();
            if let Ok(entries) = fs::read_dir(&specs_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let spec_md_path = path.join("spec.md");
                        let spec_path = if spec_md_path.is_file() {
                            spec_md_path.to_string_lossy().to_string()
                        } else {
                            path.to_string_lossy().to_string()
                        };
                        result.push(OpenSpecSpec {
                            name,
                            path: spec_path,
                        });
                    }
                }
            }
            result.sort_by(|a, b| a.name.cmp(&b.name));
            result
        } else {
            Vec::new()
        }
    };

    OpenSpecData {
        schema,
        active_changes,
        archived_changes,
        specs,
    }
}

// ========================
// Tauri Commands: Plans & Specs
// ========================

#[tauri::command]
fn get_project_plans(project_dir: String) -> Result<SisyphusData, String> {
    Ok(scan_sisyphus_internal(Path::new(&project_dir)))
}

#[tauri::command]
fn get_project_specs(project_dir: String) -> Result<OpenSpecData, String> {
    Ok(scan_openspec_internal(Path::new(&project_dir)))
}

#[tauri::command]
fn read_plan_content(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);
    if !path.is_file() {
        return Err(format!("File not found: {}", file_path));
    }
    fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(WatcherState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let settings = load_settings_internal().unwrap_or(AppSettings::default()?);
            let watcher_state = app.state::<WatcherState>();
            restart_session_watcher_internal(
                app.handle(),
                &watcher_state,
                Some(&settings.copilot_root),
                Some(&settings.opencode_root),
                &settings.enabled_providers,
            )?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_sessions,
            get_settings,
            save_settings,
            detect_terminal,
            detect_vscode,
            restart_session_watcher,
            watch_plan_file,
            stop_plan_watch,
            validate_terminal_path,
            archive_session,
            unarchive_session,
            delete_session,
            delete_empty_sessions,
            get_session_stats,
            open_terminal,
            check_directory_exists,
            read_plan,
            write_plan,
            open_plan_external,
            upsert_session_meta,
            delete_session_meta,
            get_project_plans,
            get_project_specs,
            read_plan_content
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn with_appdata<T>(appdata_dir: &Path, callback: impl FnOnce() -> T) -> T {
        unsafe {
            env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", appdata_dir);
        }
        let result = callback();
        unsafe {
            env::remove_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE");
        }
        result
    }

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        std::env::temp_dir().join(format!("session-hub-{name}-{suffix}"))
    }

    #[test]
    fn scan_sessions_reads_workspace_yaml_and_plan_flag() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let root_dir = unique_test_dir("scan");
        let appdata_dir = unique_test_dir("appdata");
        let session_dir = root_dir.join("session-state").join("session-001");

        fs::create_dir_all(&session_dir).expect("failed to create session dir");
        fs::write(
            session_dir.join("workspace.yaml"),
            "id: session-001\ncwd: D:\\\\repo\\\\demo\nsummary: Test Session\nsummary_count: 3\ncreated_at: 2025-01-01T00:00:00Z\nupdated_at: 2025-01-02T00:00:00Z\n",
        )
        .expect("failed to write workspace yaml");
        fs::write(session_dir.join("plan.md"), "# Plan").expect("failed to write plan");

        unsafe {
            env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", &appdata_dir);
        }

        let sessions = scan_sessions(&root_dir, None, false, &default_enabled_providers())
            .expect("scan should succeed");

        unsafe {
            env::remove_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE");
        }

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "session-001");
        assert_eq!(sessions[0].summary.as_deref(), Some("Test Session"));
        assert_eq!(sessions[0].summary_count, Some(3));
        assert!(sessions[0].has_plan);
        assert!(!sessions[0].parse_error);

        fs::remove_dir_all(&root_dir).expect("failed to cleanup root dir");
        fs::remove_dir_all(&appdata_dir).expect("failed to cleanup appdata dir");
    }

    #[test]
    fn validate_terminal_path_returns_true_for_existing_file() {
        let test_dir = unique_test_dir("terminal");
        fs::create_dir_all(&test_dir).expect("failed to create terminal test dir");
        let terminal_path = test_dir.join("pwsh.exe");
        fs::write(&terminal_path, "").expect("failed to create fake terminal");

        assert!(validate_terminal_path_internal(
            terminal_path.to_string_lossy().as_ref()
        ));
        assert!(!validate_terminal_path_internal(
            test_dir.join("missing.exe").to_string_lossy().as_ref()
        ));

        fs::remove_dir_all(&test_dir).expect("failed to cleanup terminal test dir");
    }

    #[test]
    fn delete_empty_sessions_returns_zero_when_no_sessions() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let root_dir = unique_test_dir("empty-del-none");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(root_dir.join("session-state")).expect("failed to create session-state");
        fs::create_dir_all(&appdata_dir).expect("failed to create appdata dir");

        unsafe {
            env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", &appdata_dir);
        }
        let result = delete_empty_sessions_internal(&root_dir.to_string_lossy());
        unsafe {
            env::remove_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE");
        }

        assert_eq!(result.expect("should succeed"), 0);

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn delete_empty_sessions_deletes_sessions_without_events() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let root_dir = unique_test_dir("empty-del-some");
        let appdata_dir = unique_test_dir("appdata");

        // session without events.jsonl (should be deleted)
        let empty_session = root_dir.join("session-state").join("session-empty");
        fs::create_dir_all(&empty_session).expect("create empty session dir");
        fs::write(empty_session.join("workspace.yaml"), "id: session-empty\n")
            .expect("write workspace.yaml");

        // session with non-empty events.jsonl (should be kept)
        let active_session = root_dir.join("session-state").join("session-active");
        fs::create_dir_all(&active_session).expect("create active session dir");
        fs::write(
            active_session.join("workspace.yaml"),
            "id: session-active\n",
        )
        .expect("write workspace.yaml");
        fs::write(
            active_session.join("events.jsonl"),
            "{\"type\":\"session_start\"}\n",
        )
        .expect("write events.jsonl");

        // session with empty events.jsonl (should be deleted)
        let no_count_session = root_dir.join("session-state").join("session-no-count");
        fs::create_dir_all(&no_count_session).expect("create no-count session dir");
        fs::write(
            no_count_session.join("workspace.yaml"),
            "id: session-no-count\n",
        )
        .expect("write workspace.yaml");
        fs::write(no_count_session.join("events.jsonl"), "").expect("write empty events.jsonl");

        unsafe {
            env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", &appdata_dir);
        }
        let result = delete_empty_sessions_internal(&root_dir.to_string_lossy());
        unsafe {
            env::remove_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE");
        }

        assert_eq!(result.expect("should succeed"), 2);
        assert!(!empty_session.exists(), "empty session should be deleted");
        assert!(active_session.exists(), "active session should remain");
        assert!(
            !no_count_session.exists(),
            "no-count session should be deleted"
        );

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn delete_empty_sessions_returns_zero_when_no_session_state_dir() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let root_dir = unique_test_dir("empty-del-nodir");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&root_dir).expect("create root dir");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");

        unsafe {
            env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", &appdata_dir);
        }
        let result = delete_empty_sessions_internal(&root_dir.to_string_lossy());
        unsafe {
            env::remove_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE");
        }

        assert_eq!(result.expect("should succeed"), 0);

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn parse_stats_empty_dir_returns_zero_values() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let session_dir = unique_test_dir("stats-empty");
        fs::create_dir_all(&session_dir).expect("create session dir");

        let stats = parse_session_stats_internal(&session_dir).expect("stats should parse");

        assert_eq!(stats, SessionStats::default());

        fs::remove_dir_all(&session_dir).expect("cleanup session dir");
    }

    #[test]
    fn parse_stats_basic_reads_top_level_counts() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let root_dir = unique_test_dir("stats-basic");
        let appdata_dir = unique_test_dir("appdata");
        let session_dir = root_dir.join("session-state").join("session-basic");
        fs::create_dir_all(&session_dir).expect("create session dir");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");
        fs::write(
            session_dir.join("events.jsonl"),
            concat!(
                "{\"type\":\"session.start\",\"data\":{\"startTime\":\"2026-03-31T10:00:00Z\",\"selectedModel\":\"gpt-4.1\"},\"timestamp\":\"2026-03-31T10:00:00Z\"}\n",
                "{\"type\":\"user.message\",\"data\":{},\"timestamp\":\"2026-03-31T10:01:00Z\"}\n",
                "{\"type\":\"tool.execution_start\",\"data\":{\"toolName\":\"grep\"},\"timestamp\":\"2026-03-31T10:02:00Z\"}\n",
                "{\"type\":\"assistant.message\",\"data\":{\"outputTokens\":120,\"reasoningOpaque\":\"opaque\"},\"timestamp\":\"2026-03-31T10:05:00Z\"}\n",
                "{\"type\":\"session.model_change\",\"data\":{\"newModel\":\"gpt-5.4\"},\"timestamp\":\"2026-03-31T10:06:00Z\"}\n"
            ),
        )
        .expect("write events");

        let stats = with_appdata(&appdata_dir, || {
            let connection = open_db_connection().expect("open db");
            init_db(&connection).expect("init db");
            get_session_stats_internal(&connection, &session_dir.to_string_lossy())
                .expect("stats should parse")
        });

        assert_eq!(stats.output_tokens, 120);
        assert_eq!(stats.interaction_count, 1);
        assert_eq!(stats.tool_call_count, 1);
        assert_eq!(stats.duration_minutes, 6);
        assert_eq!(stats.reasoning_count, 1);
        assert_eq!(
            stats.models_used,
            vec!["gpt-4.1".to_string(), "gpt-5.4".to_string()]
        );
        assert_eq!(stats.tool_breakdown.get("grep"), Some(&1));

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn parse_stats_skips_subagent_events() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let root_dir = unique_test_dir("stats-subagent");
        let appdata_dir = unique_test_dir("appdata");
        let session_dir = root_dir.join("session-state").join("session-subagent");
        fs::create_dir_all(&session_dir).expect("create session dir");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");
        fs::write(
            session_dir.join("events.jsonl"),
            concat!(
                "{\"type\":\"session.start\",\"data\":{\"startTime\":\"2026-03-31T10:00:00Z\"},\"timestamp\":\"2026-03-31T10:00:00Z\"}\n",
                "{\"type\":\"user.message\",\"data\":{},\"timestamp\":\"2026-03-31T10:01:00Z\"}\n",
                "{\"type\":\"tool.execution_start\",\"data\":{\"toolName\":\"grep\",\"parentToolCallId\":\"call-1\"},\"timestamp\":\"2026-03-31T10:02:00Z\"}\n",
                "{\"type\":\"assistant.message\",\"data\":{\"parentToolCallId\":\"call-1\",\"outputTokens\":200,\"reasoningOpaque\":\"opaque\"},\"timestamp\":\"2026-03-31T10:03:00Z\"}\n"
            ),
        )
        .expect("write events");

        let stats = with_appdata(&appdata_dir, || {
            let connection = open_db_connection().expect("open db");
            init_db(&connection).expect("init db");
            get_session_stats_internal(&connection, &session_dir.to_string_lossy())
                .expect("stats should parse")
        });

        assert_eq!(stats.interaction_count, 1);
        assert_eq!(stats.tool_call_count, 0);
        assert_eq!(stats.output_tokens, 0);
        assert_eq!(stats.reasoning_count, 0);

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }
}
