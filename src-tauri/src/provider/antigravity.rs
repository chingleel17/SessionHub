use std::path::PathBuf;

use crate::antigravity_hooks::{
    read_antigravity_hooks, write_antigravity_hooks, AntigravityHookConfig, AntigravityHookEntry,
    AntigravityHookGroup, AntigravityHookMatcher, AntigravityHookScope,
};
use crate::types::*;

use super::bridge::read_bridge_diagnostics;
use super::build_provider_integration_status;

/// SessionHub 在 Antigravity hooks.json 中佔用的群組名稱，作為「已安裝」的識別標記。
/// 目前僅登記 marker（供偵測 session 掃描觸發），不驅動即時事件管線
/// （Antigravity 掃描為 metadata-only，無 bridge watcher）。
const ANTIGRAVITY_HOOK_GROUP_NAME: &str = "sessionhub-provider-event-bridge";
const ANTIGRAVITY_MANAGED_EVENT: &str = "Stop";
const ANTIGRAVITY_MARKER_COMMAND: &str = "echo sessionhub-provider-event-bridge";

pub(crate) fn resolve_antigravity_integration_path() -> Result<PathBuf, String> {
    crate::antigravity_hooks::resolve_hooks_path(&AntigravityHookScope::Global)
}

fn managed_matcher() -> AntigravityHookMatcher {
    AntigravityHookMatcher {
        matcher: ".*".to_string(),
        hooks: vec![AntigravityHookEntry {
            hook_type: "command".to_string(),
            command: ANTIGRAVITY_MARKER_COMMAND.to_string(),
            timeout: Some(5),
        }],
    }
}

fn is_managed_group(group: &AntigravityHookGroup) -> bool {
    group
        .events
        .get(ANTIGRAVITY_MANAGED_EVENT)
        .is_some_and(|matchers| {
            matchers.iter().any(|m| {
                m.hooks
                    .iter()
                    .any(|h| h.command.contains("sessionhub-provider-event-bridge"))
            })
        })
}

fn has_managed_group(config: &AntigravityHookConfig) -> bool {
    config
        .get(ANTIGRAVITY_HOOK_GROUP_NAME)
        .is_some_and(is_managed_group)
}

pub(crate) fn install_or_update_antigravity_integration() -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(ANTIGRAVITY_PROVIDER);
    let scope = AntigravityHookScope::Global;
    let config_path = match resolve_antigravity_integration_path() {
        Ok(path) => path,
        Err(error) => {
            return build_provider_integration_status(
                ANTIGRAVITY_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                None,
                Some(error),
            );
        }
    };

    let mut config = match read_antigravity_hooks(&scope) {
        Ok(config) => config,
        Err(error) => {
            return build_provider_integration_status(
                ANTIGRAVITY_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path),
                diagnostics,
                None,
                Some(error),
            );
        }
    };

    let group = config
        .entry(ANTIGRAVITY_HOOK_GROUP_NAME.to_string())
        .or_default();
    group.events.insert(
        ANTIGRAVITY_MANAGED_EVENT.to_string(),
        vec![managed_matcher()],
    );

    if let Err(error) = write_antigravity_hooks(&scope, &config) {
        return build_provider_integration_status(
            ANTIGRAVITY_PROVIDER,
            ProviderIntegrationState::Error,
            Some(config_path),
            diagnostics,
            None,
            Some(error),
        );
    }

    detect_antigravity_integration_status()
}

pub(crate) fn detect_antigravity_integration_status() -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(ANTIGRAVITY_PROVIDER);
    let scope = AntigravityHookScope::Global;
    let config_path = match resolve_antigravity_integration_path() {
        Ok(path) => path,
        Err(error) => {
            return build_provider_integration_status(
                ANTIGRAVITY_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                None,
                Some(error),
            );
        }
    };

    if !config_path.exists() {
        return build_provider_integration_status(
            ANTIGRAVITY_PROVIDER,
            ProviderIntegrationState::Missing,
            Some(config_path),
            diagnostics,
            None,
            None,
        );
    }

    let config = match read_antigravity_hooks(&scope) {
        Ok(config) => config,
        Err(error) => {
            return build_provider_integration_status(
                ANTIGRAVITY_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path),
                diagnostics,
                None,
                Some(error),
            );
        }
    };

    if !has_managed_group(&config) {
        return build_provider_integration_status(
            ANTIGRAVITY_PROVIDER,
            ProviderIntegrationState::Missing,
            Some(config_path),
            diagnostics,
            None,
            None,
        );
    }

    build_provider_integration_status(
        ANTIGRAVITY_PROVIDER,
        ProviderIntegrationState::Installed,
        Some(config_path),
        diagnostics,
        Some(PROVIDER_INTEGRATION_VERSION),
        None,
    )
}

