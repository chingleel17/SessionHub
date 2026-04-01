use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, UNIX_EPOCH};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(default)]
    provider_integrations: Vec<ProviderIntegrationStatus>,
}

const PROVIDER_INTEGRATION_VERSION: u32 = 1;
const COPILOT_PROVIDER: &str = "copilot";
const OPENCODE_PROVIDER: &str = "opencode";
const COPILOT_HOOK_FILE_NAME: &str = "sessionhub-provider-event-bridge.json";
const OPENCODE_PLUGIN_FILE_NAME: &str = "sessionhub-provider-event-bridge.ts";
const OPENCODE_PLUGIN_METADATA_PREFIX: &str = "// sessionhub-provider-event-bridge:";

fn default_provider_bridge_version() -> u32 {
    PROVIDER_INTEGRATION_VERSION
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ProviderIntegrationState {
    Installed,
    Outdated,
    Missing,
    ManualRequired,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ProviderBridgeRecord {
    #[serde(default = "default_provider_bridge_version")]
    version: u32,
    provider: String,
    event_type: String,
    timestamp: String,
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    source_path: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ProviderIntegrationStatus {
    provider: String,
    status: ProviderIntegrationState,
    config_path: Option<String>,
    bridge_path: Option<String>,
    last_event_at: Option<String>,
    last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ManagedProviderIntegrationMetadata {
    provider: String,
    bridge_path: String,
    integration_version: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CopilotIntegrationConfig {
    #[serde(default)]
    session_hub: Option<ManagedProviderIntegrationMetadata>,
}

#[derive(Debug, Default)]
struct ProviderBridgeDiagnostics {
    bridge_path: Option<PathBuf>,
    last_event_at: Option<String>,
    last_error: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct CopilotWatchSnapshot {
    active_session_count: usize,
    archived_session_count: usize,
    active_workspace_mtime_ms: u128,
    archived_workspace_mtime_ms: u128,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct OpenCodeWatchSnapshot {
    db_exists: bool,
    wal_exists: bool,
    db_mtime_ms: u128,
    wal_mtime_ms: u128,
    max_cursor: Option<i64>,
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
    input_tokens: u64,
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
            input_tokens: 0,
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
    provider_bridge: Mutex<Option<RecommendedWatcher>>,
    last_provider_refresh: Arc<Mutex<HashMap<String, Instant>>>,
    last_bridge_records: Arc<Mutex<HashMap<String, String>>>,
}

/// Copilot watcher 防抖時間（毫秒）
const COPILOT_DEBOUNCE_MS: u64 = 800;
/// OpenCode WAL watcher 防抖時間（毫秒）
const OPENCODE_DEBOUNCE_MS: u64 = 500;
/// Provider bridge watcher 防抖時間（毫秒）
const PROVIDER_BRIDGE_DEBOUNCE_MS: u64 = 250;
/// 短時間內同 provider refresh 去重視窗（毫秒）
const PROVIDER_REFRESH_DEDUP_MS: u64 = 1_500;
/// 觸發全掃描的閾值（秒），超過此值自動執行全掃
const FULL_SCAN_THRESHOLD_SECS: u64 = 1800;

/// 單一 provider 的記憶體快取
struct ProviderCache {
    /// 上次掃描的結果
    sessions: Vec<SessionInfo>,
    /// Copilot 專用：session_id → 目錄最後修改時間（Unix 秒）
    session_mtimes: HashMap<String, i64>,
    /// 上次全掃描的時間點
    last_full_scan_at: Instant,
    /// OpenCode 專用：上次全掃描時見到的最大 time_updated 值
    last_cursor: i64,
}

/// 兩個 provider 各自持有的掃描快取
#[derive(Default)]
struct ScanCache {
    copilot: Mutex<Option<ProviderCache>>,
    opencode: Mutex<Option<ProviderCache>>,
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

fn default_opencode_config_root() -> Result<PathBuf, String> {
    let user_profile = env::var("USERPROFILE")
        .map_err(|_| "USERPROFILE environment variable is not set".to_string())?;

    Ok(PathBuf::from(user_profile).join(".config").join("opencode"))
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

fn provider_bridge_dir() -> Result<PathBuf, String> {
    Ok(default_app_data_dir()?.join("provider-bridge"))
}

fn resolve_provider_bridge_path(provider: &str) -> Result<PathBuf, String> {
    Ok(provider_bridge_dir()?.join(format!("{provider}.jsonl")))
}

fn resolve_copilot_integration_path(copilot_root: &Path) -> PathBuf {
    copilot_root.join("hooks").join(COPILOT_HOOK_FILE_NAME)
}

fn resolve_opencode_integration_path() -> Result<PathBuf, String> {
    Ok(default_opencode_config_root()?
        .join("plugins")
        .join(OPENCODE_PLUGIN_FILE_NAME))
}

fn provider_refresh_event_name(provider: &str) -> Result<&'static str, String> {
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

fn register_provider_bridge_record(
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

fn should_emit_provider_refresh_at(
    refresh_state: &Arc<Mutex<HashMap<String, Instant>>>,
    provider: &str,
    now: Instant,
) -> Result<bool, String> {
    let mut tracked = refresh_state
        .lock()
        .map_err(|_| "failed to lock provider refresh state".to_string())?;
    let dedup_window = Duration::from_millis(PROVIDER_REFRESH_DEDUP_MS);
    tracked.retain(|_, last_emit| now.duration_since(*last_emit) < dedup_window);

    if tracked
        .get(provider)
        .is_some_and(|last_emit| now.duration_since(*last_emit) < dedup_window)
    {
        return Ok(false);
    }

    tracked.insert(provider.to_string(), now);
    Ok(true)
}

fn emit_provider_refresh(
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

fn path_mtime_millis(path: &Path) -> u128 {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn is_relevant_watcher_event_kind(kind: &notify::EventKind) -> bool {
    matches!(
        kind,
        notify::EventKind::Create(_)
            | notify::EventKind::Modify(notify::event::ModifyKind::Any)
            | notify::EventKind::Modify(notify::event::ModifyKind::Data(_))
            | notify::EventKind::Modify(notify::event::ModifyKind::Name(_))
            | notify::EventKind::Remove(_)
    )
}

fn path_file_name_matches(path: &Path, expected: &str) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(expected))
}

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

fn build_copilot_watch_snapshot(root: &Path) -> CopilotWatchSnapshot {
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

fn should_emit_copilot_refresh(
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

fn is_relevant_copilot_event(event: &notify::Event, session_roots: &[PathBuf]) -> bool {
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

fn build_opencode_watch_snapshot(opencode_root: &Path) -> OpenCodeWatchSnapshot {
    let db_path = opencode_root.join("opencode.db");
    let wal_path = opencode_root.join("opencode.db-wal");

    OpenCodeWatchSnapshot {
        db_exists: db_path.exists(),
        wal_exists: wal_path.exists(),
        db_mtime_ms: path_mtime_millis(&db_path),
        wal_mtime_ms: path_mtime_millis(&wal_path),
        max_cursor: get_opencode_max_cursor(opencode_root).ok(),
    }
}

fn should_emit_opencode_refresh(
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

fn is_relevant_opencode_event(event: &notify::Event, opencode_root: &Path) -> bool {
    if !is_relevant_watcher_event_kind(&event.kind) {
        return false;
    }

    event.paths.iter().any(|path| {
        path == opencode_root
            || path_file_name_matches(path, "opencode.db")
            || path_file_name_matches(path, "opencode.db-wal")
    })
}

fn matched_bridge_providers(
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

fn process_provider_bridge_event(
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

    serde_json::from_str::<ProviderBridgeRecord>(&last_line)
        .map(Some)
        .map_err(|error| {
            format!(
                "failed to parse bridge file {}: {error}",
                bridge_path.display()
            )
        })
}

fn read_bridge_diagnostics(provider: &str) -> ProviderBridgeDiagnostics {
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

fn validate_integration_target(config_path: &Path) -> Result<(), String> {
    let parent = config_path.parent().ok_or_else(|| {
        format!(
            "integration path {} does not have a parent directory",
            config_path.display()
        )
    })?;

    if parent.exists() && !parent.is_dir() {
        return Err(format!(
            "integration parent path is not a directory: {}",
            parent.display()
        ));
    }

    if config_path.exists() && config_path.is_dir() {
        return Err(format!(
            "integration path points to a directory: {}",
            config_path.display()
        ));
    }

    Ok(())
}

fn validate_managed_metadata(
    metadata: &ManagedProviderIntegrationMetadata,
    provider: &str,
    expected_bridge_path: &Path,
) -> Result<(), String> {
    if metadata.provider != provider {
        return Err(format!(
            "integration provider mismatch: expected {}, found {}",
            provider, metadata.provider
        ));
    }

    if metadata.integration_version != PROVIDER_INTEGRATION_VERSION {
        return Err(format!(
            "integration version {} is outdated (expected {})",
            metadata.integration_version, PROVIDER_INTEGRATION_VERSION
        ));
    }

    let expected_path = expected_bridge_path.to_string_lossy();
    if metadata.bridge_path != expected_path {
        return Err(format!(
            "bridge path mismatch: expected {}, found {}",
            expected_path, metadata.bridge_path
        ));
    }

    Ok(())
}

fn build_provider_integration_status(
    provider: &str,
    status: ProviderIntegrationState,
    config_path: Option<PathBuf>,
    diagnostics: ProviderBridgeDiagnostics,
    last_error: Option<String>,
) -> ProviderIntegrationStatus {
    ProviderIntegrationStatus {
        provider: provider.to_string(),
        status,
        config_path: config_path.map(|path| path.to_string_lossy().to_string()),
        bridge_path: diagnostics
            .bridge_path
            .map(|path| path.to_string_lossy().to_string()),
        last_event_at: diagnostics.last_event_at,
        last_error: last_error.or(diagnostics.last_error),
    }
}

fn managed_provider_metadata(
    provider: &str,
    bridge_path: &Path,
) -> ManagedProviderIntegrationMetadata {
    ManagedProviderIntegrationMetadata {
        provider: provider.to_string(),
        bridge_path: bridge_path.to_string_lossy().to_string(),
        integration_version: PROVIDER_INTEGRATION_VERSION,
    }
}

fn powershell_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn render_copilot_hook_powershell(bridge_path: &Path) -> String {
    let bridge_path_literal = powershell_single_quoted(&bridge_path.to_string_lossy());
    let bridge_parent_literal = powershell_single_quoted(
        &bridge_path
            .parent()
            .unwrap_or(bridge_path)
            .to_string_lossy(),
    );

    format!(
        concat!(
            "$payload = [Console]::In.ReadToEnd(); ",
            "if ([string]::IsNullOrWhiteSpace($payload)) {{ exit 0 }}; ",
            "$event = $payload | ConvertFrom-Json; ",
            "$timestamp = if ($event.timestamp) {{ [DateTimeOffset]::FromUnixTimeMilliseconds([int64]$event.timestamp).UtcDateTime.ToString('o') }} else {{ [DateTimeOffset]::UtcNow.ToString('o') }}; ",
            "$cwd = if ($null -ne $event.cwd -and -not [string]::IsNullOrWhiteSpace([string]$event.cwd)) {{ [string]$event.cwd }} else {{ $null }}; ",
            "$error = if ($event.reason -eq 'error') {{ 'copilot session ended with error' }} else {{ $null }}; ",
            "$record = [ordered]@{{ version = {version}; provider = {provider}; eventType = 'session.ended'; timestamp = $timestamp; sessionId = $null; cwd = $cwd; sourcePath = $null; title = $null; error = $error }}; ",
            "New-Item -ItemType Directory -Force -Path {bridge_parent} | Out-Null; ",
            "[System.IO.File]::AppendAllText({bridge_path}, (($record | ConvertTo-Json -Compress) + [Environment]::NewLine), [System.Text.UTF8Encoding]::new($false));"
        ),
        version = PROVIDER_INTEGRATION_VERSION,
        provider = powershell_single_quoted(COPILOT_PROVIDER),
        bridge_parent = bridge_parent_literal,
        bridge_path = bridge_path_literal
    )
}

fn render_copilot_integration(bridge_path: &Path) -> Result<String, String> {
    let integration = serde_json::json!({
        "version": 1,
        "sessionHub": managed_provider_metadata(COPILOT_PROVIDER, bridge_path),
        "hooks": {
            "sessionEnd": [
                {
                    "type": "command",
                    "powershell": render_copilot_hook_powershell(bridge_path)
                }
            ]
        }
    });

    serde_json::to_string_pretty(&integration)
        .map_err(|error| format!("failed to serialize Copilot integration: {error}"))
}

fn render_opencode_integration(bridge_path: &Path) -> Result<String, String> {
    let metadata =
        serde_json::to_string(&managed_provider_metadata(OPENCODE_PROVIDER, bridge_path)).map_err(
            |error| format!("failed to serialize OpenCode integration metadata: {error}"),
        )?;
    let bridge_path_literal = serde_json::to_string(&bridge_path.to_string_lossy().to_string())
        .map_err(|error| format!("failed to serialize OpenCode bridge path: {error}"))?;
    let bridge_parent_literal = serde_json::to_string(
        &bridge_path
            .parent()
            .unwrap_or(bridge_path)
            .to_string_lossy()
            .to_string(),
    )
    .map_err(|error| format!("failed to serialize OpenCode bridge directory: {error}"))?;

    Ok(format!(
        concat!(
            "{metadata_prefix}{metadata}\n",
            "import {{ appendFile, mkdir }} from \"node:fs/promises\";\n",
            "import type {{ Plugin }} from \"@opencode-ai/plugin\";\n\n",
            "const BRIDGE_PATH = {bridge_path};\n",
            "const BRIDGE_DIR = {bridge_dir};\n\n",
            "function toIsoTimestamp(value: unknown): string {{\n",
            "  if (typeof value === \"string\" && value.trim().length > 0) return value;\n",
            "  if (typeof value === \"number\" && Number.isFinite(value)) return new Date(value).toISOString();\n",
            "  return new Date().toISOString();\n",
            "}}\n\n",
            "function buildRecord(eventType: string, event: any) {{\n",
            "  const session = event?.session ?? event?.output?.session ?? event ?? {{}};\n",
            "  return {{\n",
            "    version: {version},\n",
            "    provider: \"{provider}\",\n",
            "    eventType,\n",
            "    timestamp: toIsoTimestamp(event?.timestamp ?? session?.updatedAt ?? session?.timeUpdated),\n",
            "    sessionId: session?.id ?? event?.sessionId ?? null,\n",
            "    cwd: session?.cwd ?? event?.cwd ?? null,\n",
            "    sourcePath: session?.path ?? null,\n",
            "    title: session?.title ?? session?.summary ?? null,\n",
            "    error: event?.error?.message ?? null,\n",
            "  }};\n",
            "}}\n\n",
            "async function appendRecord(record: ReturnType<typeof buildRecord>) {{\n",
            "  await mkdir(BRIDGE_DIR, {{ recursive: true }});\n",
            "  await appendFile(BRIDGE_PATH, `${{JSON.stringify(record)}}\\n`, \"utf8\");\n",
            "}}\n\n",
            "export const SessionHubBridge: Plugin = async () => {{\n",
            "  return {{\n",
            "    \"session.updated\": async (event) => {{\n",
            "      await appendRecord(buildRecord(\"session.updated\", event));\n",
            "    }},\n",
            "    \"session.error\": async (event) => {{\n",
            "      await appendRecord(buildRecord(\"session.error\", event));\n",
            "    }},\n",
            "  }};\n",
            "}};\n"
        ),
        metadata_prefix = OPENCODE_PLUGIN_METADATA_PREFIX,
        metadata = metadata,
        bridge_path = bridge_path_literal,
        bridge_dir = bridge_parent_literal,
        version = PROVIDER_INTEGRATION_VERSION,
        provider = OPENCODE_PROVIDER
    ))
}

fn write_provider_integration_file(config_path: &Path, content: &str) -> Result<(), String> {
    validate_integration_target(config_path)?;
    ensure_parent_dir(config_path)?;
    fs::write(config_path, content).map_err(|error| {
        format!(
            "failed to write integration file {}: {error}",
            config_path.display()
        )
    })
}

fn build_install_failure_status(
    provider: &str,
    config_path: Option<PathBuf>,
    diagnostics: ProviderBridgeDiagnostics,
    error: String,
) -> ProviderIntegrationStatus {
    let status = if error.contains("Access is denied")
        || error.contains("Permission denied")
        || error.contains("failed to create directory")
    {
        ProviderIntegrationState::ManualRequired
    } else {
        ProviderIntegrationState::Error
    };

    build_provider_integration_status(provider, status, config_path, diagnostics, Some(error))
}

fn install_or_update_copilot_integration(copilot_root: Option<&str>) -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(COPILOT_PROVIDER);
    let copilot_root = match resolve_copilot_root(copilot_root) {
        Ok(path) => path,
        Err(error) => {
            return build_provider_integration_status(
                COPILOT_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                Some(error),
            );
        }
    };
    let config_path = resolve_copilot_integration_path(&copilot_root);
    let Some(bridge_path) = diagnostics.bridge_path.clone() else {
        return build_provider_integration_status(
            COPILOT_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            Some(config_path),
            diagnostics,
            Some("failed to resolve Copilot bridge path".to_string()),
        );
    };

    let content = match render_copilot_integration(&bridge_path) {
        Ok(content) => content,
        Err(error) => {
            return build_install_failure_status(
                COPILOT_PROVIDER,
                Some(config_path),
                diagnostics,
                error,
            );
        }
    };

    if let Err(error) = ensure_parent_dir(&bridge_path)
        .and_then(|_| write_provider_integration_file(&config_path, &content))
    {
        return build_install_failure_status(
            COPILOT_PROVIDER,
            Some(config_path),
            diagnostics,
            error,
        );
    }

    detect_copilot_integration_status(copilot_root.to_str())
}

fn install_or_update_opencode_integration() -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(OPENCODE_PROVIDER);
    let config_path = match resolve_opencode_integration_path() {
        Ok(path) => path,
        Err(error) => {
            return build_provider_integration_status(
                OPENCODE_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                Some(error),
            );
        }
    };
    let Some(bridge_path) = diagnostics.bridge_path.clone() else {
        return build_provider_integration_status(
            OPENCODE_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            Some(config_path),
            diagnostics,
            Some("failed to resolve OpenCode bridge path".to_string()),
        );
    };

    let content = match render_opencode_integration(&bridge_path) {
        Ok(content) => content,
        Err(error) => {
            return build_install_failure_status(
                OPENCODE_PROVIDER,
                Some(config_path),
                diagnostics,
                error,
            );
        }
    };

    if let Err(error) = ensure_parent_dir(&bridge_path)
        .and_then(|_| write_provider_integration_file(&config_path, &content))
    {
        return build_install_failure_status(
            OPENCODE_PROVIDER,
            Some(config_path),
            diagnostics,
            error,
        );
    }

    detect_opencode_integration_status()
}

fn recheck_provider_integration_status(
    provider: &str,
    copilot_root: Option<&str>,
) -> Result<ProviderIntegrationStatus, String> {
    match provider {
        COPILOT_PROVIDER => Ok(detect_copilot_integration_status(copilot_root)),
        OPENCODE_PROVIDER => Ok(detect_opencode_integration_status()),
        _ => Err(format!("unsupported provider: {provider}")),
    }
}

fn install_or_update_provider_integration(
    provider: &str,
    copilot_root: Option<&str>,
) -> Result<ProviderIntegrationStatus, String> {
    match provider {
        COPILOT_PROVIDER => Ok(install_or_update_copilot_integration(copilot_root)),
        OPENCODE_PROVIDER => Ok(install_or_update_opencode_integration()),
        _ => Err(format!("unsupported provider: {provider}")),
    }
}

fn parse_opencode_integration_metadata(
    content: &str,
) -> Result<Option<ManagedProviderIntegrationMetadata>, String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(raw_json) = trimmed.strip_prefix(OPENCODE_PLUGIN_METADATA_PREFIX) {
            return serde_json::from_str::<ManagedProviderIntegrationMetadata>(raw_json.trim())
                .map(Some)
                .map_err(|error| {
                    format!("failed to parse OpenCode integration metadata: {error}")
                });
        }
    }

    Ok(None)
}

fn detect_copilot_integration_status(copilot_root: Option<&str>) -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(COPILOT_PROVIDER);
    let copilot_root = match resolve_copilot_root(copilot_root) {
        Ok(path) => path,
        Err(error) => {
            return build_provider_integration_status(
                COPILOT_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                Some(error),
            );
        }
    };
    let config_path = resolve_copilot_integration_path(&copilot_root);

    if let Err(error) = validate_integration_target(&config_path) {
        return build_provider_integration_status(
            COPILOT_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            Some(config_path),
            diagnostics,
            Some(error),
        );
    }

    let Some(expected_bridge_path) = diagnostics.bridge_path.clone() else {
        return build_provider_integration_status(
            COPILOT_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            Some(config_path),
            diagnostics,
            Some("failed to resolve Copilot bridge path".to_string()),
        );
    };

    if !config_path.exists() {
        return build_provider_integration_status(
            COPILOT_PROVIDER,
            ProviderIntegrationState::Missing,
            Some(config_path),
            diagnostics,
            None,
        );
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(error) => {
            let error_message = format!(
                "failed to read Copilot integration file {}: {error}",
                config_path.display()
            );
            return build_provider_integration_status(
                COPILOT_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path),
                diagnostics,
                Some(error_message),
            );
        }
    };

    let parsed = match serde_json::from_str::<CopilotIntegrationConfig>(&content) {
        Ok(parsed) => parsed,
        Err(error) => {
            let error_message = format!(
                "failed to parse Copilot integration file {}: {error}",
                config_path.display()
            );
            return build_provider_integration_status(
                COPILOT_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path),
                diagnostics,
                Some(error_message),
            );
        }
    };

    let Some(metadata) = parsed.session_hub else {
        return build_provider_integration_status(
            COPILOT_PROVIDER,
            ProviderIntegrationState::Outdated,
            Some(config_path),
            diagnostics,
            Some("missing SessionHub integration metadata".to_string()),
        );
    };

    match validate_managed_metadata(&metadata, COPILOT_PROVIDER, &expected_bridge_path) {
        Ok(()) => build_provider_integration_status(
            COPILOT_PROVIDER,
            ProviderIntegrationState::Installed,
            Some(config_path),
            diagnostics,
            None,
        ),
        Err(error) => build_provider_integration_status(
            COPILOT_PROVIDER,
            ProviderIntegrationState::Outdated,
            Some(config_path),
            diagnostics,
            Some(error),
        ),
    }
}

fn detect_opencode_integration_status() -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(OPENCODE_PROVIDER);
    let config_path = match resolve_opencode_integration_path() {
        Ok(path) => path,
        Err(error) => {
            return build_provider_integration_status(
                OPENCODE_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                Some(error),
            );
        }
    };

    if let Err(error) = validate_integration_target(&config_path) {
        return build_provider_integration_status(
            OPENCODE_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            Some(config_path),
            diagnostics,
            Some(error),
        );
    }

    let Some(expected_bridge_path) = diagnostics.bridge_path.clone() else {
        return build_provider_integration_status(
            OPENCODE_PROVIDER,
            ProviderIntegrationState::ManualRequired,
            Some(config_path),
            diagnostics,
            Some("failed to resolve OpenCode bridge path".to_string()),
        );
    };

    if !config_path.exists() {
        return build_provider_integration_status(
            OPENCODE_PROVIDER,
            ProviderIntegrationState::Missing,
            Some(config_path),
            diagnostics,
            None,
        );
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(error) => {
            let error_message = format!(
                "failed to read OpenCode integration file {}: {error}",
                config_path.display()
            );
            return build_provider_integration_status(
                OPENCODE_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path),
                diagnostics,
                Some(error_message),
            );
        }
    };

    let metadata = match parse_opencode_integration_metadata(&content) {
        Ok(Some(metadata)) => metadata,
        Ok(None) => {
            return build_provider_integration_status(
                OPENCODE_PROVIDER,
                ProviderIntegrationState::Outdated,
                Some(config_path),
                diagnostics,
                Some("missing SessionHub integration metadata".to_string()),
            );
        }
        Err(error) => {
            return build_provider_integration_status(
                OPENCODE_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path),
                diagnostics,
                Some(error),
            );
        }
    };

    match validate_managed_metadata(&metadata, OPENCODE_PROVIDER, &expected_bridge_path) {
        Ok(()) => build_provider_integration_status(
            OPENCODE_PROVIDER,
            ProviderIntegrationState::Installed,
            Some(config_path),
            diagnostics,
            None,
        ),
        Err(error) => build_provider_integration_status(
            OPENCODE_PROVIDER,
            ProviderIntegrationState::Outdated,
            Some(config_path),
            diagnostics,
            Some(error),
        ),
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
            id: id.clone(),
            provider: "opencode".to_string(),
            cwd: worktree,
            summary,
            summary_count,
            created_at,
            updated_at,
            session_dir: opencode_root
                .join("storage")
                .join("message")
                .join(&id)
                .to_string_lossy()
                .to_string(),
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
            provider_integrations: Vec::new(),
        })
    }
}

fn collect_provider_integration_statuses(
    copilot_root: Option<&str>,
) -> Vec<ProviderIntegrationStatus> {
    vec![
        detect_copilot_integration_status(copilot_root),
        detect_opencode_integration_status(),
    ]
}

fn settings_file_path() -> Result<PathBuf, String> {
    Ok(default_app_data_dir()?.join("settings.json"))
}

fn metadata_db_path() -> Result<PathBuf, String> {
    Ok(default_app_data_dir()?.join("metadata.db"))
}

fn session_cache_path() -> Result<PathBuf, String> {
    Ok(default_app_data_dir()?.join("session_cache.json"))
}

fn save_session_cache(sessions: &[SessionInfo]) -> Result<(), String> {
    let cache_path = session_cache_path()?;
    ensure_parent_dir(&cache_path)?;
    let content = serde_json::to_string(sessions)
        .map_err(|error| format!("failed to serialize session cache: {error}"))?;
    fs::write(&cache_path, content)
        .map_err(|error| format!("failed to write session cache: {error}"))?;
    Ok(())
}

#[tauri::command]
fn load_session_cache() -> Vec<SessionInfo> {
    let Ok(cache_path) = session_cache_path() else {
        return Vec::new();
    };
    if !cache_path.exists() {
        return Vec::new();
    }
    let Ok(content) = fs::read_to_string(&cache_path) else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<SessionInfo>>(&content).unwrap_or_default()
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
    refresh_state: Arc<Mutex<HashMap<String, Instant>>>,
) -> Result<RecommendedWatcher, String> {
    let app_handle = app.clone();
    let watch_root = root.to_path_buf();
    let session_roots = vec![
        watch_root.join("session-state"),
        watch_root.join("session-state-archive"),
    ];
    let snapshot_state = Arc::new(Mutex::new(build_copilot_watch_snapshot(&watch_root)));
    let last_event: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));
    // 標記 debounce thread 是否已在執行，避免高頻事件時重複 spawn
    let debounce_running: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    let mut watcher = recommended_watcher(move |result: notify::Result<notify::Event>| {
        if let Ok(event) = result {
            if !is_relevant_copilot_event(&event, &session_roots) {
                return;
            }

            // 更新最後事件時間戳
            if let Ok(mut ts) = last_event.lock() {
                *ts = Instant::now();
            }

            // 若 debounce thread 已在跑，只更新時間即可，不再 spawn
            if debounce_running
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                return;
            }

            // spawn 唯一一個 debounce thread
            let le = Arc::clone(&last_event);
            let handle = app_handle.clone();
            let running = Arc::clone(&debounce_running);
            let refreshes = Arc::clone(&refresh_state);
            let watched_root = watch_root.clone();
            let tracked_snapshot = Arc::clone(&snapshot_state);
            thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_millis(COPILOT_DEBOUNCE_MS));
                    let elapsed = le.lock().map(|ts| ts.elapsed()).unwrap_or_default();
                    if elapsed >= Duration::from_millis(COPILOT_DEBOUNCE_MS) {
                        // 穩定後 emit 並結束 thread
                        running.store(false, Ordering::SeqCst);
                        match should_emit_copilot_refresh(&watched_root, &tracked_snapshot) {
                            Ok(true) => {
                                let _ =
                                    emit_provider_refresh(&handle, &refreshes, COPILOT_PROVIDER);
                            }
                            Ok(false) => {}
                            Err(error) => {
                                eprintln!("failed to verify copilot watcher refresh: {error}");
                            }
                        }
                        break;
                    }
                    // 尚未穩定（有新事件更新了 last_event），繼續等待
                }
            });
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
    refresh_state: Arc<Mutex<HashMap<String, Instant>>>,
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
    let watch_root = opencode_root.to_path_buf();
    let snapshot_state = Arc::new(Mutex::new(build_opencode_watch_snapshot(&watch_root)));
    let last_event: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));
    // 標記 debounce thread 是否已在執行，避免高頻事件時重複 spawn
    let debounce_running: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    let mut watcher = recommended_watcher(move |result: notify::Result<notify::Event>| {
        if let Ok(event) = result {
            if !is_relevant_opencode_event(&event, &watch_root) {
                return;
            }

            // 更新最後事件時間戳
            if let Ok(mut ts) = last_event.lock() {
                *ts = Instant::now();
            }

            // 若 debounce thread 已在跑，只更新時間即可，不再 spawn
            if debounce_running
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                return;
            }

            // spawn 唯一一個 debounce thread
            let le = Arc::clone(&last_event);
            let handle = app_handle.clone();
            let running = Arc::clone(&debounce_running);
            let refreshes = Arc::clone(&refresh_state);
            let watched_root = watch_root.clone();
            let tracked_snapshot = Arc::clone(&snapshot_state);
            thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_millis(OPENCODE_DEBOUNCE_MS));
                    let elapsed = le.lock().map(|ts| ts.elapsed()).unwrap_or_default();
                    if elapsed >= Duration::from_millis(OPENCODE_DEBOUNCE_MS) {
                        // 穩定後 emit 並結束 thread
                        running.store(false, Ordering::SeqCst);
                        match should_emit_opencode_refresh(&watched_root, &tracked_snapshot) {
                            Ok(true) => {
                                let _ =
                                    emit_provider_refresh(&handle, &refreshes, OPENCODE_PROVIDER);
                            }
                            Ok(false) => {}
                            Err(error) => {
                                eprintln!("failed to verify opencode watcher refresh: {error}");
                            }
                        }
                        break;
                    }
                    // 尚未穩定（有新事件更新了 last_event），繼續等待
                }
            });
        }
    })
    .map_err(|error| format!("failed to create opencode watcher: {error}"))?;

    watcher
        .watch(opencode_root, RecursiveMode::NonRecursive)
        .map_err(|error| format!("failed to watch {}: {error}", opencode_root.display()))?;
    if db_path.exists() {
        watcher
            .watch(&db_path, RecursiveMode::NonRecursive)
            .map_err(|error| format!("failed to watch {}: {error}", db_path.display()))?;
    }
    if wal_path.exists() {
        watcher
            .watch(&wal_path, RecursiveMode::NonRecursive)
            .map_err(|error| format!("failed to watch {}: {error}", wal_path.display()))?;
    }

    Ok(watcher)
}

