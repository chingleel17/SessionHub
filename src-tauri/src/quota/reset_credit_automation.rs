//! 程序內重置券協調器。網路、時鐘與通知由 Host 注入，測試不使用真實帳號。
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, Utc};
use rusqlite::Connection;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::db::reset_credits as journal;
use crate::settings::{load_settings_file_internal, load_settings_internal, SETTINGS_WRITE_GATE};
use crate::types::{AppSettings, QuotaCache, ResetCredits};
use crate::DbState;

use super::codex::{self, CodexCredentials, CreditError};
use super::reset_credit_policy::*;

const EVENT: &str = "reset-credit-automation-updated";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Identity {
    pub(crate) scope: String,
    pub(crate) revision: String,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AutomationStatus {
    pub(crate) account_scope: Option<String>,
    pub(crate) generation: u64,
    pub(crate) reminder_state: String,
    pub(crate) auto_use_state: String,
    pub(crate) reason: Option<String>,
    pub(crate) reminder_reason: Option<String>,
    pub(crate) auto_use_reason: Option<String>,
    pub(crate) notification_status: Option<String>,
    pub(crate) notification_error: Option<String>,
    pub(crate) next_reminder_at: Option<String>,
    pub(crate) last_attempt_at: Option<String>,
    pub(crate) last_result: Option<String>,
}

#[derive(Default)]
pub(crate) struct AutomationState {
    status: Mutex<AutomationStatus>,
    active: Mutex<HashSet<String>>,
    wake: Mutex<u64>,
    changed: Condvar,
    refresh: AtomicBool,
    quota_stamp: Mutex<Option<(String, Option<String>)>>,
}

impl AutomationState {
    fn running(&self, scope: &str) -> Result<AutomationStatus, String> {
        let mut status = self.status.lock().map_err(|_| "storage_error")?;
        status.account_scope = Some(scope.into());
        status.auto_use_state = "running".into();
        status.generation = status.generation.wrapping_add(1);
        Ok(status.clone())
    }
    pub(crate) fn request_refresh(&self) {
        self.refresh.store(true, Ordering::SeqCst);
        self.wake();
    }
    pub(crate) fn wake(&self) {
        if let Ok(mut version) = self.wake.lock() {
            *version = version.wrapping_add(1);
            self.changed.notify_all();
        }
    }
    pub(crate) fn status(&self) -> Result<AutomationStatus, String> {
        self.status
            .lock()
            .map(|s| s.clone())
            .map_err(|_| "無法讀取重置券狀態".into())
    }
    fn claim(&self, scope: &str) -> Result<AccountGuard<'_>, String> {
        if !self
            .active
            .lock()
            .map_err(|_| "無法鎖定帳號")?
            .insert(scope.to_string())
        {
            return Err("running".into());
        }
        Ok(AccountGuard {
            state: self,
            scope: scope.to_string(),
        })
    }
}

struct AccountGuard<'a> {
    state: &'a AutomationState,
    scope: String,
}
impl Drop for AccountGuard<'_> {
    fn drop(&mut self) {
        if let Ok(mut active) = self.state.active.lock() {
            active.remove(&self.scope);
        }
    }
}

pub(crate) trait Host {
    type ConsumeContext;
    fn now(&self) -> DateTime<Local>;
    fn settings(&self) -> Result<AppSettings, String>;
    fn identity(&self, settings: &AppSettings) -> Result<Identity, String>;
    fn query(
        &self,
        settings: &AppSettings,
        identity: &Identity,
    ) -> Result<ResetCredits, CreditError>;
    fn prepare_consume(
        &self,
        settings: &AppSettings,
        identity: &Identity,
    ) -> Result<Self::ConsumeContext, CreditError>;
    fn consume(
        &self,
        context: Self::ConsumeContext,
        id: &str,
        expiry: Option<i64>,
    ) -> Result<String, CreditError>;
    fn notify(
        &self,
        counts: (u32, u32),
        days: u32,
        auto: bool,
        reason: Option<&str>,
    ) -> Result<(), String>;
    fn changed(&self, identity: &Identity);
    fn publish(&self, _status: &AutomationStatus) {}
}

