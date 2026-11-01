use crate::types::{AppSettings, QuotaSnapshot};

pub mod antigravity;
pub mod cache;
pub mod claude;
pub mod codex;
pub mod copilot;
pub(crate) mod http;
pub mod opencode;
pub(crate) mod reset_credit_automation;
pub(crate) mod reset_credit_policy;

pub(crate) trait QuotaAdapter: Send + Sync {
    fn provider_key(&self) -> &str;
    fn fetch_snapshot(&self, settings: &AppSettings) -> QuotaSnapshot;
}

/// 網路查詢在資料庫鎖之外執行；所有 Codex 提交共用設定版本核對。
pub(crate) fn refresh_cached_quota(
    db: &crate::DbState,
    cache: &crate::types::QuotaCache,
    settings: &AppSettings,
    provider: Option<&str>,
) -> Vec<QuotaSnapshot> {
    let manager = QuotaManager::new();
    refresh_cached_quota_using(
        db,
        cache,
        settings,
        || match provider {
            Some(key) => manager.refresh_one(key, settings).into_iter().collect(),
            None => manager.refresh_all(settings),
        },
        crate::settings::load_settings_internal,
        |current| {
            codex::resolve_credentials(current)
                .ok()
                .map(|c| c.revision(current))
        },
    )
}

fn refresh_cached_quota_using(
    db: &crate::DbState,
    cache: &crate::types::QuotaCache,
    settings: &AppSettings,
    fetch: impl FnOnce() -> Vec<QuotaSnapshot>,
    latest: impl FnOnce() -> Result<AppSettings, String>,
    revision: impl FnOnce(&AppSettings) -> Option<String>,
) -> Vec<QuotaSnapshot> {
    let snapshots = fetch();
    let Ok(_gate) = crate::settings::SETTINGS_WRITE_GATE.lock() else {
        return Vec::new();
    };
    let Ok(current) = latest() else {
        return Vec::new();
    };
    if serde_json::to_vec(settings).ok() != serde_json::to_vec(&current).ok() {
        return Vec::new();
    }
    let current_revision = revision(&current);
    let Ok(conn) = db.conn.lock() else {
        return Vec::new();
    };
    let mut committed = Vec::new();
    for snapshot in snapshots {
        if snapshot.provider == "codex" && snapshot.account_revision != current_revision {
            continue;
        }
        if snapshot.status != "rate_limited" {
            let _ = cache::write_snapshot_to_cache_and_db(&conn, cache, &snapshot);
        }
        committed.push(snapshot);
    }
    committed
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn reset_credit_refresh_does_not_hold_db_during_network_and_drops_old_account() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::init_db(&conn).unwrap();
        let db = crate::DbState {
            conn: Mutex::new(conn),
        };
        let cache = crate::types::QuotaCache::default();
        let settings: AppSettings =
            serde_json::from_str(r#"{"copilotRoot":"","showArchived":false}"#).unwrap();
        let old = QuotaSnapshot {
            provider: "codex".into(),
            status: "ok".into(),
            account_scope: Some("a".into()),
            account_revision: Some("old".into()),
            ..Default::default()
        };
        let current = QuotaSnapshot {
            provider: "codex".into(),
            status: "ok".into(),
            account_scope: Some("b".into()),
            account_revision: Some("new".into()),
            ..Default::default()
        };
        cache
            .snapshots
            .lock()
            .unwrap()
            .insert("codex".into(), current.clone());
        let result = refresh_cached_quota_using(
            &db,
            &cache,
            &settings,
            || {
                let conn = db
                    .conn
                    .try_lock()
                    .expect("network work must not hold DB mutex");
                journal_probe(&conn);
                vec![old]
            },
            || Ok(settings.clone()),
            |_| Some("new".into()),
        );
        assert!(result.is_empty());
        assert_eq!(
            cache.snapshots.lock().unwrap()["codex"]
                .account_scope
                .as_deref(),
            Some("b")
        );
        cache::write_snapshot_to_cache_and_db(&db.conn.lock().unwrap(), &cache, &current).unwrap();
        let reloaded = crate::types::QuotaCache::default();
        cache::load_cache_from_db(&db.conn.lock().unwrap(), &reloaded, &["codex".into()]).unwrap();
        assert!(reloaded.snapshots.lock().unwrap()["codex"]
            .account_scope
            .is_none());
        crate::db::reset_credits::begin_attempt(
            &db.conn.lock().unwrap(),
            "a",
            "id",
            None,
            &crate::types::ResetCredits {
                available_count: 0,
                credits: vec![],
            },
            1,
        )
        .unwrap();
        cache::prune_disabled_provider_quota(&db.conn.lock().unwrap(), &cache, &[]).unwrap();
        assert!(
            crate::db::reset_credits::attempt(&db.conn.lock().unwrap(), "id")
                .unwrap()
                .is_some()
        );
    }

    fn journal_probe(conn: &rusqlite::Connection) {
        crate::db::reset_credits::defer(conn, "other-account", 1)
            .expect("journal writable during slow provider");
    }
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