fn create_provider_bridge_watcher(
    app: &tauri::AppHandle,
    providers: Vec<String>,
    refresh_state: Arc<Mutex<HashMap<String, Instant>>>,
    last_bridge_records: Arc<Mutex<HashMap<String, String>>>,
) -> Result<RecommendedWatcher, String> {
    let bridge_dir = provider_bridge_dir()?;
    fs::create_dir_all(&bridge_dir).map_err(|error| {
        format!(
            "failed to create provider bridge directory {}: {error}",
            bridge_dir.display()
        )
    })?;

    let app_handle = app.clone();
    let provider_bridge_paths = providers.iter().try_fold(
        HashMap::new(),
        |mut acc: HashMap<String, PathBuf>, provider| -> Result<HashMap<String, PathBuf>, String> {
            acc.insert(provider.clone(), resolve_provider_bridge_path(provider)?);
            Ok(acc)
        },
    )?;
    let watched_bridge_paths = provider_bridge_paths.clone();
    let last_event: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));
    let debounce_running: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let pending_providers: Arc<Mutex<BTreeSet<String>>> = Arc::new(Mutex::new(BTreeSet::new()));

    let mut watcher = recommended_watcher(move |result: notify::Result<notify::Event>| {
        if let Ok(event) = result {
            let changed_providers = matched_bridge_providers(&event, &watched_bridge_paths);
            if changed_providers.is_empty() {
                return;
            }

            if let Ok(mut tracked_providers) = pending_providers.lock() {
                tracked_providers.extend(changed_providers);
            }

            if let Ok(mut ts) = last_event.lock() {
                *ts = Instant::now();
            }

            if debounce_running
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                return;
            }

            let le = Arc::clone(&last_event);
            let handle = app_handle.clone();
            let running = Arc::clone(&debounce_running);
            let refreshes = Arc::clone(&refresh_state);
            let tracked_records = Arc::clone(&last_bridge_records);
            let watched_providers = providers.clone();
            let pending = Arc::clone(&pending_providers);
            thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(PROVIDER_BRIDGE_DEBOUNCE_MS));
                let elapsed = le.lock().map(|ts| ts.elapsed()).unwrap_or_default();
                if elapsed >= Duration::from_millis(PROVIDER_BRIDGE_DEBOUNCE_MS) {
                    running.store(false, Ordering::SeqCst);
                    let providers_to_process = pending
                        .lock()
                        .map(|mut tracked| {
                            let providers = tracked.iter().cloned().collect::<Vec<_>>();
                            tracked.clear();
                            providers
                        })
                        .unwrap_or_else(|_| watched_providers.clone());
                    for provider in &providers_to_process {
                        if let Err(error) = process_provider_bridge_event(
                            &handle,
                            &refreshes,
                            &tracked_records,
                            provider,
                        ) {
                            eprintln!("failed to process {provider} bridge event: {error}");
                        }
                    }
                    break;
                }
            });
        }
    })
    .map_err(|error| format!("failed to create provider bridge watcher: {error}"))?;

    watcher
        .watch(&bridge_dir, RecursiveMode::NonRecursive)
        .map_err(|error| format!("failed to watch {}: {error}", bridge_dir.display()))?;
    for bridge_path in provider_bridge_paths.values() {
        if bridge_path.exists() {
            watcher
                .watch(bridge_path, RecursiveMode::NonRecursive)
                .map_err(|error| format!("failed to watch {}: {error}", bridge_path.display()))?;
        }
    }

    Ok(watcher)
}