fn with_db<T>(
    db: &DbState,
    action: impl FnOnce(&Connection) -> Result<T, String>,
) -> Result<T, String> {
    let conn = db.conn.lock().map_err(|_| "storage_error".to_string())?;
    action(&conn).map_err(|_| "storage_error".to_string())
}

fn timestamp(value: i64) -> Option<String> {
    DateTime::<Utc>::from_timestamp(value, 0).map(|t| t.to_rfc3339())
}

/// 同 ID 只能回讀，不能再執行。呼叫端提供的帳號範圍不是登入授權。
fn prior_result(
    db: &DbState,
    identity: &Identity,
    id: &str,
    expiry: Option<i64>,
) -> Result<Option<String>, String> {
    let previous = with_db(db, |c| journal::attempt(c, id))?;
    let Some(previous) = previous else {
        return Ok(None);
    };
    if previous.scope != identity.scope
        || previous.expiry != expiry
        || previous.origin
            != if expiry.is_some() {
                "automatic"
            } else {
                "manual"
            }
    {
        return Err("request_mismatch".into());
    }
    match previous.result.as_deref() {
        Some(result @ ("reset" | "nothing_to_reset" | "no_credit" | "already_redeemed")) => {
            Ok(Some(result.to_string()))
        }
        _ => Err(if previous.state == "pending" {
            "running"
        } else {
            previous.result.as_deref().unwrap_or("unknown")
        }
        .into()),
    }
}

fn dispatch(
    host: &impl Host,
    db: &DbState,
    state: &AutomationState,
    settings: &AppSettings,
    identity: &Identity,
    id: &str,
    expiry: Option<i64>,
    credits: &ResetCredits,
) -> Result<String, String> {
    if let Some(result) = prior_result(db, identity, id, expiry)? {
        return Ok(result);
    }
    let context;
    {
        let _gate = SETTINGS_WRITE_GATE.lock().map_err(|_| "storage_error")?;
        let current = host.settings()?;
        if expiry.is_none() && !quota_enabled(&current) {
            return Err("monitoring_disabled".into());
        }
        if host.identity(&current)? != *identity {
            return Err("account_changed".into());
        }
        if expiry.is_some()
            && (!current.codex_reset_credit_auto_use_enabled
                || !settings.codex_reset_credit_auto_use_enabled)
        {
            return Err("disabled".into());
        }
        let now = host.now().timestamp();
        if let Some(expiry) = expiry {
            if !in_use_window(expiry, now) || eligible_expiry(credits, now) != Ok(expiry) {
                return Err("missed".into());
            }
        }
        if !with_db(db, |c| {
            journal::reconcile(c, &identity.scope, |original| {
                original_credits_resolved(original, credits, now)
            })
        })? {
            return Err("unknown".into());
        }
        context = host
            .prepare_consume(&current, identity)
            .map_err(|e| e.code.to_string())?;
        with_db(db, |c| {
            journal::begin_attempt(c, &identity.scope, id, expiry, credits, now)
        })?;
    }
    if expiry.is_some() {
        host.publish(&state.running(&identity.scope)?);
    }
    let result = host.consume(context, id, expiry);
    let now = host.now().timestamp();
    let (code, unknown) = match &result {
        Ok(code) => (code.as_str(), false),
        Err(error) => (error.code, error.unknown),
    };
    with_db(db, |c| {
        if let Err(error) = &result {
            if let Some(delay) = error.retry_after {
                journal::defer(c, &identity.scope, now.saturating_add(delay))?;
            }
        }
        journal::finish_attempt(c, id, code, unknown, now)
    })?;
    result.map_err(|error| error.code.to_string())
}

