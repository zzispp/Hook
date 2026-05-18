const HOURS_PER_DAY: u16 = 24;
const MINUTES_PER_HOUR: u16 = 60;
const TIME_TEXT_LENGTH: usize = 5;
const TIME_SEPARATOR_INDEX: usize = 2;

pub fn parse_provider_key_time_range_minute(value: &str) -> Option<u16> {
    let bytes = value.as_bytes();
    if bytes.len() != TIME_TEXT_LENGTH || bytes[TIME_SEPARATOR_INDEX] != b':' {
        return None;
    }
    let hour = parse_two_digits(&bytes[0..2])?;
    let minute = parse_two_digits(&bytes[3..5])?;
    provider_key_minute_of_day(hour, minute)
}

pub const fn provider_key_minute_of_day(hour: u16, minute: u16) -> Option<u16> {
    if hour >= HOURS_PER_DAY || minute >= MINUTES_PER_HOUR {
        return None;
    }
    Some(hour * MINUTES_PER_HOUR + minute)
}

pub const fn provider_key_time_range_contains(now_minute: u16, start_minute: u16, end_minute: u16) -> bool {
    if start_minute == end_minute {
        return false;
    }
    if start_minute < end_minute {
        return now_minute >= start_minute && now_minute < end_minute;
    }
    now_minute >= start_minute || now_minute < end_minute
}

fn parse_two_digits(bytes: &[u8]) -> Option<u16> {
    let [left, right] = bytes else {
        return None;
    };
    Some(u16::from(digit(*left)?) * 10 + u16::from(digit(*right)?))
}

fn digit(byte: u8) -> Option<u8> {
    if !byte.is_ascii_digit() {
        return None;
    }
    Some(byte - b'0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_provider_key_time_range_minute_accepts_hh_mm() {
        assert_eq!(parse_provider_key_time_range_minute("08:30"), provider_key_minute_of_day(8, 30));
    }

    #[test]
    fn parse_provider_key_time_range_minute_rejects_invalid_time() {
        assert_eq!(parse_provider_key_time_range_minute("8:30"), None);
        assert_eq!(parse_provider_key_time_range_minute("24:00"), None);
        assert_eq!(parse_provider_key_time_range_minute("10:60"), None);
    }

    #[test]
    fn provider_key_time_range_contains_supports_cross_midnight() {
        let start = provider_key_minute_of_day(22, 0).unwrap();
        let end = provider_key_minute_of_day(6, 0).unwrap();

        assert!(provider_key_time_range_contains(provider_key_minute_of_day(23, 0).unwrap(), start, end));
        assert!(provider_key_time_range_contains(provider_key_minute_of_day(5, 0).unwrap(), start, end));
        assert!(!provider_key_time_range_contains(provider_key_minute_of_day(12, 0).unwrap(), start, end));
    }
}