pub(crate) fn uninstall_antigravity_integration() -> ProviderIntegrationStatus {
    let diagnostics = read_bridge_diagnostics(ANTIGRAVITY_PROVIDER);
    let scope = AntigravityHookScope::Global;
    let config_path = match resolve_antigravity_integration_path() {
        Ok(path) => path,
        Err(error) => {
            return build_provider_integration_status(
                ANTIGRAVITY_PROVIDER,
                ProviderIntegrationState::ManualRequired,
                None,
                diagnostics,
                None,
                Some(error),
            );
        }
    };

    if !config_path.exists() {
        return build_provider_integration_status(
            ANTIGRAVITY_PROVIDER,
            ProviderIntegrationState::Missing,
            Some(config_path),
            diagnostics,
            None,
            None,
        );
    }

    let mut config = match read_antigravity_hooks(&scope) {
        Ok(config) => config,
        Err(error) => {
            return build_provider_integration_status(
                ANTIGRAVITY_PROVIDER,
                ProviderIntegrationState::Error,
                Some(config_path),
                diagnostics,
                None,
                Some(error),
            );
        }
    };

    config.remove(ANTIGRAVITY_HOOK_GROUP_NAME);

    if let Err(error) = write_antigravity_hooks(&scope, &config) {
        return build_provider_integration_status(
            ANTIGRAVITY_PROVIDER,
            ProviderIntegrationState::Error,
            Some(config_path),
            diagnostics,
            None,
            Some(error),
        );
    }

    build_provider_integration_status(
        ANTIGRAVITY_PROVIDER,
        ProviderIntegrationState::Missing,
        Some(config_path),
        diagnostics,
        None,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_group_detection_roundtrip() {
        let mut config = AntigravityHookConfig::new();
        assert!(!has_managed_group(&config));

        let group = config
            .entry(ANTIGRAVITY_HOOK_GROUP_NAME.to_string())
            .or_default();
        group.events.insert(
            ANTIGRAVITY_MANAGED_EVENT.to_string(),
            vec![managed_matcher()],
        );

        assert!(has_managed_group(&config));
    }
}

#[cfg(test)]
mod manual_smoke_tests {
    use super::*;

    #[test]
    #[ignore = "writes to real ~/.gemini/config/hooks.json; run manually with --ignored"]
    fn manual_smoke_install_detect_uninstall_real_global_hooks() {
        let path = resolve_antigravity_integration_path().expect("resolve path");
        println!("global hooks path: {}", path.display());

        // 起始狀態不假設（可能已由使用者透過 UI 安裝），先強制清乾淨再驗證完整循環
        let _ = uninstall_antigravity_integration();
        let before =
            crate::antigravity_hooks::read_antigravity_hooks(&AntigravityHookScope::Global)
                .expect("read before");
        assert!(
            !before.contains_key(ANTIGRAVITY_HOOK_GROUP_NAME),
            "marker group should not exist before install"
        );

        let installed = install_or_update_antigravity_integration();
        println!("install status: {:?}", installed.status);
        assert_eq!(installed.status, ProviderIntegrationState::Installed);

        let detected = detect_antigravity_integration_status();
        println!("detect status: {:?}", detected.status);
        assert_eq!(detected.status, ProviderIntegrationState::Installed);

        let raw = std::fs::read_to_string(&path).expect("read raw hooks.json");
        println!("raw hooks.json:\n{raw}");
        assert!(raw.contains(ANTIGRAVITY_HOOK_GROUP_NAME));

        let uninstalled = uninstall_antigravity_integration();
        println!("uninstall status: {:?}", uninstalled.status);
        assert_eq!(uninstalled.status, ProviderIntegrationState::Missing);

        let after = crate::antigravity_hooks::read_antigravity_hooks(&AntigravityHookScope::Global)
            .expect("read after uninstall");
        assert!(
            !after.contains_key(ANTIGRAVITY_HOOK_GROUP_NAME),
            "marker group should be removed after uninstall"
        );
    }
}
