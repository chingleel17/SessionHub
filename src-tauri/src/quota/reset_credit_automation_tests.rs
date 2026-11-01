use super::*;
use crate::types::ResetCreditEntry;
use chrono::TimeZone;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicUsize, Ordering};

struct FakeHost {
    now: AtomicI64,
    settings: Mutex<AppSettings>,
    scope: Mutex<String>,
    credits: Mutex<ResetCredits>,
    outcomes: Mutex<VecDeque<Result<String, CreditError>>>,
    query_errors: Mutex<VecDeque<CreditError>>,
    queries: AtomicUsize,
    posts: AtomicUsize,
    notices: Mutex<Vec<String>>,
    deny_notification: AtomicBool,
    switch_after_query: AtomicBool,
    disable_after_query: AtomicBool,
    toggle_after_query: AtomicBool,
    epoch: AtomicUsize,
    advance_after_query: AtomicI64,
    switch_after_dispatch: AtomicBool,
    consumed_scopes: Mutex<Vec<String>>,
    verify_notification_gate: AtomicBool,
    changes: AtomicUsize,
}

impl FakeHost {
    fn new(count: u32, seconds: i64) -> Self {
        let now = Local
            .with_ymd_and_hms(2026, 10, 30, 9, 0, 0)
            .unwrap()
            .timestamp();
        let settings=serde_json::from_value(serde_json::json!({"copilotRoot":"", "opencodeRoot":"", "codexRoot":"", "showArchived":false,
            "enabledProviders":["codex"], "quotaEnabledProviders":["codex"], "enableQuotaMonitoring":true,
            "codexResetCreditAutoUseEnabled":true, "codexResetCreditReminderEnabled":false})).expect("settings");
        Self {
            now: AtomicI64::new(now),
            settings: Mutex::new(settings),
            scope: Mutex::new("account-a".into()),
            credits: Mutex::new(ResetCredits {
                available_count: count,
                credits: (0..count)
                    .map(|_| ResetCreditEntry {
                        granted_at: Some("2026-10-01T00:00:00Z".into()),
                        expires_at: timestamp(now + seconds),
                        status: "active".into(),
                    })
                    .collect(),
            }),
            outcomes: Mutex::new(VecDeque::new()),
            query_errors: Mutex::new(VecDeque::new()),
            queries: AtomicUsize::new(0),
            posts: AtomicUsize::new(0),
            notices: Mutex::new(vec![]),
            deny_notification: AtomicBool::new(false),
            switch_after_query: AtomicBool::new(false),
            disable_after_query: AtomicBool::new(false),
            toggle_after_query: AtomicBool::new(false),
            epoch: AtomicUsize::new(0),
            advance_after_query: AtomicI64::new(0),
            switch_after_dispatch: AtomicBool::new(false),
            consumed_scopes: Mutex::new(Vec::new()),
            verify_notification_gate: AtomicBool::new(false),
            changes: AtomicUsize::new(0),
        }
    }
    fn advance(&self, seconds: i64) {
        self.now.fetch_add(seconds, Ordering::SeqCst);
    }
}

