use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use tauri::State;

use crate::openspec_scan::{
    read_openspec_file_internal, scan_openspec_internal, write_openspec_file_internal,
};
use crate::sessions::open_terminal_internal;
use crate::sisyphus::scan_sisyphus_internal;
use crate::types::*;
use crate::watcher::watch_project_files_internal;

fn which_exists(cmd: &str) -> bool {
    let mut c = std::process::Command::new("where");
    c.arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(target_os = "windows")]
    c.creation_flags(CREATE_NO_WINDOW);
    c.status().map(|s| s.success()).unwrap_or(false)
}

pub(crate) fn check_tool_availability_internal() -> ToolAvailability {
    ToolAvailability {
        copilot: which_exists("copilot"),
        opencode: which_exists("opencode"),
        gemini: which_exists("gemini"),
        vscode: which_exists("code"),
    }
}

pub(crate) fn focus_terminal_window_internal(title_hint: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        crate::platform::win32_focus::focus_window_by_title(title_hint)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = title_hint;
        Err("Terminal focus is only supported on Windows".to_string())
    }
}

pub(crate) fn open_in_tool_internal(
    tool_type: &str,
    cwd: &str,
    terminal_path: Option<&str>,
    _session_id: Option<&str>,
) -> Result<(), String> {
    match tool_type {
        "terminal" => {
            let path = terminal_path.unwrap_or("pwsh");
            open_terminal_internal(path, cwd)
        }
        "opencode" => {
            let mut cmd = Command::new("opencode");
            cmd.current_dir(cwd);
            #[cfg(target_os = "windows")]
            cmd.creation_flags(CREATE_NEW_CONSOLE);
            cmd.spawn()
                .map_err(|e| format!("failed to open opencode: {e}"))?;
            Ok(())
        }
        "copilot" => {
            let term = terminal_path.unwrap_or("pwsh");
            let term_stem = PathBuf::from(term)
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase())
                .unwrap_or_default();

            let mut cmd = Command::new(term);
            cmd.current_dir(cwd);
            if term_stem == "cmd" {
                cmd.args(["/K", &format!("cd /d \"{}\" && copilot", cwd)]);
            } else {
                cmd.args(["-NoExit", "-Command", &format!("cd '{}'; copilot", cwd)]);
            }
            #[cfg(target_os = "windows")]
            cmd.creation_flags(CREATE_NEW_CONSOLE);
            cmd.spawn()
                .map_err(|e| format!("failed to open copilot: {e}"))?;
            Ok(())
        }
        "vscode" => {
            Command::new("code")
                .arg(cwd)
                .spawn()
                .map_err(|e| format!("failed to open vscode: {e}"))?;
            Ok(())
        }
        "gemini" => {
            let term = terminal_path.unwrap_or("pwsh");
            let term_stem = PathBuf::from(term)
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase())
                .unwrap_or_default();
            let mut cmd = Command::new(term);
            cmd.current_dir(cwd);
            if term_stem == "cmd" {
                cmd.args(["/K", &format!("cd /d \"{}\" && gemini", cwd)]);
            } else {
                cmd.args(["-NoExit", "-Command", &format!("cd '{}'; gemini", cwd)]);
            }
            #[cfg(target_os = "windows")]
            cmd.creation_flags(CREATE_NEW_CONSOLE);
            cmd.spawn()
                .map_err(|e| format!("failed to open gemini: {e}"))?;
            Ok(())
        }
        "explorer" => {
            Command::new("explorer")
                .arg(cwd)
                .spawn()
                .map_err(|e| format!("failed to open explorer: {e}"))?;
            Ok(())
        }
        unknown => Err(format!("unsupported tool type: {unknown}")),
    }
}

#[tauri::command]
pub fn check_tool_availability() -> ToolAvailability {
    check_tool_availability_internal()
}

#[tauri::command]
pub fn check_jq_available() -> bool {
    which_exists("jq")
}

