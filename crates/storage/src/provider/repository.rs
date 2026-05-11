use std::collections::HashSet;

use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait};
use types::provider::{ActiveRequestRecordRequest, ActiveRequestRecordResponse, Provider, ProviderListRequest, ProviderListResponse, RequestRecordListRequest};

use crate::{Database, StorageError, StorageResult, json};

use super::{
    ProviderApiKeyRecordInput, ProviderEndpointRecordInput, ProviderEndpointRecordPatch, ProviderModelRecordInput, ProviderRecordInput, ProviderRecordPatch,
    record::{
        provider_api_keys, provider_endpoints, provider_models,
        providers::{self, ActiveModel as ProviderActiveModel},
    },
    repository_helpers::{
        ProviderFilterIds, apply_endpoint_patch, apply_provider_patch, endpoint_belongs_to_provider, filter_provider_records, provider_active_model,
        provider_model_response, remove_api_format_from_keys,
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

        let api_format = record.api_format.clone();
        let tx = self.database.connection().begin().await?;
        remove_api_format_from_keys(provider_id, &api_format, &tx).await?;
        let active: provider_endpoints::ActiveModel = record.into();
        active.delete(&tx).await?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn create_api_key(&self, input: ProviderApiKeyRecordInput) -> StorageResult<types::provider::ProviderApiKey> {
        let now = time::OffsetDateTime::now_utc();
        let record = provider_api_keys::ActiveModel {
            id: Set(self.database.next_id()),
            provider_id: Set(input.provider_id),
            name: Set(input.name),
            encrypted_api_key: Set(input.encrypted_api_key),
            note: Set(input.note),
            api_formats: Set(json::encode_optional(&input.api_formats)?),
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

    pub async fn create_model_binding(&self, input: ProviderModelRecordInput) -> StorageResult<types::provider::ProviderModelBinding> {
        let now = time::OffsetDateTime::now_utc();
        let record = provider_models::ActiveModel {
            id: Set(self.database.next_id()),
            provider_id: Set(input.provider_id),
            global_model_id: Set(input.global_model_id),
            provider_model_name: Set(input.provider_model_name),
            provider_model_mappings: Set(None),
            price_per_request: Set(input.price_per_request),
            tiered_pricing: Set(json::encode_optional(&input.tiered_pricing)?),
            config: Set(json::encode_optional(&input.config)?),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(self.database.connection())
        .await?;
        provider_model_response(record)
    }

    pub async fn model_bindings_for_provider(&self, provider_id: &str) -> StorageResult<Vec<types::provider::ProviderModelBinding>> {
        let records = provider_models::Entity::find()
            .filter(provider_models::Column::ProviderId.eq(provider_id))
            .order_by_asc(provider_models::Column::ProviderModelName)
            .all(self.database.connection())
            .await?;
        records.into_iter().map(provider_model_response).collect()
    }

    pub async fn create_request_candidate(&self, input: super::RequestCandidateRecordInput) -> StorageResult<types::provider::RequestCandidate> {
        super::request_candidate_query::create_request_candidate(self, input).await
    }

    pub async fn list_request_candidates(
        &self,
        request: types::provider::RequestCandidateListRequest,
    ) -> StorageResult<Vec<types::provider::RequestCandidate>> {
        super::request_candidate_query::list_request_candidates(self, request).await
    }

    pub async fn list_request_records(&self, request: RequestRecordListRequest) -> StorageResult<types::provider::RequestRecordListResponse> {
        super::request_record_query::list_request_records(self, request).await
    }

    pub async fn list_active_request_records(&self, request: ActiveRequestRecordRequest) -> StorageResult<ActiveRequestRecordResponse> {
        super::request_record_query::list_active_request_records(self, request).await
    }

    pub async fn get_request_record(&self, request_id: &str) -> StorageResult<types::provider::RequestRecordDetail> {
        super::request_record_query::get_request_record(self, request_id).await
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
}
