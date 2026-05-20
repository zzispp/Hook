use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set};
use types::model::{GlobalModelListRequest, GlobalModelResponse, GlobalModelWithStats, ModelCatalogItem, ModelCatalogResponse};

use crate::{Database, StorageError, StorageResult, json, usage_flush::UsageFlushApplyReport};

use super::{
    GlobalModelRecord, GlobalModelRecordInput, GlobalModelRecordPatch, GlobalModelUsageRecord, ModelRecord,
    record::{global_models, global_models::ActiveModel as GlobalModelActiveModel, provider_models},
    repository_helpers::{apply_global_model_patch, capabilities, description, record_matches, unique_provider_count},
    usage,
};

#[derive(Clone)]
pub struct ModelStore {
    database: Database,
}

impl ModelStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create_global_model(&self, input: GlobalModelRecordInput) -> StorageResult<GlobalModelResponse> {
        let now = time::OffsetDateTime::now_utc();
        let record = GlobalModelActiveModel {
            id: Set(self.database.next_id()),
            name: Set(input.name),
            display_name: Set(input.display_name),
            default_price_per_request: Set(input.default_price_per_request),
            default_tiered_pricing: Set(json::encode_required(&input.default_tiered_pricing)?),
            supported_capabilities: Set(json::encode_optional(&input.supported_capabilities)?),
            config: Set(json::encode_optional(&input.config)?),
            is_active: Set(input.is_active),
            usage_count: Set(0),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(self.database.connection())
        .await?;
        record.with_counts(0, 0)
    }

