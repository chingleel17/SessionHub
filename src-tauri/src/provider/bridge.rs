use std::collections::{BTreeSet, HashMap};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{fs, fs::File};

use tauri::Emitter;

use crate::settings::resolve_provider_bridge_path;
use crate::types::*;

// ── 路徑輔助 ────────────────────────────────────────────────────────────────

pub(crate) fn resolve_copilot_integration_path(copilot_root: &Path) -> PathBuf {
    copilot_root.join("hooks").join(COPILOT_HOOK_FILE_NAME)
}

// ── Provider Bridge 事件名稱與 dedup ────────────────────────────────────────

pub(crate) fn provider_refresh_event_name(provider: &str) -> Result<&'static str, String> {
    match provider {
        COPILOT_PROVIDER => Ok("copilot-sessions-updated"),
        OPENCODE_PROVIDER => Ok("opencode-sessions-updated"),
        _ => Err(format!("unsupported provider: {provider}")),
    }
}

fn provider_bridge_record_fingerprint(record: &ProviderBridgeRecord) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}",
        record.version,
        record.provider,
        record.event_type,
        record.timestamp,
        record.session_id.as_deref().unwrap_or_default(),
        record.cwd.as_deref().unwrap_or_default(),
        record.source_path.as_deref().unwrap_or_default(),
        record.title.as_deref().unwrap_or_default(),
        record.error.as_deref().unwrap_or_default()
    )
}

pub(crate) fn register_provider_bridge_record(
    last_bridge_records: &Arc<Mutex<HashMap<String, String>>>,
    provider: &str,
    record: &ProviderBridgeRecord,
) -> Result<bool, String> {
    let fingerprint = provider_bridge_record_fingerprint(record);
    let mut tracked = last_bridge_records
        .lock()
        .map_err(|_| "failed to lock provider bridge record state".to_string())?;

    if tracked
        .get(provider)
        .is_some_and(|previous| previous == &fingerprint)
    {
        return Ok(false);
    }

    tracked.insert(provider.to_string(), fingerprint);
    Ok(true)
}

pub(crate) fn should_emit_provider_refresh_at(
    refresh_state: &Arc<Mutex<HashMap<String, Instant>>>,
    provider: &str,
    now: Instant,
) -> Result<bool, String> {
    let mut tracked = refresh_state
        .lock()
        .map_err(|_| "failed to lock provider refresh state".to_string())?;
    let dedup_window = Duration::from_millis(PROVIDER_REFRESH_DEDUP_MS);
    tracked.retain(|_, last_emit| now.saturating_duration_since(*last_emit) < dedup_window);

    if tracked
        .get(provider)
        .is_some_and(|last_emit| now.saturating_duration_since(*last_emit) < dedup_window)
    {
        return Ok(false);
    }

    tracked.insert(provider.to_string(), now);
    Ok(true)
}

pub(crate) fn emit_provider_refresh(
    app: &tauri::AppHandle,
    refresh_state: &Arc<Mutex<HashMap<String, Instant>>>,
    provider: &str,
) -> Result<bool, String> {
    if !should_emit_provider_refresh_at(refresh_state, provider, Instant::now())? {
        return Ok(false);
    }

    app.emit(provider_refresh_event_name(provider)?, ())
        .map_err(|error| format!("failed to emit {provider} refresh: {error}"))?;
    Ok(true)
}

pub(crate) fn path_mtime_millis(path: &Path) -> u128 {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

pub(crate) fn is_relevant_watcher_event_kind(kind: &notify::EventKind) -> bool {
    matches!(
        kind,
        notify::EventKind::Create(_)
            | notify::EventKind::Modify(notify::event::ModifyKind::Any)
            | notify::EventKind::Modify(notify::event::ModifyKind::Data(_))
            | notify::EventKind::Modify(notify::event::ModifyKind::Name(_))
            | notify::EventKind::Remove(_)
    )
}

pub(crate) fn path_file_name_matches(path: &Path, expected: &str) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(expected))
}

// ── Copilot Watch Snapshot ───────────────────────────────────────────────────

fn collect_copilot_workspace_state(session_root: &Path) -> (usize, u128) {
    if !session_root.exists() {
        return (0, 0);
    }

    let Ok(entries) = fs::read_dir(session_root) else {
        return (0, 0);
    };

    let mut session_count = 0;
    let mut latest_workspace_mtime_ms = 0;
    for entry in entries.flatten() {
        let session_dir = entry.path();
        if !session_dir.is_dir() {
            continue;
        }

        let workspace_path = session_dir.join("workspace.yaml");
        if !workspace_path.is_file() {
            continue;
        }

        session_count += 1;
        latest_workspace_mtime_ms =
            latest_workspace_mtime_ms.max(path_mtime_millis(&workspace_path));
    }

    (session_count, latest_workspace_mtime_ms)
}