impl Host for FakeHost {
    type ConsumeContext = Identity;
    fn now(&self) -> DateTime<Local> {
        Local
            .timestamp_opt(self.now.load(Ordering::SeqCst), 0)
            .unwrap()
    }
    fn settings(&self) -> Result<AppSettings, String> {
        Ok(self.settings.lock().unwrap().clone())
    }
    fn identity(&self, settings: &AppSettings) -> Result<Identity, String> {
        Ok(Identity {
            scope: self.scope.lock().unwrap().clone(),
            revision: format!(
                "{}:{}",
                serde_json::to_string(&(
                    &settings.codex_root,
                    &settings.opencode_root,
                    settings.codex_reset_credit_reminder_enabled,
                    &settings.codex_reset_credit_reminder_time,
                    settings.codex_reset_credit_reminder_days_before,
                    settings.codex_reset_credit_auto_use_enabled
                ))
                .unwrap(),
                self.epoch.load(Ordering::SeqCst)
            ),
        })
    }
    fn query(&self, _: &AppSettings, _: &Identity) -> Result<ResetCredits, CreditError> {
        self.queries.fetch_add(1, Ordering::SeqCst);
        if let Some(error) = self.query_errors.lock().unwrap().pop_front() {
            return Err(error);
        }
        let credits = self.credits.lock().unwrap().clone();
        if self.switch_after_query.swap(false, Ordering::SeqCst) {
            *self.scope.lock().unwrap() = "account-b".into();
        }
        if self.disable_after_query.swap(false, Ordering::SeqCst) {
            self.settings
                .lock()
                .unwrap()
                .codex_reset_credit_auto_use_enabled = false;
        }
        if self.toggle_after_query.swap(false, Ordering::SeqCst) {
            self.epoch.fetch_add(2, Ordering::SeqCst);
        }
        self.advance(self.advance_after_query.swap(0, Ordering::SeqCst));
        Ok(credits)
    }
    fn prepare_consume(
        &self,
        settings: &AppSettings,
        identity: &Identity,
    ) -> Result<Identity, CreditError> {
        if self.identity(settings).ok().as_ref() != Some(identity) {
            return Err(CreditError::new("account_changed"));
        }
        Ok(identity.clone())
    }
    fn consume(
        &self,
        context: Identity,
        _: &str,
        expiry: Option<i64>,
    ) -> Result<String, CreditError> {
        if expiry.is_some_and(|at| !in_use_window(at, self.now().timestamp())) {
            return Err(CreditError::new("missed"));
        }
        self.posts.fetch_add(1, Ordering::SeqCst);
        self.consumed_scopes.lock().unwrap().push(context.scope);
        let result = self
            .outcomes
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Ok("reset".into()));
        if matches!(result.as_deref(), Ok("reset")) {
            let mut credits = self.credits.lock().unwrap();
            if let Some(index) = credits.credits.iter().position(|c| c.status == "active") {
                credits.credits[index].status = "used".into();
                credits.available_count = credits.available_count.saturating_sub(1);
            }
        }
        result
    }
    fn notify(
        &self,
        counts: (u32, u32),
        days: u32,
        auto: bool,
        reason: Option<&str>,
    ) -> Result<(), String> {
        if self.verify_notification_gate.load(Ordering::SeqCst) {
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                if let Ok(_guard) = SETTINGS_WRITE_GATE.try_lock() {
                    break;
                }
                assert!(
                    Instant::now() < deadline,
                    "notification must not hold settings write gate"
                );
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        if self.deny_notification.load(Ordering::SeqCst) {
            return Err("notification_denied".into());
        }
        self.notices
            .lock()
            .unwrap()
            .push(notification_body("zh-TW", counts, days, auto, reason));
        Ok(())
    }
    fn changed(&self, _: &Identity) {
        self.changes.fetch_add(1, Ordering::SeqCst);
    }
    fn publish(&self, _: &AutomationStatus) {
        if self.switch_after_dispatch.swap(false, Ordering::SeqCst) {
            self.epoch.fetch_add(1, Ordering::SeqCst);
            *self.scope.lock().unwrap() = "account-b".into();
            self.settings
                .lock()
                .unwrap()
                .codex_reset_credit_auto_use_enabled = false;
        }
    }
}

fn database() -> DbState {
    let conn = Connection::open_in_memory().expect("db");
    journal::init(&conn).expect("schema");
    DbState {
        conn: Mutex::new(conn),
    }
}

#[test]
fn reset_credit_auto_uses_only_inside_window_and_never_replays() {
    let host = FakeHost::new(1, 61);
    let db = database();
    let state = AutomationState::default();
    let mut engine = Engine::default();
    engine.tick(&host, &db, &state).unwrap();
    assert_eq!(host.posts.load(Ordering::SeqCst), 0);
    host.advance(1);
    let result = engine.tick(&host, &db, &state).unwrap();
    assert_eq!(result.last_result.as_deref(), Some("reset"));
    assert_eq!(host.posts.load(Ordering::SeqCst), 1);
    let mut restarted = Engine::default();
    journal::recover(&db.conn.lock().unwrap(), host.now().timestamp()).unwrap();
    restarted
        .tick(&host, &db, &AutomationState::default())
        .unwrap();
    assert_eq!(host.posts.load(Ordering::SeqCst), 1);
}