pub(crate) fn consume_manual_internal(
    host: &impl Host,
    db: &DbState,
    state: &AutomationState,
    id: &str,
    expected_scope: &str,
) -> Result<String, String> {
    if id.trim().is_empty() || expected_scope.is_empty() {
        return Err("invalid_request".into());
    }
    let settings = host.settings()?;
    if !quota_enabled(&settings) {
        return Err("monitoring_disabled".into());
    }
    let identity = host.identity(&settings)?;
    if identity.scope != expected_scope {
        return Err("account_changed".into());
    }
    if let Some(result) = prior_result(db, &identity, id, None)? {
        return Ok(result);
    }
    let _account = state.claim(&identity.scope)?;
    let credits = host
        .query(&settings, &identity)
        .map_err(|e| e.code.to_string())?;
    if credits.available_count == 0 {
        return Err("no_credit".into());
    }
    if credits
        .credits
        .iter()
        .filter(|c| c.status == "active")
        .count()
        != credits.available_count as usize
    {
        return Err("inconsistent".into());
    }
    let result = dispatch(host, db, state, &settings, &identity, id, None, &credits);
    host.changed(&identity);
    state.request_refresh();
    result
}

#[derive(Default)]
pub(crate) struct Engine {
    identity: Option<Identity>,
    credits: Option<ResetCredits>,
    discovery_at: i64,
    query_error: Option<String>,
}

fn configured_status(settings: &AppSettings) -> AutomationStatus {
    AutomationStatus {
        reminder_state: if settings.codex_reset_credit_reminder_enabled {
            "ready"
        } else {
            "disabled"
        }
        .into(),
        auto_use_state: if settings.codex_reset_credit_auto_use_enabled {
            "ready"
        } else {
            "disabled"
        }
        .into(),
        ..Default::default()
    }
}

fn failed_status(
    settings: Option<&AppSettings>,
    current: &AutomationStatus,
    reason: String,
) -> AutomationStatus {
    let reminder = settings
        .map(|s| s.codex_reset_credit_reminder_enabled)
        .unwrap_or(matches!(
            current.reminder_state.as_str(),
            "ready" | "paused" | "running"
        ));
    let auto = settings
        .map(|s| s.codex_reset_credit_auto_use_enabled)
        .unwrap_or(matches!(
            current.auto_use_state.as_str(),
            "ready" | "paused" | "running"
        ));
    AutomationStatus {
        reminder_state: if reminder { "paused" } else { "disabled" }.into(),
        auto_use_state: if auto { "paused" } else { "disabled" }.into(),
        reminder_reason: reminder.then(|| reason.clone()),
        auto_use_reason: auto.then(|| reason.clone()),
        reason: (reminder || auto).then_some(reason),
        ..Default::default()
    }
}

