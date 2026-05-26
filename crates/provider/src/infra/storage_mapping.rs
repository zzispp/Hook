use storage::provider::{
    ProviderApiKeyRecordInput, ProviderApiKeyRecordPatch, ProviderEndpointRecordInput, ProviderEndpointRecordPatch, ProviderModelRecordInput,
    ProviderModelCostRecordInput, ProviderModelRecordPatch, ProviderRecordInput, ProviderRecordPatch,
};
use types::provider::{
    ProviderApiKeyCreate, ProviderApiKeyUpdate, ProviderCreate, ProviderEndpointCreate, ProviderEndpointUpdate, ProviderModelBindingCreate,
    ProviderModelBindingUpdate, ProviderModelCostBatchUpsert, ProviderUpdate,
};

const DEFAULT_PROVIDER_MAX_RETRIES: i32 = 2;
const DEFAULT_PROVIDER_REQUEST_TIMEOUT_SECONDS: f64 = 300.0;
const DEFAULT_PROVIDER_STREAM_FIRST_BYTE_TIMEOUT_SECONDS: f64 = 30.0;
const DEFAULT_PROVIDER_STREAM_IDLE_TIMEOUT_SECONDS: f64 = 30.0;
const DEFAULT_PROVIDER_PRIORITY: i32 = 100;
const DEFAULT_PROVIDER_KEY_PRIORITY: i32 = 10;
const DEFAULT_PROVIDER_KEY_CACHE_TTL_MINUTES: i32 = 5;
const DEFAULT_PROVIDER_KEY_MAX_PROBE_INTERVAL_MINUTES: i32 = 32;

pub(super) fn provider_input(input: ProviderCreate) -> ProviderRecordInput {
    ProviderRecordInput {
        name: input.name,
        provider_type: input.provider_type,
        max_retries: Some(input.max_retries.unwrap_or(DEFAULT_PROVIDER_MAX_RETRIES)),
        request_timeout_seconds: Some(input.request_timeout_seconds.unwrap_or(DEFAULT_PROVIDER_REQUEST_TIMEOUT_SECONDS)),
        stream_first_byte_timeout_seconds: Some(
            input
                .stream_first_byte_timeout_seconds
                .unwrap_or(DEFAULT_PROVIDER_STREAM_FIRST_BYTE_TIMEOUT_SECONDS),
        ),
        stream_idle_timeout_seconds: Some(input.stream_idle_timeout_seconds.unwrap_or(DEFAULT_PROVIDER_STREAM_IDLE_TIMEOUT_SECONDS)),
        priority: input.priority.unwrap_or(DEFAULT_PROVIDER_PRIORITY),
        keep_priority_on_conversion: input.keep_priority_on_conversion.unwrap_or(false),
        enable_format_conversion: input.enable_format_conversion.unwrap_or(false),
        is_active: input.is_active.unwrap_or(true),
    }
}

pub(super) fn provider_patch(input: ProviderUpdate) -> ProviderRecordPatch {
    ProviderRecordPatch {
        name: input.name,
        provider_type: input.provider_type,
        max_retries: input.max_retries,
        request_timeout_seconds: input.request_timeout_seconds,
        stream_first_byte_timeout_seconds: input.stream_first_byte_timeout_seconds,
        stream_idle_timeout_seconds: input.stream_idle_timeout_seconds,
        priority: input.priority,
        keep_priority_on_conversion: input.keep_priority_on_conversion,
        enable_format_conversion: input.enable_format_conversion,
        is_active: input.is_active,
    }
}

pub(super) fn endpoint_input(provider_id: &str, input: ProviderEndpointCreate) -> ProviderEndpointRecordInput {
    ProviderEndpointRecordInput {
        provider_id: provider_id.to_owned(),
        api_format: input.api_format,
        base_url: input.base_url,
        custom_path: input.custom_path,
        max_retries: input.max_retries,
        is_active: input.is_active.unwrap_or(true),
        format_acceptance_config: input.format_acceptance_config,
        header_rules: input.header_rules,
        body_rules: input.body_rules,
    }
}