#[test]
fn reset_credit_multi_stops_on_nothing_to_reset_and_is_durable() {
    let host = FakeHost::new(3, 60);
    let db = database();
    let state = AutomationState::default();
    let mut engine = Engine::default();
    host.outcomes
        .lock()
        .unwrap()
        .extend([Ok("reset".into()), Ok("nothing_to_reset".into())]);
    engine.tick(&host, &db, &state).unwrap();
    assert_eq!(host.posts.load(Ordering::SeqCst), 2);
    engine.tick(&host, &db, &state).unwrap();
    assert_eq!(host.posts.load(Ordering::SeqCst), 2);
    assert_eq!(host.credits.lock().unwrap().available_count, 2);
}

#[test]
fn reset_credit_ambiguous_result_blocks_both_paths_after_restart() {
    let host = FakeHost::new(1, 60);
    let db = database();
    let state = AutomationState::default();
    let mut engine = Engine::default();
    host.outcomes.lock().unwrap().push_back(Err(CreditError {
        code: "unknown",
        retry_after: None,
        unknown: true,
    }));
    engine.tick(&host, &db, &state).unwrap();
    journal::recover(&db.conn.lock().unwrap(), host.now().timestamp()).unwrap();
    let restarted = AutomationState::default();
    Engine::default().tick(&host, &db, &restarted).unwrap();
    assert_eq!(
        consume_manual_internal(&host, &db, &restarted, "manual-new", "account-a"),
        Err("unknown".into())
    );
    assert_eq!(host.posts.load(Ordering::SeqCst), 1);
}

#[test]
fn reset_credit_confirmation_identity_and_inflight_account_lock() {
    let host = FakeHost::new(1, 3600);
    let db = database();
    let state = AutomationState::default();
    assert_eq!(
        consume_manual_internal(&host, &db, &state, "id", "another-account"),
        Err("account_changed".into())
    );
    let _guard = state.claim("account-a").unwrap();
    assert_eq!(
        consume_manual_internal(&host, &db, &state, "id", "account-a"),
        Err("running".into())
    );
    assert_eq!(host.posts.load(Ordering::SeqCst), 0);
    assert_eq!(host.queries.load(Ordering::SeqCst), 0);
}

#[test]
fn reset_credit_account_switch_during_query_cancels_dispatch() {
    let host = FakeHost::new(1, 3600);
    let db = database();
    let state = AutomationState::default();
    host.switch_after_query.store(true, Ordering::SeqCst);
    assert_eq!(
        consume_manual_internal(&host, &db, &state, "id", "account-a"),
        Err("account_changed".into())
    );
    assert_eq!(host.posts.load(Ordering::SeqCst), 0);
}

#[test]
fn reset_credit_same_request_id_returns_existing_result_without_query_or_post() {
    let host = FakeHost::new(1, 3600);
    let db = database();
    let state = AutomationState::default();
    assert_eq!(
        consume_manual_internal(&host, &db, &state, "id", "account-a"),
        Ok("reset".into())
    );
    let queries = host.queries.load(Ordering::SeqCst);
    assert_eq!(
        consume_manual_internal(&host, &db, &state, "id", "account-a"),
        Ok("reset".into())
    );
    assert_eq!(host.posts.load(Ordering::SeqCst), 1);
    assert_eq!(host.queries.load(Ordering::SeqCst), queries);
    *host.scope.lock().unwrap() = "account-b".into();
    assert_eq!(
        consume_manual_internal(&host, &db, &state, "id", "account-b"),
        Err("request_mismatch".into())
    );
}

#[test]
fn reset_credit_daily_notice_is_once_and_independent_of_auto() {
    let host = FakeHost::new(1, 3600);
    let db = database();
    let state = AutomationState::default();
    {
        let mut settings = host.settings.lock().unwrap();
        settings.codex_reset_credit_reminder_enabled = true;
        settings.codex_reset_credit_auto_use_enabled = false;
    }
    let mut engine = Engine::default();
    engine.tick(&host, &db, &state).unwrap();
    host.advance(360);
    Engine::default().tick(&host, &db, &state).unwrap();
    let notices = host.notices.lock().unwrap();
    assert_eq!(notices.len(), 1);
    assert!(notices[0].contains("自動使用未開啟"));
    assert_eq!(host.posts.load(Ordering::SeqCst), 0);
}