pub(crate) fn build_copilot_watch_snapshot(root: &Path) -> CopilotWatchSnapshot {
    let session_state_dir = root.join("session-state");
    let archive_dir = root.join("session-state-archive");
    let (active_session_count, active_workspace_mtime_ms) =
        collect_copilot_workspace_state(&session_state_dir);
    let (archived_session_count, archived_workspace_mtime_ms) =
        collect_copilot_workspace_state(&archive_dir);

    CopilotWatchSnapshot {
        active_session_count,
        archived_session_count,
        active_workspace_mtime_ms,
        archived_workspace_mtime_ms,
    }
}

pub(crate) fn should_emit_copilot_refresh(
    root: &Path,
    snapshot_state: &Arc<Mutex<CopilotWatchSnapshot>>,
) -> Result<bool, String> {
    let next_snapshot = build_copilot_watch_snapshot(root);
    let mut current_snapshot = snapshot_state
        .lock()
        .map_err(|_| "failed to lock copilot watcher snapshot".to_string())?;

    if *current_snapshot == next_snapshot {
        return Ok(false);
    }

    *current_snapshot = next_snapshot;
    Ok(true)
}

pub(crate) fn is_relevant_copilot_event(
    event: &notify::Event,
    session_roots: &[PathBuf],
) -> bool {
    if !is_relevant_watcher_event_kind(&event.kind) {
        return false;
    }

    event.paths.iter().any(|path| {
        path_file_name_matches(path, "workspace.yaml")
            || session_roots.iter().any(|root| path == root)
            || path
                .parent()
                .is_some_and(|parent| session_roots.iter().any(|root| parent == root))
    })
}

// ── OpenCode Watch Snapshot ──────────────────────────────────────────────────

pub(crate) fn build_opencode_watch_snapshot(opencode_root: &Path) -> OpenCodeWatchSnapshot {
    let session_storage = opencode_root.join("storage").join("session");
    let message_storage = opencode_root.join("storage").join("message");

    OpenCodeWatchSnapshot {
        db_exists: session_storage.exists(),
        wal_exists: message_storage.exists(),
        db_mtime_ms: path_mtime_millis(&session_storage),
        wal_mtime_ms: path_mtime_millis(&message_storage),
        max_cursor: None,
    }
}

pub(crate) fn should_emit_opencode_refresh(
    opencode_root: &Path,
    snapshot_state: &Arc<Mutex<OpenCodeWatchSnapshot>>,
) -> Result<bool, String> {
    let next_snapshot = build_opencode_watch_snapshot(opencode_root);
    let mut current_snapshot = snapshot_state
        .lock()
        .map_err(|_| "failed to lock opencode watcher snapshot".to_string())?;

    if *current_snapshot == next_snapshot {
        return Ok(false);
    }

    *current_snapshot = next_snapshot;
    Ok(true)
}

pub(crate) fn is_relevant_opencode_event(
    event: &notify::Event,
    opencode_root: &Path,
) -> bool {
    if !is_relevant_watcher_event_kind(&event.kind) {
        return false;
    }

    let session_storage = opencode_root.join("storage").join("session");
    let message_storage = opencode_root.join("storage").join("message");

    event.paths.iter().any(|path| {
        path == opencode_root
            || path.starts_with(&session_storage)
            || path.starts_with(&message_storage)
    })
}

// ── Bridge Provider 比對 ─────────────────────────────────────────────────────

pub(crate) fn matched_bridge_providers(
    event: &notify::Event,
    provider_bridge_paths: &HashMap<String, PathBuf>,
) -> BTreeSet<String> {
    if !is_relevant_watcher_event_kind(&event.kind) {
        return BTreeSet::new();
    }

    provider_bridge_paths
        .iter()
        .filter(|(_, bridge_path)| {
            event.paths.iter().any(|path| {
                path == *bridge_path
                    || (path.parent() == bridge_path.parent()
                        && path
                            .file_name()
                            .zip(bridge_path.file_name())
                            .is_some_and(|(left, right)| left == right))
            })
        })
        .map(|(provider, _)| provider.clone())
        .collect()
}

