use std::collections::HashSet;

use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set};
use types::provider::{Provider, ProviderListRequest, ProviderListResponse};

use crate::{Database, StorageError, StorageResult, json};

use super::{
    ProviderApiKeyRecordInput, ProviderApiKeyRecordPatch, ProviderEndpointRecordInput, ProviderEndpointRecordPatch, ProviderModelRecordInput,
    ProviderModelRecordPatch, ProviderRecordInput, ProviderRecordPatch,
    record::{
        provider_api_keys, provider_endpoints, provider_models,
        providers::{self, ActiveModel as ProviderActiveModel},
    },
    repository_helpers::{
        ProviderFilterIds, apply_endpoint_patch, apply_provider_api_key_patch, apply_provider_patch, endpoint_belongs_to_provider, filter_provider_records,
        provider_active_model,
    },
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
        let now = time::OffsetDateTime::now_utc();
        let record = provider_endpoints::ActiveModel {
            id: Set(self.database.next_id()),
            provider_id: Set(input.provider_id),
            api_format: Set(input.api_format),
            base_url: Set(input.base_url),
            custom_path: Set(input.custom_path),
            max_retries: Set(input.max_retries),
            is_active: Set(input.is_active),
            format_acceptance_config: Set(json::encode_optional(&input.format_acceptance_config)?),
            header_rules: Set(json::encode_optional(&input.header_rules)?),
            body_rules: Set(json::encode_optional(&input.body_rules)?),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(self.database.connection())
        .await?;
        record.response()
    }

    pub async fn endpoints_for_provider(&self, provider_id: &str) -> StorageResult<Vec<types::provider::ProviderEndpoint>> {
        let records = provider_endpoints::Entity::find()
            .filter(provider_endpoints::Column::ProviderId.eq(provider_id))
            .order_by_asc(provider_endpoints::Column::ApiFormat)
            .all(self.database.connection())
            .await?;
        records.into_iter().map(|record| record.response()).collect()
    }

    pub async fn update_endpoint(
        &self,
        provider_id: &str,
        endpoint_id: &str,
        input: ProviderEndpointRecordPatch,
    ) -> StorageResult<types::provider::ProviderEndpoint> {
        let record = provider_endpoints::Entity::find_by_id(endpoint_id.to_owned())
            .one(self.database.connection())
            .await?
            .ok_or(StorageError::NotFound)?;
        if !endpoint_belongs_to_provider(&record, provider_id) {
            return Err(StorageError::NotFound);
        }

        let mut active: provider_endpoints::ActiveModel = record.into();
        apply_endpoint_patch(&mut active, input)?;
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        let record = active.update(self.database.connection()).await?;
        record.response()
    }

    pub async fn delete_endpoint(&self, provider_id: &str, endpoint_id: &str) -> StorageResult<()> {
        let record = provider_endpoints::Entity::find_by_id(endpoint_id.to_owned())
            .one(self.database.connection())
            .await?
            .ok_or(StorageError::NotFound)?;
        if !endpoint_belongs_to_provider(&record, provider_id) {
            return Err(StorageError::NotFound);
        }

        let active: provider_endpoints::ActiveModel = record.into();
        active.delete(self.database.connection()).await?;
        Ok(())
    }

    pub async fn create_api_key(&self, input: ProviderApiKeyRecordInput) -> StorageResult<types::provider::ProviderApiKey> {
        let now = time::OffsetDateTime::now_utc();
        let record = provider_api_keys::ActiveModel {
            id: Set(self.database.next_id()),
            provider_id: Set(input.provider_id),
            name: Set(input.name),
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
            ..Default::default()
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