impl Engine {
    pub(crate) fn tick(
        &mut self,
        host: &impl Host,
        db: &DbState,
        state: &AutomationState,
    ) -> Result<AutomationStatus, String> {
        let settings = host.settings()?;
        let mut status = configured_status(&settings);
        let identity = match host.identity(&settings) {
            Ok(id) => id,
            Err(_) => {
                self.identity = None;
                self.credits = None;
                return Ok(failed_status(Some(&settings), &status, "no_auth".into()));
            }
        };
        status.account_scope = Some(identity.scope.clone());
        let scope_changed = self.identity.as_ref().map(|old| &old.scope) != Some(&identity.scope);
        if self.identity.as_ref() != Some(&identity) {
            self.identity = Some(identity.clone());
            self.credits = None;
            self.discovery_at = 0;
            self.query_error = None;
        }
        let now = host.now();
        let force = state.refresh.swap(false, Ordering::SeqCst);
        if (force
            && (settings.codex_reset_credit_auto_use_enabled
                || settings.codex_reset_credit_reminder_enabled))
            || (settings.codex_reset_credit_auto_use_enabled
                && now.timestamp() >= self.discovery_at)
        {
            self.discovery_at = now.timestamp().saturating_add(DISCOVERY_INTERVAL_SECONDS);
            match host.query(&settings, &identity) {
                Ok(credits) => {
                    self.credits = Some(credits);
                    self.query_error = None;
                    if scope_changed {
                        host.changed(&identity);
                    }
                }
                Err(error) => {
                    self.query_error = Some(error.code.into());
                    self.credits = None;
                }
            }
        }
        if settings.codex_reset_credit_auto_use_enabled {
            if let Some(credits) = self.credits.clone() {
                match eligible_expiry(&credits, host.now().timestamp()) {
                    Ok(expiry) if in_use_window(expiry, host.now().timestamp()) => {
                        if let Ok(_account) = state.claim(&identity.scope) {
                            self.run_auto(host, db, state, &settings, &identity, expiry, credits)?;
                        }
                    }
                    Ok(_) => {}
                    Err(reason) => status.reason = Some(reason.into()),
                }
            }
        }
        if settings.codex_reset_credit_reminder_enabled {
            let now = host.now();
            let date = now.date_naive().to_string();
            let mut record = with_db(db, |c| journal::reminder(c, &identity.scope, &date))?;
            let backoff = with_db(db, |c| journal::backoff(c, &identity.scope))?;
            if reminder_due(&now, &settings.codex_reset_credit_reminder_time)
                && now.timestamp() >= backoff
                && with_db(db, |c| {
                    journal::claim_reminder_query(c, &identity.scope, &date, now.timestamp())
                })?
            {
                match host.query(&settings, &identity) {
                    Ok(credits) => {
                        self.query_error = None;
                        self.credits = Some(credits.clone());
                        let notice = {
                            let _gate = SETTINGS_WRITE_GATE.lock().map_err(|_| "storage_error")?;
                            let current = host.settings()?;
                            let now = host.now();
                            if current.codex_reset_credit_reminder_enabled
                                && host.identity(&current)? == identity
                                && now.date_naive().to_string() == date
                            {
                                let counts = reminder_counts(
                                    &credits,
                                    &now,
                                    current.codex_reset_credit_reminder_days_before,
                                );
                                let claimed = if counts.0 + counts.1 == 0 {
                                    "checked_empty"
                                } else {
                                    "claimed"
                                };
                                with_db(db, |c| {
                                    journal::finish_reminder(
                                        c,
                                        &identity.scope,
                                        &date,
                                        counts,
                                        claimed,
                                        None,
                                        now.timestamp(),
                                    )
                                })?;
                                if claimed == "claimed" {
                                    let reconciled = with_db(db, |c| {
                                        journal::reconcile(c, &identity.scope, |original| {
                                            original_credits_resolved(
                                                original,
                                                &credits,
                                                now.timestamp(),
                                            )
                                        })
                                    })?;
                                    let reason = if !reconciled {
                                        Some("unknown")
                                    } else {
                                        eligible_expiry(&credits, now.timestamp()).err()
                                    };
                                    Some((
                                        counts,
                                        current.codex_reset_credit_reminder_days_before,
                                        current.codex_reset_credit_auto_use_enabled,
                                        reason.map(str::to_string),
                                    ))
                                } else {
                                    None
                                }
                            } else {
                                with_db(db, |c| {
                                    journal::finish_reminder(
                                        c,
                                        &identity.scope,
                                        &date,
                                        (0, 0),
                                        "cancelled",
                                        None,
                                        now.timestamp(),
                                    )
                                })?;
                                None
                            }
                        };
                        // 通知插件可能等候原生 UI，提交權已保留後不可繼續佔用設定閘門。
                        if let Some((counts, days, auto, reason)) = notice {
                            let result = host.notify(counts, days, auto, reason.as_deref());
                            with_db(db, |c| {
                                journal::finish_reminder(
                                    c,
                                    &identity.scope,
                                    &date,
                                    counts,
                                    if result.is_ok() {
                                        "dispatched"
                                    } else {
                                        "failed"
                                    },
                                    result.err().as_deref(),
                                    host.now().timestamp(),
                                )
                            })?;
                        }
                    }
                    Err(error) => {
                        self.query_error = Some(error.code.into());
                        with_db(db, |c| {
                            journal::reminder_query_failed(
                                c,
                                &identity.scope,
                                &date,
                                error.code,
                                matches!(error.code, "transient_error" | "rate_limited"),
                                host.now().timestamp(),
                            )
                        })?;
                    }
                }
                record = with_db(db, |c| journal::reminder(c, &identity.scope, &date))?;
            }
            let checked = record.as_ref().is_some_and(|r| r.state != "waiting");
            status.notification_status = record.as_ref().map(|r| r.state.clone());
            status.notification_error = record.as_ref().and_then(|r| r.detail.clone());
            status.next_reminder_at = if let Some(r) = record.filter(|r| r.state == "waiting") {
                timestamp(r.next_at.max(backoff))
            } else {
                next_reminder(
                    &host.now(),
                    &settings.codex_reset_credit_reminder_time,
                    checked,
                )
                .and_then(timestamp)
            };
        }
        if settings.codex_reset_credit_auto_use_enabled {
            if let Some(error) = &self.query_error {
                status.reason = Some(error.clone());
            }
            if let Some(credits) = &self.credits {
                if !with_db(db, |c| {
                    journal::reconcile(c, &identity.scope, |original| {
                        original_credits_resolved(original, credits, host.now().timestamp())
                    })
                })? {
                    status.reason = Some("unknown".into());
                }
                if let Ok(expiry) = eligible_expiry(credits, host.now().timestamp()) {
                    if let Some(batch) =
                        with_db(db, |c| journal::batch(c, &identity.scope, expiry))?
                    {
                        if batch.state == "terminal" {
                            status.reason = Some(match batch.detail.as_deref() {
                                Some(
                                    "reset" | "nothing_to_reset" | "no_credit" | "already_redeemed",
                                ) => "batch_processed".into(),
                                _ => batch.detail.unwrap_or_else(|| "batch_processed".into()),
                            });
                        }
                    }
                }
            }
            if status.reason.is_some() {
                status.auto_use_state = "paused".into();
            }
        }
        if let Some(last) = with_db(db, |c| journal::latest_attempt(c, &identity.scope))? {
            status.last_attempt_at = timestamp(last.updated_at);
            status.last_result = last.result;
        }
        if settings.codex_reset_credit_auto_use_enabled {
            status.auto_use_reason = status.reason.clone();
        }
        if settings.codex_reset_credit_reminder_enabled {
            status.reminder_reason = status
                .notification_error
                .clone()
                .or_else(|| self.query_error.clone());
            if status.reminder_reason.is_some() {
                status.reminder_state = "paused".into();
            }
        }
        status.reason = status
            .auto_use_reason
            .clone()
            .or_else(|| status.reminder_reason.clone());
        if host.identity(&host.settings()?)? != identity {
            return Err("account_changed".into());
        }
        Ok(status)
    }

