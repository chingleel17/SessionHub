use std::env;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::types::*;

pub(crate) fn default_copilot_root() -> Result<PathBuf, String> {
    let user_profile = env::var("USERPROFILE")
        .map_err(|_| "USERPROFILE environment variable is not set".to_string())?;

    Ok(PathBuf::from(user_profile).join(".copilot"))
}

pub(crate) fn default_opencode_root() -> Result<PathBuf, String> {
    let user_profile = env::var("USERPROFILE")
        .map_err(|_| "USERPROFILE environment variable is not set".to_string())?;

    Ok(PathBuf::from(user_profile)
        .join(".local")
        .join("share")
        .join("opencode"))
}

pub(crate) fn default_codex_root() -> Result<PathBuf, String> {
    let user_profile = env::var("USERPROFILE")
        .map_err(|_| "USERPROFILE environment variable is not set".to_string())?;

    Ok(PathBuf::from(user_profile).join(".codex"))
}

pub(crate) fn default_claude_root() -> Result<PathBuf, String> {
    let user_profile = env::var("USERPROFILE")
        .map_err(|_| "USERPROFILE environment variable is not set".to_string())?;

    Ok(PathBuf::from(user_profile).join(".claude"))
}

pub(crate) fn resolve_claude_root(root_dir: Option<&str>) -> Result<PathBuf, String> {
    match root_dir {
        Some(path) if !path.trim().is_empty() => Ok(PathBuf::from(path)),
        _ => default_claude_root(),
    }
}

pub(crate) fn default_antigravity_root() -> Result<PathBuf, String> {
    let user_profile = env::var("USERPROFILE")
        .map_err(|_| "USERPROFILE environment variable is not set".to_string())?;

    Ok(PathBuf::from(user_profile).join(".gemini"))
}

pub(crate) fn resolve_antigravity_root(root_dir: Option<&str>) -> Result<PathBuf, String> {
    match root_dir {
        Some(path) if !path.trim().is_empty() => Ok(PathBuf::from(path)),
        _ => default_antigravity_root(),
    }
}

pub(crate) fn default_agents_root() -> Result<PathBuf, String> {
    let user_profile = env::var("USERPROFILE")
        .map_err(|_| "USERPROFILE environment variable is not set".to_string())?;
    Ok(PathBuf::from(user_profile).join(".agents"))
}

/// 解析全域範圍 agents（skills/commands 正本）來源根目錄：使用者於設定頁自訂路徑優先，
/// 否則沿用預設 `~/.agents`。僅套用於全域範圍，專案範圍固定使用 `<project>/.agents`。
pub(crate) fn resolve_agents_source_root(configured_path: Option<&str>) -> Result<PathBuf, String> {
    match configured_path.filter(|value| !value.trim().is_empty()) {
        Some(value) => Ok(PathBuf::from(value)),
        None => default_agents_root(),
    }
}

pub(crate) fn resolve_claude_settings_path() -> Result<PathBuf, String> {
    Ok(default_claude_root()?.join(CLAUDE_HOOK_FILE_NAME))
}

pub(crate) fn default_hook_scripts_root() -> Result<PathBuf, String> {
    Ok(default_claude_root()?.join("hooks"))
}

pub(crate) fn default_codex_hook_scripts_root() -> Result<PathBuf, String> {
    Ok(default_codex_root()?.join("hooks"))
}

pub(crate) fn default_copilot_hook_scripts_root() -> Result<PathBuf, String> {
    Ok(default_copilot_root()?.join("hooks"))
}

/// 解析 Claude hook 腳本根目錄：使用者自訂路徑優先，否則一律使用 provider 原生目錄
/// （`~/.claude/hooks`）。三個 provider 統一安裝至各自原生目錄。
pub(crate) fn resolve_effective_hook_scripts_root(
    configured_path: Option<&str>,
) -> Result<PathBuf, String> {
    match configured_path.filter(|value| !value.trim().is_empty()) {
        Some(value) => Ok(PathBuf::from(value)),
        None => default_hook_scripts_root(),
    }
}

