use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{RoutingMetricSource, RoutingPriorSource, RoutingRequestFeatures};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingProfileId {
    #[default]
    Balanced,
    FirstByte,
    HighTps,
    CostOptimal,
    HighAvailability,
    CacheAffinityPlus,
    FixedPriorityPlus,
    Custom,
}

impl RoutingProfileId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::FirstByte => "first_byte",
            Self::HighTps => "high_tps",
            Self::CostOptimal => "cost_optimal",
            Self::HighAvailability => "high_availability",
            Self::CacheAffinityPlus => "cache_affinity_plus",
            Self::FixedPriorityPlus => "fixed_priority_plus",
            Self::Custom => "custom",
        }
    }
}

impl From<&str> for RoutingProfileId {
    fn from(value: &str) -> Self {
        match value {
            "first_byte" => Self::FirstByte,
            "high_tps" => Self::HighTps,
            "cost_optimal" => Self::CostOptimal,
            "high_availability" => Self::HighAvailability,
            "cache_affinity_plus" => Self::CacheAffinityPlus,
            "fixed_priority_plus" => Self::FixedPriorityPlus,
            "custom" => Self::Custom,
            _ => Self::Balanced,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingRouteState {
    Eligible,
    Warming,
    Degraded,
    Excluded,
    CircuitOpen,
}

impl RoutingRouteState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Eligible => "eligible",
            Self::Warming => "warming",
            Self::Degraded => "degraded",
            Self::Excluded => "excluded",
            Self::CircuitOpen => "circuit_open",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RoutingMetricWindow {
    OneMinute,
    #[default]
    FiveMinutes,
    FifteenMinutes,
    OneHour,
    OneDay,
    SevenDays,
}

impl RoutingMetricWindow {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OneMinute => "1m",
            Self::FiveMinutes => "5m",
            Self::FifteenMinutes => "15m",
            Self::OneHour => "1h",
            Self::OneDay => "24h",
            Self::SevenDays => "7d",
        }
    }

    pub const fn seconds(self) -> i64 {
        match self {
            Self::OneMinute => 60,
            Self::FiveMinutes => 300,
            Self::FifteenMinutes => 900,
            Self::OneHour => 3_600,
            Self::OneDay => 86_400,
            Self::SevenDays => 604_800,
        }
    }
}

impl From<&str> for RoutingMetricWindow {
    fn from(value: &str) -> Self {
        match value {
            "1m" => Self::OneMinute,
            "one_minute" => Self::OneMinute,
            "five_minutes" => Self::FiveMinutes,
            "15m" => Self::FifteenMinutes,
            "fifteen_minutes" => Self::FifteenMinutes,
            "1h" => Self::OneHour,
            "one_hour" => Self::OneHour,
            "24h" => Self::OneDay,
            "one_day" => Self::OneDay,
            "7d" => Self::SevenDays,
            "seven_days" => Self::SevenDays,
            _ => Self::FiveMinutes,
        }
    }
}

impl Serialize for RoutingMetricWindow {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RoutingMetricWindow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let parsed = match value.as_str() {
            "1m" | "one_minute" => Self::OneMinute,
            "5m" | "five_minutes" => Self::FiveMinutes,
            "15m" | "fifteen_minutes" => Self::FifteenMinutes,
            "1h" | "one_hour" => Self::OneHour,
            "24h" | "one_day" => Self::OneDay,
            "7d" | "seven_days" => Self::SevenDays,
            _ => {
                return Err(serde::de::Error::unknown_variant(
                    &value,
                    &[
                        "1m",
                        "5m",
                        "15m",
                        "1h",
                        "24h",
                        "7d",
                        "one_minute",
                        "five_minutes",
                        "fifteen_minutes",
                        "one_hour",
                        "one_day",
                        "seven_days",
                    ],
                ));
            }
        };
        Ok(parsed)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct RouteIdentity {
    pub provider_id: String,
    pub key_id: String,
    pub endpoint_id: String,
    pub global_model_id: String,
    pub client_api_format: String,
    pub provider_api_format: String,
    pub is_stream: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RoutingProfileWeights {
    pub success: f64,
    pub ttfb: f64,
    pub latency: f64,
    pub tps: f64,
    pub cost: f64,
    pub headroom: f64,
    pub priority: f64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RoutingProfile {
    pub id: RoutingProfileId,
    pub name: String,
    pub description: String,
    pub weights: RoutingProfileWeights,
    pub version: String,
    pub min_samples: u64,
    pub exploration_k: f64,
    pub conversion_penalty: f64,
    pub stale_metric_penalty: f64,
    pub affinity_bonus: f64,
    #[serde(default = "default_prior_sample_cap")]
    pub prior_sample_cap: u64,
    #[serde(default = "default_contextual_exploration_enabled")]
    pub contextual_exploration_enabled: bool,
    #[serde(default = "default_ema_alpha")]
    pub ema_alpha: f64,
    #[serde(default = "default_ema_max_freshness_seconds")]
    pub ema_max_freshness_seconds: i64,
    #[serde(default = "default_ema_recent_weight")]
    pub ema_recent_weight: f64,
    #[serde(default = "default_ema_recent_cap")]
    pub ema_recent_cap: f64,
    #[serde(default = "default_exploration_weight")]
    pub exploration_weight: f64,
    #[serde(default = "default_exploration_cap")]
    pub exploration_cap: f64,
    #[serde(default = "default_exploration_min_success_score")]
    pub exploration_min_success_score: f64,
    pub auto_tune_enabled: bool,
    #[serde(default)]
    pub learning: Option<RoutingProfileLearningState>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct RoutingProfileUpsert {
    pub weights: Option<RoutingProfileWeights>,
    pub min_samples: Option<u64>,
    pub exploration_k: Option<f64>,
    pub conversion_penalty: Option<f64>,
    pub stale_metric_penalty: Option<f64>,
    pub affinity_bonus: Option<f64>,
    pub prior_sample_cap: Option<u64>,
    pub contextual_exploration_enabled: Option<bool>,
    pub ema_alpha: Option<f64>,
    pub ema_max_freshness_seconds: Option<i64>,
    pub ema_recent_weight: Option<f64>,
    pub ema_recent_cap: Option<f64>,
    pub exploration_weight: Option<f64>,
    pub exploration_cap: Option<f64>,
    pub exploration_min_success_score: Option<f64>,
    pub auto_tune_enabled: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct RoutingProfilesResponse {
    pub profiles: Vec<RoutingProfile>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RoutingProfileLearningState {
    pub admin_weights: RoutingProfileWeights,
    pub learned_weights: Option<RoutingProfileWeights>,
    pub effective_weights: RoutingProfileWeights,
    pub reward_window: RoutingMetricWindow,
    pub sample_count: u64,
    pub confidence: f64,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RoutingRankingsRequest {
    pub group_code: String,
    pub model: String,
    pub api_format: String,
    pub is_stream: bool,
    #[serde(default)]
    pub window: RoutingMetricWindow,
    #[serde(default)]
    pub include_excluded: bool,
    #[serde(default)]
    pub request_id_seed: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RoutingRankingResponse {
    pub profile: RoutingProfile,
    pub window: RoutingMetricWindow,
    pub selected: Option<RouteIdentity>,
    pub request_id_seed: String,
    pub items: Vec<RouteScoreExplanation>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScoreComponent {
    pub code: String,
    pub label: String,
    pub raw_value: Option<f64>,
    pub normalized_score: f64,
    pub weight: f64,
    pub contribution: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RoutingMetricSnapshot {
    pub request_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub timeout_count: u64,
    pub rate_limited_count: u64,
    pub server_error_count: u64,
    #[serde(default)]
    pub format_conversion_failure_count: u64,
    #[serde(default)]
    pub usage_missing_count: u64,
    #[serde(default)]
    pub stream_abnormal_end_count: u64,
    #[serde(default)]
    pub schema_tool_call_failure_count: u64,
    pub latency_avg_ms: Option<f64>,
    pub ttfb_avg_ms: Option<f64>,
    pub output_tps: Option<f64>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub upstream_total_cost: Option<Decimal>,
    pub total_tokens: u64,
    pub sample_count: u64,
    pub rpm_used: u64,
    pub rpm_limit: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RouteScoreExplanation {
    pub route: RouteIdentity,
    pub provider_name: Option<String>,
    pub key_name: Option<String>,
    pub key_preview: Option<String>,
    pub endpoint_name: Option<String>,
    pub rank: u32,
    pub state: RoutingRouteState,
    pub final_score: f64,
    #[serde(default)]
    pub metric_window: RoutingMetricWindow,
    pub selected_reason: String,
    pub components: Vec<ScoreComponent>,
    pub raw_metrics: RoutingMetricSnapshot,
    pub exclusion_reason: Option<String>,
    pub metric_freshness_seconds: i64,
    #[serde(default)]
    pub metric_source: RoutingMetricSource,
    #[serde(default)]
    pub prior_source: RoutingPriorSource,
    #[serde(default)]
    pub prior_sample_count: u64,
    #[serde(default)]
    pub effective_sample_count: u64,
    #[serde(default)]
    pub routing_context_key: Option<String>,
    #[serde(default)]
    pub route_config_fingerprint: Option<String>,
    #[serde(default)]
    pub price_config_fingerprint: Option<String>,
    #[serde(default)]
    pub request_features: RoutingRequestFeatures,
}

pub const fn default_prior_sample_cap() -> u64 {
    20
}

pub const fn default_contextual_exploration_enabled() -> bool {
    true
}

pub const fn default_ema_alpha() -> f64 {
    0.35
}

pub const fn default_ema_max_freshness_seconds() -> i64 {
    300
}

pub const fn default_ema_recent_weight() -> f64 {
    0.35
}

pub const fn default_ema_recent_cap() -> f64 {
    8.0
}

pub const fn default_exploration_weight() -> f64 {
    0.05
}

pub const fn default_exploration_cap() -> f64 {
    5.0
}

pub const fn default_exploration_min_success_score() -> f64 {
    65.0
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RoutingDecisionResponse {
    pub request_id: String,
    pub profile_id: RoutingProfileId,
    pub profile_version: String,
    pub selected: Option<RouteIdentity>,
    pub candidates: Vec<RouteScoreExplanation>,
    pub created_at: String,
}