    fn run_auto(
        &mut self,
        host: &impl Host,
        db: &DbState,
        state: &AutomationState,
        settings: &AppSettings,
        identity: &Identity,
        expiry: i64,
        candidate: ResetCredits,
    ) -> Result<(), String> {
        let now = host.now().timestamp();
        if now < with_db(db, |c| journal::backoff(c, &identity.scope))? {
            return Ok(());
        }
        if !with_db(db, |c| {
            journal::claim_auto_query(c, &identity.scope, expiry, candidate.available_count, now)
        })? {
            return Ok(());
        }
        let mut credits = match host.query(settings, identity) {
            Ok(credits) => credits,
            Err(error) => {
                self.query_error = Some(error.code.into());
                if !matches!(error.code, "transient_error" | "rate_limited") {
                    with_db(db, |c| {
                        journal::finish_batch(c, &identity.scope, expiry, error.code)
                    })?;
                }
                return Ok(());
            }
        };
        self.query_error = None;
        with_db(db, |c| {
            journal::limit_batch(c, &identity.scope, expiry, credits.available_count)
        })?;
        let mut attempted = false;
        loop {
            let now = host.now().timestamp();
            if !in_use_window(expiry, now) || eligible_expiry(&credits, now) != Ok(expiry) {
                break;
            }
            let batch = with_db(db, |c| journal::batch(c, &identity.scope, expiry))?
                .ok_or("storage_error")?;
            if batch.used >= batch.max_count {
                break;
            }
            let seed = format!("{}:{expiry}:{}", identity.scope, batch.used + 1);
            let hex = Sha256::digest(seed.as_bytes())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            let id = format!(
                "{}-{}-5{}-a{}-{}",
                &hex[..8],
                &hex[8..12],
                &hex[13..16],
                &hex[17..20],
                &hex[20..32]
            );
            let previous = credits.available_count;
            let result = dispatch(
                host,
                db,
                state,
                settings,
                identity,
                &id,
                Some(expiry),
                &credits,
            );
            attempted |= with_db(db, |c| journal::attempt(c, &id))?.is_some();
            match result.as_deref() {
                Ok("reset") => match host.query(settings, identity) {
                    Ok(updated)
                        if updated
                            .credits
                            .iter()
                            .filter(|c| c.status == "active" && expires_at(c) == Some(expiry))
                            .count()
                            < previous as usize =>
                    {
                        credits = updated;
                        self.credits = Some(credits.clone());
                        with_db(db, |c| journal::verified_batch(c, &identity.scope, expiry))?;
                    }
                    _ => {
                        with_db(db, |c| {
                            journal::finish_batch(c, &identity.scope, expiry, "refresh_required")
                        })?;
                        break;
                    }
                },
                other => {
                    let reason = match other {
                        Ok(code) => code,
                        Err(error) => error,
                    };
                    if matches!(
                        reason,
                        "disabled" | "account_changed" | "monitoring_disabled"
                    ) && batch.used == 0
                    {
                        break;
                    }
                    with_db(db, |c| {
                        journal::finish_batch(c, &identity.scope, expiry, reason)
                    })?;
                    break;
                }
            }
        }
        self.credits = Some(credits);
        if attempted {
            let batch = with_db(db, |c| journal::batch(c, &identity.scope, expiry))?
                .ok_or("storage_error")?;
            if batch.state != "terminal" {
                with_db(db, |c| {
                    journal::finish_batch(c, &identity.scope, expiry, "reset")
                })?;
            }
            host.changed(identity);
        }
        Ok(())
    }
}

