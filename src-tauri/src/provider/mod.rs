use std::fs;
use std::path::{Path, PathBuf};

use crate::db::ensure_parent_dir;
use crate::types::*;

pub mod bridge;
pub mod claude;
pub mod codex;
pub mod copilot;
pub mod opencode;

pub(crate) use bridge::*;
pub(crate) use claude::*;
pub(crate) use codex::*;
pub(crate) use copilot::*;
pub(crate) use opencode::*;

// ── 共用 Integration 輔助函式（供 copilot.rs 與 opencode.rs 使用）──────────

pub(super) fn validate_integration_target(config_path: &Path) -> Result<(), String> {
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

pub(super) fn validate_managed_metadata(
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

pub(crate) fn build_provider_integration_status(
    provider: &str,
    status: ProviderIntegrationState,
    config_path: Option<PathBuf>,
    diagnostics: ProviderBridgeDiagnostics,
    installed_version: Option<u32>,
    last_error: Option<String>,
) -> ProviderIntegrationStatus {
    ProviderIntegrationStatus {
        provider: provider.to_string(),
        status,
        config_path: config_path.map(|path| path.to_string_lossy().to_string()),
        bridge_path: diagnostics
            .bridge_path
            .map(|path| path.to_string_lossy().to_string()),
        installed_version,
        last_event_at: diagnostics.last_event_at,
        last_error: last_error.or(diagnostics.last_error),
    }
}

pub(super) fn managed_provider_metadata(
    provider: &str,
    bridge_path: &Path,
) -> ManagedProviderIntegrationMetadata {
    ManagedProviderIntegrationMetadata {
        provider: provider.to_string(),
        bridge_path: bridge_path.to_string_lossy().to_string(),
        integration_version: PROVIDER_INTEGRATION_VERSION,
    }
}

pub(super) fn write_provider_integration_file(
    config_path: &Path,
    content: &str,
) -> Result<(), String> {
    validate_integration_target(config_path)?;
    ensure_parent_dir(config_path)?;
    fs::write(config_path, content).map_err(|error| {
        format!(
            "failed to write integration file {}: {error}",
            config_path.display()
        )
    })
}

pub(super) fn install_hook_scripts(
    provider_name: &str,
    root: &Path,
    entries: &[(&str, &str)],
    version: &str,
) -> Result<PathBuf, String> {
    for (relative_path, content) in entries {
        let path = root.join(relative_path);
        ensure_parent_dir(&path)?;
        fs::write(&path, content).map_err(|error| {
            format!(
                "failed to write {provider_name} hook script {}: {error}",
                path.display()
            )
        })?;
    }

    let version_path = root.join(".version");
    fs::write(&version_path, version).map_err(|error| {
        format!(
            "failed to write {provider_name} hook version marker {}: {error}",
            version_path.display()
        )
    })?;

    Ok(root.to_path_buf())
}

/// 安裝通知所需的 binary 資源（snoretoast.exe）至 hook 落地目錄的 `_bin/` 子目錄。
/// 僅在 Windows 平台有效，其他平台直接略過（不報錯）。
pub(super) fn install_notification_binary(root: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let snoretoast_bytes: &[u8] = include_bytes!("../../../hooks/_bin/snoretoast.exe");
        let bin_dir = root.join("_bin");
        ensure_parent_dir(&bin_dir.join("placeholder"))?;
        let dest = bin_dir.join("snoretoast.exe");
        fs::write(&dest, snoretoast_bytes).map_err(|error| {
            format!(
                "failed to write snoretoast.exe to {}: {error}",
                dest.display()
            )
        })?;
    }
    let _ = root; // 非 Windows 靜默略過
    Ok(())
}

/// 移除通知 binary（`_bin/snoretoast.exe`）。目錄若清空一併移除。
pub(super) fn uninstall_notification_binary(root: &Path) {
    let dest = root.join("_bin").join("snoretoast.exe");
    if dest.exists() {
        let _ = fs::remove_file(&dest);
    }
    let bin_dir = root.join("_bin");
    remove_dir_if_empty(&bin_dir);
}

/// 移除由 SessionHub 安裝的 hook 腳本檔案。僅刪除 `entries` 列出的檔案與 `.version`
/// 標記，不刪除使用者自訂的其他檔案；清理後若 `modules/` 子目錄或 root 變空才一併移除。
pub(super) fn uninstall_hook_scripts(root: &Path, entries: &[(&str, &str)]) {
    if !root.exists() {
        return;
    }

    let mut subdirs: Vec<PathBuf> = Vec::new();
    for (relative_path, _) in entries {
        let path = root.join(relative_path);
        if path.exists() {
            if let Err(e) = fs::remove_file(&path) {
                eprintln!("[uninstall] failed to remove hook script {}: {e}", path.display());
            }
        }
        if let Some(parent) = path.parent() {
            if parent != root && !subdirs.contains(&parent.to_path_buf()) {
                subdirs.push(parent.to_path_buf());
            }
        }
    }

    let version_path = root.join(".version");
    if version_path.exists() {
        let _ = fs::remove_file(&version_path);
    }

    // 僅在子目錄已空時移除（保留使用者自訂內容）
    for dir in subdirs {
        remove_dir_if_empty(&dir);
    }
    remove_dir_if_empty(root);
}

/// 僅當目錄為空時移除，否則保留（內含使用者自訂檔案）
fn remove_dir_if_empty(dir: &Path) {
    let is_empty = fs::read_dir(dir)
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(false);
    if is_empty {
        let _ = fs::remove_dir(dir);
    }
}

pub(super) fn build_install_failure_status(
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

    build_provider_integration_status(
        provider,
        status,
        config_path,
        diagnostics,
        None,
        Some(error),
    )
}

// ── 公開聚合函式 ─────────────────────────────────────────────────────────────

pub(crate) fn recheck_provider_integration_status(
    provider: &str,
    copilot_root: Option<&str>,
    codex_root: Option<&str>,
    hook_scripts_path: Option<&str>,
) -> Result<ProviderIntegrationStatus, String> {
    match provider {
        COPILOT_PROVIDER => Ok(detect_copilot_integration_status(copilot_root)),
        OPENCODE_PROVIDER => Ok(detect_opencode_integration_status()),
        CODEX_PROVIDER => Ok(detect_codex_integration_status(codex_root)),
        CLAUDE_PROVIDER => Ok(detect_claude_integration_status(hook_scripts_path)),
        _ => Err(format!("unsupported provider: {provider}")),
    }
}

pub(crate) fn install_or_update_provider_integration(
    provider: &str,
    copilot_root: Option<&str>,
    codex_root: Option<&str>,
    hook_scripts_path: Option<&str>,
) -> Result<ProviderIntegrationStatus, String> {
    match provider {
        COPILOT_PROVIDER => Ok(install_or_update_copilot_integration(copilot_root)),
        OPENCODE_PROVIDER => Ok(install_or_update_opencode_integration()),
        CODEX_PROVIDER => Ok(install_or_update_codex_integration(codex_root)),
        CLAUDE_PROVIDER => Ok(install_or_update_claude_integration(hook_scripts_path)),
        _ => Err(format!("unsupported provider: {provider}")),
    }
}

pub(crate) fn uninstall_provider_integration(
    provider: &str,
    copilot_root: Option<&str>,
    codex_root: Option<&str>,
    hook_scripts_path: Option<&str>,
) -> Result<ProviderIntegrationStatus, String> {
    match provider {
        COPILOT_PROVIDER => Ok(uninstall_copilot_integration(copilot_root)),
        OPENCODE_PROVIDER => Ok(uninstall_opencode_integration()),
        CODEX_PROVIDER => Ok(uninstall_codex_integration(codex_root)),
        CLAUDE_PROVIDER => Ok(uninstall_claude_integration(hook_scripts_path)),
        _ => Err(format!("unsupported provider: {provider}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTRIES: [(&str, &str); 2] = [
        ("modules/record-event.psm1", "managed-module"),
        ("on-session-start.ps1", "managed-script"),
    ];

    fn temp_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("session-hub-uninstall-{name}"))
    }

    #[test]
    fn uninstall_removes_managed_files_but_keeps_user_files() {
        let root = temp_root("keeps-user");
        let _ = fs::remove_dir_all(&root);
        install_hook_scripts("Test", &root, &ENTRIES, "2").unwrap();

        // 使用者自訂檔案：root 下與 modules/ 子目錄各一
        let user_root_file = root.join("my-custom-hook.ps1");
        fs::write(&user_root_file, "user").unwrap();
        let user_module_file = root.join("modules").join("user-helper.psm1");
        fs::write(&user_module_file, "user").unwrap();

        uninstall_hook_scripts(&root, &ENTRIES);

        // 受管檔案與 .version 應被移除
        assert!(!root.join("on-session-start.ps1").exists());
        assert!(!root.join("modules/record-event.psm1").exists());
        assert!(!root.join(".version").exists());
        // 使用者自訂檔案必須保留
        assert!(user_root_file.exists(), "使用者 root 檔案不應被刪除");
        assert!(user_module_file.exists(), "使用者 modules 檔案不應被刪除");
        // root 與 modules/ 因仍有使用者檔案而保留
        assert!(root.exists());
        assert!(root.join("modules").exists());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn uninstall_removes_empty_dirs_when_no_user_files() {
        let root = temp_root("empty-dirs");
        let _ = fs::remove_dir_all(&root);
        install_hook_scripts("Test", &root, &ENTRIES, "2").unwrap();

        uninstall_hook_scripts(&root, &ENTRIES);

        // 沒有使用者檔案時，modules/ 與 root 皆應被移除
        assert!(!root.exists(), "空目錄應被清除");

        let _ = fs::remove_dir_all(&root);
    }
}