#[test]
fn reset_credit_daily_retry_budget_survives_engine_restart() {
    let host = FakeHost::new(1, 3600);
    let db = database();
    let state = AutomationState::default();
    {
        let mut settings = host.settings.lock().unwrap();
        settings.codex_reset_credit_reminder_enabled = true;
        settings.codex_reset_credit_auto_use_enabled = false;
    }
    host.query_errors
        .lock()
        .unwrap()
        .extend((0..5).map(|_| CreditError::new("transient_error")));
    Engine::default().tick(&host, &db, &state).unwrap();
    host.advance(40);
    Engine::default().tick(&host, &db, &state).unwrap();
    host.advance(130);
    Engine::default().tick(&host, &db, &state).unwrap();
    host.advance(500);
    let status = Engine::default().tick(&host, &db, &state).unwrap();
    assert_eq!(host.queries.load(Ordering::SeqCst), 3);
    assert_eq!(status.notification_status.as_deref(), Some("failed"));
    assert!(host.notices.lock().unwrap().is_empty());
}

#[test]
fn reset_credit_notification_denial_does_not_block_auto() {
    let host = FakeHost::new(1, 3600);
    let db = database();
    let state = AutomationState::default();
    let mut engine = Engine::default();
    host.settings
        .lock()
        .unwrap()
        .codex_reset_credit_reminder_enabled = true;
    host.deny_notification.store(true, Ordering::SeqCst);
    let status = engine.tick(&host, &db, &state).unwrap();
    assert_eq!(
        status.notification_error.as_deref(),
        Some("notification_denied")
    );
    host.advance(3540);
    engine.tick(&host, &db, &state).unwrap();
    assert_eq!(host.posts.load(Ordering::SeqCst), 1);
}

#[test]
fn reset_credit_mixed_expiry_and_resume_after_expiry_never_consume() {
    let host = FakeHost::new(2, 60);
    let db = database();
    let state = AutomationState::default();
    let mut engine = Engine::default();
    host.credits.lock().unwrap().credits[1].expires_at = timestamp(host.now().timestamp() + 3600);
    let status = engine.tick(&host, &db, &state).unwrap();
    assert_eq!(status.reason.as_deref(), Some("selection_unverified"));
    host.advance(61);
    Engine::default().tick(&host, &db, &state).unwrap();
    assert_eq!(host.posts.load(Ordering::SeqCst), 0);
}

#[test]
fn reset_credit_disabled_before_dispatch_can_be_reenabled_without_reviving_old_job() {
    let host = FakeHost::new(1, 60);
    let db = database();
    let state = AutomationState::default();
    let mut engine = Engine::default();
    // 第一次 discovery 也可能遇到設定儲存；本輪不得使用舊版本核發 POST。
    host.disable_after_query.store(true, Ordering::SeqCst);
    let _ = engine.tick(&host, &db, &state);
    assert_eq!(host.posts.load(Ordering::SeqCst), 0);
    host.settings
        .lock()
        .unwrap()
        .codex_reset_credit_auto_use_enabled = true;
    host.advance(20);
    engine.tick(&host, &db, &state).unwrap();
    assert_eq!(host.posts.load(Ordering::SeqCst), 1);
}

#[test]
fn reset_credit_storage_failure_prevents_side_effects() {
    let host = FakeHost::new(1, 60);
    let db = database();
    let state = AutomationState::default();
    db.conn
        .lock()
        .unwrap()
        .execute_batch("PRAGMA query_only=ON")
        .unwrap();
    assert!(Engine::default().tick(&host, &db, &state).is_err());
    assert_eq!(host.posts.load(Ordering::SeqCst), 0);
    assert!(host.notices.lock().unwrap().is_empty());
}

#[test]
fn reset_credit_off_on_during_query_does_not_revive_old_generation() {
    let host = FakeHost::new(1, 60);
    let db = database();
    let state = AutomationState::default();
    let mut engine = Engine::default();
    host.toggle_after_query.store(true, Ordering::SeqCst);
    let _ = engine.tick(&host, &db, &state);
    assert_eq!(host.posts.load(Ordering::SeqCst), 0);
    host.advance(20);
    engine.tick(&host, &db, &state).unwrap();
    assert_eq!(host.posts.load(Ordering::SeqCst), 1);
}

