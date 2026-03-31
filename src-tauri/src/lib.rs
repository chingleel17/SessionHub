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
use rusqlite::{params, Connection};
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionInfo {
    id: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    copilot_root: String,
    terminal_path: Option<String>,
    external_editor_path: Option<String>,
    show_archived: bool,
    #[serde(default)]
    pinned_projects: Vec<String>,
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
}

fn default_copilot_root() -> Result<PathBuf, String> {
    let user_profile = env::var("USERPROFILE")
        .map_err(|_| "USERPROFILE environment variable is not set".to_string())?;

    Ok(PathBuf::from(user_profile).join(".copilot"))
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

impl AppSettings {
    fn default() -> Result<Self, String> {
        let terminal_path = detect_terminal_path()?;
        let external_editor_path = detect_vscode_path()?;

        Ok(Self {
            copilot_root: default_copilot_root()?.to_string_lossy().to_string(),
            terminal_path,
            external_editor_path,
            show_archived: false,
            pinned_projects: Vec::new(),
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

fn restart_session_watcher_internal(
    app: &tauri::AppHandle,
    watcher_state: &WatcherState,
    copilot_root: Option<&str>,
) -> Result<(), String> {
    let root = resolve_copilot_root(copilot_root)?;
    let watcher = create_sessions_watcher(app, &root).map_err(|error| {
        eprintln!("failed to restart session watcher: {error}");
        error
    })?;
    let mut session_watcher = watcher_state
        .sessions
        .lock()
        .map_err(|_| "failed to lock session watcher state".to_string())?;
    *session_watcher = Some(watcher);
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
    let fallback_session = || SessionInfo {
        id: fallback_id.clone(),
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
    };

    match fs::read_to_string(workspace_path) {
        Ok(content) => match serde_yaml::from_str::<WorkspaceYaml>(&content) {
            Ok(workspace) => SessionInfo {
                id: workspace.id,
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

fn scan_sessions(root_dir: &Path, show_archived: bool) -> Result<Vec<SessionInfo>, String> {
    let connection = open_db_connection()?;
    init_db(&connection)?;

    let mut sessions = scan_session_dir(&root_dir.join("session-state"), false, &connection)?;

    if show_archived {
        sessions.extend(scan_session_dir(
            &root_dir.join("session-state-archive"),
            true,
            &connection,
        )?);
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

        let content = match fs::read_to_string(&workspace_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let workspace: WorkspaceYaml = match serde_yaml::from_str(&content) {
            Ok(w) => w,
            Err(_) => continue,
        };

        let count = workspace.summary_count.unwrap_or(0);
        if count > 0 {
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
    show_archived: Option<bool>,
) -> Result<Vec<SessionInfo>, String> {
    let resolved_root = resolve_copilot_root(root_dir.as_deref())?;
    scan_sessions(&resolved_root, show_archived.unwrap_or(false))
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
) -> Result<(), String> {
    restart_session_watcher_internal(&app, &watcher_state, copilot_root.as_deref())
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
            delete_session_meta
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

        let sessions = scan_sessions(&root_dir, false).expect("scan should succeed");

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
    fn delete_empty_sessions_deletes_sessions_with_zero_summary_count() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let root_dir = unique_test_dir("empty-del-some");
        let appdata_dir = unique_test_dir("appdata");

        // session with summary_count = 0 (should be deleted)
        let empty_session = root_dir.join("session-state").join("session-empty");
        fs::create_dir_all(&empty_session).expect("create empty session dir");
        fs::write(
            empty_session.join("workspace.yaml"),
            "id: session-empty\nsummary_count: 0\n",
        )
        .expect("write workspace.yaml");

        // session with summary_count = 3 (should be kept)
        let active_session = root_dir.join("session-state").join("session-active");
        fs::create_dir_all(&active_session).expect("create active session dir");
        fs::write(
            active_session.join("workspace.yaml"),
            "id: session-active\nsummary_count: 3\n",
        )
        .expect("write workspace.yaml");

        // session with no summary_count field (should be deleted — defaults to 0)
        let no_count_session = root_dir.join("session-state").join("session-no-count");
        fs::create_dir_all(&no_count_session).expect("create no-count session dir");
        fs::write(
            no_count_session.join("workspace.yaml"),
            "id: session-no-count\n",
        )
        .expect("write workspace.yaml");

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
