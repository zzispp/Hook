mod cached_types;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use storage::{
    Database,
    group::GroupStore,
    model::global_models,
    provider::{ProviderStore, record::provider_api_keys},
    setting::SettingStore,
    user::{UserGroupStore, UserStore},
};
use types::{
    group::BillingGroupListRequest,
    pagination::{PageRequest, PageSliceRequest},
    provider::parse_provider_key_time_range_minute,
    user::UserListFilters,
    user_group::{UserGroupFilters, UserGroupListRequest},
};

pub use cached_types::{
    CachedBillingGroup, CachedEndpoint, CachedGlobalModel, CachedModelBinding, CachedProvider, CachedProviderKey, CachedUserAccess, SchedulingSnapshot,
};

use crate::llm_proxy::LlmProxyError;

const SNAPSHOT_FULL_PAGE_LIMIT: u64 = i64::MAX as u64;

pub async fn load(database: &Database, system_users: &[CachedUserAccess]) -> Result<SchedulingSnapshot, LlmProxyError> {
    let settings = SettingStore::new(database.clone()).get_system_settings().await?;
    Ok(SchedulingSnapshot {
        default_rate_limit_rpm: settings.default_rate_limit_rpm,
        scheduling_mode: settings.scheduling_mode,
        provider_priority_mode: settings.provider_priority_mode,
        cache_affinity_ttl_minutes: settings.cache_affinity_ttl_minutes,
        client_request_record_level: settings.client_request_record_level,
        client_record_request_headers: settings.client_record_request_headers,
        client_record_request_body: settings.client_record_request_body,
        client_record_response_headers: settings.client_record_response_headers,
        client_record_response_body: settings.client_record_response_body,
        client_max_request_body_size_kb: settings.client_max_request_body_size_kb,
        client_max_response_body_size_kb: settings.client_max_response_body_size_kb,
        client_sensitive_request_headers: settings.client_sensitive_request_headers,
        provider_request_record_level: settings.provider_request_record_level,
        provider_record_request_headers: settings.provider_record_request_headers,
        provider_record_request_body: settings.provider_record_request_body,
        provider_record_response_headers: settings.provider_record_response_headers,
        provider_record_response_body: settings.provider_record_response_body,
        provider_max_request_body_size_kb: settings.provider_max_request_body_size_kb,
        provider_max_response_body_size_kb: settings.provider_max_response_body_size_kb,
        provider_sensitive_request_headers: settings.provider_sensitive_request_headers,
        provider_cooldown_policy: settings.provider_cooldown_policy,
        models: load_models(database).await?,
        groups: load_groups(database).await?,
        active_user_group_codes: load_active_user_group_codes(database).await?,
        users: load_users(database, system_users).await?,
        providers: load_providers(database).await?,
    })
}

pub fn encode(snapshot: &SchedulingSnapshot) -> Result<String, LlmProxyError> {
    serde_json::to_string(snapshot).map_err(json_error)
}

pub fn decode(value: &str) -> Result<SchedulingSnapshot, LlmProxyError> {
    serde_json::from_str(value).map_err(json_error)
}

async fn load_models(database: &Database) -> Result<Vec<CachedGlobalModel>, LlmProxyError> {
    let records = global_models::Entity::find()
        .order_by_asc(global_models::Column::Name)
        .all(database.connection())
        .await?;
    records.into_iter().map(cached_model).collect()
}

async fn load_groups(database: &Database) -> Result<Vec<CachedBillingGroup>, LlmProxyError> {
    let response = GroupStore::new(database.clone())
        .list_groups(BillingGroupListRequest {
            skip: 0,
            limit: SNAPSHOT_FULL_PAGE_LIMIT,
            is_active: None,
            search: None,
        })
        .await?;
    Ok(response
        .groups
        .into_iter()
        .map(|group| CachedBillingGroup {
            code: group.code,
            billing_multiplier: group.billing_multiplier,
            allowed_model_ids: group.allowed_model_ids,
            allowed_provider_ids: group.allowed_provider_ids,
            visible_user_group_codes: group.visible_user_group_codes,
            is_active: group.is_active,
        })
        .collect())
}

async fn load_users(database: &Database, system_users: &[CachedUserAccess]) -> Result<Vec<CachedUserAccess>, LlmProxyError> {
    let page = UserStore::new(database.clone())
        .list_slice(
            PageSliceRequest {
                offset: 0,
                limit: SNAPSHOT_FULL_PAGE_LIMIT,
                page: 1,
                page_size: SNAPSHOT_FULL_PAGE_LIMIT,
            },
            UserListFilters::default(),
        )
        .await?;
    let mut users = system_users.to_vec();
    users.extend(
        page.items
            .into_iter()
            .filter(|user| !system_users.iter().any(|system| system.id == user.id.0))
            .map(|user| CachedUserAccess {
                id: user.id.0,
                username: user.username,
                group_codes: user.group_codes,
                is_active: user.is_active,
                allowed_model_ids: user.allowed_model_ids,
                allowed_provider_ids: user.allowed_provider_ids,
                quota_mode: user.quota_mode,
                rate_limit_rpm: user.rate_limit_rpm,
            })
            .collect::<Vec<_>>(),
    );
    Ok(users)
}

async fn load_active_user_group_codes(database: &Database) -> Result<Vec<String>, LlmProxyError> {
    let response = UserGroupStore::new(database.clone())
        .list_groups(UserGroupListRequest {
            page: PageRequest {
                page: 1,
                page_size: SNAPSHOT_FULL_PAGE_LIMIT,
            },
            filters: UserGroupFilters {
                search: None,
                is_active: Some(true),
            },
        })
        .await?;
    Ok(response.items.into_iter().map(|group| group.code).collect())
}

