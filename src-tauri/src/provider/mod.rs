use std::fs;
use std::path::{Path, PathBuf};

use crate::db::ensure_parent_dir;
use crate::types::*;

pub mod bridge;
pub mod copilot;
pub mod opencode;

pub(crate) use bridge::*;
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

pub(super) fn write_provider_integration_file(config_path: &Path, content: &str) -> Result<(), String> {
    validate_integration_target(config_path)?;
    ensure_parent_dir(config_path)?;
    fs::write(config_path, content).map_err(|error| {
        format!(
            "failed to write integration file {}: {error}",
            config_path.display()
        )
    })
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

    build_provider_integration_status(provider, status, config_path, diagnostics, None, Some(error))
}

// ── 公開聚合函式 ─────────────────────────────────────────────────────────────

pub(crate) fn recheck_provider_integration_status(
    provider: &str,
    copilot_root: Option<&str>,
) -> Result<ProviderIntegrationStatus, String> {
    match provider {
        COPILOT_PROVIDER => Ok(detect_copilot_integration_status(copilot_root)),
        OPENCODE_PROVIDER => Ok(detect_opencode_integration_status()),
        _ => Err(format!("unsupported provider: {provider}")),
    }
}

pub(crate) fn install_or_update_provider_integration(
    provider: &str,
    copilot_root: Option<&str>,
) -> Result<ProviderIntegrationStatus, String> {
    match provider {
        COPILOT_PROVIDER => Ok(install_or_update_copilot_integration(copilot_root)),
        OPENCODE_PROVIDER => Ok(install_or_update_opencode_integration()),
        _ => Err(format!("unsupported provider: {provider}")),
    }
}