struct QueryState {
    last: Option<(String, Instant, Result<ResetCredits, CreditError>)>,
    rejected_revisions: HashSet<String>,
}

fn query_state() -> &'static Mutex<QueryState> {
    static QUERY: OnceLock<Mutex<QueryState>> = OnceLock::new();
    QUERY.get_or_init(|| {
        Mutex::new(QueryState {
            last: None,
            rejected_revisions: HashSet::new(),
        })
    })
}

fn remember_auth_rejection(revision: &str, result: &Result<String, CreditError>) {
    if result.as_ref().is_err_and(|e| e.code == "no_auth") {
        if let Ok(mut state) = query_state().lock() {
            state.rejected_revisions.insert(revision.into());
            state.last = None;
        }
    }
}

/// 既有 quota adapter、手動及背景工作共用查詢閘門和端點退避。
pub(crate) fn query_with_backoff(
    creds: &CodexCredentials,
    settings: &AppSettings,
) -> Result<ResetCredits, CreditError> {
    let started = Instant::now();
    let revision = creds.revision(settings);
    let mut state = query_state()
        .lock()
        .map_err(|_| CreditError::new("storage_error"))?;
    if let Some((previous, finished, result)) = &state.last {
        if previous == &revision && *finished >= started {
            return result.clone();
        }
    }
    if state
        .rejected_revisions
        .contains(&creds.credential_revision())
    {
        return Err(CreditError::new("no_auth"));
    }
    let scope = creds.scope().ok_or_else(|| CreditError::new("no_auth"))?;
    let conn = crate::db::open_db_connection().map_err(|_| CreditError::new("storage_error"))?;
    let now = Utc::now().timestamp();
    let next = journal::backoff(&conn, &scope).map_err(|_| CreditError::new("storage_error"))?;
    if next > now {
        return Err(CreditError {
            code: "rate_limited",
            retry_after: Some(next - now),
            unknown: false,
        });
    }
    let result = codex::query_reset_credits(creds);
    if let Err(error) = &result {
        if error.code == "no_auth" {
            state.rejected_revisions.insert(creds.credential_revision());
        }
        if let Some(delay) = error.retry_after {
            journal::defer(&conn, &scope, Utc::now().timestamp().saturating_add(delay))
                .map_err(|_| CreditError::new("storage_error"))?;
        }
    }
    state.last = Some((revision, Instant::now(), result.clone()));
    result
}

