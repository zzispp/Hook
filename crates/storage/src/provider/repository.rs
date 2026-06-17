use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set};
use types::provider::{
    Provider, ProviderKeyGroup, ProviderKeyGroupListRequest, ProviderKeyGroupListResponse, ProviderListRequest, ProviderListResponse, ProviderOrigin,
    ProviderQuickImportKeySyncInfo, ProviderQuickImportSourceKind, ProviderQuickImportSyncStatus,
};

use crate::{Database, StorageError, StorageResult, json};

use super::{
    BillingRuleRecordInput, DimensionCollectorRecordInput, ProviderApiKeyPriorityRecordPatch, ProviderApiKeyRecordInput, ProviderApiKeyRecordPatch,
    ProviderApiKeySecretRecord, ProviderEndpointRecordInput, ProviderEndpointRecordPatch, ProviderModelCostRecordInput, ProviderModelRecordInput,
    ProviderModelRecordPatch, ProviderRecordInput, ProviderRecordPatch,
    record::{
        provider_api_keys,
        providers::{self, ActiveModel as ProviderActiveModel},
    },
    repository_helpers::{apply_provider_api_key_patch, apply_provider_patch, provider_active_model},
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
        super::provider_query::list_providers(self, request).await
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
            global_priority_by_format: Set(json::encode_required(&input.global_priority_by_format)?),
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
        let sync_infos = super::quick_import_sync_query::key_sync_info_by_provider(self, provider_id).await?;
        let source_not_configured = sync_infos.is_empty() && self.quick_import_provider(provider_id).await?;
        records
            .into_iter()
            .map(|record| {
                let mut response = record.response()?;
                response.quick_import_sync = sync_infos
                    .get(&response.id)
                    .cloned()
                    .or_else(|| source_not_configured.then(|| source_not_configured_sync_info(response.id.clone())));
                Ok(response)
            })
            .collect()
    }

    async fn quick_import_provider(&self, provider_id: &str) -> StorageResult<bool> {
        let record = providers::Entity::find_by_id(provider_id.to_owned()).one(self.database.connection()).await?;
        Ok(record.is_some_and(|provider| provider.provider_origin == ProviderOrigin::QuickImport.as_str()))
    }

    pub async fn find_api_key(&self, key_id: &str) -> StorageResult<Option<types::provider::ProviderApiKey>> {
        provider_api_keys::Entity::find_by_id(key_id.to_owned())
            .one(self.database.connection())
            .await?
            .map(|record| record.response())
            .transpose()
    }

    pub async fn create_provider_key_group(&self, input: super::ProviderKeyGroupRecordInput) -> StorageResult<ProviderKeyGroup> {
        super::provider_key_group_query::create_provider_key_group(self, input).await
    }

    pub async fn update_provider_key_group(&self, id: &str, input: super::ProviderKeyGroupRecordPatch) -> StorageResult<ProviderKeyGroup> {
        super::provider_key_group_query::update_provider_key_group(self, id, input).await
    }

    pub async fn delete_provider_key_group(&self, id: &str) -> StorageResult<()> {
        super::provider_key_group_query::delete_provider_key_group(self, id).await
    }

    pub async fn find_provider_key_group(&self, id_or_name: &str) -> StorageResult<Option<ProviderKeyGroup>> {
        super::provider_key_group_query::find_provider_key_group(self, id_or_name).await
    }

    pub async fn list_provider_key_groups(&self, request: ProviderKeyGroupListRequest) -> StorageResult<ProviderKeyGroupListResponse> {
        super::provider_key_group_query::list_provider_key_groups(self, request).await
    }

    pub async fn provider_key_ids_for_key_groups(&self, group_ids: &[String]) -> StorageResult<Vec<String>> {
        super::provider_key_group_query::provider_key_ids_for_key_groups(self, group_ids).await
    }

    pub async fn provider_key_priorities_for_key_groups(&self, group_ids: &[String]) -> StorageResult<std::collections::BTreeMap<String, i32>> {
        super::provider_key_group_query::provider_key_priorities_for_key_groups(self, group_ids).await
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
                    global_priority_by_format: json::decode_required(record.global_priority_by_format)?,
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

    pub async fn batch_update_api_key_priorities(
        &self,
        updates: Vec<ProviderApiKeyPriorityRecordPatch>,
    ) -> StorageResult<Vec<types::provider::ProviderApiKey>> {
        let mut output = Vec::with_capacity(updates.len());
        for update in updates {
            output.push(self.update_api_key_priority(update).await?);
        }
        Ok(output)
    }

    async fn update_api_key_priority(&self, update: ProviderApiKeyPriorityRecordPatch) -> StorageResult<types::provider::ProviderApiKey> {
        let record = self
            .find_api_key_record(&update.provider_id, &update.key_id)
            .await?
            .ok_or(StorageError::NotFound)?;
        let mut active: provider_api_keys::ActiveModel = record.into();
        active.global_priority_by_format = Set(json::encode_required(&update.global_priority_by_format)?);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?.response()
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

    pub async fn quick_import_source_for_provider(&self, provider_id: &str) -> StorageResult<Option<super::ProviderQuickImportSourceRecord>> {
        super::quick_import_sync_query::source_for_provider(self, provider_id).await
    }

    pub async fn list_quick_import_sync_sources(&self, limit: u64) -> StorageResult<Vec<super::ProviderQuickImportSourceRecord>> {
        super::quick_import_sync_query::list_sources(self, limit).await
    }

    pub async fn quick_import_sync_keys(&self, source_id: &str) -> StorageResult<Vec<super::ProviderQuickImportSyncKeyRecord>> {
        super::quick_import_sync_query::keys_for_source(self, source_id).await
    }

    pub async fn quick_import_sync_key(&self, provider_id: &str, key_id: &str) -> StorageResult<Option<super::ProviderQuickImportSyncKeyRecord>> {
        super::quick_import_sync_query::key_for_provider_key(self, provider_id, key_id).await
    }

    pub async fn update_quick_import_source(
        &self,
        provider_id: &str,
        patch: super::ProviderQuickImportSourceRecordPatch,
    ) -> StorageResult<super::ProviderQuickImportSourceRecord> {
        super::quick_import_sync_query::update_source(self, provider_id, patch).await
    }

    pub async fn update_quick_import_source_run(
        &self,
        source_id: &str,
        status: Option<types::provider::ProviderQuickImportSyncStatus>,
        error: Option<String>,
        failed: bool,
    ) -> StorageResult<()> {
        super::quick_import_sync_query::update_source_run(self, source_id, status, error, failed).await
    }

    pub async fn update_quick_import_sync_keys(&self, provider_id: &str, patches: Vec<super::ProviderQuickImportSyncKeyRecordPatch>) -> StorageResult<()> {
        super::quick_import_sync_query::update_keys(self, provider_id, patches).await
    }

    pub async fn create_quick_import_sync_events(&self, input: Vec<super::ProviderQuickImportSyncEventRecordInput>) -> StorageResult<()> {
        super::quick_import_sync_event_query::create_events(self, input).await
    }

    pub async fn upsert_model_costs(&self, inputs: Vec<ProviderModelCostRecordInput>) -> StorageResult<Vec<types::provider::ProviderModelCost>> {
        super::provider_model_cost_query::upsert_model_costs(self, inputs).await
    }

    pub async fn create_quick_import(&self, input: super::ProviderQuickImportRecordInput) -> StorageResult<super::ProviderQuickImportRecordOutput> {
        super::quick_import_query::create_quick_import(self, input).await
    }

    pub async fn append_quick_import(&self, input: super::ProviderQuickImportAppendRecordInput) -> StorageResult<super::ProviderQuickImportAppendRecordOutput> {
        super::quick_import_query::append_quick_import(self, input).await
    }

    pub async fn bind_quick_import(&self, input: super::ProviderQuickImportBindRecordInput) -> StorageResult<super::ProviderQuickImportBindRecordOutput> {
        super::quick_import_bind_query::bind_quick_import(self, input).await
    }

    pub async fn replace_quick_import_key(
        &self,
        input: super::ProviderQuickImportKeyReplacementRecordInput,
    ) -> StorageResult<super::ProviderQuickImportKeyReplacementRecordOutput> {
        super::quick_import_query::replace_quick_import_key(self, input).await
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

fn source_not_configured_sync_info(key_id: String) -> ProviderQuickImportKeySyncInfo {
    ProviderQuickImportKeySyncInfo {
        source_kind: ProviderQuickImportSourceKind::Newapi,
        upstream_token_id: key_id,
        upstream_group: None,
        upstream_group_ratio: Decimal::ONE,
        effective_cost_multiplier: Decimal::ONE,
        statuses: vec![ProviderQuickImportSyncStatus::SourceNotConfigured],
        last_synced_at: None,
        last_error: None,
    }
}
