use time::format_description::well_known::Rfc3339;
use types::provider::{ProviderApiKey, ProviderEndpoint, RequestCandidate};

use crate::{StorageResult, json};

#[path = "entities/mod.rs"]
pub mod entities;

pub(crate) use crate::model::provider_models;
pub use entities::{
    billing_group_providers, billing_rules, dimension_collectors, provider_api_keys, provider_cooldowns, provider_endpoints, providers, request_candidates,
    request_records,
};

pub type ProviderRecord = providers::Model;
pub type ProviderEndpointRecord = provider_endpoints::Model;
pub type ProviderApiKeyRecord = provider_api_keys::Model;
pub type ProviderCooldownRecord = provider_cooldowns::Model;
pub type ProviderModelRecord = provider_models::Model;
pub type BillingRuleRecord = billing_rules::Model;
pub type DimensionCollectorRecord = dimension_collectors::Model;
pub type RequestCandidateRecord = request_candidates::Model;
pub type RequestRecordSummaryRecord = request_records::Model;

impl ProviderEndpointRecord {
    pub fn response(self) -> StorageResult<ProviderEndpoint> {
        Ok(ProviderEndpoint {
            id: self.id,
            provider_id: self.provider_id,
            api_format: self.api_format,
            base_url: self.base_url,
            custom_path: self.custom_path,
            max_retries: self.max_retries,
            is_active: self.is_active,
            format_acceptance_config: json::decode_optional(self.format_acceptance_config)?,
            header_rules: json::decode_optional(self.header_rules)?,
            body_rules: json::decode_optional(self.body_rules)?,
            created_at: format_timestamp(self.created_at),
            updated_at: format_timestamp(self.updated_at),
        })
    }
}

impl ProviderApiKeyRecord {
    pub fn response(self) -> StorageResult<ProviderApiKey> {
        Ok(ProviderApiKey {
            id: self.id,
            provider_id: self.provider_id,
            name: self.name,
            api_formats: json::decode_required(self.api_formats)?,
            allowed_model_ids: json::decode_required(self.allowed_model_ids)?,
            note: self.note,
            internal_priority: self.internal_priority,
            rpm_limit: self.rpm_limit,
            learned_rpm_limit: self.learned_rpm_limit,
            cache_ttl_minutes: self.cache_ttl_minutes,
            max_probe_interval_minutes: self.max_probe_interval_minutes,
            time_range_enabled: self.time_range_enabled,
            time_range_start: self.time_range_start,
            time_range_end: self.time_range_end,
            health_by_format: json::decode_optional(self.health_by_format)?,
            circuit_breaker_by_format: json::decode_optional(self.circuit_breaker_by_format)?,
            is_active: self.is_active,
            has_api_key: !self.encrypted_api_key.is_empty(),
            created_at: format_timestamp(self.created_at),
            updated_at: format_timestamp(self.updated_at),
        })
    }
}

impl RequestCandidateRecord {
    pub fn response(self) -> RequestCandidate {
        RequestCandidate {
            id: self.id,
            request_id: self.request_id,
            token_id: self.token_id,
            group_code: self.group_code,
            global_model_id: self.global_model_id,
            provider_id: self.provider_id,
            endpoint_id: self.endpoint_id,
            key_id: self.key_id,
            client_api_format: self.client_api_format,
            provider_api_format: self.provider_api_format,
            needs_conversion: self.needs_conversion,
            is_stream: self.is_stream,
            is_cached: self.is_cached,
            candidate_index: self.candidate_index,
            retry_index: self.retry_index,
            status: self.status,
            skip_reason: self.skip_reason,
            status_code: self.status_code,
            prompt_tokens: self.prompt_tokens,
            completion_tokens: self.completion_tokens,
            total_tokens: self.total_tokens,
            cache_creation_input_tokens: self.cache_creation_input_tokens,
            cache_read_input_tokens: self.cache_read_input_tokens,
            input_text_tokens: self.input_text_tokens,
            input_audio_tokens: self.input_audio_tokens,
            input_image_tokens: self.input_image_tokens,
            output_text_tokens: self.output_text_tokens,
            output_audio_tokens: self.output_audio_tokens,
            output_image_tokens: self.output_image_tokens,
            reasoning_tokens: self.reasoning_tokens,
            cache_creation_5m_input_tokens: self.cache_creation_5m_input_tokens,
            cache_creation_1h_input_tokens: self.cache_creation_1h_input_tokens,
            usage_source: self.usage_source,
            usage_semantic: self.usage_semantic,
            service_tier: self.service_tier,
            input_cost: self.input_cost,
            output_cost: self.output_cost,
            cache_creation_cost: self.cache_creation_cost,
            cache_read_cost: self.cache_read_cost,
            request_cost: self.request_cost,
            input_price_per_million: self.input_price_per_million,
            output_price_per_million: self.output_price_per_million,
            cache_creation_price_per_million: self.cache_creation_price_per_million,
            cache_read_price_per_million: self.cache_read_price_per_million,
            cost_currency: self.cost_currency,
            token_cost: self.token_cost,
            base_cost: self.base_cost,
            total_cost: self.total_cost,
            billing_multiplier: self.billing_multiplier,
            billing_snapshot: json::decode_optional(self.billing_snapshot).ok().flatten(),
            latency_ms: self.latency_ms,
            first_byte_time_ms: self.first_byte_time_ms,
            error_type: self.error_type,
            error_message: self.error_message,
            error_code: self.error_code,
            error_param: self.error_param,
            created_at: format_timestamp(self.created_at),
            started_at: self.started_at.map(format_timestamp),
            finished_at: self.finished_at.map(format_timestamp),
        }
    }
}

fn format_timestamp(value: sea_orm::entity::prelude::TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("provider timestamp must format as RFC3339")
}