pub(crate) fn process_provider_bridge_event(
    app: &tauri::AppHandle,
    refresh_state: &Arc<Mutex<HashMap<String, Instant>>>,
    last_bridge_records: &Arc<Mutex<HashMap<String, String>>>,
    provider: &str,
) -> Result<bool, String> {
    let bridge_path = resolve_provider_bridge_path(provider)?;
    let Some(record) = read_last_bridge_record(&bridge_path)? else {
        return Ok(false);
    };

    if record.provider != provider {
        return Err(format!(
            "unexpected provider '{}' in {} bridge file",
            record.provider,
            bridge_path.display()
        ));
    }

    if !register_provider_bridge_record(last_bridge_records, provider, &record)? {
        return Ok(false);
    }

    emit_provider_refresh(app, refresh_state, provider)
}

// ── Bridge 記錄讀寫 ──────────────────────────────────────────────────────────

fn coerce_json_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::String(s) => {
            if s.is_empty() {
                None
            } else {
                Some(s.clone())
            }
        }
        serde_json::Value::Array(arr) => {
            let joined = arr
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<Vec<_>>()
                .join(", ");
            if joined.is_empty() {
                None
            } else {
                Some(joined)
            }
        }
        other => Some(other.to_string()),
    }
}

fn read_last_bridge_record(bridge_path: &Path) -> Result<Option<ProviderBridgeRecord>, String> {
    if !bridge_path.exists() {
        return Ok(None);
    }

    let file = File::open(bridge_path).map_err(|error| {
        format!(
            "failed to open bridge file {}: {error}",
            bridge_path.display()
        )
    })?;
    let mut last_non_empty_line: Option<String> = None;

    for line in BufReader::new(file).lines() {
        let line = line.map_err(|error| {
            format!(
                "failed to read bridge file {}: {error}",
                bridge_path.display()
            )
        })?;
        if !line.trim().is_empty() {
            last_non_empty_line = Some(line);
        }
    }

    let Some(last_line) = last_non_empty_line else {
        return Ok(None);
    };

    let raw: serde_json::Value = serde_json::from_str(&last_line).map_err(|error| {
        format!(
            "failed to parse bridge file {}: {error}",
            bridge_path.display()
        )
    })?;

    let get_str =
        |key: &str| -> Option<String> { raw.get(key).and_then(|v| coerce_json_string(v)) };
    let get_req_str = |key: &str| -> Result<String, String> {
        raw.get(key)
            .and_then(|v| coerce_json_string(v))
            .ok_or_else(|| {
                format!(
                    "missing required field '{}' in bridge file {}",
                    key,
                    bridge_path.display()
                )
            })
    };
    let get_u32 = |key: &str, default: u32| -> u32 {
        raw.get(key)
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
            .unwrap_or(default)
    };

    Ok(Some(ProviderBridgeRecord {
        version: get_u32("version", default_provider_bridge_version()),
        provider: get_req_str("provider")?,
        event_type: get_req_str("eventType")?,
        timestamp: get_req_str("timestamp")?,
        session_id: get_str("sessionId"),
        cwd: get_str("cwd"),
        source_path: get_str("sourcePath"),
        title: get_str("title"),
        error: get_str("error"),
    }))
}

pub(crate) fn read_bridge_diagnostics(provider: &str) -> ProviderBridgeDiagnostics {
    let bridge_path = match resolve_provider_bridge_path(provider) {
        Ok(path) => path,
        Err(error) => {
            return ProviderBridgeDiagnostics {
                last_error: Some(error),
                ..ProviderBridgeDiagnostics::default()
            };
        }
    };

    let mut diagnostics = ProviderBridgeDiagnostics {
        bridge_path: Some(bridge_path.clone()),
        ..ProviderBridgeDiagnostics::default()
    };

    match read_last_bridge_record(&bridge_path) {
        Ok(Some(record)) => {
            diagnostics.last_event_at = Some(record.timestamp.clone());
            diagnostics.last_error = if record.provider != provider {
                Some(format!(
                    "unexpected provider '{}' in {} bridge file",
                    record.provider, provider
                ))
            } else {
                record.error.or_else(|| {
                    record
                        .event_type
                        .ends_with(".error")
                        .then(|| format!("{provider} bridge reported {}", record.event_type))
                })
            };
        }
        Ok(None) => {}
        Err(error) => diagnostics.last_error = Some(error),
    }

    diagnostics
}
