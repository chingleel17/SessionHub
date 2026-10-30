//! 重置券的純時間與資格判斷；不存取憑證、網路或資料庫。
use chrono::{DateTime, Days, Duration, LocalResult, NaiveTime, TimeZone, Utc};

use crate::types::{AppSettings, ResetCreditEntry, ResetCredits};

pub(crate) const AUTO_USE_LEAD_SECONDS: i64 = 60;
pub(crate) const CHECK_INTERVAL_SECONDS: u64 = 10;
pub(crate) const DISCOVERY_INTERVAL_SECONDS: i64 = 300;

pub(crate) fn quota_enabled(settings: &AppSettings) -> bool {
    settings.enable_quota_monitoring
        && settings
            .quota_enabled_providers
            .iter()
            .any(|p| p == "codex")
}

pub(crate) fn expires_at(credit: &ResetCreditEntry) -> Option<i64> {
    DateTime::parse_from_rfc3339(credit.expires_at.as_deref()?)
        .ok()
        .map(|t| t.timestamp())
}

pub(crate) fn in_use_window(expiry: i64, now: i64) -> bool {
    expiry.saturating_sub(AUTO_USE_LEAD_SECONDS) <= now && now < expiry
}

pub(crate) fn eligible_expiry(credits: &ResetCredits, now: i64) -> Result<i64, &'static str> {
    if credits.available_count == 0 {
        return Err("no_credit");
    }
    let active: Vec<_> = credits
        .credits
        .iter()
        .filter(|c| c.status == "active")
        .collect();
    if active.len() != credits.available_count as usize {
        return Err("inconsistent");
    }
    let mut common = None;
    for credit in active {
        let expiry = expires_at(credit).ok_or("unknown_expiry")?;
        if expiry <= now {
            return Err("missed");
        }
        if common.is_some_and(|previous| previous != expiry) {
            return Err("selection_unverified");
        }
        common = Some(expiry);
    }
    common.ok_or("no_credit")
}

pub(crate) fn reminder_counts<T: TimeZone>(
    credits: &ResetCredits,
    now: &DateTime<T>,
    days: u32,
) -> (u32, u32) {
    let today = now.date_naive();
    let last = today
        .checked_add_days(Days::new(u64::from(days)))
        .unwrap_or(today);
    let mut counts = (0, 0);
    for credit in &credits.credits {
        if credit.status != "active" {
            continue;
        }
        let Some(expiry) = expires_at(credit).and_then(|t| DateTime::<Utc>::from_timestamp(t, 0))
        else {
            continue;
        };
        if expiry.timestamp() <= now.timestamp() {
            continue;
        }
        let date = expiry.with_timezone(&now.timezone()).date_naive();
        if date == today {
            counts.0 += 1;
        } else if today < date && date <= last {
            counts.1 += 1;
        }
    }
    counts
}

pub(crate) fn reminder_due<T: TimeZone>(now: &DateTime<T>, time: &str) -> bool {
    NaiveTime::parse_from_str(time, "%H:%M").is_ok_and(|time| now.time() >= time)
}

pub(crate) fn next_reminder<T: TimeZone>(
    now: &DateTime<T>,
    time: &str,
    checked: bool,
) -> Option<i64> {
    let time = NaiveTime::parse_from_str(time, "%H:%M").ok()?;
    let day = if checked || now.time() > time {
        now.date_naive().checked_add_days(Days::new(1))?
    } else {
        now.date_naive()
    };
    let start = day.and_time(time);
    // 春季跳時不存在的分鐘向後尋找；秋季重複時選第一次，由每日日誌去重。
    for minutes in 0..=180 {
        let local = start.checked_add_signed(Duration::minutes(minutes))?;
        if local.date() != day {
            break;
        }
        match now.timezone().from_local_datetime(&local) {
            LocalResult::Single(t) => return Some(t.timestamp()),
            LocalResult::Ambiguous(first, _) => return Some(first.timestamp()),
            LocalResult::None => {}
        }
    }
    None
}