pub(super) fn endpoint_patch(input: ProviderEndpointUpdate) -> ProviderEndpointRecordPatch {
    ProviderEndpointRecordPatch {
        api_format: input.api_format,
        base_url: input.base_url,
        custom_path: input.custom_path,
        max_retries: input.max_retries,
        is_active: input.is_active,
        format_acceptance_config: input.format_acceptance_config,
        header_rules: input.header_rules,
        body_rules: input.body_rules,
    }
}

pub(super) fn api_key_input(provider_id: &str, input: ProviderApiKeyCreate, encrypted_api_key: String) -> ProviderApiKeyRecordInput {
    ProviderApiKeyRecordInput {
        provider_id: provider_id.to_owned(),
        name: input.name,
        api_formats: input.api_formats,
        allowed_model_ids: input.allowed_model_ids,
        encrypted_api_key,
        note: input.note,
        internal_priority: input.internal_priority.unwrap_or(DEFAULT_PROVIDER_KEY_PRIORITY),
        rpm_limit: input.rpm_limit,
        cache_ttl_minutes: input.cache_ttl_minutes.unwrap_or(DEFAULT_PROVIDER_KEY_CACHE_TTL_MINUTES),
        max_probe_interval_minutes: input.max_probe_interval_minutes.unwrap_or(DEFAULT_PROVIDER_KEY_MAX_PROBE_INTERVAL_MINUTES),
        time_range_enabled: input.time_range_enabled.unwrap_or(false),
        time_range_start: input.time_range_start,
        time_range_end: input.time_range_end,
        is_active: input.is_active.unwrap_or(true),
    }
}

pub(super) fn api_key_patch(input: ProviderApiKeyUpdate, encrypted_api_key: Option<String>) -> ProviderApiKeyRecordPatch {
    ProviderApiKeyRecordPatch {
        name: input.name,
        api_formats: input.api_formats,
        allowed_model_ids: input.allowed_model_ids,
        encrypted_api_key,
        note: input.note,
        internal_priority: input.internal_priority,
        rpm_limit: input.rpm_limit,
        cache_ttl_minutes: input.cache_ttl_minutes,
        max_probe_interval_minutes: input.max_probe_interval_minutes,
        time_range_enabled: input.time_range_enabled,
        time_range_start: input.time_range_start,
        time_range_end: input.time_range_end,
        is_active: input.is_active,
    }
}

pub(super) fn model_binding_input(provider_id: &str, input: ProviderModelBindingCreate) -> ProviderModelRecordInput {
    ProviderModelRecordInput {
        provider_id: provider_id.to_owned(),
        global_model_id: input.global_model_id,
        provider_model_name: input.provider_model_name,
        provider_model_mapping: input.provider_model_mapping,
        is_active: true,
        config: input.config,
    }
}

pub(super) fn model_binding_patch(input: ProviderModelBindingUpdate) -> ProviderModelRecordPatch {
    ProviderModelRecordPatch {
        provider_model_name: input.provider_model_name,
        is_active: input.is_active,
        provider_model_mapping: input.provider_model_mapping,
        config: input.config,
    }
}

pub(super) fn model_cost_inputs(provider_id: &str, key_id: &str, input: ProviderModelCostBatchUpsert) -> Vec<ProviderModelCostRecordInput> {
    input
        .costs
        .into_iter()
        .map(|cost| ProviderModelCostRecordInput {
            provider_id: provider_id.to_owned(),
            key_id: key_id.to_owned(),
            provider_model_id: cost.provider_model_id,
            cost_mode: cost.cost_mode,
            price_per_request: cost.price_per_request,
            input_price_per_million: cost.input_price_per_million,
            output_price_per_million: cost.output_price_per_million,
            cache_creation_price_per_million: cost.cache_creation_price_per_million,
            cache_read_price_per_million: cost.cache_read_price_per_million,
        })
        .collect()
}
