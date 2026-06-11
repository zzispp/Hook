use types::recharge::{
    RECHARGE_ORDER_DATE_PRESET_CUSTOM, RECHARGE_ORDER_DATE_PRESET_LAST_7_DAYS, RECHARGE_ORDER_DATE_PRESET_LAST_30_DAYS, RECHARGE_ORDER_DATE_PRESET_TODAY,
    RechargeOrderDatePreset, RechargeOrderListFilters,
};

use super::{RechargeError, RechargeResult};

const ONE_DAY: i64 = 1;
const LAST_7_DAYS: i64 = 7;
const LAST_30_DAYS: i64 = 30;

pub(super) fn resolve_order_filters(filters: RechargeOrderListFilters, now_utc: time::OffsetDateTime) -> RechargeResult<RechargeOrderListFilters> {
    let Some(window) = date_window(&filters, now_utc)? else {
        return Ok(filters);
    };
    Ok(RechargeOrderListFilters {
        paid_at_start: Some(window.started_at),
        paid_at_end: Some(window.ended_at),
        ..filters
    })
}

fn date_window(filters: &RechargeOrderListFilters, now_utc: time::OffsetDateTime) -> RechargeResult<Option<DateWindow>> {
    match filters.date_preset {
        RechargeOrderDatePreset::All => Ok(None),
        RechargeOrderDatePreset::Today => preset_window(now_utc, filters.tz_offset_minutes, RECHARGE_ORDER_DATE_PRESET_TODAY, ONE_DAY).map(Some),
        RechargeOrderDatePreset::Last7Days => preset_window(now_utc, filters.tz_offset_minutes, RECHARGE_ORDER_DATE_PRESET_LAST_7_DAYS, LAST_7_DAYS).map(Some),
        RechargeOrderDatePreset::Last30Days => {
            preset_window(now_utc, filters.tz_offset_minutes, RECHARGE_ORDER_DATE_PRESET_LAST_30_DAYS, LAST_30_DAYS).map(Some)
        }
        RechargeOrderDatePreset::Custom => custom_window(filters).map(Some),
    }
}

fn preset_window(now_utc: time::OffsetDateTime, offset_minutes: i32, field: &str, days: i64) -> RechargeResult<DateWindow> {
    let today = now_utc.to_offset(utc_offset(offset_minutes)?).date();
    let start_date = today - time::Duration::days(days - ONE_DAY);
    date_window_from_dates(start_date, today, offset_minutes, field)
}

fn custom_window(filters: &RechargeOrderListFilters) -> RechargeResult<DateWindow> {
    let start_date = parse_date(required_date(filters.start_date.as_deref(), "start_date")?, "start_date")?;
    let end_date = parse_date(required_date(filters.end_date.as_deref(), "end_date")?, "end_date")?;
    if start_date > end_date {
        return Err(RechargeError::InvalidInput("start_date must be before or equal to end_date".into()));
    }
    date_window_from_dates(start_date, end_date, filters.tz_offset_minutes, RECHARGE_ORDER_DATE_PRESET_CUSTOM)
}

fn date_window_from_dates(start_date: time::Date, end_date: time::Date, offset_minutes: i32, field: &str) -> RechargeResult<DateWindow> {
    Ok(DateWindow {
        started_at: local_date_start_utc(start_date, offset_minutes, field)?,
        ended_at: local_date_start_utc(next_day(end_date, "end_date")?, offset_minutes, field)?,
    })
}

fn required_date<'a>(value: Option<&'a str>, field: &str) -> RechargeResult<&'a str> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| RechargeError::InvalidInput(format!("{field} is required for custom range")))
}

fn parse_date(value: &str, field: &str) -> RechargeResult<time::Date> {
    time::Date::parse(value, &time::format_description::well_known::Iso8601::DEFAULT)
        .map_err(|error| RechargeError::InvalidInput(format!("{field} must use YYYY-MM-DD: {error}")))
}

fn next_day(date: time::Date, field: &str) -> RechargeResult<time::Date> {
    date.next_day().ok_or_else(|| RechargeError::InvalidInput(format!("{field} is out of range")))
}