fn restart_session_watcher_internal(
    app: &tauri::AppHandle,
    watcher_state: &WatcherState,
    copilot_root: Option<&str>,
    opencode_root: Option<&str>,
    enabled_providers: &[String],
) -> Result<(), String> {
    let copilot_bridge_active = enabled_providers.iter().any(|p| p == COPILOT_PROVIDER)
        && matches!(
            detect_copilot_integration_status(copilot_root).status,
            ProviderIntegrationState::Installed
        );
    let opencode_bridge_active = enabled_providers.iter().any(|p| p == OPENCODE_PROVIDER)
        && matches!(
            detect_opencode_integration_status().status,
            ProviderIntegrationState::Installed
        );

    let mut bridge_providers = Vec::new();
    if copilot_bridge_active {
        bridge_providers.push(COPILOT_PROVIDER.to_string());
    }
    if opencode_bridge_active {
        bridge_providers.push(OPENCODE_PROVIDER.to_string());
    }

    if bridge_providers.is_empty() {
        let mut provider_bridge = watcher_state
            .provider_bridge
            .lock()
            .map_err(|_| "failed to lock provider bridge watcher state".to_string())?;
        *provider_bridge = None;
    } else {
        let watcher = create_provider_bridge_watcher(
            app,
            bridge_providers,
            Arc::clone(&watcher_state.last_provider_refresh),
            Arc::clone(&watcher_state.last_bridge_records),
        )?;
        let mut provider_bridge = watcher_state
            .provider_bridge
            .lock()
            .map_err(|_| "failed to lock provider bridge watcher state".to_string())?;
        *provider_bridge = Some(watcher);
    }

    // Copilot watcher
    if enabled_providers.iter().any(|p| p == COPILOT_PROVIDER) && !copilot_bridge_active {
        let root = resolve_copilot_root(copilot_root)?;
        match create_sessions_watcher(app, &root, Arc::clone(&watcher_state.last_provider_refresh))
        {
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
    if enabled_providers.iter().any(|p| p == OPENCODE_PROVIDER) && !opencode_bridge_active {
        if let Ok(oc_root) = resolve_opencode_root(opencode_root) {
            match create_opencode_watcher(
                app,
                &oc_root,
                Arc::clone(&watcher_state.last_provider_refresh),
            ) {
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

    // Migration: 新增 input_tokens 欄位（舊資料庫相容）
    if let Err(error) = connection.execute(
        "ALTER TABLE session_stats ADD COLUMN input_tokens INTEGER NOT NULL DEFAULT 0",
        [],
    ) {
        let error_message = error.to_string();
        if !error_message.contains("duplicate column name") {
            eprintln!("Warning: failed to add input_tokens column: {error}");
        }
    }

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
                   duration_minutes, models_used, reasoning_count, tool_breakdown, input_tokens
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
                    input_tokens: row
                        .get(8)
                        .map_err(|error| format!("failed to read input_tokens column: {error}"))?,
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
                session_id, events_mtime, output_tokens, input_tokens, interaction_count,
                tool_call_count, duration_minutes, models_used, reasoning_count, tool_breakdown
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(session_id) DO UPDATE SET
                events_mtime = excluded.events_mtime,
                output_tokens = excluded.output_tokens,
                input_tokens = excluded.input_tokens,
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
                stats.input_tokens,
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

// ── OpenCode JSON Storage 解析 ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct OpencodeTokens {
    #[serde(default)]
    input: Option<u64>,
    #[serde(default)]
    output: Option<u64>,
    #[serde(default)]
    reasoning: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OpencodeMessageTime {
    #[serde(default)]
    created: Option<i64>,
    #[serde(default)]
    completed: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpencodeMessage {
    id: String,
    #[serde(default)]
    role: String,
    #[serde(default)]
    time: Option<OpencodeMessageTime>,
    #[serde(default)]
    tokens: Option<OpencodeTokens>,
    #[serde(default)]
    model_id: Option<String>,
}

/// 判斷 session_dir 是否為 OpenCode session（目錄名以 ses_ 開頭）
fn is_opencode_session_dir(session_dir: &Path) -> bool {
    session_dir
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with("ses_"))
        .unwrap_or(false)
}

/// 判斷 OpenCode message 目錄是否為活躍狀態（最近 5 分鐘內有修改）
fn is_opencode_session_live(message_dir: &Path) -> bool {
    let mtime = dir_mtime_secs(message_dir);
    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    now - mtime < 300
}

/// 解析單一 msg_*.json 檔案為 OpencodeMessage
fn parse_opencode_message_json(path: &Path) -> Option<OpencodeMessage> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str::<OpencodeMessage>(&content).ok()
}

/// 掃描 storage/message/{sessionID}/ 目錄，回傳所有 OpencodeMessage
fn scan_opencode_message_dir(message_dir: &Path) -> Vec<OpencodeMessage> {
    if !message_dir.exists() {
        return Vec::new();
    }
    let Ok(entries) = fs::read_dir(message_dir) else {
        return Vec::new();
    };
    let mut messages = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(msg) = parse_opencode_message_json(&path) {
                messages.push(msg);
            }
        }
    }
    messages
}

/// 掃描 storage/part/{messageID}/ 目錄，回傳 type=tool 的工具名稱清單
fn scan_opencode_parts_for_message(storage_root: &Path, message_id: &str) -> Vec<String> {
    let part_dir = storage_root.join("part").join(message_id);
    if !part_dir.exists() {
        return Vec::new();
    }
    let Ok(entries) = fs::read_dir(&part_dir) else {
        return Vec::new();
    };
    let mut tools = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };
        if value.get("type").and_then(|v| v.as_str()) == Some("tool") {
            let tool_name = value
                .pointer("/state/tool")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            tools.push(tool_name);
        }
    }
    tools
}

/// 計算 OpenCode session 的統計資料
/// message_dir: storage/message/{sessionID}/
fn calculate_opencode_session_stats(message_dir: &Path) -> Result<SessionStats, String> {
    let storage_root = message_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "cannot determine storage root from message_dir".to_string())?;

    let messages = scan_opencode_message_dir(message_dir);
    if messages.is_empty() {
        return Ok(SessionStats::default());
    }

    let mut stats = SessionStats::default();
    let mut models_used = BTreeSet::new();
    let mut min_time: Option<i64> = None;
    let mut max_time: Option<i64> = None;

    for msg in &messages {
        // 追蹤時間範圍
        if let Some(time) = &msg.time {
            let t = time.completed.or(time.created);
            if let Some(t) = t {
                min_time = Some(min_time.map_or(t, |m: i64| m.min(t)));
                max_time = Some(max_time.map_or(t, |m: i64| m.max(t)));
            }
        }

        match msg.role.as_str() {
            "user" => {
                stats.interaction_count += 1;
            }
            "assistant" => {
                if let Some(tokens) = &msg.tokens {
                    stats.output_tokens += tokens.output.unwrap_or(0);
                    stats.input_tokens += tokens.input.unwrap_or(0);
                    if tokens.reasoning.unwrap_or(0) > 0 {
                        stats.reasoning_count += 1;
                    }
                }
                if let Some(model) = msg.model_id.as_deref().filter(|m| !m.is_empty()) {
                    models_used.insert(model.to_string());
                }
                // 掃描 part 目錄取得工具呼叫
                let tool_names = scan_opencode_parts_for_message(storage_root, &msg.id);
                for tool_name in tool_names {
                    stats.tool_call_count += 1;
                    *stats.tool_breakdown.entry(tool_name).or_insert(0) += 1;
                }
            }
            _ => {}
        }
    }

    stats.models_used = models_used.into_iter().collect();

    if let (Some(min_t), Some(max_t)) = (min_time, max_time) {
        let diff_ms = max_t - min_t;
        if diff_ms > 0 {
            stats.duration_minutes = (diff_ms as u64) / 60_000;
        }
    }

    Ok(stats)
}

