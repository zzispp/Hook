use storage::provider::{
    ProviderApiKeyRecordInput, ProviderApiKeyRecordPatch, ProviderEndpointRecordInput, ProviderEndpointRecordPatch, ProviderKeyGroupRecordInput,
    ProviderKeyGroupRecordPatch, ProviderModelCostRecordInput, ProviderModelRecordInput, ProviderModelRecordPatch, ProviderQuickImportApiKeyRecordInput,
    ProviderQuickImportEndpointRecordInput, ProviderQuickImportKeyModelRecordInput, ProviderQuickImportKeyReplacementRecordInput,
    ProviderQuickImportModelCostRecordInput, ProviderQuickImportModelRecordInput, ProviderQuickImportRecordInput, ProviderQuickImportSourceRecordInput,
    ProviderRecordInput, ProviderRecordPatch,
};
use types::provider::{
    ProviderApiKeyCreate, ProviderApiKeyUpdate, ProviderCreate, ProviderEndpointCreate, ProviderEndpointUpdate, ProviderKeyGroupCreate, ProviderKeyGroupUpdate,
    ProviderModelBindingBatchUpdate, ProviderModelBindingCreate, ProviderModelBindingUpdate, ProviderModelCostBatchUpsert, ProviderOrigin, ProviderUpdate,
};

use crate::application::{ProviderQuickImportAppend, ProviderQuickImportCreate, ProviderQuickImportKeyReplacement};

const DEFAULT_PROVIDER_MAX_RETRIES: i32 = 2;
const DEFAULT_PROVIDER_REQUEST_TIMEOUT_SECONDS: f64 = 300.0;
const DEFAULT_PROVIDER_STREAM_FIRST_BYTE_TIMEOUT_SECONDS: f64 = 60.0;
const DEFAULT_PROVIDER_STREAM_IDLE_TIMEOUT_SECONDS: f64 = 300.0;
const DEFAULT_PROVIDER_PRIORITY: i32 = 100;
const DEFAULT_PROVIDER_KEY_PRIORITY: i32 = 10;
const DEFAULT_PROVIDER_KEY_CACHE_TTL_MINUTES: i32 = 5;
const DEFAULT_PROVIDER_KEY_MAX_PROBE_INTERVAL_MINUTES: i32 = 32;

