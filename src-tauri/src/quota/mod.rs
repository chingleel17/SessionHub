use crate::types::{AppSettings, QuotaSnapshot};

pub mod antigravity;
pub mod cache;
pub mod claude;
pub mod codex;
pub mod copilot;
pub mod opencode;

pub(crate) trait QuotaAdapter: Send + Sync {
    fn provider_key(&self) -> &str;
    fn fetch_snapshot(&self, settings: &AppSettings) -> QuotaSnapshot;
}

pub(crate) struct QuotaManager {
    adapters: Vec<Box<dyn QuotaAdapter>>,
}

impl QuotaManager {
    pub(crate) fn new() -> Self {
        QuotaManager {
            adapters: vec![
                Box::new(claude::ClaudeAdapter),
                Box::new(copilot::CopilotAdapter),
                Box::new(opencode::OpenCodeAdapter),
                Box::new(codex::CodexAdapter),
                Box::new(antigravity::AntigravityAdapter),
            ],
        }
    }

    pub(crate) fn refresh_all(&self, settings: &AppSettings) -> Vec<QuotaSnapshot> {
        self.adapters
            .iter()
            .filter(|a| {
                settings
                    .quota_enabled_providers
                    .contains(&a.provider_key().to_string())
            })
            .map(|a| a.fetch_snapshot(settings))
            .collect()
    }

    pub(crate) fn refresh_one(
        &self,
        provider_key: &str,
        settings: &AppSettings,
    ) -> Option<QuotaSnapshot> {
        self.adapters
            .iter()
            .find(|a| {
                a.provider_key() == provider_key
                    && settings
                        .quota_enabled_providers
                        .contains(&a.provider_key().to_string())
            })
            .map(|a| a.fetch_snapshot(settings))
    }
}