/// OpenCode stats 的快取讀取、計算、寫入入口
fn get_opencode_session_stats_internal(
    connection: &Connection,
    message_dir: &Path,
    session_id: &str,
) -> Result<SessionStats, String> {
    if !message_dir.exists() {
        return Ok(SessionStats::default());
    }

    let is_live = is_opencode_session_live(message_dir);
    let dir_mtime = dir_mtime_secs(message_dir);

    if !is_live {
        if let Some((cached_mtime, mut cached_stats)) =
            get_session_stats_cache(connection, session_id)?
        {
            if cached_mtime == dir_mtime {
                cached_stats.is_live = false;
                return Ok(cached_stats);
            }
        }
    }

    let mut stats = calculate_opencode_session_stats(message_dir)?;
    stats.is_live = is_live;

    if !stats.is_live {
        upsert_session_stats_cache(connection, session_id, dir_mtime, &stats)?;
    }

    Ok(stats)
}

// ── End OpenCode JSON Storage 解析 ───────────────────────────────────────────

fn get_session_stats_internal(
    connection: &Connection,
    session_dir: &str,
) -> Result<SessionStats, String> {
    let session_path = PathBuf::from(session_dir);
    let session_id = session_id_from_dir(&session_path);

    // OpenCode session：session_dir 為 storage/message/{ses_xxx}，目錄名以 ses_ 開頭
    if is_opencode_session_dir(&session_path) {
        return get_opencode_session_stats_internal(connection, &session_path, &session_id);
    }

    // Copilot session：原有邏輯
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

/// 判斷是否需要執行全掃描
fn should_full_scan(cache: &Option<ProviderCache>, force_full: bool) -> bool {
    if force_full {
        return true;
    }
    match cache {
        None => true,
        Some(c) => c.last_full_scan_at.elapsed().as_secs() > FULL_SCAN_THRESHOLD_SECS,
    }
}

/// 取得目錄的最後修改時間（Unix 秒），失敗時回傳 0
fn dir_mtime_secs(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Copilot 增量掃描：只重新解析 mtime 有變化或新增的 session 目錄
fn scan_copilot_incremental_internal(
    session_state_dir: &Path,
    is_archived: bool,
    connection: &Connection,
    cache: &mut ProviderCache,
) -> Result<(), String> {
    if !session_state_dir.exists() {
        // 目錄消失則清空對應快取
        cache.sessions.retain(|s| s.is_archived != is_archived);
        return Ok(());
    }

    let entries = fs::read_dir(session_state_dir)
        .map_err(|error| format!("failed to read {}: {error}", session_state_dir.display()))?;

    let mut current_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

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
            // mtime 有變化或新增：重新解析
            let meta = read_session_meta(connection, &session_id)?;
            let info = parse_workspace_file(&session_dir, &workspace_path, is_archived, meta);
            // 更新 sessions 快取（replace or push）
            if let Some(pos) = cache.sessions.iter().position(|s| s.id == session_id) {
                cache.sessions[pos] = info;
            } else {
                cache.sessions.push(info);
            }
            cache.session_mtimes.insert(session_id, current_mtime);
        }
    }

    // 移除已消失的 session（只處理對應 is_archived 的 bucket）
    cache.sessions.retain(|s| {
        if s.is_archived != is_archived {
            return true; // 不同 bucket，保留
        }
        current_ids.contains(&s.id)
    });
    cache
        .session_mtimes
        .retain(|id, _| current_ids.contains(id.as_str()));

    Ok(())
}

/// OpenCode 增量掃描：只查詢 time_updated > last_cursor 的 session
fn scan_opencode_incremental_internal(
    opencode_root: &Path,
    show_archived: bool,
    metadata_conn: &Connection,
    cache: &mut ProviderCache,
) -> Result<(), String> {
    let oc_conn = match open_opencode_db_readonly(opencode_root) {
        Ok(conn) => conn,
        Err(error) => {
            eprintln!("opencode incremental scan: {error}");
            return Ok(());
        }
    };

    let query = if show_archived {
        "SELECT s.id, s.title, s.time_created, s.time_updated, s.time_archived, \
                p.worktree, s.summary_additions, s.summary_deletions, s.summary_files \
         FROM session s \
         LEFT JOIN project p ON s.project_id = p.id \
         WHERE s.time_updated > ?1 \
         ORDER BY s.time_updated ASC"
    } else {
        "SELECT s.id, s.title, s.time_created, s.time_updated, s.time_archived, \
                p.worktree, s.summary_additions, s.summary_deletions, s.summary_files \
         FROM session s \
         LEFT JOIN project p ON s.project_id = p.id \
         WHERE s.time_updated > ?1 AND s.time_archived IS NULL \
         ORDER BY s.time_updated ASC"
    };

    let mut statement = oc_conn
        .prepare(query)
        .map_err(|error| format!("opencode incremental query prepare error: {error}"))?;

    let rows = statement
        .query_map([cache.last_cursor], |row| {
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
        .map_err(|error| format!("opencode incremental query error: {error}"))?;

    let mut new_cursor = cache.last_cursor;

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
                eprintln!("opencode incremental: failed to read row: {error}");
                continue;
            }
        };

        let is_archived = time_archived.is_some();
        let created_at = time_created.and_then(unix_ms_to_iso8601);
        let updated_at = time_updated.and_then(unix_ms_to_iso8601);

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

        let info = SessionInfo {
            id: id.clone(),
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
        };

        // upsert by session_id
        if let Some(pos) = cache.sessions.iter().position(|s| s.id == id) {
            cache.sessions[pos] = info;
        } else {
            cache.sessions.push(info);
        }

        // 更新 cursor 為見到的最大 time_updated
        if let Some(ts) = time_updated {
            if ts > new_cursor {
                new_cursor = ts;
            }
        }
    }

    cache.last_cursor = new_cursor;
    Ok(())
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

/// 合法終端機可執行檔名稱白名單（不區分大小寫）
const VALID_TERMINAL_STEMS: &[&str] = &["pwsh", "powershell", "cmd", "bash", "sh"];

fn validate_terminal_path_internal(path: &str) -> bool {
    let candidate = PathBuf::from(path);

    if !candidate.exists() || !candidate.is_file() {
        return false;
    }

    // 確認 file_stem 為合法終端機名稱
    candidate
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| {
            let stem_lower = stem.to_lowercase();
            VALID_TERMINAL_STEMS.contains(&stem_lower.as_str())
        })
        .unwrap_or(false)
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

fn open_terminal_internal(terminal_path: &str, cwd: &str) -> Result<(), String> {
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
        // pwsh、powershell 及其他未知終端機皆走 PowerShell 語法
        _ => {
            cmd.args(["-NoExit", "-Command", &format!("cd '{}'", cwd)])
                .current_dir(cwd);
        }
    }

    // CMD 和 PowerShell 用 current_dir 以外的方式切換，仍需設定 current_dir 作為 fallback
    if stem != "bash" && stem != "sh" {
        cmd.current_dir(cwd);
    }

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

/// 從 OpenCode DB 取得最大 time_updated 值，用於全掃描後設置 last_cursor
fn get_opencode_max_cursor(opencode_root: &Path) -> Result<i64, String> {
    let oc_conn = open_opencode_db_readonly(opencode_root)?;
    let cursor: i64 = oc_conn
        .query_row(
            "SELECT COALESCE(MAX(time_updated), 0) FROM session",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("failed to get max cursor: {e}"))?;
    Ok(cursor)
}

fn get_sessions_internal(
    root_dir: Option<String>,
    opencode_root: Option<String>,
    show_archived: Option<bool>,
    enabled_providers: Option<Vec<String>>,
    force_full: Option<bool>,
    scan_cache: &ScanCache,
) -> Result<Vec<SessionInfo>, String> {
    let resolved_copilot = resolve_copilot_root(root_dir.as_deref())?;
    let resolved_opencode = resolve_opencode_root(opencode_root.as_deref()).ok();
    let providers = enabled_providers.unwrap_or_else(default_enabled_providers);
    let show_archived = show_archived.unwrap_or(false);
    let force = force_full.unwrap_or(false);

    let connection = open_db_connection()?;
    init_db(&connection)?;

    let mut all_sessions: Vec<SessionInfo> = Vec::new();

    // ── Copilot provider ──────────────────────────────────────────────────────
    if providers.iter().any(|p| p == "copilot") {
        let mut copilot_guard = scan_cache
            .copilot
            .lock()
            .map_err(|_| "failed to lock copilot scan cache".to_string())?;

        if should_full_scan(&copilot_guard, force) {
            // 全掃描
            let mut sessions =
                scan_session_dir(&resolved_copilot.join("session-state"), false, &connection)?;
            if show_archived {
                sessions.extend(scan_session_dir(
                    &resolved_copilot.join("session-state-archive"),
                    true,
                    &connection,
                )?);
            }

            // 建立 mtime 索引
            let mut mtimes = HashMap::new();
            for session in &sessions {
                let dir = PathBuf::from(&session.session_dir);
                mtimes.insert(session.id.clone(), dir_mtime_secs(&dir));
            }

            *copilot_guard = Some(ProviderCache {
                sessions: sessions.clone(),
                session_mtimes: mtimes,
                last_full_scan_at: Instant::now(),
                last_cursor: 0,
            });

            all_sessions.extend(sessions);
        } else {
            // 增量掃描
            let cache = copilot_guard
                .as_mut()
                .expect("cache is Some after should_full_scan check");
            scan_copilot_incremental_internal(
                &resolved_copilot.join("session-state"),
                false,
                &connection,
                cache,
            )?;
            if show_archived {
                scan_copilot_incremental_internal(
                    &resolved_copilot.join("session-state-archive"),
                    true,
                    &connection,
                    cache,
                )?;
            }
            all_sessions.extend(cache.sessions.iter().cloned());
        }
    }

    // ── OpenCode provider ─────────────────────────────────────────────────────
    if providers.iter().any(|p| p == "opencode") {
        if let Some(oc_root) = &resolved_opencode {
            let mut oc_guard = scan_cache
                .opencode
                .lock()
                .map_err(|_| "failed to lock opencode scan cache".to_string())?;

            if should_full_scan(&oc_guard, force) {
                // 全掃描
                match scan_opencode_sessions_internal(oc_root, show_archived, &connection) {
                    Ok(sessions) => {
                        // 計算 last_cursor（最大 time_updated）
                        // OpenCode 的 time_updated 是毫秒 timestamp，從 ISO8601 反推不方便，
                        // 直接查 DB 取最大值
                        let max_cursor = get_opencode_max_cursor(oc_root).unwrap_or(0);
                        *oc_guard = Some(ProviderCache {
                            sessions: sessions.clone(),
                            session_mtimes: HashMap::new(),
                            last_full_scan_at: Instant::now(),
                            last_cursor: max_cursor,
                        });
                        all_sessions.extend(sessions);
                    }
                    Err(error) => {
                        eprintln!("opencode provider error (ignored): {error}");
                    }
                }
            } else {
                // 增量掃描
                let cache = oc_guard
                    .as_mut()
                    .expect("cache is Some after should_full_scan check");
                if let Err(e) =
                    scan_opencode_incremental_internal(oc_root, show_archived, &connection, cache)
                {
                    eprintln!("opencode incremental scan error (ignored): {e}");
                }
                all_sessions.extend(cache.sessions.iter().cloned());
            }
        }
    }

    all_sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

    // 掃描完成後儲存快取（失敗不影響主流程）
    if let Err(error) = save_session_cache(&all_sessions) {
        eprintln!("session cache save error (ignored): {error}");
    }

    Ok(all_sessions)
}