#[test]
fn reset_credit_manual_rejects_unconfirmed_active_count() {
    let host = FakeHost::new(1, 3600);
    let db = database();
    let state = AutomationState::default();
    host.credits.lock().unwrap().credits[0].status = "unknown_status".into();
    assert_eq!(
        consume_manual_internal(&host, &db, &state, "manual", "account-a"),
        Err("inconsistent".into())
    );
    assert_eq!(host.posts.load(Ordering::SeqCst), 0);
}

#[test]
fn reset_credit_post_auth_rejection_survives_unrelated_setting_changes() {
    let host = FakeHost::new(1, 3600);
    let creds = CodexCredentials::test_credentials("test-rejected-post-credential");
    remember_auth_rejection(
        &creds.credential_revision(),
        &Err(CreditError::new("no_auth")),
    );
    let mut settings = host.settings().unwrap();
    assert_eq!(
        query_with_backoff(&creds, &settings).unwrap_err().code,
        "no_auth"
    );
    settings.codex_reset_credit_auto_use_enabled = false;
    assert_eq!(
        query_with_backoff(&creds, &settings).unwrap_err().code,
        "no_auth"
    );
    let updated = CodexCredentials::test_credentials("test-refreshed-credential");
    assert!(!query_state()
        .lock()
        .unwrap()
        .rejected_revisions
        .contains(&updated.credential_revision()));
}

#[test]
fn reset_credit_notification_text_reflects_selection_and_language() {
    assert!(
        notification_body("zh-TW", (2, 1), 3, true, Some("selection_unverified"))
            .contains("無法確認使用順序")
    );
    assert!(
        notification_body("en-US", (0, 2), 3, true, None).contains("0 reset credits expire today")
    );
    assert!(notification_body("en-US", (0, 2), 3, false, None).contains("Auto-use is off"));
}

#[test]
fn reset_credit_slow_preflight_query_crossing_expiry_never_posts() {
    let host = FakeHost::new(1, 61);
    let db = database();
    let state = AutomationState::default();
    let mut engine = Engine::default();
    engine.tick(&host, &db, &state).unwrap();
    host.advance(1);
    host.advance_after_query.store(61, Ordering::SeqCst);
    engine.tick(&host, &db, &state).unwrap();
    assert_eq!(host.posts.load(Ordering::SeqCst), 0);
}

#[test]
fn reset_credit_committed_context_survives_settings_and_account_switch() {
    let host = FakeHost::new(1, 60);
    let db = database();
    let state = AutomationState::default();
    host.switch_after_dispatch.store(true, Ordering::SeqCst);
    let _ = Engine::default().tick(&host, &db, &state);
    assert_eq!(host.posts.load(Ordering::SeqCst), 1);
    assert_eq!(*host.consumed_scopes.lock().unwrap(), vec!["account-a"]);
    let last = journal::latest_attempt(&db.conn.lock().unwrap(), "account-a")
        .unwrap()
        .unwrap();
    assert_eq!(last.result.as_deref(), Some("reset"));
    assert!(
        journal::latest_attempt(&db.conn.lock().unwrap(), "account-b")
            .unwrap()
            .is_none()
    );
}

#[test]
fn reset_credit_manual_keeps_quota_only_provider_compatibility() {
    let host = FakeHost::new(1, 3600);
    let db = database();
    let state = AutomationState::default();
    host.settings.lock().unwrap().enabled_providers.clear();
    let status = Engine::default().tick(&host, &db, &state).unwrap();
    assert_eq!(status.auto_use_state, "ready");
    assert_eq!(
        consume_manual_internal(&host, &db, &state, "manual-quota-only", "account-a"),
        Ok("reset".into())
    );
}

