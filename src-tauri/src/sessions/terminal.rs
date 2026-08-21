use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::sessions::{
    ensure_herdr_client_console, herdr_pane_run, herdr_tab_create, HerdrTab,
};
use crate::settings::TERMINAL_LAUNCHER_HERDR;
use crate::types::HerdrTabState;
#[cfg(target_os = "windows")]
use crate::types::CREATE_NEW_CONSOLE;

pub(crate) struct TerminalLaunchSpec<'a> {
    pub(crate) cwd: &'a str,
    pub(crate) command: Option<&'a str>,
    pub(crate) label: &'a str,
}

pub(crate) fn project_terminal_label(cwd: &str, tool: Option<&str>) -> String {
    let project = Path::new(cwd)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(cwd);
    match tool.filter(|value| !value.trim().is_empty()) {
        Some(tool) => format!("{project} · {tool}"),
        None => project.to_string(),
    }
}

pub(crate) fn remember_herdr_tab(
    state: &HerdrTabState,
    session_id: Option<&str>,
    cwd: &str,
    tab_id: &str,
) -> Result<(), String> {
    let mut tabs = state
        .session_tabs
        .lock()
        .map_err(|_| "failed to lock herdr tab state".to_string())?;
    if let Some(session_id) = session_id.filter(|value| !value.trim().is_empty()) {
        tabs.insert(session_id.to_string(), tab_id.to_string());
    }
    if let Some(project_name) = Path::new(cwd)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
    {
        tabs.insert(project_name.to_string(), tab_id.to_string());
    }
    Ok(())
}

pub(crate) fn launch_terminal(
    launcher: &str,
    terminal_path: &str,
    spec: TerminalLaunchSpec<'_>,
) -> Result<Option<HerdrTab>, String> {
    if launcher == TERMINAL_LAUNCHER_HERDR {
        launch_via_herdr(spec)
    } else {
        launch_via_shell(terminal_path, spec)?;
        Ok(None)
    }
}

fn launch_via_shell(terminal_path: &str, spec: TerminalLaunchSpec<'_>) -> Result<(), String> {
    let terminal = PathBuf::from(terminal_path);
    let stem = terminal
        .file_stem()
        .and_then(|value| value.to_str())
        .map(|value| value.to_lowercase())
        .unwrap_or_default();
    let mut command = Command::new(terminal_path);
    command.current_dir(spec.cwd);

    match spec.command {
        Some(initial_command) => {
            if stem == "cmd" {
                command.args(["/K", &format!("cd /d \"{}\" && {}", spec.cwd, initial_command)]);
            } else {
                command.args([
                    "-NoExit",
                    "-Command",
                    &format!("cd '{}'; {}", spec.cwd, initial_command),
                ]);
            }
        }
        None => match stem.as_str() {
            "cmd" => {
                command.args(["/K", &format!("cd /d \"{}\"", spec.cwd)]);
            }
            "bash" | "sh" => {
                command.arg("-i");
            }
            _ => {
                command.args(["-NoExit", "-Command", &format!("cd '{}'", spec.cwd)]);
            }
        },
    };

    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NEW_CONSOLE);
    crate::sessions::configure_msys_stackdump_suppression(&mut command);
    command
        .spawn()
        .map_err(|error| format!("failed to open terminal: {error}"))?;
    Ok(())
}

fn launch_via_herdr(spec: TerminalLaunchSpec<'_>) -> Result<Option<HerdrTab>, String> {
    // 先確保有 TUI client 承載，否則 tab 只會建立在 headless server 中，畫面上看不到任何東西。
    ensure_herdr_client_console()?;

    let tab = herdr_tab_create(spec.cwd, spec.label, true)?;
    if let Some(command) = spec.command {
        herdr_pane_run(&tab.pane_id, command)?;
    }
    Ok(Some(tab))
}