pub(crate) fn default_opencode_config_root() -> Result<PathBuf, String> {
    let user_profile = env::var("USERPROFILE")
        .map_err(|_| "USERPROFILE environment variable is not set".to_string())?;

    Ok(PathBuf::from(user_profile).join(".config").join("opencode"))
}

pub(crate) fn default_app_data_dir() -> Result<PathBuf, String> {
    if let Ok(override_dir) = env::var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE") {
        return Ok(PathBuf::from(override_dir).join("SessionHub"));
    }

    let app_data =
        env::var("APPDATA").map_err(|_| "APPDATA environment variable is not set".to_string())?;

    Ok(PathBuf::from(app_data).join("SessionHub"))
}

pub(crate) fn resolve_copilot_root(root_dir: Option<&str>) -> Result<PathBuf, String> {
    match root_dir {
        Some(path) if !path.trim().is_empty() => Ok(PathBuf::from(path)),
        _ => default_copilot_root(),
    }
}

pub(crate) fn resolve_opencode_root(root_dir: Option<&str>) -> Result<PathBuf, String> {
    match root_dir {
        Some(path) if !path.trim().is_empty() => Ok(PathBuf::from(path)),
        _ => default_opencode_root(),
    }
}

pub(crate) fn resolve_codex_root(root_dir: Option<&str>) -> Result<PathBuf, String> {
    match root_dir {
        Some(path) if !path.trim().is_empty() => Ok(PathBuf::from(path)),
        _ => default_codex_root(),
    }
}

pub(crate) fn provider_bridge_dir() -> Result<PathBuf, String> {
    Ok(default_app_data_dir()?.join("provider-bridge"))
}

pub(crate) fn resolve_provider_bridge_path(provider: &str) -> Result<PathBuf, String> {
    Ok(provider_bridge_dir()?.join(format!("{provider}.jsonl")))
}

pub(crate) fn resolve_opencode_integration_path() -> Result<PathBuf, String> {
    Ok(default_opencode_config_root()?
        .join("plugins")
        .join(OPENCODE_PLUGIN_FILE_NAME))
}

pub(crate) fn settings_file_path() -> Result<PathBuf, String> {
    Ok(default_app_data_dir()?.join("settings.json"))
}

pub(crate) fn metadata_db_path() -> Result<PathBuf, String> {
    Ok(default_app_data_dir()?.join("metadata.db"))
}

pub(crate) fn hook_logs_dir() -> Result<PathBuf, String> {
    Ok(default_app_data_dir()?.join("logs"))
}

pub(crate) fn ensure_logs_dir() {
    if let Ok(dir) = hook_logs_dir() {
        let _ = std::fs::create_dir_all(&dir);
    }
}

pub(crate) fn legacy_session_cache_path() -> Result<PathBuf, String> {
    Ok(default_app_data_dir()?.join("session_cache.json"))
}

