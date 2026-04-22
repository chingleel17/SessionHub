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
            terminal_path,
            external_editor_path,
            show_archived: false,
            pinned_projects: Vec::new(),
            enabled_providers: default_enabled_providers(),
            provider_integrations: Vec::new(),
            default_launcher: None,
            enable_intervention_notification: true,
            enable_session_end_notification: false,
        })
    }
}

pub(crate) fn collect_provider_integration_statuses(
    copilot_root: Option<&str>,
) -> Vec<ProviderIntegrationStatus> {
    vec![
        crate::provider::detect_copilot_integration_status(copilot_root),
        crate::provider::detect_opencode_integration_status(),
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
