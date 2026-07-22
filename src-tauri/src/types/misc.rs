use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use notify::RecommendedWatcher;

use crate::types::{ProviderIntegrationStatus, SessionActivityStatus, SessionInfo};

pub(crate) struct WatcherState {
    pub(crate) sessions: Mutex<Option<RecommendedWatcher>>,
    pub(crate) plan: Mutex<Option<RecommendedWatcher>>,
    pub(crate) project: Mutex<Option<RecommendedWatcher>>,
    pub(crate) opencode: Mutex<Option<RecommendedWatcher>>,
    pub(crate) codex: Mutex<Option<RecommendedWatcher>>,
    pub(crate) claude: Mutex<Option<RecommendedWatcher>>,
    pub(crate) provider_bridge: Mutex<Option<RecommendedWatcher>>,
    pub(crate) last_provider_refresh: Arc<Mutex<HashMap<String, Instant>>>,
    pub(crate) last_bridge_records: Arc<Mutex<HashMap<String, String>>>,
    /// 最後一次 get_settings 取得的 integration 狀態，供 restart_session_watcher 使用，避免重讀磁碟
    pub(crate) known_integrations: Mutex<Vec<ProviderIntegrationStatus>>,
    /// per-provider quota refresh 的最後觸發時間（用於 bridge 事件後 30 秒 debounce）
    pub(crate) last_quota_refresh_trigger: Arc<Mutex<HashMap<String, Instant>>>,
}

impl Default for WatcherState {
    fn default() -> Self {
        WatcherState {
            sessions: Mutex::new(None),
            plan: Mutex::new(None),
            project: Mutex::new(None),
            opencode: Mutex::new(None),
            codex: Mutex::new(None),
            claude: Mutex::new(None),
            provider_bridge: Mutex::new(None),
            last_provider_refresh: Arc::new(Mutex::new(HashMap::new())),
            last_bridge_records: Arc::new(Mutex::new(HashMap::new())),
            known_integrations: Mutex::new(Vec::new()),
            last_quota_refresh_trigger: Arc::new(Mutex::new(HashMap::new())),
        }
    }
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
pub(crate) struct ScanCache {
    pub(crate) copilot: Mutex<Option<ProviderCache>>,
    pub(crate) opencode: Mutex<Option<ProviderCache>>,
    pub(crate) codex: Mutex<Option<ProviderCache>>,
    pub(crate) claude: Mutex<Option<ProviderCache>>,
    pub(crate) antigravity: Mutex<Option<ProviderCache>>,
    // session_id → (events_mtime_secs, SessionActivityStatus)
    pub(crate) activity: Mutex<HashMap<String, (i64, SessionActivityStatus)>>,
    // 防止同時進行多個掃描的全局互斥體
    pub(crate) scan_lock: Mutex<()>,
}

impl Default for ScanCache {
    fn default() -> Self {
        ScanCache {
            copilot: Mutex::new(None),
            opencode: Mutex::new(None),
            codex: Mutex::new(None),
            claude: Mutex::new(None),
            antigravity: Mutex::new(None),
            activity: Mutex::new(HashMap::new()),
            scan_lock: Mutex::new(()),
        }
    }
}

// ── Tool Availability ────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ToolAvailability {
    pub copilot: bool,
    pub opencode: bool,
    pub claude: bool,
    pub codex: bool,
    pub gemini: bool,
    pub vscode: bool,
}
