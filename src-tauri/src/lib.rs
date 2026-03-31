use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

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

    Ok(())
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
}