async fn load_providers(database: &Database) -> Result<Vec<CachedProvider>, LlmProxyError> {
    let store = ProviderStore::new(database.clone());
    let mut providers = Vec::new();
    for provider in store.active_providers_for_scheduling().await? {
        providers.push(CachedProvider {
            endpoints: load_endpoints(&store, &provider.id).await?,
            keys: load_keys(database, &provider.id).await?,
            models: load_model_bindings(&store, &provider.id).await?,
            id: provider.id,
            name: provider.name,
            max_retries: provider.max_retries,
            request_timeout_seconds: provider.request_timeout_seconds,
            stream_first_byte_timeout_seconds: provider.stream_first_byte_timeout_seconds,
            stream_idle_timeout_seconds: provider.stream_idle_timeout_seconds,
            priority: provider.priority,
            keep_priority_on_conversion: provider.keep_priority_on_conversion,
            enable_format_conversion: provider.enable_format_conversion,
            is_active: provider.is_active,
        });
    }
    Ok(providers)
}

async fn load_model_bindings(store: &ProviderStore, provider_id: &str) -> Result<Vec<CachedModelBinding>, LlmProxyError> {
    Ok(store
        .model_bindings_for_provider(provider_id)
        .await?
        .into_iter()
        .map(|model| CachedModelBinding {
            id: model.id,
            provider_id: model.provider_id,
            global_model_id: model.global_model_id,
            provider_model_name: model.provider_model_name,
            provider_model_mapping: model.provider_model_mapping,
            is_active: model.is_active,
        })
        .collect())
}

async fn load_endpoints(store: &ProviderStore, provider_id: &str) -> Result<Vec<CachedEndpoint>, LlmProxyError> {
    Ok(store
        .endpoints_for_provider(provider_id)
        .await?
        .into_iter()
        .map(|endpoint| CachedEndpoint {
            id: endpoint.id,
            provider_id: endpoint.provider_id,
            api_format: endpoint.api_format,
            base_url: endpoint.base_url,
            custom_path: endpoint.custom_path,
            max_retries: endpoint.max_retries,
            is_active: endpoint.is_active,
            format_acceptance_config: endpoint.format_acceptance_config,
            header_rules: endpoint.header_rules,
            body_rules: endpoint.body_rules,
        })
        .collect())
}

async fn load_keys(database: &Database, provider_id: &str) -> Result<Vec<CachedProviderKey>, LlmProxyError> {
    let records = provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
        .order_by_asc(provider_api_keys::Column::InternalPriority)
        .all(database.connection())
        .await?;
    records
        .into_iter()
        .map(|record| {
            let (time_range_start_minute, time_range_end_minute) = cached_key_time_range(&record)?;
            Ok(CachedProviderKey {
                id: record.id,
                provider_id: record.provider_id,
                name: record.name.clone(),
                api_formats: decode_key_api_formats(record.api_formats)?,
                allowed_model_ids: decode_key_allowed_model_ids(record.allowed_model_ids)?,
                key_preview: record.name,
                encrypted_api_key: record.encrypted_api_key,
                internal_priority: record.internal_priority,
                global_priority: record.global_priority,
                rpm_limit: record.rpm_limit,
                cache_ttl_minutes: record.cache_ttl_minutes,
                time_range_enabled: record.time_range_enabled,
                time_range_start_minute,
                time_range_end_minute,
                is_active: record.is_active,
            })
        })
        .collect()
}

fn cached_key_time_range(record: &provider_api_keys::Model) -> Result<(Option<u16>, Option<u16>), LlmProxyError> {
    if !record.time_range_enabled {
        return Ok((None, None));
    }
    let start = cached_key_time_range_minute(&record.id, "time_range_start", record.time_range_start.as_deref())?;
    let end = cached_key_time_range_minute(&record.id, "time_range_end", record.time_range_end.as_deref())?;
    if start == end {
        return Err(LlmProxyError::Infrastructure(format!(
            "provider key {} time_range_start and time_range_end cannot be equal",
            record.id
        )));
    }
    Ok((Some(start), Some(end)))
}

fn cached_key_time_range_minute(key_id: &str, field: &str, value: Option<&str>) -> Result<u16, LlmProxyError> {
    let Some(value) = value else {
        return Err(LlmProxyError::Infrastructure(format!(
            "provider key {key_id} {field} is required when time_range_enabled is true"
        )));
    };
    parse_provider_key_time_range_minute(value).ok_or_else(|| LlmProxyError::Infrastructure(format!("provider key {key_id} {field} must use HH:mm format")))
}

fn decode_key_api_formats(value: String) -> Result<Vec<String>, LlmProxyError> {
    serde_json::from_str(&value).map_err(|error| LlmProxyError::Infrastructure(format!("provider key api_formats decode error: {error}")))
}

fn decode_key_allowed_model_ids(value: String) -> Result<Vec<String>, LlmProxyError> {
    serde_json::from_str(&value).map_err(|error| LlmProxyError::Infrastructure(format!("provider key allowed_model_ids decode error: {error}")))
}

fn cached_model(record: global_models::Model) -> Result<CachedGlobalModel, LlmProxyError> {
    Ok(CachedGlobalModel {
        id: record.id,
        name: record.name,
        is_active: record.is_active,
        default_price_per_request: record.default_price_per_request,
        default_tiered_pricing: serde_json::from_str(&record.default_tiered_pricing)
            .map_err(|error| LlmProxyError::Infrastructure(format!("invalid model pricing config: {error}")))?,
    })
}

fn json_error(error: serde_json::Error) -> LlmProxyError {
    LlmProxyError::Infrastructure(format!("proxy scheduling cache json error: {error}"))
}
