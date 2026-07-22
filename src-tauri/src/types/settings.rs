use serde::{Deserialize, Serialize};

use crate::types::{default_false, default_true, ProviderIntegrationStatus};
use crate::types::{
    ANTIGRAVITY_PROVIDER, CLAUDE_PROVIDER, CODEX_PROVIDER, COPILOT_PROVIDER, OPENCODE_PROVIDER,
};

pub(crate) fn default_enabled_providers() -> Vec<String> {
    vec![
        "copilot".to_string(),
        "opencode".to_string(),
        "codex".to_string(),
        ANTIGRAVITY_PROVIDER.to_string(),
    ]
}

pub(crate) fn default_enabled_providers_all() -> Vec<String> {
    vec![
        CLAUDE_PROVIDER.to_string(),
        COPILOT_PROVIDER.to_string(),
        OPENCODE_PROVIDER.to_string(),
        CODEX_PROVIDER.to_string(),
        ANTIGRAVITY_PROVIDER.to_string(),
    ]
}

pub(crate) fn default_claude_quota_reset_day() -> u8 {
    1
}

pub(crate) fn default_hook_scripts_path() -> String {
    String::new()
}

pub(crate) fn default_minimize_to_tray() -> bool {
    false
}

pub(crate) fn default_notification_enabled() -> bool {
    true
}

pub(crate) fn default_quota_overlay_opacity() -> f64 {
    0.3
}

pub(crate) fn default_quota_overlay_theme() -> OverlayTheme {
    OverlayTheme::Dark
}

/// 系統匣圖示的 quota 顯示模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TrayQuotaMode {
    #[default]
    IconOnly,
    Percentage,
    Bar,
    Hidden,
}

/// 桌面 quota overlay 的視覺主題
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OverlayTheme {
    #[default]
    Dark,
    Light,
}

/// 桌面 quota overlay 的版型：完整（進度條列表）或精簡（圓環一列）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OverlayStyle {
    Full,
    #[default]
    Compact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSettings {
    pub(crate) copilot_root: String,
    #[serde(default)]
    pub(crate) opencode_root: String,
    #[serde(default)]
    pub(crate) codex_root: String,
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
    #[serde(default)]
    pub(crate) claude_root: String,
    #[serde(default)]
    pub(crate) antigravity_root: String,
    #[serde(default = "default_hook_scripts_path")]
    pub(crate) hook_scripts_path: String,
    #[serde(default = "default_claude_quota_reset_day")]
    pub(crate) claude_quota_reset_day: u8,
    #[serde(default = "default_minimize_to_tray")]
    pub(crate) minimize_to_tray: bool,
    #[serde(default = "default_true")]
    pub(crate) enable_quota_monitoring: bool,
    #[serde(default = "default_enabled_providers_all")]
    pub(crate) quota_enabled_providers: Vec<String>,
    #[serde(default = "default_false")]
    pub(crate) allow_create_project_config_dir: bool,
    #[serde(default)]
    pub(crate) agents_source_root: String,
    // ── Tray / Overlay quota widget 設定 ──
    #[serde(default)]
    pub(crate) tray_quota_mode: TrayQuotaMode,
    #[serde(default)]
    pub(crate) tray_quota_primary_provider: Option<String>,
    #[serde(default = "default_true")]
    pub(crate) tray_quota_panel_enabled: bool,
    #[serde(default = "default_false")]
    pub(crate) quota_overlay_enabled: bool,
    #[serde(default = "default_true")]
    pub(crate) quota_overlay_locked: bool,
    #[serde(default = "default_quota_overlay_opacity")]
    pub(crate) quota_overlay_opacity: f64,
    #[serde(default)]
    pub(crate) quota_overlay_providers: Vec<String>,
    #[serde(default = "default_quota_overlay_theme")]
    pub(crate) quota_overlay_theme: OverlayTheme,
    #[serde(default)]
    pub(crate) quota_overlay_style: OverlayStyle,
}

pub(crate) fn default_analytics_refresh_interval() -> u32 {
    30
}
