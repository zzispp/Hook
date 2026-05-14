use time::format_description::well_known::Rfc3339;
use types::provider::{ProviderApiKey, ProviderEndpoint, RequestCandidate};

use crate::{StorageResult, json};

#[path = "entities/mod.rs"]
pub mod entities;

pub(crate) use crate::model::provider_models;
pub use entities::{billing_group_providers, provider_api_keys, provider_endpoints, providers, request_candidates, request_records};

pub type ProviderRecord = providers::Model;
pub type ProviderEndpointRecord = provider_endpoints::Model;
pub type ProviderApiKeyRecord = provider_api_keys::Model;
pub type ProviderModelRecord = provider_models::Model;
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
            cost_currency: self.cost_currency,
            token_cost: self.token_cost,
            base_cost: self.base_cost,
            total_cost: self.total_cost,
            billing_multiplier: self.billing_multiplier,
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
