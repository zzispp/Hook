use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderSchedulingMode {
    FixedOrder,
    CacheAffinity,
    LoadBalance,
}

impl ProviderSchedulingMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FixedOrder => "fixed_order",
            Self::CacheAffinity => "cache_affinity",
            Self::LoadBalance => "load_balance",
        }
    }
}

impl Default for ProviderSchedulingMode {
    fn default() -> Self {
        Self::FixedOrder
    }
}

impl From<&str> for ProviderSchedulingMode {
    fn from(value: &str) -> Self {
        match value {
            "cache_affinity" => Self::CacheAffinity,
            "load_balance" => Self::LoadBalance,
            _ => Self::FixedOrder,
        }
    }
}