fn local_date_start_utc(date: time::Date, offset_minutes: i32, field: &str) -> RechargeResult<time::OffsetDateTime> {
    let offset = utc_offset(offset_minutes)?;
    date.with_hms(0, 0, 0)
        .map(|value| value.assume_offset(offset).to_offset(time::UtcOffset::UTC))
        .map_err(|error| RechargeError::InvalidInput(format!("invalid {field} boundary: {error}")))
}

fn utc_offset(offset_minutes: i32) -> RechargeResult<time::UtcOffset> {
    let seconds = offset_minutes
        .checked_mul(60)
        .ok_or_else(|| RechargeError::InvalidInput("tz_offset_minutes exceeds supported range".into()))?;
    time::UtcOffset::from_whole_seconds(seconds).map_err(|_| RechargeError::InvalidInput("tz_offset_minutes must be between -1439 and 1439".into()))
}

struct DateWindow {
    started_at: time::OffsetDateTime,
    ended_at: time::OffsetDateTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_preset_leaves_paid_window_empty() {
        let filters = resolve_order_filters(RechargeOrderListFilters::default(), now()).unwrap();

        assert_eq!(filters.paid_at_start, None);
        assert_eq!(filters.paid_at_end, None);
    }

    #[test]
    fn today_uses_local_day_boundaries() {
        let filters = resolve_order_filters(
            RechargeOrderListFilters {
                date_preset: RechargeOrderDatePreset::Today,
                tz_offset_minutes: 480,
                ..Default::default()
            },
            now(),
        )
        .unwrap();

        assert_eq!(timestamp(filters.paid_at_start), "2026-06-10T16:00:00Z");
        assert_eq!(timestamp(filters.paid_at_end), "2026-06-11T16:00:00Z");
    }

    #[test]
    fn last_7_and_30_days_include_today() {
        let seven_days = resolve_order_filters(preset(RechargeOrderDatePreset::Last7Days), now()).unwrap();
        let thirty_days = resolve_order_filters(preset(RechargeOrderDatePreset::Last30Days), now()).unwrap();

        assert_eq!(timestamp(seven_days.paid_at_start), "2026-06-04T16:00:00Z");
        assert_eq!(timestamp(seven_days.paid_at_end), "2026-06-11T16:00:00Z");
        assert_eq!(timestamp(thirty_days.paid_at_start), "2026-05-12T16:00:00Z");
        assert_eq!(timestamp(thirty_days.paid_at_end), "2026-06-11T16:00:00Z");
    }

    #[test]
    fn custom_requires_valid_ordered_dates() {
        let missing = resolve_order_filters(preset(RechargeOrderDatePreset::Custom), now()).unwrap_err();
        let invalid = resolve_order_filters(custom("bad", "2026-06-11"), now()).unwrap_err();
        let reversed = resolve_order_filters(custom("2026-06-12", "2026-06-11"), now()).unwrap_err();

        assert_eq!(missing.to_string(), "invalid input: start_date is required for custom range");
        assert!(invalid.to_string().contains("start_date must use YYYY-MM-DD"));
        assert_eq!(reversed.to_string(), "invalid input: start_date must be before or equal to end_date");
    }

    #[test]
    fn custom_uses_inclusive_local_dates() {
        let filters = resolve_order_filters(custom("2026-06-01", "2026-06-03"), now()).unwrap();

        assert_eq!(timestamp(filters.paid_at_start), "2026-05-31T16:00:00Z");
        assert_eq!(timestamp(filters.paid_at_end), "2026-06-03T16:00:00Z");
    }

    fn preset(date_preset: RechargeOrderDatePreset) -> RechargeOrderListFilters {
        RechargeOrderListFilters {
            date_preset,
            tz_offset_minutes: 480,
            ..Default::default()
        }
    }

    fn custom(start_date: &str, end_date: &str) -> RechargeOrderListFilters {
        RechargeOrderListFilters {
            start_date: Some(start_date.into()),
            end_date: Some(end_date.into()),
            ..preset(RechargeOrderDatePreset::Custom)
        }
    }

    fn now() -> time::OffsetDateTime {
        time::Date::from_calendar_date(2026, time::Month::June, 11)
            .unwrap()
            .with_hms(8, 30, 0)
            .unwrap()
            .assume_utc()
    }

    fn timestamp(value: Option<time::OffsetDateTime>) -> String {
        value.unwrap().format(&time::format_description::well_known::Rfc3339).unwrap()
    }
}
