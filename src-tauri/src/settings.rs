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

/// 在 PATH 上查找執行檔的系統指令（Windows: where；其他平台: which）。
#[cfg(target_os = "windows")]
pub(crate) const PATH_LOOKUP_COMMAND: &str = "where";
#[cfg(not(target_os = "windows"))]
pub(crate) const PATH_LOOKUP_COMMAND: &str = "which";

pub(crate) fn command_exists_on_path(command: &str) -> bool {
    let mut process = Command::new(PATH_LOOKUP_COMMAND);
    process
        .arg(command)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(target_os = "windows")]
    process.creation_flags(CREATE_NO_WINDOW);
    process
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn command_path_on_path(command: &str) -> Option<PathBuf> {
    let mut process = Command::new(PATH_LOOKUP_COMMAND);
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
            user_profile_env().ok().map(PathBuf::from),
        )
        .into_iter()
        .find(|path| path.is_file())
    })
}

pub(crate) fn default_copilot_root() -> Result<PathBuf, String> {
    let user_profile = user_profile_env()?;

    Ok(PathBuf::from(user_profile).join(".copilot"))
}

pub(crate) fn default_opencode_root() -> Result<PathBuf, String> {
    let user_profile = user_profile_env()?;

    Ok(PathBuf::from(user_profile)
        .join(".local")
        .join("share")
        .join("opencode"))
}

pub(crate) fn default_codex_root() -> Result<PathBuf, String> {
    let user_profile = user_profile_env()?;

    Ok(PathBuf::from(user_profile).join(".codex"))
}

pub(crate) fn default_claude_root() -> Result<PathBuf, String> {
    let user_profile = user_profile_env()?;

    Ok(PathBuf::from(user_profile).join(".claude"))
}

pub(crate) fn resolve_claude_root(root_dir: Option<&str>) -> Result<PathBuf, String> {
    match root_dir {
        Some(path) if !path.trim().is_empty() => Ok(PathBuf::from(path)),
        _ => default_claude_root(),
    }
}

pub(crate) fn default_antigravity_root() -> Result<PathBuf, String> {
    let user_profile = user_profile_env()?;

    Ok(PathBuf::from(user_profile).join(".gemini"))
}

pub(crate) fn resolve_antigravity_root(root_dir: Option<&str>) -> Result<PathBuf, String> {
    match root_dir {
        Some(path) if !path.trim().is_empty() => Ok(PathBuf::from(path)),
        _ => default_antigravity_root(),
    }
}

pub(crate) fn default_agents_root() -> Result<PathBuf, String> {
    let user_profile = user_profile_env()?;
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
    let user_profile = user_profile_env()?;

    Ok(PathBuf::from(user_profile).join(".config").join("opencode"))
}

/// 使用者家目錄：Windows 用 USERPROFILE；其他平台（開發用）退回 HOME。
pub(crate) fn user_profile_env() -> Result<String, String> {
    if let Ok(value) = env::var("USERPROFILE") {
        return Ok(value);
    }
    #[cfg(not(target_os = "windows"))]
    if let Ok(value) = env::var("HOME") {
        return Ok(value);
    }
    Err("USERPROFILE environment variable is not set".to_string())
}

pub(crate) fn default_app_data_dir() -> Result<PathBuf, String> {
    if let Ok(override_dir) = env::var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE") {
        return Ok(PathBuf::from(override_dir).join("SessionHub"));
    }

    if let Ok(app_data) = env::var("APPDATA") {
        return Ok(PathBuf::from(app_data).join("SessionHub"));
    }

    // 非 Windows（開發用）：退回 ~/Library/Application Support 或 XDG 路徑
    #[cfg(target_os = "macos")]
    if let Ok(home) = env::var("HOME") {
        return Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("SessionHub"));
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    if let Ok(home) = env::var("HOME") {
        return Ok(PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("SessionHub"));
    }

    Err("APPDATA environment variable is not set".to_string())
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
    // 新協定使用獨立目錄，舊版未加鎖的 hook 不得與清理程序共用檔案。
    Ok(default_app_data_dir()?
        .join("provider-bridge")
        .join("locked-v1"))
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
        let mut cmd = Command::new(PATH_LOOKUP_COMMAND);
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
        let mut cmd = Command::new(PATH_LOOKUP_COMMAND);
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

/// 目錄內是否有「SessionHub 以外」的內容。SessionHub 啟動時會在 ~/.codex、~/.copilot
/// 建立 `hooks` 目錄，單憑目錄存在無法判斷使用者是否真的安裝了該平台。
fn dir_has_user_content(path: &std::path::Path) -> bool {
    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .flatten()
                .any(|entry| entry.file_name() != std::ffi::OsStr::new("hooks"))
        })
        .unwrap_or(false)
}

