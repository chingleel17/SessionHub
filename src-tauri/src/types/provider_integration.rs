use serde::{Deserialize, Serialize};

pub(crate) const PROVIDER_INTEGRATION_VERSION: u32 = 6;
pub(crate) const COPILOT_PROVIDER: &str = "copilot";
pub(crate) const OPENCODE_PROVIDER: &str = "opencode";
pub(crate) const CODEX_PROVIDER: &str = "codex";
pub(crate) const CLAUDE_PROVIDER: &str = "claude";
pub(crate) const ANTIGRAVITY_PROVIDER: &str = "antigravity";
pub(crate) const AGENTS_PROVIDER: &str = "agents";
pub(crate) const COPILOT_HOOK_FILE_NAME: &str = "sessionhub-provider-event-bridge.json";
pub(crate) const CODEX_HOOK_FILE_NAME: &str = "hooks.json";
pub(crate) const OPENCODE_PLUGIN_FILE_NAME: &str = "sessionhub-provider-event-bridge.ts";
pub(crate) const OPENCODE_PLUGIN_METADATA_PREFIX: &str = "// sessionhub-provider-event-bridge:";
pub(crate) const CLAUDE_HOOK_FILE_NAME: &str = "settings.json";

pub(crate) fn default_provider_bridge_version() -> u32 {
    PROVIDER_INTEGRATION_VERSION
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
