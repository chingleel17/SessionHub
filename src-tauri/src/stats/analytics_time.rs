use chrono::{DateTime, Datelike, Days, Duration, LocalResult, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;

#[derive(Debug, Clone)]
pub(crate) struct AnalyticsDateRange {
    pub(crate) start_utc: DateTime<Utc>,
    pub(crate) end_utc: DateTime<Utc>,
    pub(crate) time_zone: Tz,
}

pub(crate) fn resolve_analytics_date_range(
    start_date: &str,
    end_date: &str,
    time_zone: &str,
) -> Result<AnalyticsDateRange, String> {
    let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
        .map_err(|_| "invalid date format: startDate".to_string())?;
    let end = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
        .map_err(|_| "invalid date format: endDate".to_string())?;
    if start > end {
        return Err("startDate must be before endDate".to_string());
    }
    let time_zone = time_zone
        .parse::<Tz>()
        .map_err(|_| format!("unsupported timeZone: {time_zone}"))?;
    let end_exclusive = end
        .checked_add_days(Days::new(1))
        .ok_or_else(|| "endDate is outside the supported range".to_string())?;

    Ok(AnalyticsDateRange {
        start_utc: local_date_start_utc(start, time_zone)?,
        end_utc: local_date_start_utc(end_exclusive, time_zone)?,
        time_zone,
    })
}

pub(crate) fn analytics_bucket_label(
    timestamp: DateTime<Utc>,
    time_zone: Tz,
    group_by: &str,
) -> Result<String, String> {
    let local_date = timestamp.with_timezone(&time_zone).date_naive();
    match group_by {
        "day" => Ok(local_date.format("%Y-%m-%d").to_string()),
        "week" => {
            let iso_week = local_date.iso_week();
            Ok(format!("{}-W{:02}", iso_week.year(), iso_week.week()))
        }
        "month" => Ok(local_date.format("%Y-%m").to_string()),
        _ => Err("invalid groupBy value".to_string()),
    }
}

pub(crate) fn local_date_start_utc(
    date: NaiveDate,
    time_zone: Tz,
) -> Result<DateTime<Utc>, String> {
    let midnight = date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| "invalid local date boundary".to_string())?;
    let mut candidate = midnight;
    for _ in 0..=(24 * 60) {
        match time_zone.from_local_datetime(&candidate) {
            LocalResult::Single(value) => return Ok(value.with_timezone(&Utc)),
            LocalResult::Ambiguous(first, second) => {
                return Ok(first.min(second).with_timezone(&Utc));
            }
            LocalResult::None => {
                candidate = candidate
                    .checked_add_signed(Duration::minutes(1))
                    .ok_or_else(|| "local date boundary is outside supported range".to_string())?;
            }
        }
    }
    Err("unable to resolve local date boundary".to_string())
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::{analytics_bucket_label, resolve_analytics_date_range};

    #[test]
    fn local_day_ranges_use_iana_timezone_and_dst_boundaries() {
        let taipei = resolve_analytics_date_range("2026-04-10", "2026-04-10", "Asia/Taipei")
            .expect("resolve Taipei day");
        assert_eq!(taipei.start_utc.to_rfc3339(), "2026-04-09T16:00:00+00:00");
        assert_eq!(taipei.end_utc.to_rfc3339(), "2026-04-10T16:00:00+00:00");
        assert_eq!(taipei.time_zone.name(), "Asia/Taipei");

        let spring =
            resolve_analytics_date_range("2026-03-08", "2026-03-08", "America/Los_Angeles")
                .expect("resolve spring DST day");
        assert_eq!((spring.end_utc - spring.start_utc).num_hours(), 23);
        let fall = resolve_analytics_date_range("2026-11-01", "2026-11-01", "America/Los_Angeles")
            .expect("resolve fall DST day");
        assert_eq!((fall.end_utc - fall.start_utc).num_hours(), 25);
    }

    #[test]
    fn labels_follow_local_calendar_and_iso_week_year() {
        let utc = Utc
            .with_ymd_and_hms(2021, 1, 1, 1, 0, 0)
            .single()
            .expect("UTC time");
        assert_eq!(
            analytics_bucket_label(utc, "America/Los_Angeles".parse().expect("timezone"), "day")
                .expect("day label"),
            "2020-12-31"
        );
        assert_eq!(
            analytics_bucket_label(utc, "Asia/Taipei".parse().expect("timezone"), "week")
                .expect("week label"),
            "2020-W53"
        );
        assert_eq!(
            analytics_bucket_label(
                Utc.with_ymd_and_hms(2024, 2, 29, 12, 0, 0)
                    .single()
                    .expect("leap date"),
                "UTC".parse().expect("UTC timezone"),
                "month"
            )
            .expect("month label"),
            "2024-02"
        );
    }

    #[test]
    fn rejects_invalid_dates_order_and_unknown_timezone() {
        assert!(resolve_analytics_date_range("2026-02-30", "2026-03-01", "UTC").is_err());
        assert!(resolve_analytics_date_range("2026-04-02", "2026-04-01", "UTC").is_err());
        assert!(
            resolve_analytics_date_range("2026-04-01", "2026-04-01", "Pacific Standard Time")
                .is_err()
        );
    }
}
