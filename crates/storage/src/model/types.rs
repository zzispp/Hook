use rust_decimal::Decimal;
use types::model::{PatchField, TieredPricingConfig};
use types::provider::RoutingProfileId;

#[derive(Clone, Debug, PartialEq)]
pub struct GlobalModelUsageRecord {
    pub model_id: String,
    pub count: i64,
    pub user_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlobalModelUserUsageRecord {
    pub user_id: String,
    pub model_id: String,
    pub count: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlobalModelRecordInput {
    pub name: String,
    pub display_name: String,
    pub default_price_per_request: Option<Decimal>,
    pub default_tiered_pricing: TieredPricingConfig,
    pub supported_capabilities: Option<Vec<String>>,
    pub config: Option<serde_json::Value>,
    pub routing_profile_id: Option<RoutingProfileId>,
    pub is_active: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GlobalModelRecordPatch {
    pub display_name: Option<String>,
    pub is_active: Option<bool>,
    pub default_price_per_request: PatchField<Decimal>,
    pub default_tiered_pricing: Option<TieredPricingConfig>,
    pub supported_capabilities: PatchField<Vec<String>>,
    pub config: PatchField<serde_json::Value>,
    pub routing_profile_id: PatchField<RoutingProfileId>,
}