struct ProductionHost<'a>(&'a tauri::AppHandle);
impl Host for ProductionHost<'_> {
    type ConsumeContext = CodexCredentials;
    fn now(&self) -> DateTime<Local> {
        Local::now()
    }
    fn settings(&self) -> Result<AppSettings, String> {
        load_settings_file_internal()
    }
    fn identity(&self, settings: &AppSettings) -> Result<Identity, String> {
        let creds = codex::resolve_credentials(settings).map_err(|_| "no_auth")?;
        Ok(Identity {
            scope: creds.scope().ok_or("no_auth")?,
            revision: creds.revision(settings),
        })
    }
    fn query(
        &self,
        settings: &AppSettings,
        identity: &Identity,
    ) -> Result<ResetCredits, CreditError> {
        let creds =
            codex::resolve_credentials(settings).map_err(|_| CreditError::new("no_auth"))?;
        if creds.scope().as_deref() != Some(&identity.scope)
            || creds.revision(settings) != identity.revision
        {
            return Err(CreditError::new("account_changed"));
        }
        query_with_backoff(&creds, settings)
    }
    fn prepare_consume(
        &self,
        settings: &AppSettings,
        identity: &Identity,
    ) -> Result<CodexCredentials, CreditError> {
        let creds =
            codex::resolve_credentials(settings).map_err(|_| CreditError::new("no_auth"))?;
        if creds.scope().as_deref() != Some(&identity.scope)
            || creds.revision(settings) != identity.revision
        {
            return Err(CreditError::new("account_changed"));
        }
        Ok(creds)
    }
    fn consume(
        &self,
        creds: CodexCredentials,
        id: &str,
        expiry: Option<i64>,
    ) -> Result<String, CreditError> {
        if expiry.is_some_and(|at| !in_use_window(at, Utc::now().timestamp())) {
            return Err(CreditError::new("missed"));
        }
        let result = codex::consume_with_credentials(&creds, id);
        remember_auth_rejection(&creds.credential_revision(), &result);
        result
    }
    fn notify(
        &self,
        counts: (u32, u32),
        days: u32,
        auto: bool,
        reason: Option<&str>,
    ) -> Result<(), String> {
        let locale = with_db(&self.0.state::<DbState>(), journal::locale)?;
        let notification = self.0.notification();
        if notification
            .permission_state()
            .map_err(|_| "notification_failed")?
            != tauri_plugin_notification::PermissionState::Granted
        {
            return Err("notification_denied".into());
        }
        notification
            .builder()
            .title("SessionHub · Codex")
            .body(notification_body(&locale, counts, days, auto, reason))
            .show()
            .map_err(|_| "notification_failed".into())
    }
    fn changed(&self, identity: &Identity) {
        let app = self.0.clone();
        let identity = identity.clone();
        std::thread::spawn(move || {
            let host = ProductionHost(&app);
            let Ok(settings) = load_settings_internal() else {
                return;
            };
            if !quota_enabled(&settings)
                || host.identity(&settings).ok().as_ref().map(|id| &id.scope)
                    != Some(&identity.scope)
            {
                return;
            }
            super::refresh_cached_quota(
                &app.state::<DbState>(),
                &app.state::<QuotaCache>(),
                &settings,
                Some("codex"),
            );
            crate::emit_quota_snapshots_updated(&app);
            crate::tray_icon::update_tray_from_cache(&app, &settings);
        });
    }
    fn publish(&self, status: &AutomationStatus) {
        let _ = self.0.emit(EVENT, status);
    }
}

