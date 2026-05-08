use rust_decimal::Decimal;
use types::model::{GlobalModelResponse, ModelCatalogProviderDetail, ModelPriceRange, PricingTier, TieredPricingConfig};

#[derive(Clone, Debug, toasty::Model)]
#[table = "global_models"]
pub struct GlobalModelRecord {
    #[key]
    #[column(type = varchar(36))]
    pub id: String,
    #[unique]
    #[column(type = varchar(100))]
    pub name: String,
    #[column(type = varchar(100))]
    pub display_name: String,
    #[column(type = numeric(20, 8))]
    pub default_price_per_request: Option<Decimal>,
    #[serialize(json)]
    pub default_tiered_pricing: TieredPricingConfig,
    #[serialize(json, nullable)]
    pub supported_capabilities: Option<Vec<String>>,
    #[serialize(json, nullable)]
    pub config: Option<serde_json::Value>,
    pub is_active: bool,
    #[index]
    pub usage_count: i64,
    #[auto]
    #[column(type = timestamp(6))]
    pub created_at: jiff::Timestamp,
    #[auto]
    #[column(type = timestamp(6))]
    pub updated_at: jiff::Timestamp,
}

#[derive(Clone, Debug, toasty::Model)]
#[table = "models"]
pub struct ModelRecord {
    #[key]
    #[column(type = varchar(36))]
    pub id: String,
    #[index]
    #[column(type = varchar(36))]
    pub provider_id: String,
    #[index]
    #[column(type = varchar(36))]
    pub global_model_id: String,
    #[column(type = varchar(200))]
    pub provider_model_name: String,
    #[serialize(json, nullable)]
    pub provider_model_mappings: Option<serde_json::Value>,
    #[column(type = numeric(20, 8))]
    pub price_per_request: Option<Decimal>,
    #[serialize(json, nullable)]
    pub tiered_pricing: Option<TieredPricingConfig>,
    pub supports_vision: Option<bool>,
    pub supports_function_calling: Option<bool>,
    pub supports_streaming: Option<bool>,
    pub supports_extended_thinking: Option<bool>,
    pub supports_image_generation: Option<bool>,
    pub is_active: bool,
    pub is_available: bool,
    #[serialize(json, nullable)]
    pub config: Option<serde_json::Value>,
    #[auto]
    #[column(type = timestamp(6))]
    pub created_at: jiff::Timestamp,
    #[auto]
    #[column(type = timestamp(6))]
    pub updated_at: jiff::Timestamp,
}

impl GlobalModelRecord {
    pub fn with_counts(self, provider_count: u64, active_provider_count: u64) -> GlobalModelResponse {
        GlobalModelResponse {
            id: self.id,
            name: self.name,
            display_name: self.display_name,
            is_active: self.is_active,
            default_price_per_request: self.default_price_per_request,
            default_tiered_pricing: self.default_tiered_pricing,
            supported_capabilities: self.supported_capabilities,
            config: self.config,
            provider_count: Some(provider_count),
            active_provider_count: Some(active_provider_count),
            usage_count: Some(self.usage_count),
            created_at: self.created_at.to_string(),
            updated_at: Some(self.updated_at.to_string()),
        }
    }

    pub fn price_range(&self) -> ModelPriceRange {
        first_tier_range(self.default_tiered_pricing.tiers.first())
    }
}

impl ModelRecord {
    pub fn provider_detail(self, global_model: &GlobalModelRecord) -> ModelCatalogProviderDetail {
        let tiered = self.tiered_pricing.clone().unwrap_or_else(|| global_model.default_tiered_pricing.clone());
        let tier = tiered.tiers.first();
        ModelCatalogProviderDetail {
            provider_id: self.provider_id.clone(),
            provider_name: self.provider_id,
            model_id: Some(self.id),
            target_model: self.provider_model_name,
            input_price_per_1m: tier.map(|item| item.input_price_per_1m),
            output_price_per_1m: tier.map(|item| item.output_price_per_1m),
            cache_creation_price_per_1m: tier.and_then(|item| item.cache_creation_price_per_1m),
            cache_read_price_per_1m: tier.and_then(|item| item.cache_read_price_per_1m),
            cache_1h_creation_price_per_1m: tier.and_then(cache_1h_creation_price),
            price_per_request: self.price_per_request.or(global_model.default_price_per_request),
            effective_tiered_pricing: Some(tiered.clone()),
            tier_count: tiered.tiers.len() as u64,
            supports_vision: Some(self.supports_vision.unwrap_or_else(|| config_bool(global_model, "vision"))),
            supports_function_calling: Some(self.supports_function_calling.unwrap_or_else(|| config_bool(global_model, "function_calling"))),
            supports_streaming: Some(
                self.supports_streaming
                    .unwrap_or_else(|| config_bool_with_default(global_model, "streaming", true)),
            ),
            is_active: self.is_active,
        }
    }
}

fn first_tier_range(tier: Option<&PricingTier>) -> ModelPriceRange {
    ModelPriceRange {
        min_input: tier.map(|item| item.input_price_per_1m),
        max_input: tier.map(|item| item.input_price_per_1m),
        min_output: tier.map(|item| item.output_price_per_1m),
        max_output: tier.map(|item| item.output_price_per_1m),
    }
}

fn cache_1h_creation_price(tier: &PricingTier) -> Option<Decimal> {
    tier.cache_ttl_pricing
        .as_ref()?
        .iter()
        .find(|item| item.ttl_minutes == 60)
        .map(|item| item.cache_creation_price_per_1m)
}

fn config_bool(global_model: &GlobalModelRecord, key: &str) -> bool {
    config_bool_with_default(global_model, key, false)
}

fn config_bool_with_default(global_model: &GlobalModelRecord, key: &str, default: bool) -> bool {
    global_model
        .config
        .as_ref()
        .and_then(|config| config.get(key))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(default)
}
