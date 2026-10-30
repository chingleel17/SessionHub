//! 重置券操作日誌與配額快取分離；所有函式只執行短時間資料庫操作。
use rusqlite::{params, Connection, OptionalExtension};

use crate::types::{ResetCreditEntry, ResetCredits};

pub(crate) fn init(conn: &Connection) -> Result<(), String> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS reset_credit_reminder_checks (
            account_scope TEXT NOT NULL, local_date TEXT NOT NULL,
            state TEXT NOT NULL DEFAULT 'waiting', attempts INTEGER NOT NULL DEFAULT 0,
            next_at INTEGER NOT NULL DEFAULT 0, today_count INTEGER NOT NULL DEFAULT 0,
            future_count INTEGER NOT NULL DEFAULT 0, detail TEXT, updated_at INTEGER NOT NULL,
            PRIMARY KEY(account_scope, local_date));
        CREATE TABLE IF NOT EXISTS reset_credit_auto_batches (
            account_scope TEXT NOT NULL, expires_at INTEGER NOT NULL,
            state TEXT NOT NULL DEFAULT 'waiting', get_attempts INTEGER NOT NULL DEFAULT 0,
            next_at INTEGER NOT NULL DEFAULT 0, max_count INTEGER NOT NULL,
            used INTEGER NOT NULL DEFAULT 0, detail TEXT,
            PRIMARY KEY(account_scope, expires_at));
        CREATE TABLE IF NOT EXISTS reset_credit_consume_attempts (
            request_id TEXT PRIMARY KEY, account_scope TEXT NOT NULL, origin TEXT NOT NULL,
            batch_expiry INTEGER, ordinal INTEGER, observed TEXT NOT NULL,
            state TEXT NOT NULL, result TEXT, released INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);
        CREATE UNIQUE INDEX IF NOT EXISTS reset_credit_batch_ordinal
            ON reset_credit_consume_attempts(account_scope, batch_expiry, ordinal) WHERE origin = 'automatic';
        CREATE UNIQUE INDEX IF NOT EXISTS reset_credit_one_unresolved
            ON reset_credit_consume_attempts(account_scope)
            WHERE state IN ('pending','unknown') AND released = 0;
        CREATE TABLE IF NOT EXISTS reset_credit_query_backoff (
            account_scope TEXT NOT NULL, endpoint TEXT NOT NULL, next_at INTEGER NOT NULL,
            PRIMARY KEY(account_scope, endpoint));
        CREATE TABLE IF NOT EXISTS reset_credit_preferences (key TEXT PRIMARY KEY, value TEXT NOT NULL);
    ").map_err(|e| format!("重置券日誌初始化失敗: {e}"))
}