pub(crate) fn notification_body(
    locale: &str,
    counts: (u32, u32),
    days: u32,
    auto: bool,
    reason: Option<&str>,
) -> String {
    let en = locale == "en-US";
    let mut text = if en {
        format!("{} reset credits expire today.", counts.0)
    } else {
        format!("今天有 {} 張重置券到期。", counts.0)
    };
    if days > 0 {
        text.push_str(&if en {
            format!(" Another {} expire in the next {days} days.", counts.1)
        } else {
            format!("未來 {days} 天另有 {} 張到期。", counts.1)
        });
    }
    text.push_str(if !auto {
        if en {
            " Auto-use is off; use them before expiry."
        } else {
            "自動使用未開啟，請於到期前手動使用。"
        }
    } else if reason.is_none() {
        if en {
            " Auto-use will be attempted one minute before expiry. Keep SessionHub running."
        } else {
            "將在到期前 1 分鐘嘗試自動使用，請保持 SessionHub 執行。"
        }
    } else {
        match reason {
            Some("selection_unverified") => {
                if en {
                    " Auto-use is paused: selection order is unverified. Handle manually."
                } else {
                    "自動使用暫停：無法確認使用順序，請手動處理。"
                }
            }
            Some("unknown_expiry") => {
                if en {
                    " Auto-use is paused: some expiry times are unknown."
                } else {
                    "自動使用暫停：部分可用券的到期時間不明。"
                }
            }
            Some("inconsistent") => {
                if en {
                    " Auto-use is paused: available count and entries differ."
                } else {
                    "自動使用暫停：可用張數與明細不一致。"
                }
            }
            Some("unknown") => {
                if en {
                    " Auto-use is paused: the previous result is unknown. Refresh to reconcile."
                } else {
                    "自動使用暫停：上次結果不明，請重新整理核對。"
                }
            }
            _ => {
                if en {
                    " Auto-use is paused: no eligible credits were confirmed."
                } else {
                    "自動使用暫停：目前無法確認符合使用條件的券。"
                }
            }
        }
    });
    text
}

pub(crate) fn manual_consume(
    app: &tauri::AppHandle,
    id: &str,
    scope: &str,
) -> Result<String, String> {
    consume_manual_internal(
        &ProductionHost(app),
        &app.state::<DbState>(),
        &app.state::<AutomationState>(),
        id,
        scope,
    )
}

pub(crate) fn set_locale(db: &DbState, locale: &str) -> Result<(), String> {
    with_db(db, |c| journal::save_locale(c, locale))
}

pub(crate) fn quota_updated(app: &tauri::AppHandle) {
    let (Some(state), Some(cache)) = (
        app.try_state::<AutomationState>(),
        app.try_state::<QuotaCache>(),
    ) else {
        return;
    };
    let Ok(snapshots) = cache.snapshots.lock() else {
        return;
    };
    let Some(snapshot) = snapshots.get("codex") else {
        return;
    };
    let next = (
        snapshot.fetched_at.clone(),
        snapshot.account_revision.clone(),
    );
    if let Ok(mut previous) = state.quota_stamp.lock() {
        if previous.as_ref() != Some(&next) {
            *previous = Some(next);
            state.request_refresh();
        }
    };
}

pub(crate) fn start(app: &tauri::AppHandle) -> Result<(), String> {
    with_db(&app.state::<DbState>(), |c| {
        journal::recover(c, Utc::now().timestamp())
    })?;
    let app = app.clone();
    std::thread::spawn(move || {
        let mut engine = Engine::default();
        loop {
            let state = app.state::<AutomationState>();
            let version = state.wake.lock().map(|v| *v).unwrap_or_default();
            let started = Instant::now();
            let result = engine.tick(&ProductionHost(&app), &app.state::<DbState>(), &state);
            let latest_settings = load_settings_file_internal().ok();
            if let Ok(mut current) = state.status.lock() {
                let mut next = match result {
                    Ok(next) => next,
                    Err(reason) => failed_status(latest_settings.as_ref(), &current, reason),
                };
                next.generation = current.generation;
                if next != *current {
                    next.generation = current.generation.wrapping_add(1);
                    *current = next.clone();
                    let _ = app.emit(EVENT, &next);
                }
            }
            if let Ok(guard) = state.wake.lock() {
                let wait =
                    Duration::from_secs(CHECK_INTERVAL_SECONDS).saturating_sub(started.elapsed());
                let _ = state
                    .changed
                    .wait_timeout_while(guard, wait, |v| *v == version);
            };
        }
    });
    Ok(())
}

#[cfg(test)]
#[path = "reset_credit_automation_tests.rs"]
mod tests;