    pub async fn update_global_model(&self, id: &str, input: GlobalModelRecordPatch) -> StorageResult<GlobalModelResponse> {
        let record = self.find_global_model_record(id).await?.ok_or(StorageError::NotFound)?;
        let mut active: GlobalModelActiveModel = record.into();
        apply_global_model_patch(&mut active, input)?;
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?;
        self.get_global_model(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_global_model(&self, id: &str) -> StorageResult<()> {
        let record = self.find_global_model_record(id).await?.ok_or(StorageError::NotFound)?;
        provider_models::Entity::delete_many()
            .filter(provider_models::Column::GlobalModelId.eq(record.id.as_str()))
            .exec(self.database.connection())
            .await?;
        let active: GlobalModelActiveModel = record.into();
        active.delete(self.database.connection()).await?;
        Ok(())
    }

    pub async fn find_global_model_by_name(&self, name: &str) -> StorageResult<Option<GlobalModelResponse>> {
        match self.find_global_model_record_by_name(name).await? {
            Some(record) => self.to_response(record).await.map(Some),
            None => Ok(None),
        }
    }

    pub async fn get_global_model(&self, id: &str) -> StorageResult<Option<GlobalModelResponse>> {
        match self.find_global_model_record(id).await? {
            Some(record) => self.to_response(record).await.map(Some),
            None => Ok(None),
        }
    }

    pub async fn get_global_model_with_stats(&self, id: &str) -> StorageResult<Option<GlobalModelWithStats>> {
        let Some(record) = self.find_global_model_record(id).await? else {
            return Ok(None);
        };
        let provider_count = self.provider_count(&record.id).await?;
        let active_provider_count = self.active_provider_count(&record.id).await?;
        let total_models = self.model_count(&record.id).await?;
        let price_range = record.price_range()?;
        Ok(Some(GlobalModelWithStats {
            total_models,
            total_providers: provider_count,
            price_range,
            model: record.with_counts(provider_count, active_provider_count)?,
        }))
    }

    pub async fn list_global_models(&self, request: GlobalModelListRequest) -> StorageResult<types::model::GlobalModelListResponse> {
        let records = self.filtered_records(&request).await?;
        let total = records.len() as u64;
        let page = records.into_iter().skip(request.skip as usize).take(request.limit as usize);
        let mut models = Vec::new();
        for record in page {
            models.push(self.to_response(record).await?);
        }
        Ok(types::model::GlobalModelListResponse { models, total })
    }

    pub async fn global_model_providers(&self, id: &str) -> StorageResult<types::model::GlobalModelProvidersResponse> {
        let global_model = self.find_global_model_record(id).await?.ok_or(StorageError::NotFound)?;
        let mut providers = Vec::new();
        for model in self.models_for_global_model(&global_model.id).await? {
            providers.push(model.provider_detail(&global_model)?);
        }
        Ok(types::model::GlobalModelProvidersResponse {
            total: providers.len() as u64,
            providers,
        })
    }

    pub async fn catalog(&self) -> StorageResult<ModelCatalogResponse> {
        let mut items = Vec::new();
        for record in self.active_global_model_records().await? {
            items.push(self.catalog_item(record).await?);
        }
        Ok(ModelCatalogResponse {
            total: items.len() as u64,
            models: items,
        })
    }

    pub async fn record_usage(&self, input: GlobalModelUsageRecord) -> StorageResult<()> {
        usage::record_usage(self.database.connection(), &input).await
    }

    pub async fn record_usage_batch(&self, inputs: &[GlobalModelUsageRecord]) -> StorageResult<()> {
        usage::record_usage_batch(self.database.connection(), inputs).await
    }

    pub async fn record_usage_batch_once(&self, batch_id: &str, inputs: &[GlobalModelUsageRecord]) -> StorageResult<UsageFlushApplyReport> {
        usage::record_usage_batch_once(self.database.connection(), batch_id, inputs).await
    }
}

impl ModelStore {
    async fn catalog_item(&self, record: GlobalModelRecord) -> StorageResult<ModelCatalogItem> {
        let providers = self.active_provider_details(&record).await?;
        let price_range = record.price_range()?;
        let capabilities = capabilities(&record)?;
        let description = description(record.config()?.as_ref());
        Ok(ModelCatalogItem {
            global_model_id: record.id,
            global_model_name: record.name,
            display_name: record.display_name,
            description,
            total_providers: providers.len() as u64,
            providers,
            price_range,
            capabilities,
        })
    }

    async fn active_provider_details(&self, record: &GlobalModelRecord) -> StorageResult<Vec<types::model::ModelCatalogProviderDetail>> {
        let mut providers = Vec::new();
        for model in self.models_for_global_model(&record.id).await? {
            providers.push(model.provider_detail(record)?);
        }
        Ok(providers)
    }

    async fn to_response(&self, record: GlobalModelRecord) -> StorageResult<GlobalModelResponse> {
        let provider_count = self.provider_count(&record.id).await?;
        let active_provider_count = self.active_provider_count(&record.id).await?;
        record.with_counts(provider_count, active_provider_count)
    }

    async fn filtered_records(&self, request: &GlobalModelListRequest) -> StorageResult<Vec<GlobalModelRecord>> {
        let search = request.search.as_ref().map(|value| value.to_ascii_lowercase());
        let mut records = self.global_model_records().await?;
        records.retain(|record| record_matches(record, request.is_active, search.as_deref()));
        records.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(records)
    }

    async fn provider_count(&self, id: &str) -> StorageResult<u64> {
        Ok(unique_provider_count(self.models_for_global_model(id).await?))
    }

    async fn active_provider_count(&self, id: &str) -> StorageResult<u64> {
        Ok(unique_provider_count(self.models_for_global_model(id).await?))
    }

    async fn model_count(&self, id: &str) -> StorageResult<u64> {
        provider_models::Entity::find()
            .filter(provider_models::Column::GlobalModelId.eq(id))
            .count(self.database.connection())
            .await
            .map_err(StorageError::from)
    }

    async fn global_model_records(&self) -> StorageResult<Vec<GlobalModelRecord>> {
        global_models::Entity::find()
            .order_by_asc(global_models::Column::Name)
            .all(self.database.connection())
            .await
            .map_err(StorageError::from)
    }

    async fn active_global_model_records(&self) -> StorageResult<Vec<GlobalModelRecord>> {
        global_models::Entity::find()
            .filter(global_models::Column::IsActive.eq(true))
            .order_by_asc(global_models::Column::Name)
            .all(self.database.connection())
            .await
            .map_err(StorageError::from)
    }

    async fn models_for_global_model(&self, id: &str) -> StorageResult<Vec<ModelRecord>> {
        provider_models::Entity::find()
            .filter(provider_models::Column::GlobalModelId.eq(id))
            .all(self.database.connection())
            .await
            .map_err(StorageError::from)
    }

    async fn find_global_model_record(&self, id: &str) -> StorageResult<Option<GlobalModelRecord>> {
        let record = global_models::Entity::find_by_id(id.to_owned()).one(self.database.connection()).await?;
        match record {
            Some(record) => Ok(Some(record)),
            None => self.find_global_model_record_by_name(id).await,
        }
    }

    async fn find_global_model_record_by_name(&self, name: &str) -> StorageResult<Option<GlobalModelRecord>> {
        global_models::Entity::find()
            .filter(global_models::Column::Name.eq(name))
            .one(self.database.connection())
            .await
            .map_err(StorageError::from)
    }
}
