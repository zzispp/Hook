use rust_decimal::Decimal;
use time::format_description::well_known::Rfc3339;
use types::model::{GlobalModelResponse, ModelCatalogProviderDetail, ModelPriceRange, PricingTier, TieredPricingConfig};

use crate::{StorageResult, json};

#[path = "entities/mod.rs"]
pub mod entities;

pub use entities::{global_model_user_usage_counts, global_models, provider_models};

pub type GlobalModelRecord = global_models::Model;
pub type ModelRecord = provider_models::Model;

impl GlobalModelRecord {
    pub fn with_counts(self, provider_count: u64, active_provider_count: u64) -> StorageResult<GlobalModelResponse> {
        let default_tiered_pricing = self.default_tiered_pricing()?;
        let supported_capabilities = self.supported_capabilities()?;
        let config = self.config()?;
        let routing_profile_id = self.routing_profile_id();
        Ok(GlobalModelResponse {
            id: self.id,
            name: self.name,
            display_name: self.display_name,
            is_active: self.is_active,
            default_price_per_request: self.default_price_per_request,
            default_tiered_pricing,
            supported_capabilities,
            config,
            routing_profile_id,
            provider_count: Some(provider_count),
            active_provider_count: Some(active_provider_count),
            usage_count: Some(self.usage_count),
            created_at: format_timestamp(self.created_at),
            updated_at: Some(format_timestamp(self.updated_at)),
        })
    }

    pub fn price_range(&self) -> StorageResult<ModelPriceRange> {
        Ok(first_tier_range(self.default_tiered_pricing()?.tiers.first()))
    }

    pub fn default_tiered_pricing(&self) -> StorageResult<TieredPricingConfig> {
        json::decode_required(self.default_tiered_pricing.clone())
    }

    pub fn supported_capabilities(&self) -> StorageResult<Option<Vec<String>>> {
        json::decode_optional(self.supported_capabilities.clone())
    }

    pub fn config(&self) -> StorageResult<Option<serde_json::Value>> {
        json::decode_optional(self.config.clone())
    }
}

impl ModelRecord {
    pub fn provider_detail(self, global_model: &GlobalModelRecord) -> StorageResult<ModelCatalogProviderDetail> {
        let tiered = global_model.default_tiered_pricing()?;
        let tier = tiered.tiers.first();
        Ok(ModelCatalogProviderDetail {
            provider_id: self.provider_id.clone(),
            provider_name: self.provider_id,
            model_id: Some(self.id),
            target_model: self.provider_model_name,
            input_price_per_1m: tier.map(|item| item.input_price_per_1m),
            output_price_per_1m: tier.map(|item| item.output_price_per_1m),
            cache_creation_price_per_1m: tier.and_then(|item| item.cache_creation_price_per_1m),
            cache_read_price_per_1m: tier.and_then(|item| item.cache_read_price_per_1m),
            cache_1h_creation_price_per_1m: tier.and_then(cache_1h_creation_price),
            price_per_request: global_model.default_price_per_request,
            effective_tiered_pricing: Some(tiered.clone()),
            tier_count: tiered.tiers.len() as u64,
            supports_vision: Some(global_model.config_bool("vision")?),
            supports_function_calling: Some(global_model.config_bool("function_calling")?),
            supports_streaming: Some(global_model.config_bool_with_default("streaming", true)?),
        })
    }
}

impl GlobalModelRecord {
    fn config_bool(&self, key: &str) -> StorageResult<bool> {
        self.config_bool_with_default(key, false)
    }

    fn config_bool_with_default(&self, key: &str, default: bool) -> StorageResult<bool> {
        Ok(self
            .config()?
            .and_then(|config| config.get(key).and_then(serde_json::Value::as_bool))
            .unwrap_or(default))
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

fn format_timestamp(value: sea_orm::prelude::TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("global model timestamp must format as RFC3339")
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rust_decimal::Decimal;
    use time::OffsetDateTime;

    use super::{GlobalModelRecord, format_timestamp};

    #[test]
    fn with_counts_formats_model_timestamps_as_rfc3339() {
        let created_at = OffsetDateTime::parse("2026-05-13T08:30:45Z", &super::Rfc3339).expect("test created_at should parse");
        let updated_at = OffsetDateTime::parse("2026-05-13T09:31:46Z", &super::Rfc3339).expect("test updated_at should parse");
        let record = GlobalModelRecord {
            id: "model-1".into(),
            name: "gpt-test".into(),
            display_name: "GPT Test".into(),
            default_price_per_request: Some(Decimal::from_str("0.5").expect("decimal")),
            default_tiered_pricing: "{\"tiers\":[{\"up_to\":null,\"input_price_per_1m\":0.1,\"output_price_per_1m\":0.2}]}".into(),
            supported_capabilities: Some("[\"vision\"]".into()),
            config: Some("{\"streaming\":true}".into()),
            routing_profile_id: None,
            is_active: true,
            usage_count: 42,
            created_at,
            updated_at,
        };

        let response = record.with_counts(3, 2).expect("record should convert");

        assert_eq!(response.created_at, "2026-05-13T08:30:45Z");
        assert_eq!(response.updated_at.as_deref(), Some("2026-05-13T09:31:46Z"));
    }

    #[test]
    fn format_timestamp_uses_rfc3339() {
        let value = OffsetDateTime::parse("2026-05-13T08:30:45+08:00", &super::Rfc3339).expect("timestamp should parse");

        assert_eq!(format_timestamp(value), "2026-05-13T08:30:45+08:00");
    }
}
