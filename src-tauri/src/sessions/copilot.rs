use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use rusqlite::Connection;

use crate::db::{delete_session_meta_internal, ensure_parent_dir, read_session_meta};
use crate::settings::detect_vscode_path;
use crate::types::*;

/// 判斷是否需要執行全掃描
pub(crate) fn should_full_scan(cache: &Option<ProviderCache>, force_full: bool) -> bool {
    if force_full {
        return true;
    }
    match cache {
        None => true,
        Some(c) => c.last_full_scan_at.elapsed().as_secs() > FULL_SCAN_THRESHOLD_SECS,
    }
}

/// 取得目錄的最後修改時間（Unix 秒），失敗時回傳 0
pub(crate) fn dir_mtime_secs(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub(crate) fn parse_workspace_file(
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

pub(crate) fn scan_session_dir(
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

/// Copilot 增量掃描：只重新解析 mtime 有變化或新增的 session 目錄
pub(crate) fn scan_copilot_incremental_internal(
    session_state_dir: &Path,
    is_archived: bool,
    connection: &Connection,
    cache: &mut ProviderCache,
) -> Result<(), String> {
    if !session_state_dir.exists() {
        cache.sessions.retain(|s| s.is_archived != is_archived);
        return Ok(());
    }

    let entries = fs::read_dir(session_state_dir)
        .map_err(|error| format!("failed to read {}: {error}", session_state_dir.display()))?;

    let mut current_ids: HashSet<String> = HashSet::new();

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

        current_ids.insert(session_id.clone());

        let current_mtime = dir_mtime_secs(&session_dir);
        let cached_mtime = cache.session_mtimes.get(&session_id).copied().unwrap_or(-1);

        if current_mtime != cached_mtime {
            let meta = read_session_meta(connection, &session_id)?;
            let info = parse_workspace_file(&session_dir, &workspace_path, is_archived, meta);
            if let Some(pos) = cache.sessions.iter().position(|s| s.id == session_id) {
                cache.sessions[pos] = info;
            } else {
                cache.sessions.push(info);
            }
            cache.session_mtimes.insert(session_id, current_mtime);
        }
    }

    cache.sessions.retain(|s| {
        if s.is_archived != is_archived {
            return true;
        }
        current_ids.contains(&s.id)
    });
    cache
        .session_mtimes
        .retain(|id, _| current_ids.contains(id.as_str()));

    Ok(())
}

pub(crate) fn find_session_by_cwd_internal(
    copilot_root: &Path,
    target_cwd: &str,
    connection: &Connection,
) -> Result<Option<SessionInfo>, String> {
    let normalize = |p: &str| p.replace('\\', "/").to_lowercase();
    let normalized_target = normalize(target_cwd);

    for (dir, is_archived) in [
        (copilot_root.join("session-state"), false),
        (copilot_root.join("session-state-archive"), true),
    ] {
        if !dir.exists() {
            continue;
        }

        let entries = fs::read_dir(&dir)
            .map_err(|e| format!("failed to read {}: {e}", dir.display()))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("failed to read entry: {e}"))?;
            let session_dir = entry.path();
            if !session_dir.is_dir() {
                continue;
            }
            let workspace_path = session_dir.join("workspace.yaml");
            if !workspace_path.exists() {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&workspace_path) {
                if let Ok(workspace) = serde_yaml::from_str::<WorkspaceYaml>(&content) {
                    if workspace
                        .cwd
                        .as_deref()
                        .is_some_and(|c| normalize(c) == normalized_target)
                    {
                        let session_id = session_dir
                            .file_name()
                            .map(|v| v.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let meta = read_session_meta(&connection, &session_id)?;
                        return Ok(Some(parse_workspace_file(
                            &session_dir,
                            &workspace_path,
                            is_archived,
                            meta,
                        )));
                    }
                }
            }
        }
    }

    Ok(None)
}



pub(crate) fn archive_session_internal(root_dir: &Path, session_id: &str) -> Result<(), String> {
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

pub(crate) fn unarchive_session_internal(
    root_dir: &Path,
    session_id: &str,
) -> Result<(), String> {
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

pub(crate) fn delete_session_internal(root_dir: &Path, session_id: &str, connection: &Connection) -> Result<(), String> {
    for candidate in [
        root_dir.join("session-state").join(session_id),
        root_dir.join("session-state-archive").join(session_id),
    ] {
        if candidate.exists() {
            fs::remove_dir_all(&candidate)
                .map_err(|error| format!("failed to delete session {}: {error}", session_id))?;

            delete_session_meta_internal(connection, session_id)?;

            return Ok(());
        }
    }

    Err(format!("session {} does not exist", session_id))
}

pub(crate) fn delete_empty_sessions_internal(copilot_root: &str, connection: &Connection) -> Result<usize, String> {
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
                let _ = delete_session_meta_internal(connection, &session_id);
                deleted_count += 1;
            }
            Err(error) => {
                eprintln!("failed to delete empty session {}: {error}", session_id);
            }
        }
    }

    Ok(deleted_count)
}

// ── Plan / Terminal 操作 ─────────────────────────────────────────────────────

pub(crate) fn open_terminal_internal(terminal_path: &str, cwd: &str) -> Result<(), String> {
    let terminal = PathBuf::from(terminal_path);
    let stem = terminal
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let mut cmd = Command::new(terminal_path);

    match stem.as_str() {
        "cmd" => {
            cmd.args(["/K", &format!("cd /d \"{}\"", cwd)]);
        }
        "bash" | "sh" => {
            cmd.arg("-i").current_dir(cwd);
        }
        _ => {
            cmd.args(["-NoExit", "-Command", &format!("cd '{}'", cwd)])
                .current_dir(cwd);
        }
    }

    if stem != "bash" && stem != "sh" {
        cmd.current_dir(cwd);
    }

    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NEW_CONSOLE);

    cmd.spawn()
        .map_err(|error| format!("failed to open terminal: {error}"))?;

    Ok(())
}

pub(crate) fn directory_exists(path: &str) -> bool {
    PathBuf::from(path).is_dir()
}

pub(crate) fn read_plan_internal(session_dir: &str) -> Result<Option<String>, String> {
    let plan_path = PathBuf::from(session_dir).join("plan.md");

    if !plan_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&plan_path)
        .map_err(|error| format!("failed to read plan file {}: {error}", plan_path.display()))?;

    Ok(Some(content))
}

pub(crate) fn write_plan_internal(session_dir: &str, content: &str) -> Result<(), String> {
    let plan_path = PathBuf::from(session_dir).join("plan.md");
    fs::write(&plan_path, content)
        .map_err(|error| format!("failed to write plan file {}: {error}", plan_path.display()))?;

    Ok(())
}

pub(crate) fn open_plan_external_internal(
    session_dir: &str,
    editor_cmd: Option<&str>,
) -> Result<(), String> {
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
