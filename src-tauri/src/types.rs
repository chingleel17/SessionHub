use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use notify::RecommendedWatcher;
use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
pub(crate) const CREATE_NEW_CONSOLE: u32 = 0x00000010;

#[cfg(target_os = "windows")]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Deserialize)]
pub(crate) struct WorkspaceYaml {
    pub(crate) id: String,
    pub(crate) cwd: Option<String>,
    pub(crate) summary: Option<String>,
    pub(crate) summary_count: Option<u32>,
    pub(crate) created_at: Option<String>,
    pub(crate) updated_at: Option<String>,
}

pub(crate) fn default_provider() -> String {
    "copilot".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionInfo {
    pub(crate) id: String,
    #[serde(default = "default_provider")]
    pub(crate) provider: String,
    pub(crate) cwd: Option<String>,
    pub(crate) summary: Option<String>,
    pub(crate) summary_count: Option<u32>,
    pub(crate) created_at: Option<String>,
    pub(crate) updated_at: Option<String>,
    pub(crate) session_dir: String,
    pub(crate) parse_error: bool,
    pub(crate) is_archived: bool,
    pub(crate) notes: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) has_plan: bool,
    pub(crate) has_events: bool,
}

pub(crate) fn default_enabled_providers() -> Vec<String> {
    vec!["copilot".to_string(), "opencode".to_string()]
}

pub(crate) fn default_notification_enabled() -> bool {
    true
}

pub(crate) fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionActivityStatus {
    pub(crate) session_id: String,
    /// "idle" | "active" | "waiting" | "done"
    pub(crate) status: String,
    /// "thinking" | "tool_call" | "file_op" | "sub_agent" | "working" | "completed"
    pub(crate) detail: Option<String>,
    pub(crate) last_activity_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionTargetedPayload {
    pub(crate) session_id: String,
    pub(crate) cwd: String,
    pub(crate) event_type: String,
}

/// tool.pre / tool.post / prompt.submitted 等純活動提示事件的輕量 payload，
/// 前端可直接用 cwd 比對更新 activity status，不需要任何 IPC 或檔案掃描。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActivityHintPayload {
    pub(crate) cwd: String,
    pub(crate) event_type: String,
    pub(crate) title: Option<String>,
    pub(crate) error: Option<String>,
}

/// 每次 provider bridge 收到事件時發送給前端的 debug log 記錄。
/// status 可能值："targeted" | "fallback" | "full_refresh" | "skipped_dedup" | "skipped_rate_limit" | "activity_hint"
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BridgeEventLogEntry {
    pub(crate) id: String,
    pub(crate) provider: String,
    pub(crate) event_type: String,
    pub(crate) timestamp: String,
    pub(crate) cwd: Option<String>,
    pub(crate) session_id: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) error: Option<String>,
    /// "targeted" | "fallback" | "full_refresh" | "skipped_dedup" | "skipped_rate_limit"
    pub(crate) status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSettings {
    pub(crate) copilot_root: String,
    #[serde(default)]
    pub(crate) opencode_root: String,
    pub(crate) terminal_path: Option<String>,
    pub(crate) external_editor_path: Option<String>,
    pub(crate) show_archived: bool,
    #[serde(default)]
    pub(crate) pinned_projects: Vec<String>,
    #[serde(default = "default_enabled_providers")]
    pub(crate) enabled_providers: Vec<String>,
    #[serde(default)]
    pub(crate) provider_integrations: Vec<ProviderIntegrationStatus>,
    #[serde(default)]
    pub(crate) default_launcher: Option<String>,
    #[serde(default = "default_notification_enabled")]
    pub(crate) enable_intervention_notification: bool,
    #[serde(default)]
    pub(crate) enable_session_end_notification: bool,
    #[serde(default = "default_true")]
    pub(crate) show_status_bar: bool,
    #[serde(default = "default_analytics_refresh_interval")]
    pub(crate) analytics_refresh_interval: u32,
    #[serde(default)]
    pub(crate) analytics_panel_collapsed: bool,
}

pub(crate) const PROVIDER_INTEGRATION_VERSION: u32 = 3;
pub(crate) const COPILOT_PROVIDER: &str = "copilot";
pub(crate) const OPENCODE_PROVIDER: &str = "opencode";
pub(crate) const COPILOT_HOOK_FILE_NAME: &str = "sessionhub-provider-event-bridge.json";
pub(crate) const OPENCODE_PLUGIN_FILE_NAME: &str = "sessionhub-provider-event-bridge.ts";
pub(crate) const OPENCODE_PLUGIN_METADATA_PREFIX: &str = "// sessionhub-provider-event-bridge:";