pub(crate) fn detect_terminal_path() -> Result<Option<String>, String> {
    for terminal_name in ["pwsh", "powershell"] {
        let mut cmd = Command::new("where");
        cmd.arg(terminal_name);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let output = cmd
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

pub(crate) fn detect_vscode_path() -> Result<Option<String>, String> {
    let mut cmd = Command::new("where");
    cmd.arg("code");
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd
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

impl AppSettings {
    pub(crate) fn default() -> Result<Self, String> {
        let terminal_path = detect_terminal_path()?;
        let external_editor_path = detect_vscode_path()?;

        Ok(Self {
            copilot_root: default_copilot_root()?.to_string_lossy().to_string(),
            opencode_root: default_opencode_root()?.to_string_lossy().to_string(),
            codex_root: default_codex_root()?.to_string_lossy().to_string(),
            claude_root: default_claude_root()?.to_string_lossy().to_string(),
            antigravity_root: default_antigravity_root()?.to_string_lossy().to_string(),
            hook_scripts_path: default_hook_scripts_root()?.to_string_lossy().to_string(),
            claude_quota_reset_day: 1,
            minimize_to_tray: false,
            terminal_path,
            external_editor_path,
            show_archived: false,
            pinned_projects: Vec::new(),
            enabled_providers: default_enabled_providers(),
            provider_integrations: Vec::new(),
            default_launcher: None,
            enable_intervention_notification: true,
            enable_session_end_notification: false,
            show_status_bar: true,
            analytics_refresh_interval: 30,
            analytics_panel_collapsed: false,
            enable_quota_monitoring: true,
            quota_enabled_providers: crate::types::default_enabled_providers_all(),
            allow_create_project_config_dir: false,
            agents_source_root: String::new(),
            tray_quota_mode: crate::types::TrayQuotaMode::default(),
            tray_quota_primary_provider: None,
            tray_quota_panel_enabled: true,
            quota_overlay_enabled: false,
            quota_overlay_locked: true,
            quota_overlay_opacity: 0.85,
            quota_overlay_providers: Vec::new(),
            quota_overlay_theme: crate::types::OverlayTheme::default(),
            quota_overlay_style: crate::types::OverlayStyle::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_agents_source_root_uses_configured_path_when_present() {
        let resolved = resolve_agents_source_root(Some("D:/custom/agents")).expect("resolve");
        assert_eq!(resolved, PathBuf::from("D:/custom/agents"));
    }

    #[test]
    fn resolve_agents_source_root_falls_back_to_default_when_blank() {
        let resolved = resolve_agents_source_root(Some("   ")).expect("resolve");
        assert_eq!(resolved, default_agents_root().expect("default"));
    }

    #[test]
    fn resolve_agents_source_root_falls_back_to_default_when_none() {
        let resolved = resolve_agents_source_root(None).expect("resolve");
        assert_eq!(resolved, default_agents_root().expect("default"));
    }
}

pub(crate) fn collect_provider_integration_statuses(
    copilot_root: Option<&str>,
    codex_root: Option<&str>,
    hook_scripts_path: Option<&str>,
) -> Vec<ProviderIntegrationStatus> {
    vec![
        crate::provider::detect_copilot_integration_status(copilot_root),
        crate::provider::detect_opencode_integration_status(),
        crate::provider::detect_codex_integration_status(codex_root),
        crate::provider::detect_claude_integration_status(hook_scripts_path),
        crate::provider::detect_antigravity_integration_status(),
    ]
}

pub(crate) fn load_settings_internal() -> Result<AppSettings, String> {
    let settings_path = settings_file_path()?;

    if !settings_path.exists() {
        return AppSettings::default();
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|error| format!("failed to read settings file: {error}"))?;

    serde_json::from_str::<AppSettings>(&content)
        .map_err(|error| format!("failed to parse settings file: {error}"))
}

pub(crate) fn save_settings_internal(settings: &AppSettings) -> Result<(), String> {
    let settings_path = settings_file_path()?;
    crate::db::ensure_parent_dir(&settings_path)?;

    let content = serde_json::to_string_pretty(settings)
        .map_err(|error| format!("failed to serialize settings: {error}"))?;

    std::fs::write(&settings_path, content)
        .map_err(|error| format!("failed to write settings file: {error}"))?;

    Ok(())
}

/// 合法終端機可執行檔名稱白名單（不區分大小寫）
pub(crate) const VALID_TERMINAL_STEMS: &[&str] = &["pwsh", "powershell", "cmd", "bash", "sh"];

pub(crate) fn validate_terminal_path_internal(path: &str) -> bool {
    let candidate = PathBuf::from(path);

    if !candidate.exists() || !candidate.is_file() {
        return false;
    }

    candidate
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| {
            let stem_lower = stem.to_lowercase();
            VALID_TERMINAL_STEMS.contains(&stem_lower.as_str())
        })
        .unwrap_or(false)
}