pub(super) fn provider_input(input: ProviderCreate) -> ProviderRecordInput {
    ProviderRecordInput {
        name: input.name,
        provider_type: input.provider_type,
        provider_origin: ProviderOrigin::Manual,
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
    let internal_priority = input.internal_priority.unwrap_or(DEFAULT_PROVIDER_KEY_PRIORITY);
    ProviderApiKeyRecordInput {
        provider_id: provider_id.to_owned(),
        name: input.name,
        global_priority_by_format: global_priority_by_format(&input.api_formats, internal_priority),
        api_formats: input.api_formats,
        allowed_model_ids: input.allowed_model_ids,
        encrypted_api_key,
        note: input.note,
        internal_priority,
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
        global_priority_by_format: None,
        rpm_limit: input.rpm_limit,
        cache_ttl_minutes: input.cache_ttl_minutes,
        max_probe_interval_minutes: input.max_probe_interval_minutes,
        time_range_enabled: input.time_range_enabled,
        time_range_start: input.time_range_start,
        time_range_end: input.time_range_end,
        is_active: input.is_active,
    }
}

pub(super) fn provider_key_group_input(input: ProviderKeyGroupCreate) -> ProviderKeyGroupRecordInput {
    ProviderKeyGroupRecordInput {
        name: input.name,
        description: input.description,
        sort_order: input.sort_order.unwrap_or(0),
        provider_key_members: input.provider_key_members,
    }
}

pub(super) fn provider_key_group_patch(input: ProviderKeyGroupUpdate) -> ProviderKeyGroupRecordPatch {
    ProviderKeyGroupRecordPatch {
        name: input.name,
        description: input.description,
        sort_order: input.sort_order,
        provider_key_members: input.provider_key_members,
    }
}

fn global_priority_by_format(api_formats: &[String], priority: i32) -> std::collections::BTreeMap<String, i32> {
    api_formats.iter().map(|format| (format.clone(), priority)).collect()
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

pub(super) fn model_binding_batch_input(provider_id: &str, input: ProviderModelBindingBatchUpdate) -> storage::provider::ProviderModelRecordBatchUpdate {
    storage::provider::ProviderModelRecordBatchUpdate {
        provider_id: provider_id.to_owned(),
        create: input.create.into_iter().map(|binding| model_binding_input(provider_id, binding)).collect(),
        delete_ids: input.delete_ids,
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

pub(super) fn quick_import_input(input: ProviderQuickImportCreate) -> ProviderQuickImportRecordInput {
    ProviderQuickImportRecordInput {
        provider: provider_input(input.provider),
        sync_source: input.sync_source.map(quick_import_source_input),
        endpoints: input.endpoints.into_iter().map(quick_import_endpoint_input).collect(),
        api_keys: input.api_keys.into_iter().map(quick_import_key_input).collect(),
        model_bindings: input.model_bindings.into_iter().map(quick_import_model_input).collect(),
        model_costs: input.model_costs.into_iter().map(quick_import_cost_input).collect(),
    }
}

pub(super) fn quick_import_append_input(input: ProviderQuickImportAppend) -> storage::provider::ProviderQuickImportAppendRecordInput {
    storage::provider::ProviderQuickImportAppendRecordInput {
        provider_id: input.provider_id,
        source_id: input.source_id,
        endpoints: input.endpoints.into_iter().map(quick_import_endpoint_input).collect(),
        api_keys: input.api_keys.into_iter().map(quick_import_key_input).collect(),
        model_bindings: input.model_bindings.into_iter().map(quick_import_model_input).collect(),
        model_costs: input.model_costs.into_iter().map(quick_import_cost_input).collect(),
    }
}

pub(super) fn quick_import_key_replacement_input(input: ProviderQuickImportKeyReplacement) -> ProviderQuickImportKeyReplacementRecordInput {
    ProviderQuickImportKeyReplacementRecordInput {
        provider_id: input.provider_id,
        source_id: input.source_id,
        key_id: input.key_id,
        upstream_token_id: input.upstream_token_id,
        upstream_token_name: input.upstream_token_name,
        upstream_masked_key: input.upstream_masked_key,
        upstream_group: input.upstream_group,
        upstream_group_ratio: input.upstream_group_ratio,
        effective_cost_multiplier: input.effective_cost_multiplier,
        model_mappings: input.model_mappings.into_iter().map(quick_import_key_model_input).collect(),
        key_patch: api_key_patch(input.input, input.encrypted_api_key),
        model_bindings: input.model_bindings.into_iter().map(quick_import_model_input).collect(),
        model_costs: input.model_costs.into_iter().map(quick_import_cost_input).collect(),
        delete_provider_model_ids: Vec::new(),
    }
}

fn quick_import_source_input(input: crate::application::ProviderQuickImportSyncSourceCreate) -> ProviderQuickImportSourceRecordInput {
    ProviderQuickImportSourceRecordInput {
        source_kind: input.source_kind.as_str().to_owned(),
        base_url: input.base_url,
        encrypted_system_access_token: input.encrypted_system_access_token,
        user_id: input.user_id,
        recharge_multiplier: input.recharge_multiplier,
        sync_config: input.sync_config,
    }
}

fn quick_import_endpoint_input(input: ProviderEndpointCreate) -> ProviderQuickImportEndpointRecordInput {
    ProviderQuickImportEndpointRecordInput {
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

fn quick_import_key_input(input: crate::application::ProviderQuickImportApiKeyCreate) -> ProviderQuickImportApiKeyRecordInput {
    let internal_priority = input.input.internal_priority.unwrap_or(DEFAULT_PROVIDER_KEY_PRIORITY);
    ProviderQuickImportApiKeyRecordInput {
        upstream_token_id: input.upstream_token_id,
        upstream_token_name: input.upstream_token_name,
        upstream_masked_key: input.upstream_masked_key,
        upstream_group: input.upstream_group,
        upstream_group_ratio: input.upstream_group_ratio,
        effective_cost_multiplier: input.effective_cost_multiplier,
        model_mappings: input.model_mappings.into_iter().map(quick_import_key_model_input).collect(),
        name: input.input.name,
        global_priority_by_format: global_priority_by_format(&input.input.api_formats, internal_priority),
        api_formats: input.input.api_formats,
        allowed_model_ids: input.input.allowed_model_ids,
        encrypted_api_key: input.encrypted_api_key,
        note: input.input.note,
        internal_priority,
        rpm_limit: input.input.rpm_limit,
        cache_ttl_minutes: input.input.cache_ttl_minutes.unwrap_or(DEFAULT_PROVIDER_KEY_CACHE_TTL_MINUTES),
        max_probe_interval_minutes: input
            .input
            .max_probe_interval_minutes
            .unwrap_or(DEFAULT_PROVIDER_KEY_MAX_PROBE_INTERVAL_MINUTES),
        time_range_enabled: input.input.time_range_enabled.unwrap_or(false),
        time_range_start: input.input.time_range_start,
        time_range_end: input.input.time_range_end,
        is_active: input.input.is_active.unwrap_or(true),
    }
}

fn quick_import_key_model_input(input: crate::application::ProviderQuickImportKeyModelCreate) -> ProviderQuickImportKeyModelRecordInput {
    ProviderQuickImportKeyModelRecordInput {
        upstream_model_id: input.upstream_model_id,
        global_model_id: input.global_model_id,
    }
}

fn quick_import_model_input(input: ProviderModelBindingCreate) -> ProviderQuickImportModelRecordInput {
    ProviderQuickImportModelRecordInput {
        global_model_id: input.global_model_id,
        provider_model_name: input.provider_model_name,
        provider_model_mapping: input.provider_model_mapping,
        is_active: true,
        config: input.config,
    }
}

fn quick_import_cost_input(input: crate::application::ProviderQuickImportModelCostCreate) -> ProviderQuickImportModelCostRecordInput {
    ProviderQuickImportModelCostRecordInput {
        upstream_token_id: input.upstream_token_id,
        global_model_id: input.global_model_id,
        cost_mode: input.cost.cost_mode,
        price_per_request: input.cost.price_per_request,
        input_price_per_million: input.cost.input_price_per_million,
        output_price_per_million: input.cost.output_price_per_million,
        cache_creation_price_per_million: input.cost.cache_creation_price_per_million,
        cache_read_price_per_million: input.cost.cache_read_price_per_million,
    }
}