/// 依本機實際存在的平台目錄，回傳 (啟用的平台, 用量監控的平台)。
/// 偵測不到任何平台時回傳全部，維持原本的預設行為。
pub(crate) fn detect_default_providers() -> (Vec<String>, Vec<String>) {
    let mut enabled = Vec::new();
    let mut quota = Vec::new();
    let mut push = |provider: &str, installed: bool, quota_ok: bool| {
        if installed {
            enabled.push(provider.to_string());
        }
        if installed || quota_ok {
            quota.push(provider.to_string());
        }
    };

    let opencode_root = default_opencode_root().ok();
    let opencode_installed = opencode_root.as_ref().is_some_and(|p| p.is_dir());
    let opencode_has_openai = opencode_root
        .as_ref()
        .is_some_and(|p| crate::quota::codex::opencode_has_openai_oauth(&p.to_string_lossy()));

    push(
        "claude",
        default_claude_root().is_ok_and(|p| p.is_dir()),
        false,
    );
    push(
        "copilot",
        default_copilot_root().is_ok_and(|p| dir_has_user_content(&p)),
        false,
    );
    push("opencode", opencode_installed, false);
    // Codex 用量可由 OpenCode 的 OpenAI 授權取得，即使未安裝 Codex CLI
    push(
        "codex",
        default_codex_root().is_ok_and(|p| dir_has_user_content(&p)),
        opencode_has_openai,
    );
    push(
        "antigravity",
        default_antigravity_root().is_ok_and(|p| p.is_dir()),
        false,
    );

    if enabled.is_empty() {
        enabled = default_enabled_providers_all();
    }
    if quota.is_empty() {
        quota = default_enabled_providers_all();
    }
    (enabled, quota)
}