/// 無穩定券 ID 時保守比對原始授予／到期組合，不能只靠總張數下降解除不明結果。
pub(crate) fn original_credits_resolved(
    original: &[ResetCreditEntry],
    current: &ResetCredits,
    now: i64,
) -> bool {
    let inactive = |c: &ResetCreditEntry| {
        matches!(
            c.status.as_str(),
            "expired" | "used" | "redeemed" | "consumed"
        )
    };
    if original
        .iter()
        .filter(|c| !inactive(c))
        .all(|c| expires_at(c).is_some_and(|t| t <= now))
    {
        return true;
    }
    if current
        .credits
        .iter()
        .filter(|c| c.status == "active")
        .count()
        != current.available_count as usize
    {
        return false;
    }
    original.iter().filter(|c| !inactive(c)).all(|old| {
        expires_at(old).is_some_and(|t| t <= now)
            || !current.credits.iter().any(|new| {
                !inactive(new)
                    && new.granted_at == old.granted_at
                    && new.expires_at == old.expires_at
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn credit(expiry: Option<&str>, status: &str) -> ResetCreditEntry {
        ResetCreditEntry {
            granted_at: Some("2026-10-01T00:00:00Z".into()),
            expires_at: expiry.map(str::to_string),
            status: status.into(),
        }
    }
    #[test]
    fn reset_credit_window_is_exact_and_selection_is_conservative() {
        assert!(!in_use_window(1000, 939));
        assert!(in_use_window(1000, 940));
        assert!(in_use_window(1000, 999));
        assert!(!in_use_window(1000, 1000));
        let mut credits = ResetCredits {
            available_count: 2,
            credits: vec![credit(Some("2026-10-30T10:00:00Z"), "active"); 2],
        };
        assert!(eligible_expiry(&credits, 1).is_ok());
        credits.credits[1].expires_at = Some("2026-11-01T10:00:00Z".into());
        assert_eq!(eligible_expiry(&credits, 1), Err("selection_unverified"));
        credits.credits[1].expires_at = None;
        assert_eq!(eligible_expiry(&credits, 1), Err("unknown_expiry"));
        credits.available_count = 1;
        assert_eq!(eligible_expiry(&credits, 1), Err("inconsistent"));
    }
    #[test]
    fn reset_credit_counts_use_local_calendar_and_exclude_expired_or_unknown() {
        let now = chrono_tz::Asia::Taipei
            .with_ymd_and_hms(2026, 10, 30, 9, 0, 0)
            .unwrap();
        let credits = ResetCredits {
            available_count: 6,
            credits: vec![
                credit(Some("2026-10-30T15:59:00Z"), "active"),
                credit(Some("2026-10-30T16:00:00Z"), "active"),
                credit(Some("2026-11-02T16:00:00Z"), "active"),
                credit(Some("2026-10-30T01:00:00Z"), "active"),
                credit(Some("2026-10-30T12:00:00Z"), "expired"),
                credit(None, "active"),
            ],
        };
        assert_eq!(reminder_counts(&credits, &now, 0), (1, 0));
        assert_eq!(reminder_counts(&credits, &now, 3), (1, 1));
        assert!(reminder_due(&now, "09:00"));
    }
    #[test]
    fn reset_credit_dst_calendar_is_not_a_fixed_24_hours() {
        let tz = chrono_tz::America::New_York;
        let now = tz.with_ymd_and_hms(2026, 3, 8, 0, 0, 0).unwrap();
        let credits = ResetCredits {
            available_count: 1,
            credits: vec![credit(Some("2026-03-09T04:30:00Z"), "active")],
        };
        assert_eq!(reminder_counts(&credits, &now, 0), (0, 0));
        assert_eq!(reminder_counts(&credits, &now, 1), (0, 1));
        let next = next_reminder(&now, "02:30", false).unwrap();
        assert_eq!(
            DateTime::from_timestamp(next, 0)
                .unwrap()
                .with_timezone(&tz)
                .time(),
            NaiveTime::from_hms_opt(3, 0, 0).unwrap()
        );
    }
    #[test]
    fn reset_credit_unknown_without_expiry_does_not_unlock_on_other_batch_expiry() {
        let original = vec![credit(None, "active")];
        let current = ResetCredits {
            available_count: 1,
            credits: original.clone(),
        };
        assert!(!original_credits_resolved(&original, &current, i64::MAX));
        assert!(original_credits_resolved(
            &original,
            &ResetCredits {
                available_count: 0,
                credits: vec![]
            },
            1
        ));
        let unknown = vec![credit(None, "future_unknown_status")];
        assert!(!original_credits_resolved(
            &unknown,
            &ResetCredits {
                available_count: 1,
                credits: unknown.clone()
            },
            100
        ));
        assert!(!original_credits_resolved(
            &original,
            &ResetCredits {
                available_count: 1,
                credits: vec![]
            },
            100
        ));
    }
}