#[tauri::command]
fn get_sessions(
    root_dir: Option<String>,
    opencode_root: Option<String>,
    show_archived: Option<bool>,
    enabled_providers: Option<Vec<String>>,
    force_full: Option<bool>,
    scan_cache: State<'_, ScanCache>,
) -> Result<Vec<SessionInfo>, String> {
    get_sessions_internal(
        root_dir,
        opencode_root,
        show_archived,
        enabled_providers,
        force_full,
        scan_cache.inner(),
    )
}

fn get_settings_internal() -> Result<AppSettings, String> {
    let mut settings = load_settings_internal()?;
    // 若 opencode_root 為空（舊版 settings.json 無此欄位），補填預設值供前端顯示
    if settings.opencode_root.trim().is_empty() {
        if let Ok(default_root) = default_opencode_root() {
            settings.opencode_root = default_root.to_string_lossy().to_string();
        }
    }
    settings.provider_integrations =
        collect_provider_integration_statuses(Some(settings.copilot_root.as_str()));
    Ok(settings)
}

#[tauri::command]
fn get_settings() -> Result<AppSettings, String> {
    get_settings_internal()
}

#[tauri::command]
fn save_settings(settings: AppSettings) -> Result<(), String> {
    save_settings_internal(&settings)
}

fn restart_provider_watchers_after_integration_change(
    app: &tauri::AppHandle,
    watcher_state: &WatcherState,
    copilot_root_override: Option<&str>,
) -> Result<(), String> {
    let settings = get_settings_internal().unwrap_or_else(|_| AppSettings {
        copilot_root: copilot_root_override.unwrap_or_default().to_string(),
        opencode_root: default_opencode_root()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default(),
        terminal_path: None,
        external_editor_path: None,
        show_archived: false,
        pinned_projects: Vec::new(),
        enabled_providers: default_enabled_providers(),
        provider_integrations: Vec::new(),
    });

    let copilot_root = copilot_root_override.unwrap_or(settings.copilot_root.as_str());
    restart_session_watcher_internal(
        app,
        watcher_state,
        Some(copilot_root),
        Some(settings.opencode_root.as_str()),
        &settings.enabled_providers,
    )
}

#[tauri::command]
fn install_provider_integration(
    app: tauri::AppHandle,
    watcher_state: State<'_, WatcherState>,
    provider: String,
    copilot_root: Option<String>,
) -> Result<ProviderIntegrationStatus, String> {
    let status = install_or_update_provider_integration(&provider, copilot_root.as_deref())?;
    restart_provider_watchers_after_integration_change(
        &app,
        &watcher_state,
        copilot_root.as_deref(),
    )?;
    Ok(status)
}

#[tauri::command]
fn update_provider_integration(
    app: tauri::AppHandle,
    watcher_state: State<'_, WatcherState>,
    provider: String,
    copilot_root: Option<String>,
) -> Result<ProviderIntegrationStatus, String> {
    let status = install_or_update_provider_integration(&provider, copilot_root.as_deref())?;
    restart_provider_watchers_after_integration_change(
        &app,
        &watcher_state,
        copilot_root.as_deref(),
    )?;
    Ok(status)
}

