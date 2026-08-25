use std::env;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::types::*;

pub(crate) const TERMINAL_LAUNCHER_SHELL: &str = "shell";
pub(crate) const TERMINAL_LAUNCHER_HERDR: &str = "herdr";

pub(crate) fn resolve_terminal_launcher(value: Option<&str>) -> &'static str {
    match value {
        Some(value) if value.eq_ignore_ascii_case(TERMINAL_LAUNCHER_HERDR) => {
            TERMINAL_LAUNCHER_HERDR
        }
        _ => TERMINAL_LAUNCHER_SHELL,
    }
}

pub(crate) fn command_exists_on_path(command: &str) -> bool {
    let mut process = Command::new("where");
    process
        .arg(command)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(target_os = "windows")]
    process.creation_flags(CREATE_NO_WINDOW);
    process.status().map(|status| status.success()).unwrap_or(false)
}

fn command_path_on_path(command: &str) -> Option<PathBuf> {
    let mut process = Command::new("where");
    process.arg(command);
    #[cfg(target_os = "windows")]
    process.creation_flags(CREATE_NO_WINDOW);

    let output = process.output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

fn herdr_standard_install_paths(
    local_app_data: Option<PathBuf>,
    user_profile: Option<PathBuf>,
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(root) = local_app_data {
        roots.push(root);
    }
    if let Some(profile) = user_profile {
        let root = profile.join("AppData").join("Local");
        if !roots.contains(&root) {
            roots.push(root);
        }
    }

    roots
        .into_iter()
        .map(|root| {
            root.join("Programs")
                .join("Herdr")
                .join("bin")
                .join("herdr.exe")
        })
        .collect()
}

/// 解析 Herdr 執行檔。GUI 程序可能繼承到安裝前的舊 PATH，因此補查官方安裝器位置。
pub(crate) fn resolve_herdr_executable() -> Option<PathBuf> {
    command_path_on_path("herdr").or_else(|| {
        herdr_standard_install_paths(
            env::var_os("LOCALAPPDATA").map(PathBuf::from),
            env::var_os("USERPROFILE").map(PathBuf::from),
        )
        .into_iter()
        .find(|path| path.is_file())
    })
}

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

/// 偵測 PATH 上的 VS Code 指令。
///
/// 依序嘗試正式版與 Insiders 版，讓只安裝 Insiders 的環境也能偵測到。
pub(crate) fn detect_vscode_path() -> Result<Option<String>, String> {
    for candidate in ["code", "code-insiders"] {
        let mut cmd = Command::new("where");
        cmd.arg(candidate);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let output = cmd
            .output()
            .map_err(|error| format!("failed to execute where command: {error}"))?;

        if output.status.success() {
            let found = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty());

            if found.is_some() {
                return Ok(found);
            }
        }
    }

    Ok(None)
}

/// 解析實際要使用的 VS Code 執行檔。
///
/// 優先採用設定中的 `external_editor_path`（使用者可能指定 VS Code Insiders 等變體），
/// 該路徑存在時直接回傳；否則退回 PATH 上的 `code`。
pub(crate) fn resolve_vscode_command() -> Option<String> {
    let configured = load_settings_internal()
        .ok()
        .and_then(|settings| settings.external_editor_path)
        .map(|path| path.trim().to_string())
        .filter(|path| !path.is_empty());

    if let Some(path) = configured {
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }

    detect_vscode_path().ok().flatten()
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
            launch_on_startup: false,
            start_minimized_on_startup: true,
            terminal_path,
            terminal_launcher: Some(TERMINAL_LAUNCHER_SHELL.to_string()),
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
            quota_overlay_opacity: crate::types::default_quota_overlay_opacity(),
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
    fn unknown_terminal_launcher_falls_back_to_shell() {
        assert_eq!(resolve_terminal_launcher(Some("unknown")), TERMINAL_LAUNCHER_SHELL);
        assert_eq!(resolve_terminal_launcher(None), TERMINAL_LAUNCHER_SHELL);
        assert_eq!(resolve_terminal_launcher(Some("herdr")), TERMINAL_LAUNCHER_HERDR);
    }

    #[test]
    fn missing_terminal_launcher_is_compatible_with_old_settings() {
        let settings = AppSettings::default().expect("default settings");
        let mut value = serde_json::to_value(settings).expect("serialize settings");
        value
            .as_object_mut()
            .expect("settings object")
            .remove("terminalLauncher");
        let parsed = serde_json::from_value::<AppSettings>(value).expect("parse old settings");
        assert_eq!(
            resolve_terminal_launcher(parsed.terminal_launcher.as_deref()),
            TERMINAL_LAUNCHER_SHELL
        );
    }

    #[test]
    fn herdr_standard_install_paths_supports_msi_environment_without_local_app_data() {
        let paths = herdr_standard_install_paths(None, Some(PathBuf::from(r"C:\Users\Test")));

        assert_eq!(
            paths,
            vec![PathBuf::from(
                r"C:\Users\Test\AppData\Local\Programs\Herdr\bin\herdr.exe"
            )]
        );
    }

    #[test]
    fn herdr_standard_install_paths_deduplicates_matching_roots() {
        let paths = herdr_standard_install_paths(
            Some(PathBuf::from(r"C:\Users\Test\AppData\Local")),
            Some(PathBuf::from(r"C:\Users\Test")),
        );

        assert_eq!(paths.len(), 1);
    }

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

    let mut settings = serde_json::from_str::<AppSettings>(&content)
        .map_err(|error| format!("failed to parse settings file: {error}"))?;
    settings.terminal_launcher = Some(resolve_terminal_launcher(
        settings.terminal_launcher.as_deref(),
    )
    .to_string());
    Ok(settings)
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

pub(crate) fn validate_terminal_path_internal(path: &str, launcher: Option<&str>) -> bool {
    let candidate = PathBuf::from(path);

    if resolve_terminal_launcher(launcher) == TERMINAL_LAUNCHER_HERDR {
        return resolve_herdr_executable().is_some()
            && (candidate.is_file()
                || (!candidate.components().any(|component| {
                    matches!(component, std::path::Component::RootDir | std::path::Component::Prefix(_))
                })
                    && command_exists_on_path(path)));
    }

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
