#[derive(Clone, Copy)]
pub(super) enum BucketGranularity {
    Minute,
    Hour,
    Day,
}

impl BucketGranularity {
    pub(super) fn all() -> [Self; 3] {
        [Self::Minute, Self::Hour, Self::Day]
    }

    pub(super) fn bounds(self, value: time::OffsetDateTime) -> BucketBounds {
        let started_at = match self {
            Self::Minute => floor_minute(value),
            Self::Hour => floor_hour(value),
            Self::Day => value.date().midnight().assume_utc(),
        };
        BucketBounds {
            granularity: self.as_str(),
            started_at,
            ended_at: started_at + self.duration(),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Minute => "minute",
            Self::Hour => "hour",
            Self::Day => "day",
        }
    }

    fn duration(self) -> time::Duration {
        match self {
            Self::Minute => time::Duration::minutes(1),
            Self::Hour => time::Duration::hours(1),
            Self::Day => time::Duration::days(1),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct BucketBounds {
    pub(super) granularity: &'static str,
    pub(super) started_at: time::OffsetDateTime,
    pub(super) ended_at: time::OffsetDateTime,
}

fn floor_minute(value: time::OffsetDateTime) -> time::OffsetDateTime {
    value.replace_second(0).and_then(|v| v.replace_nanosecond(0)).unwrap_or(value)
}

fn floor_hour(value: time::OffsetDateTime) -> time::OffsetDateTime {
    floor_minute(value).replace_minute(0).unwrap_or(value)
}