#[tauri::command]
fn recheck_provider_integration(
    app: tauri::AppHandle,
    watcher_state: State<'_, WatcherState>,
    provider: String,
    copilot_root: Option<String>,
) -> Result<ProviderIntegrationStatus, String> {
    let status = recheck_provider_integration_status(&provider, copilot_root.as_deref())?;
    restart_provider_watchers_after_integration_change(
        &app,
        &watcher_state,
        copilot_root.as_deref(),
    )?;
    Ok(status)
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
fn open_terminal(terminal_path: String, cwd: String, _session_id: String) -> Result<(), String> {
    open_terminal_internal(&terminal_path, &cwd)
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
        .manage(ScanCache::default())
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
            load_session_cache,
            get_settings,
            save_settings,
            install_provider_integration,
            update_provider_integration,
            recheck_provider_integration,
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
    use std::ffi::OsString;
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

    fn with_env_var<T>(key: &str, value: &Path, callback: impl FnOnce() -> T) -> T {
        let previous: Option<OsString> = env::var_os(key);
        unsafe {
            env::set_var(key, value);
        }
        let result = callback();
        unsafe {
            match previous {
                Some(previous) => env::set_var(key, previous),
                None => env::remove_var(key),
            }
        }
        result
    }

    fn without_env_var<T>(key: &str, callback: impl FnOnce() -> T) -> T {
        let previous: Option<OsString> = env::var_os(key);
        unsafe {
            env::remove_var(key);
        }
        let result = callback();
        unsafe {
            if let Some(previous) = previous {
                env::set_var(key, previous);
            }
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

    fn bridge_record_json(
        provider: &str,
        event_type: &str,
        timestamp: &str,
        error: Option<&str>,
    ) -> String {
        serde_json::to_string(&ProviderBridgeRecord {
            version: PROVIDER_INTEGRATION_VERSION,
            provider: provider.to_string(),
            event_type: event_type.to_string(),
            timestamp: timestamp.to_string(),
            session_id: Some("session-001".to_string()),
            cwd: Some("D:\\repo".to_string()),
            source_path: None,
            title: Some("Test".to_string()),
            error: error.map(|value| value.to_string()),
        })
        .expect("serialize bridge record")
    }

    #[test]
    fn provider_refresh_dedup_suppresses_duplicate_refreshes_within_window() {
        let refresh_state = Arc::new(Mutex::new(HashMap::new()));
        let now = Instant::now();

        assert!(
            should_emit_provider_refresh_at(&refresh_state, COPILOT_PROVIDER, now)
                .expect("first refresh should emit")
        );
        assert!(!should_emit_provider_refresh_at(
            &refresh_state,
            COPILOT_PROVIDER,
            now + Duration::from_millis(PROVIDER_REFRESH_DEDUP_MS - 1)
        )
        .expect("duplicate refresh should dedupe"));
        assert!(should_emit_provider_refresh_at(
            &refresh_state,
            COPILOT_PROVIDER,
            now + Duration::from_millis(PROVIDER_REFRESH_DEDUP_MS + 1)
        )
        .expect("refresh after window should emit"));
    }

    #[test]
    fn opencode_refresh_snapshot_skips_unchanged_state_and_emits_after_change() {
        let _guard = test_lock().lock().expect("failed to lock test mutex");
        let oc_dir = unique_test_dir("oc-refresh-snapshot");
        fs::create_dir_all(&oc_dir).expect("create opencode dir");

        let snapshot_state = Arc::new(Mutex::new(build_opencode_watch_snapshot(&oc_dir)));
        assert!(!should_emit_opencode_refresh(&oc_dir, &snapshot_state)
            .expect("unchanged snapshot should not emit"));

        create_opencode_db(&oc_dir, &[("oc-refresh", "Refresh", 1000, 2000, None)]);

        assert!(should_emit_opencode_refresh(&oc_dir, &snapshot_state)
            .expect("db change should emit once"));
        assert!(!should_emit_opencode_refresh(&oc_dir, &snapshot_state)
            .expect("unchanged snapshot after refresh should not emit"));

        fs::remove_dir_all(&oc_dir).expect("cleanup opencode dir");
    }

    #[test]
    fn register_provider_bridge_record_skips_duplicate_last_record() {
        let bridge_records = Arc::new(Mutex::new(HashMap::new()));
        let record = ProviderBridgeRecord {
            version: PROVIDER_INTEGRATION_VERSION,
            provider: OPENCODE_PROVIDER.to_string(),
            event_type: "session.updated".to_string(),
            timestamp: "2026-04-01T09:00:00Z".to_string(),
            session_id: Some("session-001".to_string()),
            cwd: Some("D:\\repo".to_string()),
            source_path: None,
            title: Some("Demo".to_string()),
            error: None,
        };

        assert!(
            register_provider_bridge_record(&bridge_records, OPENCODE_PROVIDER, &record)
                .expect("first record should register")
        );
        assert!(
            !register_provider_bridge_record(&bridge_records, OPENCODE_PROVIDER, &record)
                .expect("duplicate record should skip")
        );
    }

    #[test]
    fn register_provider_bridge_record_treats_error_change_as_distinct() {
        let bridge_records = Arc::new(Mutex::new(HashMap::new()));
        let mut record = ProviderBridgeRecord {
            version: PROVIDER_INTEGRATION_VERSION,
            provider: OPENCODE_PROVIDER.to_string(),
            event_type: "session.updated".to_string(),
            timestamp: "2026-04-01T09:00:00Z".to_string(),
            session_id: Some("session-001".to_string()),
            cwd: Some("D:\\repo".to_string()),
            source_path: None,
            title: Some("Demo".to_string()),
            error: None,
        };

        assert!(
            register_provider_bridge_record(&bridge_records, OPENCODE_PROVIDER, &record)
                .expect("first record should register")
        );

        record.error = Some("refresh failed".to_string());

        assert!(
            register_provider_bridge_record(&bridge_records, OPENCODE_PROVIDER, &record)
                .expect("record with different error should not be deduplicated")
        );
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

        // 在 with_appdata 閉包內開啟 DB，閉包結束後 connection 自動 drop，
        // 確保 SQLite 檔案在 remove_dir_all 前已被釋放
        let sessions = with_appdata(&appdata_dir, || {
            let connection = open_db_connection().expect("open db");
            init_db(&connection).expect("init db");
            scan_session_dir(&root_dir.join("session-state"), false, &connection)
                .expect("scan should succeed")
        });

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

    // ──────────────────────────────────────────────────────────────────────────
    // should_full_scan
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn should_full_scan_returns_true_when_cache_is_none() {
        // 快取為 None（首次啟動）→ 必須執行全掃
        assert!(should_full_scan(&None, false));
    }

    #[test]
    fn should_full_scan_returns_true_when_force_full_is_set() {
        // force_full = true，無論快取狀態都必須全掃
        let cache = Some(ProviderCache {
            sessions: Vec::new(),
            session_mtimes: HashMap::new(),
            last_full_scan_at: Instant::now(),
            last_cursor: 0,
        });
        assert!(should_full_scan(&cache, true));
    }

    #[test]
    fn should_full_scan_returns_false_when_cache_is_fresh() {
        // 快取剛建立（elapsed ≈ 0），不需全掃
        let cache = Some(ProviderCache {
            sessions: Vec::new(),
            session_mtimes: HashMap::new(),
            last_full_scan_at: Instant::now(),
            last_cursor: 0,
        });
        assert!(!should_full_scan(&cache, false));
    }

    // ──────────────────────────────────────────────────────────────────────────
    // dir_mtime_secs
    // ──────────────────────────────────────────────────────────────────────────

    #[test]
    fn dir_mtime_secs_returns_zero_for_missing_path() {
        let missing = std::env::temp_dir().join("session-hub-nonexistent-dir-xyz");
        assert_eq!(dir_mtime_secs(&missing), 0);
    }

    #[test]
    fn dir_mtime_secs_returns_positive_for_existing_dir() {
        let dir = unique_test_dir("mtime");
        fs::create_dir_all(&dir).expect("create dir");

        let mtime = dir_mtime_secs(&dir);
        assert!(mtime > 0, "mtime should be a positive unix timestamp");

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    // ──────────────────────────────────────────────────────────────────────────
    // scan_copilot_incremental_internal
    // ──────────────────────────────────────────────────────────────────────────

    /// 建立最小測試用 ProviderCache（空快取）
    fn empty_copilot_cache() -> ProviderCache {
        ProviderCache {
            sessions: Vec::new(),
            session_mtimes: HashMap::new(),
            last_full_scan_at: Instant::now(),
            last_cursor: 0,
        }
    }

    #[test]
    fn incremental_copilot_picks_up_new_session() {
        // 對一個空快取執行增量掃描，應偵測到新建的 session 目錄
        let _guard = test_lock().lock().expect("lock");
        let root_dir = unique_test_dir("inc-new");
        let appdata_dir = unique_test_dir("appdata");
        let session_dir = root_dir.join("session-state").join("session-inc-001");
        fs::create_dir_all(&session_dir).expect("create session dir");
        fs::write(
            session_dir.join("workspace.yaml"),
            "id: session-inc-001\ncwd: D:\\repo\\demo\nsummary: Inc Test\nsummary_count: 1\ncreated_at: 2025-01-01T00:00:00Z\nupdated_at: 2025-01-02T00:00:00Z\n",
        )
        .expect("write workspace.yaml");

        let sessions = with_appdata(&appdata_dir, || {
            let conn = open_db_connection().expect("open db");
            init_db(&conn).expect("init db");
            let mut cache = empty_copilot_cache();
            scan_copilot_incremental_internal(
                &root_dir.join("session-state"),
                false,
                &conn,
                &mut cache,
            )
            .expect("incremental scan");
            cache.sessions
        });

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "session-inc-001");
        assert_eq!(sessions[0].summary.as_deref(), Some("Inc Test"));
        assert!(!sessions[0].is_archived);

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_copilot_skips_unchanged_session() {
        // mtime 未變化的 session 不應重新解析（快取命中）
        let _guard = test_lock().lock().expect("lock");
        let root_dir = unique_test_dir("inc-skip");
        let appdata_dir = unique_test_dir("appdata");
        let session_dir = root_dir.join("session-state").join("session-skip-001");
        fs::create_dir_all(&session_dir).expect("create session dir");
        fs::write(
            session_dir.join("workspace.yaml"),
            "id: session-skip-001\ncwd: D:\\repo\nsummary: Original\nsummary_count: 1\ncreated_at: 2025-01-01T00:00:00Z\nupdated_at: 2025-01-01T00:00:00Z\n",
        )
        .expect("write workspace.yaml");

        with_appdata(&appdata_dir, || {
            let conn = open_db_connection().expect("open db");
            init_db(&conn).expect("init db");

            // 第一次掃描：讀入快取並記錄 mtime
            let mut cache = empty_copilot_cache();
            scan_copilot_incremental_internal(
                &root_dir.join("session-state"),
                false,
                &conn,
                &mut cache,
            )
            .expect("first scan");
            assert_eq!(cache.sessions.len(), 1);

            // 手動竄改快取中的 summary，模擬「已有舊資料」
            cache.sessions[0].summary = Some("Cached Value".to_string());

            // 第二次掃描：mtime 未變，不應覆蓋快取內容
            scan_copilot_incremental_internal(
                &root_dir.join("session-state"),
                false,
                &conn,
                &mut cache,
            )
            .expect("second scan");

            // 快取命中，summary 仍是手動設定的值
            assert_eq!(cache.sessions[0].summary.as_deref(), Some("Cached Value"));
        });

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_copilot_removes_deleted_session() {
        // 目錄被刪除後，再次增量掃描應從快取中移除對應 session
        let _guard = test_lock().lock().expect("lock");
        let root_dir = unique_test_dir("inc-del");
        let appdata_dir = unique_test_dir("appdata");
        let session_state = root_dir.join("session-state");
        let session_dir = session_state.join("session-del-001");
        fs::create_dir_all(&session_dir).expect("create session dir");
        fs::write(
            session_dir.join("workspace.yaml"),
            "id: session-del-001\ncwd: D:\\repo\nsummary: To Delete\nsummary_count: 1\ncreated_at: 2025-01-01T00:00:00Z\nupdated_at: 2025-01-01T00:00:00Z\n",
        )
        .expect("write workspace.yaml");

        with_appdata(&appdata_dir, || {
            let conn = open_db_connection().expect("open db");
            init_db(&conn).expect("init db");

            let mut cache = empty_copilot_cache();

            // 第一次掃描：快取 1 個 session
            scan_copilot_incremental_internal(&session_state, false, &conn, &mut cache)
                .expect("first scan");
            assert_eq!(cache.sessions.len(), 1);

            // 刪除目錄
            fs::remove_dir_all(&session_dir).expect("remove session dir");

            // 第二次掃描：session 應從快取消失
            scan_copilot_incremental_internal(&session_state, false, &conn, &mut cache)
                .expect("second scan");
            assert!(
                cache.sessions.is_empty(),
                "deleted session should be removed from cache"
            );
            assert!(
                cache.session_mtimes.is_empty(),
                "mtime entry should be removed"
            );
        });

        fs::remove_dir_all(&root_dir).expect("cleanup root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_copilot_clears_cache_when_dir_missing() {
        // session-state 目錄本身不存在時，對應 bucket 的快取應被清空
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("appdata");
        let missing_dir = unique_test_dir("inc-missing").join("session-state");

        // 預先塞入一筆假資料到快取
        let mut cache = empty_copilot_cache();
        cache.sessions.push(SessionInfo {
            id: "ghost-session".to_string(),
            provider: "copilot".to_string(),
            cwd: None,
            summary: None,
            summary_count: None,
            created_at: None,
            updated_at: None,
            session_dir: String::new(),
            parse_error: false,
            is_archived: false,
            notes: None,
            tags: Vec::new(),
            has_plan: false,
            has_events: false,
        });

        with_appdata(&appdata_dir, || {
            let conn = open_db_connection().expect("open db");
            init_db(&conn).expect("init db");

            scan_copilot_incremental_internal(&missing_dir, false, &conn, &mut cache)
                .expect("scan on missing dir");

            assert!(
                cache.sessions.is_empty(),
                "cache should be cleared when dir is missing"
            );
        });

        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_copilot_preserves_other_bucket_on_dir_missing() {
        // session-state 消失時，只清除 is_archived=false 的 bucket，
        // is_archived=true 的 session 應保留
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("appdata");
        let missing_dir = unique_test_dir("inc-bucket").join("session-state");

        let mut cache = empty_copilot_cache();
        // 塞入 active session（is_archived=false）
        cache.sessions.push(SessionInfo {
            id: "active-session".to_string(),
            provider: "copilot".to_string(),
            cwd: None,
            summary: None,
            summary_count: None,
            created_at: None,
            updated_at: None,
            session_dir: String::new(),
            parse_error: false,
            is_archived: false,
            notes: None,
            tags: Vec::new(),
            has_plan: false,
            has_events: false,
        });
        // 塞入 archived session（is_archived=true）
        cache.sessions.push(SessionInfo {
            id: "archived-session".to_string(),
            provider: "copilot".to_string(),
            cwd: None,
            summary: None,
            summary_count: None,
            created_at: None,
            updated_at: None,
            session_dir: String::new(),
            parse_error: false,
            is_archived: true,
            notes: None,
            tags: Vec::new(),
            has_plan: false,
            has_events: false,
        });

        with_appdata(&appdata_dir, || {
            let conn = open_db_connection().expect("open db");
            init_db(&conn).expect("init db");

            // 掃描 is_archived=false 的目錄（不存在）→ 只清除 active bucket
            scan_copilot_incremental_internal(&missing_dir, false, &conn, &mut cache)
                .expect("scan");

            assert_eq!(
                cache.sessions.len(),
                1,
                "only archived session should remain"
            );
            assert_eq!(cache.sessions[0].id, "archived-session");
        });

        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    // ──────────────────────────────────────────────────────────────────────────
    // scan_opencode_incremental_internal
    // ──────────────────────────────────────────────────────────────────────────

    fn create_full_opencode_db(dir: &Path) {
        let db_path = dir.join("opencode.db");
        let conn = Connection::open(&db_path).expect("create opencode.db");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS project (
                id TEXT PRIMARY KEY,
                worktree TEXT
             );
             CREATE TABLE IF NOT EXISTS session (
                id TEXT PRIMARY KEY,
                title TEXT,
                time_created INTEGER,
                time_updated INTEGER,
                time_archived INTEGER,
                project_id TEXT,
                summary_additions INTEGER,
                summary_deletions INTEGER,
                summary_files INTEGER
             );",
        )
        .expect("create tables");
    }

    /// 建立最小的 opencode.db，包含指定 session 資料
    fn create_opencode_db(dir: &Path, sessions: &[(&str, &str, i64, i64, Option<i64>)]) {
        // sessions 欄位：(id, title, time_created, time_updated, time_archived)
        create_full_opencode_db(dir);
        let db_path = dir.join("opencode.db");
        let conn = Connection::open(&db_path).expect("reopen opencode.db");

        for (id, title, time_created, time_updated, time_archived) in sessions {
            conn.execute(
                "INSERT INTO session (id, title, time_created, time_updated, time_archived)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, title, time_created, time_updated, time_archived],
            )
            .expect("insert session");
        }
    }

    #[test]
    fn scan_opencode_sessions_reads_sqlite_rows_and_maps_metadata() {
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-full-scan");
        fs::create_dir_all(&oc_dir).expect("create oc dir");
        create_full_opencode_db(&oc_dir);

        let db_path = oc_dir.join("opencode.db");
        let oc_conn = Connection::open(&db_path).expect("open opencode db");
        oc_conn
            .execute(
                "INSERT INTO project (id, worktree) VALUES (?1, ?2)",
                params!["project-001", "D:\\repo\\demo"],
            )
            .expect("insert project");
        oc_conn
            .execute(
                "INSERT INTO session (
                    id, title, time_created, time_updated, time_archived, project_id,
                    summary_additions, summary_deletions, summary_files
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    "oc-session-001",
                    "OpenCode Title",
                    1_710_000_000_000_i64,
                    1_710_000_300_000_i64,
                    Option::<i64>::None,
                    "project-001",
                    12_i64,
                    3_i64,
                    5_i64
                ],
            )
            .expect("insert session");

        let metadata_conn = Connection::open_in_memory().expect("open metadata db");
        init_db(&metadata_conn).expect("init metadata db");
        upsert_session_meta_internal(
            &metadata_conn,
            "oc-session-001",
            Some("同步備註".to_string()),
            vec!["research".to_string(), "multi-platform".to_string()],
        )
        .expect("insert metadata");

        let sessions =
            scan_opencode_sessions_internal(&oc_dir, false, &metadata_conn).expect("scan sessions");

        assert_eq!(sessions.len(), 1);
        let session = &sessions[0];
        assert_eq!(session.id, "oc-session-001");
        assert_eq!(session.provider, OPENCODE_PROVIDER);
        assert_eq!(session.cwd.as_deref(), Some("D:\\repo\\demo"));
        assert_eq!(session.summary.as_deref(), Some("OpenCode Title"));
        assert_eq!(session.summary_count, Some(20));
        assert_eq!(session.created_at.as_deref(), Some("2024-03-09T16:00:00Z"));
        assert_eq!(session.updated_at.as_deref(), Some("2024-03-09T16:05:00Z"));
        assert_eq!(
            session.session_dir,
            oc_dir.join("storage")
                .join("message")
                .join("oc-session-001")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(session.notes.as_deref(), Some("同步備註"));
        assert_eq!(
            session.tags,
            vec!["research".to_string(), "multi-platform".to_string()]
        );
        assert!(!session.is_archived);
        assert!(!session.parse_error);

        drop(oc_conn);
        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
    }

    #[test]
    fn scan_opencode_sessions_returns_empty_when_db_missing() {
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-missing-db");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        let metadata_conn = Connection::open_in_memory().expect("open metadata db");
        init_db(&metadata_conn).expect("init metadata db");

        let sessions =
            scan_opencode_sessions_internal(&oc_dir, false, &metadata_conn).expect("scan sessions");

        assert!(sessions.is_empty(), "missing opencode db should be ignored");

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
    }

    #[test]
    fn get_sessions_filters_by_enabled_providers() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("providers-appdata");
        let copilot_root = unique_test_dir("providers-copilot");
        let opencode_root = unique_test_dir("providers-opencode");
        let copilot_session_dir = copilot_root.join("session-state").join("cp-session-001");

        fs::create_dir_all(&copilot_session_dir).expect("create copilot session dir");
        fs::create_dir_all(&opencode_root).expect("create opencode dir");
        fs::write(
            copilot_session_dir.join("workspace.yaml"),
            concat!(
                "id: cp-session-001\n",
                "cwd: D:\\repo\\copilot\n",
                "summary: Copilot Session\n",
                "updated_at: 2025-01-02T00:00:00Z\n"
            ),
        )
        .expect("write workspace yaml");

        create_full_opencode_db(&opencode_root);
        let oc_conn = Connection::open(opencode_root.join("opencode.db")).expect("open opencode db");
        oc_conn
            .execute(
                "INSERT INTO session (
                    id, title, time_created, time_updated, time_archived, project_id,
                    summary_additions, summary_deletions, summary_files
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    "oc-session-001",
                    "OpenCode Session",
                    1_735_689_600_000_i64,
                    1_735_693_200_000_i64,
                    Option::<i64>::None,
                    Option::<String>::None,
                    1_i64,
                    1_i64,
                    1_i64
                ],
            )
            .expect("insert opencode session");

        let scan_cache = ScanCache::default();

        with_appdata(&appdata_dir, || {
            let copilot_only = get_sessions_internal(
                Some(copilot_root.to_string_lossy().to_string()),
                Some(opencode_root.to_string_lossy().to_string()),
                Some(false),
                Some(vec![COPILOT_PROVIDER.to_string()]),
                Some(true),
                &scan_cache,
            )
            .expect("scan copilot only");
            assert_eq!(copilot_only.len(), 1);
            assert_eq!(copilot_only[0].provider, COPILOT_PROVIDER);

            let opencode_only = get_sessions_internal(
                Some(copilot_root.to_string_lossy().to_string()),
                Some(opencode_root.to_string_lossy().to_string()),
                Some(false),
                Some(vec![OPENCODE_PROVIDER.to_string()]),
                Some(true),
                &scan_cache,
            )
            .expect("scan opencode only");
            assert_eq!(opencode_only.len(), 1);
            assert_eq!(opencode_only[0].provider, OPENCODE_PROVIDER);

            let all_providers = get_sessions_internal(
                Some(copilot_root.to_string_lossy().to_string()),
                Some(opencode_root.to_string_lossy().to_string()),
                Some(false),
                Some(vec![
                    COPILOT_PROVIDER.to_string(),
                    OPENCODE_PROVIDER.to_string(),
                ]),
                Some(true),
                &scan_cache,
            )
            .expect("scan all providers");
            assert_eq!(all_providers.len(), 2);
            assert!(all_providers.iter().any(|session| session.provider == COPILOT_PROVIDER));
            assert!(all_providers.iter().any(|session| session.provider == OPENCODE_PROVIDER));
        });

        drop(oc_conn);
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
        fs::remove_dir_all(&copilot_root).expect("cleanup copilot root");
        fs::remove_dir_all(&opencode_root).expect("cleanup opencode root");
    }

    #[test]
    fn scan_sisyphus_reads_project_metadata() {
        let _guard = test_lock().lock().expect("lock");
        let project_dir = unique_test_dir("sisyphus-project");
        let sisyphus_dir = project_dir.join(".sisyphus");
        fs::create_dir_all(sisyphus_dir.join("plans")).expect("create plans dir");
        fs::create_dir_all(sisyphus_dir.join("notepads").join("alpha")).expect("create alpha notepad");
        fs::create_dir_all(sisyphus_dir.join("notepads").join("beta")).expect("create beta notepad");
        fs::create_dir_all(sisyphus_dir.join("evidence")).expect("create evidence dir");
        fs::create_dir_all(sisyphus_dir.join("drafts")).expect("create drafts dir");

        fs::write(
            sisyphus_dir.join("boulder.json"),
            r#"{
                "activePlan": "plans/alpha.md",
                "planName": "Alpha Plan",
                "agent": "copilot",
                "sessionIds": ["session-001", "session-002"],
                "startedAt": "2026-04-01T09:00:00Z"
            }"#,
        )
        .expect("write boulder.json");
        fs::write(
            sisyphus_dir.join("plans").join("alpha.md"),
            "# Alpha Title\n\n## TL;DR\n第一行摘要\n第二行摘要\n\n## Details\n內容\n",
        )
        .expect("write alpha plan");
        fs::write(
            sisyphus_dir.join("plans").join("beta.md"),
            "# Beta Title\n\n一般內容\n",
        )
        .expect("write beta plan");
        fs::write(
            sisyphus_dir.join("notepads").join("alpha").join("issues.md"),
            "- issue",
        )
        .expect("write alpha issues");
        fs::write(
            sisyphus_dir.join("notepads").join("alpha").join("learnings.md"),
            "- learning",
        )
        .expect("write alpha learnings");
        fs::write(
            sisyphus_dir.join("notepads").join("beta").join("issues.md"),
            "- beta issue",
        )
        .expect("write beta issues");
        fs::write(sisyphus_dir.join("evidence").join("b.txt"), "b").expect("write evidence b");
        fs::write(sisyphus_dir.join("evidence").join("a.txt"), "a").expect("write evidence a");
        fs::write(sisyphus_dir.join("drafts").join("draft-b.md"), "# Draft B")
            .expect("write draft b");
        fs::write(sisyphus_dir.join("drafts").join("draft-a.md"), "# Draft A")
            .expect("write draft a");

        let data = scan_sisyphus_internal(&project_dir);

        assert_eq!(data.active_plan.as_ref().and_then(|plan| plan.plan_name.as_deref()), Some("Alpha Plan"));
        assert_eq!(data.active_plan.as_ref().and_then(|plan| plan.agent.as_deref()), Some("copilot"));
        assert_eq!(
            data.active_plan
                .as_ref()
                .map(|plan| plan.session_ids.clone())
                .unwrap_or_default(),
            vec!["session-001".to_string(), "session-002".to_string()]
        );
        assert_eq!(data.plans.len(), 2);
        assert_eq!(data.plans[0].name, "alpha");
        assert_eq!(data.plans[0].title.as_deref(), Some("Alpha Title"));
        assert_eq!(data.plans[0].tldr.as_deref(), Some("第一行摘要\n第二行摘要"));
        assert!(data.plans[0].is_active);
        assert_eq!(data.plans[1].name, "beta");
        assert!(!data.plans[1].is_active);
        assert_eq!(data.notepads.len(), 2);
        assert_eq!(data.notepads[0].name, "alpha");
        assert!(data.notepads[0].has_issues);
        assert!(data.notepads[0].has_learnings);
        assert_eq!(data.notepads[1].name, "beta");
        assert!(data.notepads[1].has_issues);
        assert!(!data.notepads[1].has_learnings);
        assert_eq!(data.evidence_files, vec!["a.txt".to_string(), "b.txt".to_string()]);
        assert_eq!(
            data.draft_files,
            vec!["draft-a.md".to_string(), "draft-b.md".to_string()]
        );

        fs::remove_dir_all(&project_dir).expect("cleanup project dir");
    }

    #[test]
    fn scan_openspec_reads_project_metadata() {
        let _guard = test_lock().lock().expect("lock");
        let project_dir = unique_test_dir("openspec-project");
        let openspec_dir = project_dir.join("openspec");
        fs::create_dir_all(openspec_dir.join("changes").join("feature-b").join("specs").join("auth"))
            .expect("create feature-b specs dir");
        fs::create_dir_all(openspec_dir.join("changes").join("archive").join("legacy-a"))
            .expect("create archive dir");
        fs::create_dir_all(openspec_dir.join("specs").join("api"))
            .expect("create api spec dir");
        fs::create_dir_all(openspec_dir.join("specs").join("workflow"))
            .expect("create workflow spec dir");

        fs::write(openspec_dir.join("config.yaml"), "schema: v2\n").expect("write config");
        fs::write(
            openspec_dir.join("changes").join("feature-b").join("proposal.md"),
            "# Proposal",
        )
        .expect("write proposal");
        fs::write(
            openspec_dir.join("changes").join("feature-b").join("tasks.md"),
            "- [ ] task",
        )
        .expect("write tasks");
        fs::write(
            openspec_dir
                .join("changes")
                .join("feature-b")
                .join("specs")
                .join("auth")
                .join("spec.md"),
            "# Auth Spec",
        )
        .expect("write change spec");
        fs::write(
            openspec_dir.join("changes").join("archive").join("legacy-a").join("design.md"),
            "# Design",
        )
        .expect("write archive design");
        fs::write(
            openspec_dir.join("specs").join("api").join("spec.md"),
            "# API Spec",
        )
        .expect("write api spec");

        let data = scan_openspec_internal(&project_dir);

        assert_eq!(data.schema.as_deref(), Some("v2"));
        assert_eq!(data.active_changes.len(), 1);
        assert_eq!(data.active_changes[0].name, "feature-b");
        assert!(data.active_changes[0].has_proposal);
        assert!(!data.active_changes[0].has_design);
        assert!(data.active_changes[0].has_tasks);
        assert_eq!(data.active_changes[0].specs_count, 1);
        assert_eq!(data.archived_changes.len(), 1);
        assert_eq!(data.archived_changes[0].name, "legacy-a");
        assert!(!data.archived_changes[0].has_proposal);
        assert!(data.archived_changes[0].has_design);
        assert!(!data.archived_changes[0].has_tasks);
        assert_eq!(data.specs.len(), 2);
        assert_eq!(data.specs[0].name, "api");
        assert_eq!(
            data.specs[0].path,
            openspec_dir
                .join("specs")
                .join("api")
                .join("spec.md")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(data.specs[1].name, "workflow");
        assert_eq!(
            data.specs[1].path,
            openspec_dir
                .join("specs")
                .join("workflow")
                .to_string_lossy()
                .to_string()
        );

        fs::remove_dir_all(&project_dir).expect("cleanup project dir");
    }

    #[test]
    fn scan_project_metadata_returns_empty_structures_when_dirs_missing() {
        let _guard = test_lock().lock().expect("lock");
        let project_dir = unique_test_dir("project-empty-metadata");
        fs::create_dir_all(&project_dir).expect("create project dir");

        let sisyphus_data = scan_sisyphus_internal(&project_dir);
        assert!(sisyphus_data.active_plan.is_none());
        assert!(sisyphus_data.plans.is_empty());
        assert!(sisyphus_data.notepads.is_empty());
        assert!(sisyphus_data.evidence_files.is_empty());
        assert!(sisyphus_data.draft_files.is_empty());

        let openspec_data = scan_openspec_internal(&project_dir);
        assert!(openspec_data.schema.is_none());
        assert!(openspec_data.active_changes.is_empty());
        assert!(openspec_data.archived_changes.is_empty());
        assert!(openspec_data.specs.is_empty());

        fs::remove_dir_all(&project_dir).expect("cleanup project dir");
    }

    #[test]
    fn incremental_opencode_picks_up_new_session() {
        // cursor = 0 時應掃到所有 session
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-new");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        create_opencode_db(&oc_dir, &[("oc-session-001", "OC Title", 1000, 2000, None)]);

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = empty_copilot_cache(); // cursor = 0
            scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache)
                .expect("incremental scan");

            assert_eq!(cache.sessions.len(), 1);
            assert_eq!(cache.sessions[0].id, "oc-session-001");
            assert_eq!(cache.sessions[0].provider, "opencode");
            assert_eq!(cache.sessions[0].summary.as_deref(), Some("OC Title"));
            assert_eq!(
                cache.last_cursor, 2000,
                "cursor should advance to max time_updated"
            );
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_opencode_cursor_advances_after_scan() {
        // 掃描後 cursor 應更新為最大 time_updated
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-cursor");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        create_opencode_db(
            &oc_dir,
            &[
                ("oc-a", "A", 1000, 3000, None),
                ("oc-b", "B", 1000, 5000, None),
                ("oc-c", "C", 1000, 4000, None),
            ],
        );

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = empty_copilot_cache();
            scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache)
                .expect("scan");

            assert_eq!(cache.sessions.len(), 3);
            assert_eq!(cache.last_cursor, 5000, "cursor should be max time_updated");
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_opencode_skips_sessions_before_cursor() {
        // cursor 設為 3000，time_updated <= 3000 的 session 不應被撈到
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-skip");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        create_opencode_db(
            &oc_dir,
            &[
                ("oc-old", "Old", 1000, 2000, None),
                ("oc-new", "New", 1000, 5000, None),
            ],
        );

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = ProviderCache {
                sessions: Vec::new(),
                session_mtimes: HashMap::new(),
                last_full_scan_at: Instant::now(),
                last_cursor: 3000, // 只掃 time_updated > 3000
            };
            scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache)
                .expect("scan");

            assert_eq!(
                cache.sessions.len(),
                1,
                "only new session should be picked up"
            );
            assert_eq!(cache.sessions[0].id, "oc-new");
            assert_eq!(cache.last_cursor, 5000);
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_opencode_upserts_existing_session() {
        // cursor 推進後，若同一 session time_updated 再次超過 cursor 應 upsert 而非 duplicate
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-upsert");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        create_opencode_db(&oc_dir, &[("oc-x", "Title v1", 1000, 2000, None)]);

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = empty_copilot_cache();

            // 第一次掃描
            scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache)
                .expect("first scan");
            assert_eq!(cache.sessions.len(), 1);
            assert_eq!(cache.last_cursor, 2000);

            // 手動更新 DB 模擬 session 被修改（time_updated 推進）
            let db_path = oc_dir.join("opencode.db");
            let oc_conn = Connection::open(&db_path).expect("reopen db");
            oc_conn
                .execute(
                    "UPDATE session SET title = 'Title v2', time_updated = 4000 WHERE id = 'oc-x'",
                    [],
                )
                .expect("update session");

            // 第二次增量掃描：只撈 time_updated > 2000
            scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache)
                .expect("second scan");

            assert_eq!(cache.sessions.len(), 1, "should upsert, not duplicate");
            assert_eq!(
                cache.sessions[0].summary.as_deref(),
                Some("Title v2"),
                "summary should be updated"
            );
            assert_eq!(cache.last_cursor, 4000);
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_opencode_excludes_archived_when_show_archived_false() {
        // show_archived=false 時，已封存的 session 不應出現在結果中
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-arch");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        create_opencode_db(
            &oc_dir,
            &[
                ("oc-active", "Active", 1000, 2000, None),
                ("oc-archived", "Archived", 1000, 3000, Some(9000)),
            ],
        );

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = empty_copilot_cache();
            scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache)
                .expect("scan");

            assert_eq!(cache.sessions.len(), 1);
            assert_eq!(cache.sessions[0].id, "oc-active");
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_opencode_includes_archived_when_show_archived_true() {
        // show_archived=true 時，封存 session 也應出現
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-arch-all");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        create_opencode_db(
            &oc_dir,
            &[
                ("oc-active", "Active", 1000, 2000, None),
                ("oc-archived", "Archived", 1000, 3000, Some(9000)),
            ],
        );

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = empty_copilot_cache();
            scan_opencode_incremental_internal(&oc_dir, true, &metadata_conn, &mut cache)
                .expect("scan");

            assert_eq!(cache.sessions.len(), 2);
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_opencode_noop_when_db_missing() {
        // opencode.db 不存在時應靜默回傳 Ok，不修改快取
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-no-db");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");
        // 故意不建立 opencode.db

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = empty_copilot_cache();
            let result =
                scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache);

            assert!(result.is_ok(), "should not error when db is missing");
            assert!(cache.sessions.is_empty(), "cache should remain empty");
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn incremental_opencode_cursor_unchanged_when_no_new_rows() {
        // 沒有新 row 時 cursor 不應改變
        let _guard = test_lock().lock().expect("lock");
        let oc_dir = unique_test_dir("oc-no-new");
        let appdata_dir = unique_test_dir("appdata");
        fs::create_dir_all(&oc_dir).expect("create oc dir");

        create_opencode_db(&oc_dir, &[("oc-z", "Z", 1000, 2000, None)]);

        with_appdata(&appdata_dir, || {
            let metadata_conn = open_db_connection().expect("open metadata db");
            init_db(&metadata_conn).expect("init db");

            let mut cache = ProviderCache {
                sessions: Vec::new(),
                session_mtimes: HashMap::new(),
                last_full_scan_at: Instant::now(),
                last_cursor: 9999, // cursor 已超過所有 time_updated
            };
            scan_opencode_incremental_internal(&oc_dir, false, &metadata_conn, &mut cache)
                .expect("scan");

            assert!(cache.sessions.is_empty(), "no new sessions");
            assert_eq!(cache.last_cursor, 9999, "cursor should not change");
        });

        fs::remove_dir_all(&oc_dir).expect("cleanup oc dir");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn provider_bridge_paths_use_appdata_override() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("provider-bridge-appdata");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");

        with_appdata(&appdata_dir, || {
            let copilot_path =
                resolve_provider_bridge_path(COPILOT_PROVIDER).expect("resolve copilot bridge");
            let opencode_path =
                resolve_provider_bridge_path(OPENCODE_PROVIDER).expect("resolve opencode bridge");

            assert_eq!(
                copilot_path,
                appdata_dir
                    .join("SessionHub")
                    .join("provider-bridge")
                    .join("copilot.jsonl")
            );
            assert_eq!(
                opencode_path,
                appdata_dir
                    .join("SessionHub")
                    .join("provider-bridge")
                    .join("opencode.jsonl")
            );
        });

        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn detect_copilot_integration_status_reads_installed_state_and_last_error() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("copilot-provider-appdata");
        let copilot_root = unique_test_dir("copilot-provider-root");
        let config_path = resolve_copilot_integration_path(&copilot_root);

        fs::create_dir_all(config_path.parent().expect("config parent")).expect("create hooks dir");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");

        let status = with_appdata(&appdata_dir, || {
            let bridge_path =
                resolve_provider_bridge_path(COPILOT_PROVIDER).expect("resolve copilot bridge");
            ensure_parent_dir(&bridge_path).expect("create bridge dir");
            fs::write(
                &bridge_path,
                format!(
                    "{}\n",
                    bridge_record_json(
                        COPILOT_PROVIDER,
                        "session.error",
                        "2026-04-01T12:00:00Z",
                        Some("hook failed")
                    )
                ),
            )
            .expect("write bridge record");

            let integration = serde_json::json!({
                "version": 1,
                "sessionHub": {
                    "provider": COPILOT_PROVIDER,
                    "bridgePath": bridge_path.to_string_lossy().to_string(),
                    "integrationVersion": PROVIDER_INTEGRATION_VERSION
                },
                "hooks": {
                    "sessionEnd": []
                }
            });
            fs::write(
                &config_path,
                serde_json::to_string_pretty(&integration).expect("serialize integration"),
            )
            .expect("write copilot integration");

            let root_string = copilot_root.to_string_lossy().to_string();
            detect_copilot_integration_status(Some(root_string.as_str()))
        });

        assert_eq!(status.status, ProviderIntegrationState::Installed);
        assert_eq!(
            status.last_event_at.as_deref(),
            Some("2026-04-01T12:00:00Z")
        );
        assert_eq!(status.last_error.as_deref(), Some("hook failed"));
        assert_eq!(
            status.config_path.as_deref(),
            Some(config_path.to_string_lossy().as_ref())
        );

        fs::remove_dir_all(&copilot_root).expect("cleanup copilot root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn detect_copilot_integration_status_marks_outdated_when_version_mismatches() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("copilot-provider-outdated-appdata");
        let copilot_root = unique_test_dir("copilot-provider-outdated-root");
        let config_path = resolve_copilot_integration_path(&copilot_root);

        fs::create_dir_all(config_path.parent().expect("config parent")).expect("create hooks dir");
        fs::create_dir_all(&appdata_dir).expect("create appdata dir");

        let status = with_appdata(&appdata_dir, || {
            let bridge_path =
                resolve_provider_bridge_path(COPILOT_PROVIDER).expect("resolve copilot bridge");
            let integration = serde_json::json!({
                "version": 1,
                "sessionHub": {
                    "provider": COPILOT_PROVIDER,
                    "bridgePath": bridge_path.to_string_lossy().to_string(),
                    "integrationVersion": PROVIDER_INTEGRATION_VERSION - 1
                },
                "hooks": {
                    "sessionEnd": []
                }
            });
            fs::write(
                &config_path,
                serde_json::to_string_pretty(&integration).expect("serialize integration"),
            )
            .expect("write copilot integration");

            let root_string = copilot_root.to_string_lossy().to_string();
            detect_copilot_integration_status(Some(root_string.as_str()))
        });

        assert_eq!(status.status, ProviderIntegrationState::Outdated);
        assert!(status
            .last_error
            .as_deref()
            .is_some_and(|error| error.contains("outdated")));

        fs::remove_dir_all(&copilot_root).expect("cleanup copilot root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn install_copilot_integration_writes_managed_hook_file() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("copilot-provider-install-appdata");
        let copilot_root = unique_test_dir("copilot-provider-install-root");

        fs::create_dir_all(&appdata_dir).expect("create appdata dir");
        fs::create_dir_all(&copilot_root).expect("create copilot root");

        let status = with_appdata(&appdata_dir, || {
            let root_string = copilot_root.to_string_lossy().to_string();
            install_or_update_copilot_integration(Some(root_string.as_str()))
        });
        let config_path = resolve_copilot_integration_path(&copilot_root);
        let content = fs::read_to_string(&config_path).expect("read copilot hook file");

        assert_eq!(status.status, ProviderIntegrationState::Installed);
        assert!(content.contains("\"sessionHub\""));
        assert!(content.contains("\"sessionEnd\""));
        assert!(content.contains("AppendAllText"));

        fs::remove_dir_all(&copilot_root).expect("cleanup copilot root");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn detect_opencode_integration_status_reads_installed_state_from_plugin_metadata() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("opencode-provider-appdata");
        let user_profile = unique_test_dir("opencode-provider-user");

        fs::create_dir_all(&appdata_dir).expect("create appdata dir");
        fs::create_dir_all(&user_profile).expect("create user profile dir");

        let status = with_appdata(&appdata_dir, || {
            with_env_var("USERPROFILE", &user_profile, || {
                let config_path =
                    resolve_opencode_integration_path().expect("resolve opencode integration path");
                fs::create_dir_all(config_path.parent().expect("plugin parent"))
                    .expect("create plugin dir");

                let bridge_path =
                    resolve_provider_bridge_path(OPENCODE_PROVIDER).expect("resolve bridge path");
                ensure_parent_dir(&bridge_path).expect("create bridge dir");
                fs::write(
                    &bridge_path,
                    format!(
                        "{}\n",
                        bridge_record_json(
                            OPENCODE_PROVIDER,
                            "session.updated",
                            "2026-04-02T08:30:00Z",
                            None
                        )
                    ),
                )
                .expect("write bridge record");

                let metadata = serde_json::json!({
                    "provider": OPENCODE_PROVIDER,
                    "bridgePath": bridge_path.to_string_lossy().to_string(),
                    "integrationVersion": PROVIDER_INTEGRATION_VERSION
                });
                fs::write(
                    &config_path,
                    format!(
                        "{OPENCODE_PLUGIN_METADATA_PREFIX}{}\nexport const SessionHubBridge = () => ({{}});\n",
                        serde_json::to_string(&metadata).expect("serialize metadata")
                    ),
                )
                .expect("write plugin file");

                detect_opencode_integration_status()
            })
        });

        assert_eq!(status.status, ProviderIntegrationState::Installed);
        assert_eq!(
            status.last_event_at.as_deref(),
            Some("2026-04-02T08:30:00Z")
        );
        assert!(status.last_error.is_none());
        assert!(status
            .config_path
            .as_deref()
            .is_some_and(|path| path.ends_with(OPENCODE_PLUGIN_FILE_NAME)));

        fs::remove_dir_all(&user_profile).expect("cleanup user profile");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn detect_opencode_integration_status_marks_missing_and_manual_required() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("opencode-provider-missing-appdata");
        let user_profile = unique_test_dir("opencode-provider-missing-user");

        fs::create_dir_all(&appdata_dir).expect("create appdata dir");
        fs::create_dir_all(&user_profile).expect("create user profile dir");

        let missing_status = with_appdata(&appdata_dir, || {
            with_env_var("USERPROFILE", &user_profile, || {
                let config_path =
                    resolve_opencode_integration_path().expect("resolve opencode integration path");
                fs::create_dir_all(config_path.parent().expect("plugin parent"))
                    .expect("create plugin dir");
                detect_opencode_integration_status()
            })
        });

        assert_eq!(missing_status.status, ProviderIntegrationState::Missing);
        assert!(missing_status.last_error.is_none());
        assert!(missing_status
            .config_path
            .as_deref()
            .is_some_and(|path| path.ends_with(OPENCODE_PLUGIN_FILE_NAME)));

        let manual_required_status = with_appdata(&appdata_dir, || {
            without_env_var("USERPROFILE", detect_opencode_integration_status)
        });

        assert_eq!(
            manual_required_status.status,
            ProviderIntegrationState::ManualRequired
        );
        assert!(manual_required_status.config_path.is_none());
        assert!(manual_required_status
            .last_error
            .as_deref()
            .is_some_and(|error| error.contains("USERPROFILE")));

        fs::remove_dir_all(&user_profile).expect("cleanup user profile");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }

    #[test]
    fn install_opencode_integration_writes_managed_plugin_file() {
        let _guard = test_lock().lock().expect("lock");
        let appdata_dir = unique_test_dir("opencode-provider-install-appdata");
        let user_profile = unique_test_dir("opencode-provider-install-user");

        fs::create_dir_all(&appdata_dir).expect("create appdata dir");
        fs::create_dir_all(&user_profile).expect("create user profile dir");

        let status = with_appdata(&appdata_dir, || {
            with_env_var(
                "USERPROFILE",
                &user_profile,
                install_or_update_opencode_integration,
            )
        });
        let config_path = with_env_var("USERPROFILE", &user_profile, || {
            resolve_opencode_integration_path().expect("resolve OpenCode plugin path")
        });
        let content = fs::read_to_string(&config_path).expect("read OpenCode plugin file");

        assert_eq!(status.status, ProviderIntegrationState::Installed);
        assert!(content.contains(OPENCODE_PLUGIN_METADATA_PREFIX));
        assert!(content.contains("\"session.updated\""));
        assert!(content.contains("appendFile"));

        fs::remove_dir_all(&user_profile).expect("cleanup user profile");
        fs::remove_dir_all(&appdata_dir).expect("cleanup appdata");
    }
}