#[tauri::command]
pub fn focus_terminal_window(title_hint: String) -> Result<(), String> {
    focus_terminal_window_internal(&title_hint)
}

#[tauri::command]
pub fn open_in_tool(
    tool_type: String,
    cwd: String,
    terminal_path: Option<String>,
    session_id: Option<String>,
) -> Result<(), String> {
    open_in_tool_internal(
        &tool_type,
        &cwd,
        terminal_path.as_deref(),
        session_id.as_deref(),
    )
}

/// Provider → resume 指令對照。與前端 `src/App.tsx` 的 `getSessionOpenCommand`（複製指令功能）保持同步。
pub(crate) fn resume_session_command(provider: &str, session_id: &str) -> Result<String, String> {
    match provider {
        "claude" => Ok(format!("claude --resume={session_id}")),
        "codex" => Ok(format!("codex resume {session_id}")),
        "copilot" => Ok(format!("copilot --resume={session_id}")),
        "opencode" => Ok(format!("opencode --session {session_id}")),
        unknown => Err(format!("unsupported provider: {unknown}")),
    }
}

pub(crate) fn resume_session_in_terminal_internal(
    provider: &str,
    session_id: &str,
    cwd: &str,
    terminal_path: Option<&str>,
) -> Result<(), String> {
    let resume_cmd = resume_session_command(provider, session_id)?;
    let term = terminal_path.unwrap_or("pwsh");
    let term_stem = PathBuf::from(term)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let mut cmd = Command::new(term);
    cmd.current_dir(cwd);
    if term_stem == "cmd" {
        cmd.args(["/K", &format!("cd /d \"{}\" && {}", cwd, resume_cmd)]);
    } else {
        cmd.args([
            "-NoExit",
            "-Command",
            &format!("cd '{}'; {}", cwd, resume_cmd),
        ]);
    }
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NEW_CONSOLE);
    cmd.spawn()
        .map_err(|e| format!("failed to resume session: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn resume_session_in_terminal(
    provider: String,
    session_id: String,
    cwd: String,
    terminal_path: Option<String>,
) -> Result<(), String> {
    resume_session_in_terminal_internal(&provider, &session_id, &cwd, terminal_path.as_deref())
}

#[tauri::command]
pub async fn get_project_plans(project_dir: String) -> Result<SisyphusData, String> {
    // 在後台執行掃描，避免阻塞 UI 執行緒
    let result =
        std::thread::spawn(move || scan_sisyphus_internal(std::path::Path::new(&project_dir)))
            .join();

    match result {
        Ok(data) => Ok(data),
        Err(_) => Err("plan scan thread panicked".to_string()),
    }
}

#[tauri::command]
pub async fn get_project_specs(project_dir: String) -> Result<OpenSpecData, String> {
    // 在後台執行掃描，避免阻塞 UI 執行緒
    let result =
        std::thread::spawn(move || scan_openspec_internal(std::path::Path::new(&project_dir)))
            .join();

    match result {
        Ok(data) => Ok(data),
        Err(_) => Err("scan thread panicked".to_string()),
    }
}

#[tauri::command]
pub fn read_openspec_file(project_cwd: String, relative_path: String) -> Result<String, String> {
    read_openspec_file_internal(&project_cwd, &relative_path)
}

#[tauri::command]
pub fn write_openspec_file(
    project_cwd: String,
    relative_path: String,
    content: String,
) -> Result<(), String> {
    write_openspec_file_internal(&project_cwd, &relative_path, &content)
}

#[tauri::command]
pub fn watch_project_files(
    app: tauri::AppHandle,
    watcher_state: State<'_, WatcherState>,
    project_dir: String,
) -> Result<(), String> {
    watch_project_files_internal(&app, &watcher_state, &project_dir)
}

#[tauri::command]
pub fn stop_project_watch(watcher_state: State<'_, WatcherState>) -> Result<(), String> {
    let mut project_watcher = watcher_state
        .project
        .lock()
        .map_err(|_| "failed to lock project watcher state".to_string())?;
    *project_watcher = None;
    Ok(())
}