#[test]
fn reset_credit_reminder_does_not_submit_previous_day_after_midnight() {
    let host = FakeHost::new(1, 86400);
    let db = database();
    let state = AutomationState::default();
    {
        let mut settings = host.settings.lock().unwrap();
        settings.codex_reset_credit_auto_use_enabled = false;
        settings.codex_reset_credit_reminder_enabled = true;
        settings.codex_reset_credit_reminder_time = "23:59".into();
        settings.codex_reset_credit_reminder_days_before = 1;
    }
    host.now.store(
        Local
            .with_ymd_and_hms(2026, 10, 30, 23, 59, 50)
            .unwrap()
            .timestamp(),
        Ordering::SeqCst,
    );
    host.advance_after_query.store(20, Ordering::SeqCst);
    let mut engine = Engine::default();
    engine.tick(&host, &db, &state).unwrap();
    engine.tick(&host, &db, &state).unwrap();
    assert!(host.notices.lock().unwrap().is_empty());
    assert_eq!(host.queries.load(Ordering::SeqCst), 1);
    assert_eq!(
        journal::reminder(&db.conn.lock().unwrap(), "account-a", "2026-10-30")
            .unwrap()
            .unwrap()
            .state,
        "cancelled"
    );
}

#[test]
fn reset_credit_late_start_clock_rollback_and_empty_check_are_deduplicated() {
    let host = FakeHost::new(1, 43200);
    let db = database();
    let state = AutomationState::default();
    {
        let mut settings = host.settings.lock().unwrap();
        settings.codex_reset_credit_auto_use_enabled = false;
        settings.codex_reset_credit_reminder_enabled = true;
    }
    host.advance(10800);
    let mut engine = Engine::default();
    engine.tick(&host, &db, &state).unwrap();
    host.advance(-7200);
    host.settings
        .lock()
        .unwrap()
        .codex_reset_credit_reminder_time = "10:00".into();
    engine.tick(&host, &db, &state).unwrap();
    assert_eq!(host.notices.lock().unwrap().len(), 1);
    let empty = FakeHost::new(0, 3600);
    let empty_db = database();
    {
        let mut settings = empty.settings.lock().unwrap();
        settings.codex_reset_credit_auto_use_enabled = false;
        settings.codex_reset_credit_reminder_enabled = true;
    }
    let mut engine = Engine::default();
    engine.tick(&empty, &empty_db, &state).unwrap();
    engine.tick(&empty, &empty_db, &state).unwrap();
    assert_eq!(empty.queries.load(Ordering::SeqCst), 1);
    assert!(empty.notices.lock().unwrap().is_empty());
}

#[test]
fn reset_credit_reminder_is_independent_and_disabled_auto_stays_disabled() {
    let host = FakeHost::new(1, 3600);
    let db = database();
    let state = AutomationState::default();
    {
        let mut settings = host.settings.lock().unwrap();
        settings.enable_quota_monitoring = false;
        settings.enabled_providers.clear();
        settings.quota_enabled_providers.clear();
        settings.codex_reset_credit_reminder_enabled = true;
        settings.codex_reset_credit_auto_use_enabled = false;
    }
    host.verify_notification_gate.store(true, Ordering::SeqCst);
    let result = Engine::default().tick(&host, &db, &state).unwrap();
    assert_eq!(result.reminder_state, "ready");
    assert_eq!(result.auto_use_state, "disabled");
    assert_eq!(host.notices.lock().unwrap().len(), 1);
    assert_eq!(host.posts.load(Ordering::SeqCst), 0);
    let failed = failed_status(
        Some(&host.settings().unwrap()),
        &result,
        "storage_error".into(),
    );
    assert_eq!(failed.reminder_state, "paused");
    assert_eq!(failed.auto_use_state, "disabled");
    assert!(failed.auto_use_reason.is_none());
}

#[test]
fn reset_credit_auto_use_does_not_require_platform_monitoring() {
    let host = FakeHost::new(1, 60);
    let db = database();
    let state = AutomationState::default();
    {
        let mut settings = host.settings.lock().unwrap();
        settings.enable_quota_monitoring = false;
        settings.enabled_providers.clear();
        settings.quota_enabled_providers.clear();
    }
    Engine::default().tick(&host, &db, &state).unwrap();
    assert_eq!(host.posts.load(Ordering::SeqCst), 1);
    host.settings
        .lock()
        .unwrap()
        .codex_reset_credit_auto_use_enabled = false;
    let settings = host.settings().unwrap();
    let failed = failed_status(
        Some(&settings),
        &AutomationStatus::default(),
        "storage_error".into(),
    );
    assert_eq!(failed.auto_use_state, "disabled");
    assert_eq!(failed.reminder_state, "disabled");
    assert!(failed.reason.is_none());
}