pub(crate) fn default_provider_bridge_version() -> u32 {
    PROVIDER_INTEGRATION_VERSION
}

pub(crate) fn default_analytics_refresh_interval() -> u32 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProviderIntegrationState {
    Installed,
    Outdated,
    Missing,
    ManualRequired,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderBridgeRecord {
    #[serde(default = "default_provider_bridge_version")]
    pub(crate) version: u32,
    pub(crate) provider: String,
    pub(crate) event_type: String,
    pub(crate) timestamp: String,
    #[serde(default)]
    pub(crate) session_id: Option<String>,
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(default)]
    pub(crate) source_path: Option<String>,
    #[serde(default)]
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderIntegrationStatus {
    pub(crate) provider: String,
    pub(crate) status: ProviderIntegrationState,
    pub(crate) config_path: Option<String>,
    pub(crate) bridge_path: Option<String>,
    /// 目前安裝的 integration 版本號（None 表示未安裝或無法讀取）
    pub(crate) installed_version: Option<u32>,
    pub(crate) last_event_at: Option<String>,
    pub(crate) last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManagedProviderIntegrationMetadata {
    pub(crate) provider: String,
    pub(crate) bridge_path: String,
    pub(crate) integration_version: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CopilotIntegrationConfig {
    #[serde(default)]
    pub(crate) session_hub: Option<ManagedProviderIntegrationMetadata>,
}

#[derive(Debug, Default)]
pub(crate) struct ProviderBridgeDiagnostics {
    pub(crate) bridge_path: Option<std::path::PathBuf>,
    pub(crate) last_event_at: Option<String>,
    pub(crate) last_error: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CopilotWatchSnapshot {
    pub(crate) active_session_count: usize,
    pub(crate) archived_session_count: usize,
    pub(crate) active_workspace_mtime_ms: u128,
    pub(crate) archived_workspace_mtime_ms: u128,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct OpenCodeWatchSnapshot {
    pub(crate) db_exists: bool,
    pub(crate) wal_exists: bool,
    pub(crate) db_mtime_ms: u128,
    pub(crate) wal_mtime_ms: u128,
    pub(crate) max_cursor: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionMeta {
    pub(crate) notes: Option<String>,
    pub(crate) tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModelMetricsEntry {
    pub(crate) requests_count: f64,
    pub(crate) requests_cost: f64,
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionStats {
    pub(crate) output_tokens: u64,
    pub(crate) input_tokens: u64,
    pub(crate) interaction_count: u32,
    pub(crate) tool_call_count: u32,
    pub(crate) duration_minutes: u64,
    pub(crate) models_used: Vec<String>,
    pub(crate) reasoning_count: u32,
    pub(crate) tool_breakdown: BTreeMap<String, u32>,
    pub(crate) model_metrics: BTreeMap<String, ModelMetricsEntry>,
    pub(crate) is_live: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnalyticsDataPoint {
    pub(crate) label: String,
    pub(crate) output_tokens: u64,
    pub(crate) input_tokens: u64,
    pub(crate) interaction_count: u32,
    pub(crate) cost_points: f64,
    pub(crate) session_count: u32,
    pub(crate) missing_count: u32,
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
            model_metrics: BTreeMap::new(),
            is_live: false,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionStartData {
    #[serde(default)]
    pub(crate) start_time: Option<String>,
    #[serde(default)]
    pub(crate) selected_model: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionModelChangeData {
    #[serde(default)]
    pub(crate) new_model: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TopLevelFilterData {
    #[serde(default)]
    pub(crate) parent_tool_call_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ToolExecutionStartData {
    #[serde(default)]
    pub(crate) parent_tool_call_id: Option<String>,
    #[serde(default)]
    pub(crate) tool_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssistantMessageData {
    #[serde(default)]
    pub(crate) parent_tool_call_id: Option<String>,
    #[serde(default)]
    pub(crate) output_tokens: Option<u64>,
    #[serde(default)]
    pub(crate) reasoning_opaque: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionShutdownRequestData {
    #[serde(default)]
    pub(crate) count: Option<f64>,
    #[serde(default)]
    pub(crate) cost: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionShutdownUsageData {
    #[serde(default)]
    pub(crate) input_tokens: Option<u64>,
    #[serde(default)]
    pub(crate) output_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionShutdownModelMetric {
    #[serde(default)]
    pub(crate) requests: Option<SessionShutdownRequestData>,
    #[serde(default)]
    pub(crate) usage: Option<SessionShutdownUsageData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionShutdownData {
    #[serde(default)]
    pub(crate) model_metrics: BTreeMap<String, SessionShutdownModelMetric>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SessionEvent {
    #[serde(rename = "type")]
    pub(crate) event_type: String,
    #[serde(default)]
    pub(crate) timestamp: Option<String>,
    #[serde(default)]
    pub(crate) data: serde_json::Value,
}

#[derive(Default)]
pub(crate) struct WatcherState {
    pub(crate) sessions: Mutex<Option<RecommendedWatcher>>,
    pub(crate) plan: Mutex<Option<RecommendedWatcher>>,
    pub(crate) project: Mutex<Option<RecommendedWatcher>>,
    pub(crate) opencode: Mutex<Option<RecommendedWatcher>>,
    pub(crate) provider_bridge: Mutex<Option<RecommendedWatcher>>,
    pub(crate) last_provider_refresh: Arc<Mutex<HashMap<String, Instant>>>,
    pub(crate) last_bridge_records: Arc<Mutex<HashMap<String, String>>>,
}

/// Copilot watcher 防抖時間（毫秒）
pub(crate) const COPILOT_DEBOUNCE_MS: u64 = 800;
/// OpenCode WAL watcher 防抖時間（毫秒）
pub(crate) const OPENCODE_DEBOUNCE_MS: u64 = 500;
/// 專案 plans/specs watcher 防抖時間（毫秒）
pub(crate) const PROJECT_FILES_DEBOUNCE_MS: u64 = 400;
/// Provider bridge watcher 防抖時間（毫秒）
pub(crate) const PROVIDER_BRIDGE_DEBOUNCE_MS: u64 = 250;
/// 短時間內同 provider refresh 去重視窗（毫秒）
pub(crate) const PROVIDER_REFRESH_DEDUP_MS: u64 = 1_500;
/// 觸發全掃描的閾值（秒），超過此值自動執行全掃
pub(crate) const FULL_SCAN_THRESHOLD_SECS: u64 = 1800;

/// 單一 provider 的記憶體快取
pub(crate) struct ProviderCache {
    /// 上次掃描的結果
    pub(crate) sessions: Vec<SessionInfo>,
    /// Copilot 專用：session_id → 目錄最後修改時間（Unix 秒）
    pub(crate) session_mtimes: HashMap<String, i64>,
    /// 上次全掃描的時間點
    pub(crate) last_full_scan_at: Instant,
    /// OpenCode 專用：上次全掃描時見到的最大 time_updated 值
    pub(crate) last_cursor: i64,
}

/// 兩個 provider 各自持有的掃描快取
#[derive(Default)]
pub(crate) struct ScanCache {
    pub(crate) copilot: Mutex<Option<ProviderCache>>,
    pub(crate) opencode: Mutex<Option<ProviderCache>>,
}

// ── OpenCode JSON 儲存格式（session/*.json / project/*.json）────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct OpencodeProjectJson {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) worktree: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpencodeSessionJsonTime {
    #[serde(default)]
    pub(crate) created: Option<i64>,
    #[serde(default)]
    pub(crate) updated: Option<i64>,
    #[serde(default)]
    pub(crate) archived: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OpencodeSessionJsonSummary {
    #[serde(default)]
    pub(crate) additions: Option<i64>,
    #[serde(default)]
    pub(crate) deletions: Option<i64>,
    #[serde(default)]
    pub(crate) files: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpencodeSessionJson {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) directory: Option<String>,
    #[serde(default)]
    pub(crate) time: Option<OpencodeSessionJsonTime>,
    #[serde(default)]
    pub(crate) summary: Option<OpencodeSessionJsonSummary>,
}

// ── OpenCode Stats 解析相關型別 ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct OpencodeTokens {
    #[serde(default)]
    pub(crate) input: Option<u64>,
    #[serde(default)]
    pub(crate) output: Option<u64>,
    #[serde(default)]
    pub(crate) reasoning: Option<u64>,
    #[serde(default, rename = "inputTokens")]
    pub(crate) input_tokens: Option<u64>,
    #[serde(default, rename = "outputTokens")]
    pub(crate) output_tokens: Option<u64>,
}

impl OpencodeTokens {
    pub(crate) fn effective_input(&self) -> u64 {
        self.input.or(self.input_tokens).unwrap_or(0)
    }
    pub(crate) fn effective_output(&self) -> u64 {
        self.output.or(self.output_tokens).unwrap_or(0)
    }
    pub(crate) fn effective_reasoning(&self) -> u64 {
        self.reasoning.unwrap_or(0)
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct OpencodeMessageTime {
    #[serde(default)]
    pub(crate) created: Option<i64>,
    #[serde(default)]
    pub(crate) completed: Option<i64>,
}

/// metadata.assistant 子物件（modelID、tokens）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpencodeAssistantMeta {
    #[serde(default, alias = "modelID")]
    pub(crate) model_id: Option<String>,
    #[serde(default)]
    pub(crate) tokens: Option<OpencodeTokens>,
}

/// metadata 子物件
#[derive(Debug, Deserialize)]
pub(crate) struct OpencodeMessageMetadata {
    #[serde(default)]
    pub(crate) time: Option<OpencodeMessageTime>,
    #[serde(default)]
    pub(crate) assistant: Option<OpencodeAssistantMeta>,
    /// 有些版本的 token 統計直接放在 metadata.usage
    #[serde(default)]
    pub(crate) usage: Option<OpencodeTokens>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpencodeMessage {
    pub(crate) id: String,
    #[serde(default, alias = "sessionID")]
    pub(crate) session_id: Option<String>,
    #[serde(default)]
    pub(crate) role: String,
    #[serde(default)]
    pub(crate) metadata: Option<OpencodeMessageMetadata>,
}

impl OpencodeMessage {
    pub(crate) fn time(&self) -> Option<&OpencodeMessageTime> {
        self.metadata.as_ref()?.time.as_ref()
    }
    pub(crate) fn model_id(&self) -> Option<&str> {
        self.metadata
            .as_ref()?
            .assistant
            .as_ref()?
            .model_id
            .as_deref()
            .filter(|s| !s.is_empty())
    }
    pub(crate) fn tokens(&self) -> Option<&OpencodeTokens> {
        self.metadata.as_ref().and_then(|m| {
            m.assistant
                .as_ref()
                .and_then(|a| a.tokens.as_ref())
                .or(m.usage.as_ref())
        })
    }
}

// ── Sisyphus 型別 ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SisyphusBoulder {
    pub(crate) active_plan: Option<String>,
    pub(crate) plan_name: Option<String>,
    pub(crate) agent: Option<String>,
    pub(crate) session_ids: Vec<String>,
    pub(crate) started_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SisyphusPlan {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) title: Option<String>,
    pub(crate) tldr: Option<String>,
    pub(crate) is_active: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SisyphusNotepad {
    pub(crate) name: String,
    pub(crate) has_issues: bool,
    pub(crate) has_learnings: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SisyphusData {
    pub(crate) active_plan: Option<SisyphusBoulder>,
    pub(crate) plans: Vec<SisyphusPlan>,
    pub(crate) notepads: Vec<SisyphusNotepad>,
    pub(crate) evidence_files: Vec<String>,
    pub(crate) draft_files: Vec<String>,
}

// ── OpenSpec 型別 ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenSpecChange {
    pub(crate) name: String,
    pub(crate) has_proposal: bool,
    pub(crate) has_design: bool,
    pub(crate) has_tasks: bool,
    pub(crate) specs_count: usize,
    pub(crate) specs: Vec<OpenSpecSpec>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenSpecSpec {
    pub(crate) name: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenSpecData {
    pub(crate) schema: Option<String>,
    pub(crate) active_changes: Vec<OpenSpecChange>,
    pub(crate) archived_changes: Vec<OpenSpecChange>,
    pub(crate) specs: Vec<OpenSpecSpec>,
}

// ── Activity 相關型別 ────────────────────────────────────────────────────────

/// OpenCode message 檔案結構（只解析需要的欄位）
#[derive(Debug, Deserialize)]
pub(crate) struct OpenCodeMessageFile {
    pub(crate) role: Option<String>,
    pub(crate) finish: Option<String>,
    pub(crate) time: Option<OpenCodeMessageTime2>,
}

/// Activity 模組專用的 time 型別（與 stats 的 OpencodeMessageTime 不同）
#[derive(Debug, Deserialize)]
pub(crate) struct OpenCodeMessageTime2 {
    pub(crate) created: Option<i64>,
    pub(crate) completed: Option<i64>,
}

// ── Tool Availability ────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ToolAvailability {
    pub copilot: bool,
    pub opencode: bool,
    pub gemini: bool,
    pub vscode: bool,
}
