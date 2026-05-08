use types::model::{GlobalModelListRequest, GlobalModelResponse, GlobalModelWithStats, ModelCapabilities, ModelCatalogItem, ModelCatalogResponse, PatchField};

use crate::{Database, StorageError, StorageResult};

use super::{GlobalModelRecord, GlobalModelRecordInput, GlobalModelRecordPatch, ModelRecord};

#[derive(Clone)]
pub struct ModelStore {
    database: Database,
}

impl ModelStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create_global_model(&self, input: GlobalModelRecordInput) -> StorageResult<GlobalModelResponse> {
        let mut db = self.database.connection();
        toasty::create!(GlobalModelRecord {
            id: self.database.next_id(),
            name: input.name,
            display_name: input.display_name,
            default_price_per_request: input.default_price_per_request,
            default_tiered_pricing: input.default_tiered_pricing,
            supported_capabilities: input.supported_capabilities,
            config: input.config,
            is_active: input.is_active,
            usage_count: 0,
        })
        .exec(&mut db)
        .await
        .map(|record| record.with_counts(0, 0))
        .map_err(StorageError::from)
    }

    pub async fn update_global_model(&self, id: &str, input: GlobalModelRecordPatch) -> StorageResult<GlobalModelResponse> {
        let mut db = self.database.connection();
        let mut record = self.find_global_model_record(id).await?.ok_or(StorageError::NotFound)?;
        let mut update = record.update();
        if let Some(display_name) = input.display_name {
            update.set_display_name(display_name);
        }
        if let Some(is_active) = input.is_active {
            update.set_is_active(is_active);
        }
        match input.default_price_per_request {
            PatchField::Value(value) => {
                update.set_default_price_per_request(Some(value));
            }
            PatchField::Null => {
                update.set_default_price_per_request(None);
            }
            PatchField::Missing => {}
        };
        if let Some(pricing) = input.default_tiered_pricing {
            update.set_default_tiered_pricing(pricing);
        }
        match input.supported_capabilities {
            PatchField::Value(value) => {
                update.set_supported_capabilities(Some(value));
            }
            PatchField::Null => {
                update.set_supported_capabilities(None);
            }
            PatchField::Missing => {}
        };
        match input.config {
            PatchField::Value(value) => {
                update.set_config(Some(value));
            }
            PatchField::Null => {
                update.set_config(None);
            }
            PatchField::Missing => {}
        };
        update.exec(&mut db).await.map_err(StorageError::from)?;
        self.get_global_model(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_global_model(&self, id: &str) -> StorageResult<()> {
        let mut db = self.database.connection();
        let record = self.find_global_model_record(id).await?.ok_or(StorageError::NotFound)?;
        ModelRecord::filter(ModelRecord::fields().global_model_id().eq(record.id.as_str()))
            .delete()
            .exec(&mut db)
            .await?;
        record.delete().exec(&mut db).await?;
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
        let record_id = record.id.clone();
        Ok(Some(GlobalModelWithStats {
            total_models: self.model_count(&record_id).await?,
            total_providers: provider_count,
            price_range: record.price_range(),
            model: record.with_counts(provider_count, active_provider_count),
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
        let providers: Vec<_> = self
            .models_for_global_model(&global_model.id)
            .await?
            .into_iter()
            .map(|model| model.provider_detail(&global_model))
            .collect();
        let total = providers.len() as u64;
        Ok(types::model::GlobalModelProvidersResponse { providers, total })
    }

    pub async fn catalog(&self) -> StorageResult<ModelCatalogResponse> {
        let mut items = Vec::new();
        for record in self.active_global_model_records().await? {
            let providers = self.models_for_global_model(&record.id).await?;
            let details = providers
                .into_iter()
                .filter(|model| model.is_active)
                .map(|model| model.provider_detail(&record))
                .collect::<Vec<_>>();
            items.push(catalog_item(record, details));
        }
        Ok(ModelCatalogResponse {
            total: items.len() as u64,
            models: items,
        })
    }

    async fn to_response(&self, record: GlobalModelRecord) -> StorageResult<GlobalModelResponse> {
        let provider_count = self.provider_count(&record.id).await?;
        let active_provider_count = self.active_provider_count(&record.id).await?;
        Ok(record.with_counts(provider_count, active_provider_count))
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
        Ok(unique_provider_count(
            self.models_for_global_model(id)
                .await?
                .into_iter()
                .filter(|model| model.is_active && model.is_available)
                .collect(),
        ))
    }

    async fn model_count(&self, id: &str) -> StorageResult<u64> {
        let mut db = self.database.connection();
        ModelRecord::filter(ModelRecord::fields().global_model_id().eq(id))
            .count()
            .exec(&mut db)
            .await
            .map_err(StorageError::from)
    }

    async fn global_model_records(&self) -> StorageResult<Vec<GlobalModelRecord>> {
        let mut db = self.database.connection();
        GlobalModelRecord::all()
            .order_by(GlobalModelRecord::fields().name().asc())
            .exec(&mut db)
            .await
            .map_err(StorageError::from)
    }

    async fn active_global_model_records(&self) -> StorageResult<Vec<GlobalModelRecord>> {
        let mut db = self.database.connection();
        GlobalModelRecord::filter(GlobalModelRecord::fields().is_active().eq(true))
            .order_by(GlobalModelRecord::fields().name().asc())
            .exec(&mut db)
            .await
            .map_err(StorageError::from)
    }

    async fn models_for_global_model(&self, id: &str) -> StorageResult<Vec<ModelRecord>> {
        let mut db = self.database.connection();
        ModelRecord::filter(ModelRecord::fields().global_model_id().eq(id))
            .exec(&mut db)
            .await
            .map_err(StorageError::from)
    }

    async fn find_global_model_record(&self, id: &str) -> StorageResult<Option<GlobalModelRecord>> {
        let mut db = self.database.connection();
        let record = GlobalModelRecord::filter(GlobalModelRecord::fields().id().eq(id)).first().exec(&mut db).await?;
        match record {
            Some(record) => Ok(Some(record)),
            None => GlobalModelRecord::filter(GlobalModelRecord::fields().name().eq(id))
                .first()
                .exec(&mut db)
                .await
                .map_err(StorageError::from),
        }
    }

    async fn find_global_model_record_by_name(&self, name: &str) -> StorageResult<Option<GlobalModelRecord>> {
        let mut db = self.database.connection();
        GlobalModelRecord::filter(GlobalModelRecord::fields().name().eq(name))
            .first()
            .exec(&mut db)
            .await
            .map_err(StorageError::from)
    }
}

fn catalog_item(record: GlobalModelRecord, providers: Vec<types::model::ModelCatalogProviderDetail>) -> ModelCatalogItem {
    let price_range = record.price_range();
    let capabilities = capabilities(&record, &providers);
    ModelCatalogItem {
        global_model_name: record.name,
        display_name: record.display_name,
        description: description(record.config.as_ref()),
        total_providers: providers.len() as u64,
        providers,
        price_range,
        capabilities,
    }
}

fn capabilities(record: &GlobalModelRecord, providers: &[types::model::ModelCatalogProviderDetail]) -> ModelCapabilities {
    ModelCapabilities {
        supports_vision: config_bool(record.config.as_ref(), "vision") || providers.iter().any(|provider| provider.supports_vision == Some(true)),
        supports_function_calling: config_bool(record.config.as_ref(), "function_calling")
            || providers.iter().any(|provider| provider.supports_function_calling == Some(true)),
        supports_streaming: config_bool_default(record.config.as_ref(), "streaming", true)
            || providers.iter().any(|provider| provider.supports_streaming == Some(true)),
    }
}

fn config_bool(config: Option<&serde_json::Value>, key: &str) -> bool {
    config_bool_default(config, key, false)
}

fn config_bool_default(config: Option<&serde_json::Value>, key: &str, default: bool) -> bool {
    config.and_then(|value| value.get(key)).and_then(serde_json::Value::as_bool).unwrap_or(default)
}

fn description(config: Option<&serde_json::Value>) -> Option<String> {
    config
        .and_then(|value| value.get("description"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
}

fn record_matches(record: &GlobalModelRecord, active: Option<bool>, search: Option<&str>) -> bool {
    let active_matches = active.is_none_or(|expected| record.is_active == expected);
    active_matches && search.is_none_or(|query| record.name.to_ascii_lowercase().contains(query) || record.display_name.to_ascii_lowercase().contains(query))
}

fn unique_provider_count(mut records: Vec<ModelRecord>) -> u64 {
    records.sort_by(|left, right| left.provider_id.cmp(&right.provider_id));
    records.dedup_by(|left, right| left.provider_id == right.provider_id);
    records.len() as u64
}
