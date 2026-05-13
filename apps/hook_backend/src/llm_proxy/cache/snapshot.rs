use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use storage::{
    Database,
    group::GroupStore,
    model::global_models,
    provider::{ProviderStore, record::provider_api_keys},
    setting::SettingStore,
};
use types::{group::BillingGroupListRequest, model::TieredPricingConfig, provider::ProviderSchedulingMode};

use crate::llm_proxy::LlmProxyError;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SchedulingSnapshot {
    pub scheduling_mode: ProviderSchedulingMode,
    pub models: Vec<CachedGlobalModel>,
    pub groups: Vec<CachedBillingGroup>,
    pub providers: Vec<CachedProvider>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedGlobalModel {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub default_price_per_request: Option<Decimal>,
    pub default_tiered_pricing: TieredPricingConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedBillingGroup {
    pub code: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub billing_multiplier: Decimal,
    pub allowed_model_ids: Vec<String>,
    pub allowed_provider_ids: Vec<String>,
    pub is_active: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedProvider {
    pub id: String,
    pub name: String,
    pub max_retries: Option<i32>,
    pub request_timeout_seconds: Option<f64>,
    pub stream_first_byte_timeout_seconds: Option<f64>,
    pub priority: i32,
    pub keep_priority_on_conversion: bool,
    pub enable_format_conversion: bool,
    pub is_active: bool,
    pub endpoints: Vec<CachedEndpoint>,
    pub keys: Vec<CachedProviderKey>,
    pub models: Vec<CachedModelBinding>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedEndpoint {
    pub id: String,
    pub provider_id: String,
    pub api_format: String,
    pub base_url: String,
    pub custom_path: Option<String>,
    pub max_retries: Option<i32>,
    pub is_active: bool,
    pub format_acceptance_config: Option<serde_json::Value>,
    pub header_rules: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedProviderKey {
    pub id: String,
    pub provider_id: String,
    pub encrypted_api_key: String,
    pub internal_priority: i32,
    pub cache_ttl_minutes: i32,
    pub is_active: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedModelBinding {
    pub id: String,
    pub provider_id: String,
    pub global_model_id: String,
    pub provider_model_name: String,
    pub is_active: bool,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub price_per_request: Option<Decimal>,
    pub tiered_pricing: Option<TieredPricingConfig>,
}

pub async fn load(database: &Database) -> Result<SchedulingSnapshot, LlmProxyError> {
    let settings = SettingStore::new(database.clone()).get_system_settings().await?;
    Ok(SchedulingSnapshot {
        scheduling_mode: settings.scheduling_mode,
        models: load_models(database).await?,
        groups: load_groups(database).await?,
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
            limit: u64::MAX,
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
            is_active: group.is_active,
        })
        .collect())
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
            is_active: model.is_active,
            price_per_request: model.price_per_request,
            tiered_pricing: model.tiered_pricing,
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
        })
        .collect())
}

async fn load_keys(database: &Database, provider_id: &str) -> Result<Vec<CachedProviderKey>, LlmProxyError> {
    let records = provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
        .order_by_asc(provider_api_keys::Column::InternalPriority)
        .all(database.connection())
        .await?;
    Ok(records
        .into_iter()
        .map(|record| CachedProviderKey {
            id: record.id,
            provider_id: record.provider_id,
            encrypted_api_key: record.encrypted_api_key,
            internal_priority: record.internal_priority,
            cache_ttl_minutes: record.cache_ttl_minutes,
            is_active: record.is_active,
        })
        .collect())
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
