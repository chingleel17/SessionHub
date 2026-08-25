use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use tauri::{Emitter, Manager, State};

use crate::openspec_scan::{
    read_openspec_file_internal, scan_openspec_internal, write_openspec_file_internal,
};
use crate::sessions::{
    herdr_server_is_running, herdr_tab_focus, launch_terminal, project_terminal_label,
    remember_herdr_tab, TerminalLaunchSpec,
};
use crate::settings::{
    load_settings_internal, resolve_herdr_executable, resolve_terminal_launcher,
    resolve_vscode_command, TERMINAL_LAUNCHER_HERDR,
};
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
    let herdr = resolve_herdr_executable().is_some();

    ToolAvailability {
        copilot: which_exists("copilot"),
        opencode: which_exists("opencode"),
        claude: which_exists("claude"),
        codex: which_exists("codex"),
        gemini: which_exists("gemini"),
        vscode: resolve_vscode_command().is_some(),
        herdr,
        herdr_server_running: herdr && herdr_server_is_running().unwrap_or(false),
    }
}

pub(crate) fn focus_terminal_window_internal(
    title_hint: &str,
    tab_state: &HerdrTabState,
) -> Result<(), String> {
    let launcher = load_settings_internal()
        .ok()
        .map(|settings| resolve_terminal_launcher(settings.terminal_launcher.as_deref()));
    if launcher == Some(TERMINAL_LAUNCHER_HERDR) {
        let (session_key, _) = title_hint.split_once('\n').unwrap_or((title_hint, ""));
        let tab_id = tab_state
            .session_tabs
            .lock()
            .map_err(|_| "failed to lock herdr tab state".to_string())?
            .get(session_key)
            .cloned()
            .ok_or_else(|| {
                "找不到此 session 對應的 herdr tab，請手動切換至 herdr 分頁".to_string()
            })?;
        return herdr_tab_focus(&tab_id);
    }

    #[cfg(target_os = "windows")]
    {
        let (_, window_hint) = title_hint.split_once('\n').unwrap_or(("", title_hint));
        crate::platform::win32_focus::focus_window_by_title(window_hint)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = title_hint;
        Err("Terminal focus is only supported on Windows".to_string())
    }
}

pub(crate) fn show_main_window_internal(
    app: &tauri::AppHandle,
    view: Option<&str>,
) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    if let Some(view) = view {
        let _ = app.emit("navigate-main-view", view.to_string());
    }
    Ok(())
}

pub(crate) fn open_in_tool_internal(
    tool_type: &str,
    cwd: &str,
    terminal_path: Option<&str>,
    session_id: Option<&str>,
    tab_state: &HerdrTabState,
) -> Result<(), String> {
    let launcher = load_settings_internal()
        .ok()
        .map(|settings| resolve_terminal_launcher(settings.terminal_launcher.as_deref()))
        .unwrap_or(crate::settings::TERMINAL_LAUNCHER_SHELL);
    let terminal = terminal_path.unwrap_or("pwsh");
    let launch_command = |command: Option<&str>, tool: Option<&str>| {
        let tab = launch_terminal(
            launcher,
            terminal,
            TerminalLaunchSpec {
                cwd,
                command,
                label: &project_terminal_label(cwd, tool),
            },
        )?;
        if let Some(tab) = tab {
            remember_herdr_tab(tab_state, session_id, cwd, &tab.tab_id)?;
        }
        Ok(())
    };

    match tool_type {
        "terminal" => launch_command(None, None),
        "opencode" => launch_command(Some("opencode"), Some("opencode")),
        "claude" => launch_command(Some("claude"), Some("claude")),
        "codex" => launch_command(Some("codex"), Some("codex")),
        "copilot" => launch_command(Some("copilot"), Some("copilot")),
        "vscode" => {
            let editor = resolve_vscode_command()
                .ok_or_else(|| "failed to open vscode: no VS Code executable found".to_string())?;
            let editor_ext = PathBuf::from(&editor)
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase())
                .unwrap_or_default();

            // .cmd / .bat / .ps1 為 shim 腳本，Command 只解析 .exe，需透過終端機執行。
            let mut cmd = if matches!(editor_ext.as_str(), "cmd" | "bat" | "ps1") {
                let term = terminal_path.unwrap_or("pwsh");
                let term_stem = PathBuf::from(term)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();

                let mut c = Command::new(term);
                if term_stem == "cmd" {
                    c.args(["/C", &format!("\"{}\" \"{}\"", editor, cwd)]);
                } else {
                    c.args(["-NoProfile", "-Command", &format!("& '{}' '{}'", editor, cwd)]);
                }
                c
            } else {
                let mut c = Command::new(&editor);
                c.arg(cwd);
                c
            };

            #[cfg(target_os = "windows")]
            cmd.creation_flags(CREATE_NO_WINDOW);
            cmd.spawn()
                .map_err(|e| format!("failed to open vscode: {e}"))?;
            Ok(())
        }
        "gemini" => launch_command(Some("gemini"), Some("gemini")),
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
pub fn focus_terminal_window(
    title_hint: String,
    tab_state: State<'_, HerdrTabState>,
) -> Result<(), String> {
    focus_terminal_window_internal(&title_hint, &tab_state)
}

#[tauri::command]
pub fn show_main_window(app: tauri::AppHandle, view: Option<String>) -> Result<(), String> {
    show_main_window_internal(&app, view.as_deref())
}

#[tauri::command]
pub fn open_in_tool(
    tool_type: String,
    cwd: String,
    terminal_path: Option<String>,
    session_id: Option<String>,
    tab_state: State<'_, HerdrTabState>,
) -> Result<(), String> {
    open_in_tool_internal(
        &tool_type,
        &cwd,
        terminal_path.as_deref(),
        session_id.as_deref(),
        &tab_state,
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
    tab_state: &HerdrTabState,
) -> Result<(), String> {
    let resume_cmd = resume_session_command(provider, session_id)?;
    let launcher = load_settings_internal()
        .ok()
        .map(|settings| resolve_terminal_launcher(settings.terminal_launcher.as_deref()))
        .unwrap_or(crate::settings::TERMINAL_LAUNCHER_SHELL);
    let tab = launch_terminal(
        launcher,
        terminal_path.unwrap_or("pwsh"),
        TerminalLaunchSpec {
            cwd,
            command: Some(&resume_cmd),
            label: &project_terminal_label(cwd, Some(provider)),
        },
    )?;
    if let Some(tab) = tab {
        remember_herdr_tab(tab_state, Some(session_id), cwd, &tab.tab_id)?;
    }
    Ok(())
}

#[tauri::command]
pub fn resume_session_in_terminal(
    provider: String,
    session_id: String,
    cwd: String,
    terminal_path: Option<String>,
    tab_state: State<'_, HerdrTabState>,
) -> Result<(), String> {
    resume_session_in_terminal_internal(
        &provider,
        &session_id,
        &cwd,
        terminal_path.as_deref(),
        &tab_state,
    )
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
