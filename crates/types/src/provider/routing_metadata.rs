use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingMetricSource {
    #[default]
    Unknown,
    Exact,
    WindowFallback,
    Prior,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingPriorSource {
    #[default]
    Unknown,
    ExactRoute,
    ProviderModelFormat,
    ProviderModel,
    Provider,
    Neutral,
}
