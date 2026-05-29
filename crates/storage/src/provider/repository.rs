use std::collections::HashSet;

use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set};
use types::provider::{Provider, ProviderListRequest, ProviderListResponse};

use crate::{Database, StorageError, StorageResult, json};

use super::{
    BillingRuleRecordInput, DimensionCollectorRecordInput, ProviderApiKeyRecordInput, ProviderApiKeyRecordPatch, ProviderApiKeySecretRecord,
    ProviderEndpointRecordInput, ProviderEndpointRecordPatch, ProviderModelCostRecordInput, ProviderModelRecordInput, ProviderModelRecordPatch,
    ProviderRecordInput, ProviderRecordPatch,
    record::{
        provider_api_keys, provider_endpoints, provider_models,
        providers::{self, ActiveModel as ProviderActiveModel},
    },
    repository_helpers::{ProviderFilterIds, apply_provider_api_key_patch, apply_provider_patch, filter_provider_records, provider_active_model},
};

#[derive(Clone)]
pub struct ProviderStore {
    database: Database,
}

impl ProviderStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create_provider(&self, input: ProviderRecordInput) -> StorageResult<Provider> {
        let record = provider_active_model(self.database.next_id(), input).insert(self.database.connection()).await?;
        Ok(record.into())
    }

    pub async fn update_provider(&self, id: &str, input: ProviderRecordPatch) -> StorageResult<Provider> {
        let record = self.find_provider_record(id).await?.ok_or(StorageError::NotFound)?;
        let mut active: ProviderActiveModel = record.into();
        apply_provider_patch(&mut active, input);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        let record = active.update(self.database.connection()).await?;
        Ok(record.into())
    }

    pub async fn delete_provider(&self, id: &str) -> StorageResult<()> {
        let record = self.find_provider_record(id).await?.ok_or(StorageError::NotFound)?;
        let active: ProviderActiveModel = record.into();
        active.delete(self.database.connection()).await?;
        Ok(())
    }

    pub async fn find_provider(&self, id_or_name: &str) -> StorageResult<Option<Provider>> {
        self.find_provider_record(id_or_name).await.map(|record| record.map(Into::into))
    }

    pub async fn list_providers(&self, request: ProviderListRequest) -> StorageResult<ProviderListResponse> {
        let records = self.provider_records().await?;
        let ids = self.provider_filter_ids(&request).await?;
        let records = filter_provider_records(records, &request, ids);
        let total = records.len() as u64;
        let providers = records
            .into_iter()
            .skip(request.skip as usize)
            .take(request.limit as usize)
            .map(Into::into)
            .collect();
        Ok(ProviderListResponse { providers, total })
    }

    pub async fn active_providers_for_scheduling(&self) -> StorageResult<Vec<Provider>> {
        Ok(self
            .provider_records()
            .await?
            .into_iter()
            .filter(|record| record.is_active)
            .map(Into::into)
            .collect())
    }

    pub async fn create_endpoint(&self, input: ProviderEndpointRecordInput) -> StorageResult<types::provider::ProviderEndpoint> {
        super::provider_endpoint_query::create_endpoint(self, input).await
    }

    pub async fn endpoints_for_provider(&self, provider_id: &str) -> StorageResult<Vec<types::provider::ProviderEndpoint>> {
        super::provider_endpoint_query::endpoints_for_provider(self, provider_id).await
    }

    pub async fn update_endpoint(
        &self,
        provider_id: &str,
        endpoint_id: &str,
        input: ProviderEndpointRecordPatch,
    ) -> StorageResult<types::provider::ProviderEndpoint> {
        super::provider_endpoint_query::update_endpoint(self, provider_id, endpoint_id, input).await
    }

    pub async fn delete_endpoint(&self, provider_id: &str, endpoint_id: &str) -> StorageResult<()> {
        super::provider_endpoint_query::delete_endpoint(self, provider_id, endpoint_id).await
    }

    pub async fn create_api_key(&self, input: ProviderApiKeyRecordInput) -> StorageResult<types::provider::ProviderApiKey> {
        let now = time::OffsetDateTime::now_utc();
        let record = provider_api_keys::ActiveModel {
            id: Set(self.database.next_id()),
            provider_id: Set(input.provider_id),
            name: Set(input.name),
            api_formats: Set(json::encode_required(&input.api_formats)?),
            allowed_model_ids: Set(json::encode_required(&input.allowed_model_ids)?),
            encrypted_api_key: Set(input.encrypted_api_key),
            note: Set(input.note),
            internal_priority: Set(input.internal_priority),
            rpm_limit: Set(input.rpm_limit),
            learned_rpm_limit: Set(None),
            cache_ttl_minutes: Set(input.cache_ttl_minutes),
            max_probe_interval_minutes: Set(input.max_probe_interval_minutes),
            time_range_enabled: Set(input.time_range_enabled),
            time_range_start: Set(input.time_range_start),
            time_range_end: Set(input.time_range_end),
            health_by_format: Set(None),
            circuit_breaker_by_format: Set(None),
            is_active: Set(input.is_active),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(self.database.connection())
        .await?;
        record.response()
    }

    pub async fn api_keys_for_provider(&self, provider_id: &str) -> StorageResult<Vec<types::provider::ProviderApiKey>> {
        let records = provider_api_keys::Entity::find()
            .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
            .order_by_asc(provider_api_keys::Column::InternalPriority)
            .all(self.database.connection())
            .await?;
        records.into_iter().map(|record| record.response()).collect()
    }

    pub async fn api_key_secrets_for_provider(&self, provider_id: &str) -> StorageResult<Vec<ProviderApiKeySecretRecord>> {
        let records = provider_api_keys::Entity::find()
            .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
            .order_by_asc(provider_api_keys::Column::InternalPriority)
            .all(self.database.connection())
            .await?;
        records
            .into_iter()
            .map(|record| {
                Ok(ProviderApiKeySecretRecord {
                    id: record.id,
                    name: record.name,
                    api_formats: json::decode_required(record.api_formats)?,
                    allowed_model_ids: json::decode_required(record.allowed_model_ids)?,
                    encrypted_api_key: record.encrypted_api_key,
                    internal_priority: record.internal_priority,
                    is_active: record.is_active,
                })
            })
            .collect()
    }

    pub async fn update_api_key(&self, provider_id: &str, key_id: &str, input: ProviderApiKeyRecordPatch) -> StorageResult<types::provider::ProviderApiKey> {
        let record = self.find_api_key_record(provider_id, key_id).await?.ok_or(StorageError::NotFound)?;
        let mut active: provider_api_keys::ActiveModel = record.into();
        apply_provider_api_key_patch(&mut active, input)?;
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        let record = active.update(self.database.connection()).await?;
        record.response()
    }

    pub async fn delete_api_key(&self, provider_id: &str, key_id: &str) -> StorageResult<()> {
        let record = self.find_api_key_record(provider_id, key_id).await?.ok_or(StorageError::NotFound)?;
        let active: provider_api_keys::ActiveModel = record.into();
        active.delete(self.database.connection()).await?;
        Ok(())
    }

    pub async fn create_model_binding(&self, input: ProviderModelRecordInput) -> StorageResult<types::provider::ProviderModelBinding> {
        super::provider_model_query::create_model_binding(self, input).await
    }

    pub async fn model_bindings_for_provider(&self, provider_id: &str) -> StorageResult<Vec<types::provider::ProviderModelBinding>> {
        super::provider_model_query::model_bindings_for_provider(self, provider_id).await
    }

    pub async fn update_model_binding(
        &self,
        provider_id: &str,
        model_id: &str,
        input: ProviderModelRecordPatch,
    ) -> StorageResult<types::provider::ProviderModelBinding> {
        super::provider_model_query::update_model_binding(self, provider_id, model_id, input).await
    }

    pub async fn delete_model_binding(&self, provider_id: &str, model_id: &str) -> StorageResult<()> {
        super::provider_model_query::delete_model_binding(self, provider_id, model_id).await
    }

    pub async fn list_model_costs(&self, provider_id: &str) -> StorageResult<Vec<types::provider::ProviderModelCost>> {
        super::provider_model_cost_query::list_model_costs(self, provider_id).await
    }

    pub async fn upsert_model_costs(&self, inputs: Vec<ProviderModelCostRecordInput>) -> StorageResult<Vec<types::provider::ProviderModelCost>> {
        super::provider_model_cost_query::upsert_model_costs(self, inputs).await
    }

    pub async fn delete_model_cost(&self, provider_id: &str, key_id: &str, provider_model_id: &str) -> StorageResult<()> {
        super::provider_model_cost_query::delete_model_cost(self, provider_id, key_id, provider_model_id).await
    }

    pub async fn find_model_cost(&self, key_id: &str, provider_model_id: &str) -> StorageResult<Option<types::provider::ProviderModelCost>> {
        super::provider_model_cost_query::find_model_cost(self, key_id, provider_model_id).await
    }

    pub async fn create_billing_rule(&self, input: BillingRuleRecordInput) -> StorageResult<super::record::BillingRuleRecord> {
        super::billing_config_query::create_billing_rule(self, input).await
    }

    pub async fn enabled_billing_rule_for_model(&self, model_id: &str, task_type: &str) -> StorageResult<Option<super::record::BillingRuleRecord>> {
        super::billing_config_query::enabled_rule_for_model(self, model_id, task_type).await
    }

    pub async fn enabled_billing_rule_for_global_model(
        &self,
        global_model_id: &str,
        task_type: &str,
    ) -> StorageResult<Option<super::record::BillingRuleRecord>> {
        super::billing_config_query::enabled_rule_for_global_model(self, global_model_id, task_type).await
    }

    pub async fn create_dimension_collector(&self, input: DimensionCollectorRecordInput) -> StorageResult<super::record::DimensionCollectorRecord> {
        super::billing_config_query::create_dimension_collector(self, input).await
    }

    pub async fn enabled_dimension_collectors(&self, api_format: &str, task_type: &str) -> StorageResult<Vec<super::record::DimensionCollectorRecord>> {
        super::billing_config_query::enabled_collectors(self, api_format, task_type).await
    }

    pub(crate) fn connection(&self) -> &DatabaseConnection {
        self.database.connection()
    }

    pub(crate) fn next_id(&self) -> String {
        self.database.next_id()
    }

    async fn provider_records(&self) -> StorageResult<Vec<super::ProviderRecord>> {
        providers::Entity::find()
            .order_by_asc(providers::Column::Priority)
            .order_by_asc(providers::Column::Name)
            .all(self.database.connection())
            .await
            .map_err(StorageError::from)
    }

    async fn provider_filter_ids(&self, request: &ProviderListRequest) -> StorageResult<ProviderFilterIds> {
        Ok(ProviderFilterIds {
            api_format: self.provider_ids_by_api_format(request.api_format.as_deref()).await?,
            model: self.provider_ids_by_model(request.model_id.as_deref()).await?,
        })
    }

    async fn provider_ids_by_api_format(&self, api_format: Option<&str>) -> StorageResult<Option<HashSet<String>>> {
        let Some(api_format) = api_format else { return Ok(None) };
        let records = provider_endpoints::Entity::find()
            .filter(provider_endpoints::Column::ApiFormat.eq(api_format))
            .all(self.database.connection())
            .await?;
        Ok(Some(records.into_iter().map(|record| record.provider_id).collect()))
    }

    async fn provider_ids_by_model(&self, model_id: Option<&str>) -> StorageResult<Option<HashSet<String>>> {
        let Some(model_id) = model_id else { return Ok(None) };
        let records = provider_models::Entity::find()
            .filter(provider_models::Column::GlobalModelId.eq(model_id))
            .all(self.database.connection())
            .await?;
        Ok(Some(records.into_iter().map(|record| record.provider_id).collect()))
    }

    async fn find_provider_record(&self, id_or_name: &str) -> StorageResult<Option<super::ProviderRecord>> {
        let by_id = providers::Entity::find_by_id(id_or_name.to_owned()).one(self.database.connection()).await?;
        match by_id {
            Some(record) => Ok(Some(record)),
            None => providers::Entity::find()
                .filter(providers::Column::Name.eq(id_or_name))
                .one(self.database.connection())
                .await
                .map_err(StorageError::from),
        }
    }

    async fn find_api_key_record(&self, provider_id: &str, key_id: &str) -> StorageResult<Option<super::record::ProviderApiKeyRecord>> {
        provider_api_keys::Entity::find_by_id(key_id.to_owned())
            .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
            .one(self.database.connection())
            .await
            .map_err(StorageError::from)
    }
}