pub(crate) fn recover(conn: &Connection, now: i64) -> Result<(), String> {
    conn.execute("UPDATE reset_credit_consume_attempts SET state='unknown', result='unknown', updated_at=?1 WHERE state='pending'", [now]).map_err(|e| e.to_string())?;
    conn.execute("UPDATE reset_credit_auto_batches SET state='terminal', detail='interrupted' WHERE state='verifying'", []).map_err(|e| e.to_string())?;
    conn.execute("UPDATE reset_credit_reminder_checks SET state='failed', detail='transient_error' WHERE state='waiting' AND attempts>=3", []).map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn backoff(conn: &Connection, scope: &str) -> Result<i64, String> {
    conn.query_row("SELECT next_at FROM reset_credit_query_backoff WHERE account_scope=?1 AND endpoint='reset_credits'", [scope], |r| r.get(0))
        .optional().map(|v| v.unwrap_or(0)).map_err(|e| e.to_string())
}

pub(crate) fn defer(conn: &Connection, scope: &str, next: i64) -> Result<(), String> {
    conn.execute("INSERT INTO reset_credit_query_backoff(account_scope,endpoint,next_at) VALUES(?1,'reset_credits',?2)
        ON CONFLICT(account_scope,endpoint) DO UPDATE SET next_at=MAX(next_at,excluded.next_at)", params![scope, next])
        .map(|_| ()).map_err(|e| e.to_string())
}

#[derive(Debug, Clone)]
pub(crate) struct Reminder {
    pub(crate) state: String,
    pub(crate) next_at: i64,
    pub(crate) detail: Option<String>,
}

pub(crate) fn reminder(
    conn: &Connection,
    scope: &str,
    date: &str,
) -> Result<Option<Reminder>, String> {
    conn.query_row("SELECT state,next_at,detail FROM reset_credit_reminder_checks WHERE account_scope=?1 AND local_date=?2", params![scope,date],
        |r| Ok(Reminder { state:r.get(0)?, next_at:r.get(1)?, detail:r.get(2)? })).optional().map_err(|e| e.to_string())
}

/// 先持久化查詢名額與保守退避，程序中止也不能重新獲得三次機會。
pub(crate) fn claim_reminder_query(
    conn: &Connection,
    scope: &str,
    date: &str,
    now: i64,
) -> Result<bool, String> {
    conn.execute("INSERT OR IGNORE INTO reset_credit_reminder_checks(account_scope,local_date,updated_at) VALUES(?1,?2,?3)", params![scope,date,now]).map_err(|e| e.to_string())?;
    conn.execute("UPDATE reset_credit_reminder_checks SET attempts=attempts+1,
        next_at=?3 + CASE WHEN attempts=0 THEN 40 ELSE 130 END, updated_at=?3
        WHERE account_scope=?1 AND local_date=?2 AND state='waiting' AND attempts<3 AND next_at<=?3", params![scope,date,now])
        .map(|n| n == 1).map_err(|e| e.to_string())
}

pub(crate) fn reminder_query_failed(
    conn: &Connection,
    scope: &str,
    date: &str,
    code: &str,
    transient: bool,
    now: i64,
) -> Result<(), String> {
    conn.execute("UPDATE reset_credit_reminder_checks SET state=CASE WHEN ?4 AND attempts<3 THEN 'waiting' ELSE 'failed' END,
        detail=?3, updated_at=?5 WHERE account_scope=?1 AND local_date=?2 AND state='waiting'", params![scope,date,code,transient,now])
        .map(|_| ()).map_err(|e| e.to_string())
}

pub(crate) fn finish_reminder(
    conn: &Connection,
    scope: &str,
    date: &str,
    counts: (u32, u32),
    state: &str,
    detail: Option<&str>,
    now: i64,
) -> Result<(), String> {
    conn.execute("UPDATE reset_credit_reminder_checks SET state=?3,today_count=?4,future_count=?5,detail=?6,updated_at=?7
        WHERE account_scope=?1 AND local_date=?2", params![scope,date,state,i64::from(counts.0),i64::from(counts.1),detail,now])
        .map(|_| ()).map_err(|e| e.to_string())
}

#[derive(Debug, Clone)]
pub(crate) struct Batch {
    pub(crate) state: String,
    pub(crate) used: u32,
    pub(crate) max_count: u32,
    pub(crate) detail: Option<String>,
}

pub(crate) fn batch(conn: &Connection, scope: &str, expiry: i64) -> Result<Option<Batch>, String> {
    conn.query_row("SELECT state,used,max_count,detail FROM reset_credit_auto_batches WHERE account_scope=?1 AND expires_at=?2", params![scope,expiry],
        |r| Ok(Batch {state:r.get(0)?, used:r.get::<_,i64>(1)? as u32, max_count:r.get::<_,i64>(2)? as u32, detail:r.get(3)?})).optional().map_err(|e| e.to_string())
}

pub(crate) fn claim_auto_query(
    conn: &Connection,
    scope: &str,
    expiry: i64,
    count: u32,
    now: i64,
) -> Result<bool, String> {
    conn.execute("INSERT OR IGNORE INTO reset_credit_auto_batches(account_scope,expires_at,max_count) VALUES(?1,?2,?3)", params![scope,expiry,i64::from(count)]).map_err(|e| e.to_string())?;
    conn.execute("UPDATE reset_credit_auto_batches SET get_attempts=get_attempts+1,
        next_at=?3 + CASE WHEN get_attempts=0 THEN 20 ELSE 30 END
        WHERE account_scope=?1 AND expires_at=?2 AND state='waiting' AND get_attempts<3 AND next_at<=?3", params![scope,expiry,now])
        .map(|n| n == 1).map_err(|e| e.to_string())
}

pub(crate) fn finish_batch(
    conn: &Connection,
    scope: &str,
    expiry: i64,
    detail: &str,
) -> Result<(), String> {
    conn.execute("UPDATE reset_credit_auto_batches SET state='terminal',detail=?3 WHERE account_scope=?1 AND expires_at=?2", params![scope,expiry,detail])
        .map(|_| ()).map_err(|e| e.to_string())
}

#[derive(Debug, Clone)]
pub(crate) struct Attempt {
    pub(crate) scope: String,
    pub(crate) origin: String,
    pub(crate) expiry: Option<i64>,
    pub(crate) state: String,
    pub(crate) result: Option<String>,
    pub(crate) updated_at: i64,
}

pub(crate) fn attempt(conn: &Connection, id: &str) -> Result<Option<Attempt>, String> {
    conn.query_row("SELECT account_scope,origin,batch_expiry,state,result,updated_at FROM reset_credit_consume_attempts WHERE request_id=?1", [id],
        |r| Ok(Attempt { scope:r.get(0)?, origin:r.get(1)?, expiry:r.get(2)?, state:r.get(3)?, result:r.get(4)?, updated_at:r.get(5)? })).optional().map_err(|e| e.to_string())
}

pub(crate) fn latest_attempt(conn: &Connection, scope: &str) -> Result<Option<Attempt>, String> {
    let id: Option<String> = conn.query_row("SELECT request_id FROM reset_credit_consume_attempts WHERE account_scope=?1 AND origin='automatic' ORDER BY updated_at DESC,rowid DESC LIMIT 1", [scope], |r| r.get(0)).optional().map_err(|e| e.to_string())?;
    id.map(|id| attempt(conn, &id))
        .transpose()
        .map(Option::flatten)
}

pub(crate) fn reconcile(
    conn: &Connection,
    scope: &str,
    resolved: impl FnOnce(&[ResetCreditEntry]) -> bool,
) -> Result<bool, String> {
    let pending: Option<(String, String)> = conn
        .query_row(
            "SELECT request_id,observed FROM reset_credit_consume_attempts
        WHERE account_scope=?1 AND state IN ('pending','unknown') AND released=0",
            [scope],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let Some((id, json)) = pending else {
        return Ok(true);
    };
    let original: Vec<ResetCreditEntry> = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    if resolved(&original) {
        conn.execute(
            "UPDATE reset_credit_consume_attempts SET released=1 WHERE request_id=?1",
            [id],
        )
        .map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub(crate) fn limit_batch(
    conn: &Connection,
    scope: &str,
    expiry: i64,
    count: u32,
) -> Result<(), String> {
    conn.execute("UPDATE reset_credit_auto_batches SET max_count=MIN(max_count,?3) WHERE account_scope=?1 AND expires_at=?2 AND used=0",params![scope,expiry,i64::from(count)]).map(|_|()).map_err(|e|e.to_string())
}

pub(crate) fn locale(conn: &Connection) -> Result<String, String> {
    conn.query_row(
        "SELECT value FROM reset_credit_preferences WHERE key='locale'",
        [],
        |r| r.get(0),
    )
    .optional()
    .map(|v| v.unwrap_or_else(|| "zh-TW".into()))
    .map_err(|e| e.to_string())
}

pub(crate) fn save_locale(conn: &Connection, locale: &str) -> Result<(), String> {
    if !matches!(locale, "zh-TW" | "en-US") {
        return Err("不支援的通知語言".into());
    }
    conn.execute("INSERT INTO reset_credit_preferences(key,value) VALUES('locale',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[locale]).map(|_|()).map_err(|e|e.to_string())
}

pub(crate) fn begin_attempt(
    conn: &Connection,
    scope: &str,
    id: &str,
    expiry: Option<i64>,
    credits: &ResetCredits,
    now: i64,
) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let ordinal = if let Some(expiry) = expiry {
        let batch = batch(&tx, scope, expiry)?.ok_or("找不到到期批次")?;
        if batch.state != "waiting" || batch.used >= batch.max_count {
            return Err("批次已處理".into());
        }
        tx.execute("UPDATE reset_credit_auto_batches SET used=used+1,state='verifying' WHERE account_scope=?1 AND expires_at=?2", params![scope,expiry]).map_err(|e| e.to_string())?;
        Some(i64::from(batch.used + 1))
    } else {
        None
    };
    let observed = serde_json::to_string(&credits.credits).map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO reset_credit_consume_attempts(request_id,account_scope,origin,batch_expiry,ordinal,observed,state,created_at,updated_at)
        VALUES(?1,?2,?3,?4,?5,?6,'pending',?7,?7)", params![id,scope,if expiry.is_some(){"automatic"}else{"manual"},expiry,ordinal,observed,now])
        .map_err(|_| "此帳號已有進行中或結果不明的重置請求".to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

pub(crate) fn finish_attempt(
    conn: &Connection,
    id: &str,
    result: &str,
    unknown: bool,
    now: i64,
) -> Result<(), String> {
    conn.execute("UPDATE reset_credit_consume_attempts SET state=?2,result=?3,updated_at=?4 WHERE request_id=?1 AND state='pending'",
        params![id,if unknown {"unknown"} else {"complete"},result,now]).map(|_| ()).map_err(|e| e.to_string())
}

pub(crate) fn verified_batch(conn: &Connection, scope: &str, expiry: i64) -> Result<(), String> {
    conn.execute("UPDATE reset_credit_auto_batches SET state='waiting',next_at=0 WHERE account_scope=?1 AND expires_at=?2 AND state='verifying'", params![scope,expiry])
        .map(|_| ()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quota::reset_credit_policy::original_credits_resolved;
    fn credits() -> ResetCredits {
        ResetCredits {
            available_count: 1,
            credits: vec![ResetCreditEntry {
                status: "active".into(),
                granted_at: None,
                expires_at: None,
            }],
        }
    }
    #[test]
    fn reset_credit_journal_recovery_preserves_unknown_and_identity() {
        let conn = Connection::open_in_memory().expect("db");
        init(&conn).expect("schema");
        init(&conn).expect("reentrant");
        begin_attempt(&conn, "a", "one", None, &credits(), 10).expect("claim");
        assert!(begin_attempt(&conn, "a", "two", None, &credits(), 11).is_err());
        recover(&conn, 12).expect("recover");
        assert_eq!(
            attempt(&conn, "one").expect("read").expect("one").state,
            "unknown"
        );
        assert!(!reconcile(&conn, "a", |original| original_credits_resolved(
            original,
            &credits(),
            100
        ))
        .expect("still unknown"));
        assert!(reconcile(&conn, "a", |original| original_credits_resolved(
            original,
            &ResetCredits {
                available_count: 0,
                credits: vec![]
            },
            100
        ))
        .expect("resolved"));
        assert!(begin_attempt(&conn, "b", "one", None, &credits(), 20).is_err());
        begin_attempt(&conn, "a", "two", None, &credits(), 21).expect("unlocked");
        assert_eq!(
            attempt(&conn, "one").expect("read").expect("one").scope,
            "a"
        );
    }
    #[test]
    fn reset_credit_reminder_retry_survives_restart_and_is_bounded() {
        let conn = Connection::open_in_memory().expect("db");
        init(&conn).expect("schema");
        assert!(claim_reminder_query(&conn, "a", "2026-10-30", 100).expect("first"));
        recover(&conn, 101).expect("restart");
        assert!(!claim_reminder_query(&conn, "a", "2026-10-30", 101).expect("backoff"));
        assert!(claim_reminder_query(&conn, "a", "2026-10-30", 140).expect("second"));
        assert!(claim_reminder_query(&conn, "a", "2026-10-30", 270).expect("third"));
        assert!(!claim_reminder_query(&conn, "a", "2026-10-30", 1000).expect("exhausted"));
        finish_reminder(&conn, "a", "2026-10-30", (1, 0), "claimed", None, 1000)
            .expect("claim notification");
        recover(&conn, 1001).expect("restart");
        assert!(!claim_reminder_query(&conn, "a", "2026-10-30", 1001).expect("no replay"));
        defer(&conn, "a", 2000).expect("defer");
        defer(&conn, "a", 1500).expect("keep longer");
        assert_eq!(backoff(&conn, "a").expect("read"), 2000);
    }
    #[test]
    fn reset_credit_batch_cap_and_terminal_state_are_durable() {
        let conn = Connection::open_in_memory().expect("db");
        init(&conn).expect("schema");
        assert!(claim_auto_query(&conn, "a", 1000, 1, 940).expect("query"));
        begin_attempt(&conn, "a", "first", Some(1000), &credits(), 940).expect("post");
        finish_attempt(&conn, "first", "reset", false, 941).expect("result");
        verified_batch(&conn, "a", 1000).expect("verified");
        assert!(begin_attempt(&conn, "a", "second", Some(1000), &credits(), 942).is_err());
        finish_batch(&conn, "a", 1000, "reset").expect("terminal");
        assert!(!claim_auto_query(&conn, "a", 1000, 2, 950).expect("no recreate"));
    }
}