impl AppSettings {
    pub(crate) fn default() -> Result<Self, String> {
        let terminal_path = detect_terminal_path()?;
        let external_editor_path = detect_vscode_path()?;

        let (enabled_providers, quota_enabled_providers) = detect_default_providers();

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
            enabled_providers,
            provider_integrations: Vec::new(),
            default_launcher: None,
            enable_intervention_notification: true,
            enable_session_end_notification: false,
            show_status_bar: true,
            analytics_refresh_interval: 30,
            analytics_panel_collapsed: false,
            enable_quota_monitoring: true,
            quota_enabled_providers,
            codex_reset_credit_reminder_enabled: false,
            codex_reset_credit_reminder_time: "09:00".to_string(),
            codex_reset_credit_reminder_days_before: 0,
            codex_reset_credit_auto_use_enabled: false,
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
            quota_overlay_intervention_enabled: true,
            quota_overlay_intervention_style: crate::types::OverlayInterventionStyle::default(),
            quota_overlay_indicator_side: crate::types::OverlayIndicatorSide::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hooks_only_directory_does_not_count_as_installed_provider() {
        let dir =
            std::env::temp_dir().join(format!("sessionhub-hooks-only-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("hooks")).unwrap();
        assert!(!dir_has_user_content(&dir));
        std::fs::write(dir.join("auth.json"), "{}").unwrap();
        assert!(dir_has_user_content(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_terminal_launcher_falls_back_to_shell() {
        assert_eq!(
            resolve_terminal_launcher(Some("unknown")),
            TERMINAL_LAUNCHER_SHELL
        );
        assert_eq!(resolve_terminal_launcher(None), TERMINAL_LAUNCHER_SHELL);
        assert_eq!(
            resolve_terminal_launcher(Some("herdr")),
            TERMINAL_LAUNCHER_HERDR
        );
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
    let mut settings = load_settings_file_internal()?;
    prune_unavailable_providers(&mut settings);
    Ok(settings)
}

/// 輕量讀取已儲存的設定；背景排程與儲存比對不需要探查平台授權或安裝目錄。
pub(crate) fn load_settings_file_internal() -> Result<AppSettings, String> {
    let settings_path = settings_file_path()?;

    if !settings_path.exists() {
        return AppSettings::default();
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|error| format!("failed to read settings file: {error}"))?;

    let mut settings = parse_settings_content(&content)?;
    settings.terminal_launcher =
        Some(resolve_terminal_launcher(settings.terminal_launcher.as_deref()).to_string());
    Ok(settings)
}

/// 平台根目錄不存在（未安裝）時，從「啟用平台」與「用量監控平台」移除，
/// 避免舊設定把未安裝的平台留在 overlay / dashboard / 背景刷新中。
/// Codex 用量可由 OpenCode 的 OpenAI 授權取得，因此該情況下保留 codex 的用量監控。
fn prune_unavailable_providers(settings: &mut AppSettings) {
    let exists = |root: &str| root.trim().is_empty() || std::path::Path::new(root.trim()).is_dir();
    let opencode_has_openai =
        crate::quota::codex::opencode_has_openai_oauth(&settings.opencode_root);
    let root_available = |provider: &str| match provider {
        "copilot" => exists(&settings.copilot_root),
        "opencode" => exists(&settings.opencode_root),
        "codex" => exists(&settings.codex_root),
        "claude" => exists(&settings.claude_root),
        "antigravity" => exists(&settings.antigravity_root),
        _ => true,
    };
    let enabled: Vec<String> = settings
        .enabled_providers
        .iter()
        .filter(|p| root_available(p))
        .cloned()
        .collect();
    let quota: Vec<String> = settings
        .quota_enabled_providers
        .iter()
        .filter(|p| root_available(p) || (p.as_str() == "codex" && opencode_has_openai))
        .cloned()
        .collect();
    settings.enabled_providers = enabled;
    settings.quota_enabled_providers = quota;
}

pub(crate) fn save_settings_internal(settings: &AppSettings) -> Result<(), String> {
    save_settings_with_reset_credit_mode(settings, true)
}

/// 背景保存其他偏好時，在同一設定閘門內沿用伺服端目前有效的自動化設定。
pub(crate) fn save_settings_with_reset_credit_mode(
    settings: &AppSettings,
    update_reset_credits: bool,
) -> Result<(), String> {
    let _gate = SETTINGS_WRITE_GATE
        .lock()
        .map_err(|_| "無法鎖定設定".to_string())?;
    let mut settings = settings.clone();
    let previous = load_settings_file_internal()?;
    if !update_reset_credits {
        let saved = &previous;
        settings.codex_reset_credit_reminder_enabled = saved.codex_reset_credit_reminder_enabled;
        settings.codex_reset_credit_reminder_time = saved.codex_reset_credit_reminder_time.clone();
        settings.codex_reset_credit_reminder_days_before =
            saved.codex_reset_credit_reminder_days_before;
        settings.codex_reset_credit_auto_use_enabled = saved.codex_reset_credit_auto_use_enabled;
    }
    validate_reset_credit_settings(&settings)?;
    let settings_path = settings_file_path()?;
    crate::db::ensure_parent_dir(&settings_path)?;

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|error| format!("failed to serialize settings: {error}"))?;

    let temporary_path = settings_path.with_extension(format!("json.{}.tmp", std::process::id()));
    std::fs::write(&temporary_path, content)
        .map_err(|error| format!("failed to write settings file: {error}"))?;
    if let Err(error) = std::fs::rename(&temporary_path, &settings_path) {
        let _ = std::fs::remove_file(&temporary_path);
        return Err(format!("failed to replace settings file: {error}"));
    }
    if reset_credit_settings_changed(&previous, &settings) {
        SETTINGS_GENERATION.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    Ok(())
}

pub(crate) fn reset_credit_settings_changed(previous: &AppSettings, next: &AppSettings) -> bool {
    previous.codex_root != next.codex_root
        || previous.opencode_root != next.opencode_root
        || previous.codex_reset_credit_reminder_enabled != next.codex_reset_credit_reminder_enabled
        || previous.codex_reset_credit_reminder_time != next.codex_reset_credit_reminder_time
        || previous.codex_reset_credit_reminder_days_before
            != next.codex_reset_credit_reminder_days_before
        || previous.codex_reset_credit_auto_use_enabled != next.codex_reset_credit_auto_use_enabled
}

/// 與背景工作核發副作用執行權共用，鎖定期間不得執行網路呼叫。
pub(crate) static SETTINGS_WRITE_GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());
pub(crate) static SETTINGS_GENERATION: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

pub(crate) fn valid_reminder_time(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 5
        && bytes[2] == b':'
        && [bytes[0], bytes[1], bytes[3], bytes[4]]
            .iter()
            .all(u8::is_ascii_digit)
        && &value[..2] <= "23"
        && &value[3..] <= "59"
}

pub(crate) fn validate_reset_credit_settings(settings: &AppSettings) -> Result<(), String> {
    if !valid_reminder_time(&settings.codex_reset_credit_reminder_time) {
        return Err("提醒時間必須為 00:00–23:59 的 HH:mm 格式".to_string());
    }
    if settings.codex_reset_credit_reminder_days_before > 30 {
        return Err("提前提醒天數必須為 0–30 的整數".to_string());
    }
    Ok(())
}

fn parse_settings_content(content: &str) -> Result<AppSettings, String> {
    let mut value: serde_json::Value = serde_json::from_str(content)
        .map_err(|error| format!("failed to parse settings file: {error}"))?;
    if let Some(object) = value.as_object_mut() {
        if !object
            .get("codexResetCreditReminderTime")
            .and_then(|v| v.as_str())
            .is_some_and(valid_reminder_time)
        {
            object.insert("codexResetCreditReminderTime".into(), "09:00".into());
        }
        if !object
            .get("codexResetCreditReminderDaysBefore")
            .and_then(|v| v.as_u64())
            .is_some_and(|n| n <= 30)
        {
            object.insert("codexResetCreditReminderDaysBefore".into(), 0.into());
        }
        for key in [
            "codexResetCreditReminderEnabled",
            "codexResetCreditAutoUseEnabled",
        ] {
            if !object.get(key).is_some_and(serde_json::Value::is_boolean) {
                object.insert(key.into(), false.into());
            }
        }
    }
    serde_json::from_value(value).map_err(|error| format!("failed to parse settings file: {error}"))
}

#[cfg(test)]
mod reset_credit_settings_tests {
    use super::*;

    #[test]
    fn reset_credit_load_normalizes_only_invalid_fields() {
        let base = serde_json::json!({"copilotRoot":"", "showArchived":true,
            "codexResetCreditReminderEnabled":true, "codexResetCreditAutoUseEnabled":true});
        for bad in [
            serde_json::json!(-1),
            serde_json::json!(1.5),
            serde_json::json!(31),
            serde_json::json!("3"),
        ] {
            let mut value = base.clone();
            value["codexResetCreditReminderDaysBefore"] = bad;
            value["codexResetCreditReminderTime"] = "25:00".into();
            let settings = parse_settings_content(&value.to_string()).expect("load");
            assert_eq!(settings.codex_reset_credit_reminder_days_before, 0);
            assert_eq!(settings.codex_reset_credit_reminder_time, "09:00");
            assert!(
                settings.show_archived
                    && settings.codex_reset_credit_auto_use_enabled
                    && settings.codex_reset_credit_reminder_enabled
            );
        }
        let old =
            parse_settings_content(r#"{"copilotRoot":"", "showArchived":false}"#).expect("old");
        assert!(
            !old.codex_reset_credit_auto_use_enabled && !old.codex_reset_credit_reminder_enabled
        );
        for bad in ["9:00", "24:00", "23:60", "ab:cd", "０９:00"] {
            assert!(!valid_reminder_time(bad));
        }
        assert!(valid_reminder_time("00:00") && valid_reminder_time("23:59"));
    }

    #[test]
    fn reset_credit_invalid_save_does_not_touch_existing_settings() {
        let _guard = crate::shared_env_test_lock().lock().expect("env lock");
        let root =
            std::env::temp_dir().join(format!("sessionhub-reset-settings-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("mkdir");
        let previous = std::env::var_os("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE");
        std::env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", &root);
        let mut settings =
            parse_settings_content(r#"{"copilotRoot":"", "showArchived":true}"#).expect("parse");
        save_settings_internal(&settings).expect("save");
        let before = std::fs::read(settings_file_path().expect("path")).expect("before");
        settings.codex_reset_credit_auto_use_enabled = true;
        save_settings_internal(&settings).expect("explicit enable");
        let mut stale_background = settings.clone();
        stale_background.codex_reset_credit_auto_use_enabled = false;
        stale_background.codex_reset_credit_reminder_time = "25:00".into();
        stale_background.show_archived = false;
        save_settings_with_reset_credit_mode(&stale_background, false)
            .expect("background preserves current automation");
        let saved = load_settings_internal().expect("saved");
        assert!(saved.codex_reset_credit_auto_use_enabled);
        assert!(!saved.show_archived);
        settings.codex_reset_credit_auto_use_enabled = false;
        save_settings_internal(&settings).expect("restore baseline");
        settings.codex_reset_credit_reminder_time = "9:00".into();
        assert!(save_settings_internal(&settings).is_err());
        assert_eq!(
            std::fs::read(settings_file_path().expect("path")).expect("after"),
            before
        );
        match previous {
            Some(v) => std::env::set_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE", v),
            None => std::env::remove_var("COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE"),
        }
        let _ = std::fs::remove_dir_all(root);
    }
}

/// 合法終端機可執行檔名稱白名單（不區分大小寫）
pub(crate) const VALID_TERMINAL_STEMS: &[&str] = &["pwsh", "powershell", "cmd", "bash", "sh"];

pub(crate) fn validate_terminal_path_internal(path: &str, launcher: Option<&str>) -> bool {
    let candidate = PathBuf::from(path);

    if resolve_terminal_launcher(launcher) == TERMINAL_LAUNCHER_HERDR {
        return resolve_herdr_executable().is_some()
            && (candidate.is_file()
                || (!candidate.components().any(|component| {
                    matches!(
                        component,
                        std::path::Component::RootDir | std::path::Component::Prefix(_)
                    )
                }) && command_exists_on_path(path)));
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
