use serde::{Deserialize, Serialize};

mod analytics;
mod snapshot;

pub use analytics::*;
pub use snapshot::*;

pub const MAX_SERIES_POINTS: usize = 720;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PerformanceMonitoringRange {
    Realtime,
    #[default]
    Today,
    #[serde(rename = "7d")]
    SevenDays,
    #[serde(rename = "30d")]
    ThirtyDays,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotGranularity {
    Minute,
    Hour,
    Day,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EffectiveTimeRange {
    pub started_at: String,
    pub ended_at: String,
}

impl SnapshotGranularity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Minute => "minute",
            Self::Hour => "hour",
            Self::Day => "day",
        }
    }

    pub const fn bucket_seconds(self) -> i64 {
        match self {
            Self::Minute => 60,
            Self::Hour => 3_600,
            Self::Day => 86_400,
        }
    }
}

impl TryFrom<&str> for SnapshotGranularity {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "minute" => Ok(Self::Minute),
            "hour" => Ok(Self::Hour),
            "day" => Ok(Self::Day),
            _ => Err(format!("unsupported snapshot granularity: {value}")),
        }
    }
}
