use async_trait::async_trait;
use storage::{
    Database, StorageError,
    model_status::{ModelStatusRunRecordInput, ModelStatusRunStatusValue, ModelStatusStore},
};
use types::{api_token::ApiToken, model_status::*};

use crate::application::{
    ModelStatusError, ModelStatusProbeInput, ModelStatusRepository, ModelStatusResult, ModelStatusRunRecord, ModelStatusRunStatus, ModelStatusTokenCatalog,
};

#[derive(Clone)]
pub struct StorageModelStatusRepository {
    store: ModelStatusStore,
}

#[derive(Clone)]
pub struct StorageModelStatusTokenCatalog {
    store: ModelStatusStore,
}

impl StorageModelStatusRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: ModelStatusStore::new(database),
        }
    }
}

impl StorageModelStatusTokenCatalog {
    pub fn new(database: Database) -> Self {
        Self {
            store: ModelStatusStore::new(database),
        }
    }
}

#[async_trait]
impl ModelStatusRepository for StorageModelStatusRepository {
    async fn list_public(&self, request: ModelStatusListRequest) -> ModelStatusResult<ModelStatusCheckListResponse> {
        self.store.list_public(request).await.map_err(storage_error)
    }

    async fn list_admin(&self, request: ModelStatusListRequest) -> ModelStatusResult<ModelStatusCheckListResponse> {
        self.store.list_admin(request).await.map_err(storage_error)
    }

    async fn create_check(&self, input: ModelStatusCheckCreate) -> ModelStatusResult<ModelStatusCheckResponse> {
        self.store.create_check(input).await.map_err(storage_error)
    }

    async fn batch_create_checks(&self, input: ModelStatusCheckBatchCreateRequest) -> ModelStatusResult<ModelStatusCheckBatchCreateResponse> {
        self.store.batch_create_checks(input).await.map_err(storage_error)
    }

    async fn update_check(&self, id: &str, input: ModelStatusCheckUpdate) -> ModelStatusResult<ModelStatusCheckResponse> {
        self.store.update_check(id, input).await.map_err(storage_error)
    }

    async fn delete_check(&self, id: &str) -> ModelStatusResult<()> {
        self.store.delete_check(id).await.map_err(storage_error)
    }

    async fn batch_update_checks(&self, input: ModelStatusCheckBatchUpdateRequest) -> ModelStatusResult<ModelStatusCheckBatchUpdateResponse> {
        self.store.batch_update_checks(input).await.map_err(storage_error)
    }

    async fn list_runs(&self, request: ModelStatusRunListRequest) -> ModelStatusResult<ModelStatusRunListResponse> {
        self.store.list_runs(request).await.map_err(storage_error)
    }

    async fn due_checks(&self, limit: u64, now: time::OffsetDateTime) -> ModelStatusResult<Vec<ModelStatusProbeInput>> {
        let records = self.store.due_checks(limit, now).await.map_err(storage_error)?;
        Ok(records
            .into_iter()
            .map(|record| ModelStatusProbeInput {
                check_id: record.check_id,
                model_name: record.model_name,
                api_format: record.api_format,
                interval_seconds: record.interval_seconds,
                token: record.token,
            })
            .collect())
    }

    async fn record_run(&self, record: ModelStatusRunRecord, interval_seconds: i64) -> ModelStatusResult<()> {
        self.store.record_run(record_input(record), interval_seconds).await.map_err(storage_error)
    }

    async fn defer_check(&self, check_id: &str, next_due_at: time::OffsetDateTime) -> ModelStatusResult<()> {
        self.store.defer_check(check_id, next_due_at).await.map_err(storage_error)
    }

    async fn token_has_checks(&self, token_id: &str) -> ModelStatusResult<bool> {
        self.store.token_has_checks(token_id).await.map_err(storage_error)
    }
}

#[async_trait]
impl ModelStatusTokenCatalog for StorageModelStatusTokenCatalog {
    async fn independent_token(&self, id: &str) -> ModelStatusResult<Option<ApiToken>> {
        self.store.independent_token(id).await.map_err(storage_error)
    }
}

fn record_input(record: ModelStatusRunRecord) -> ModelStatusRunRecordInput {
    ModelStatusRunRecordInput {
        check_id: record.check_id,
        status: status_value(record.status),
        latency_ms: record.latency_ms,
        status_code: record.status_code,
        message: record.message,
        checked_at: record.checked_at,
    }
}

fn status_value(status: ModelStatusRunStatus) -> ModelStatusRunStatusValue {
    match status {
        ModelStatusRunStatus::Operational => ModelStatusRunStatusValue::Operational,
        ModelStatusRunStatus::Degraded => ModelStatusRunStatusValue::Degraded,
        ModelStatusRunStatus::Failed => ModelStatusRunStatusValue::Failed,
        ModelStatusRunStatus::Error => ModelStatusRunStatusValue::Error,
    }
}

fn storage_error(error: StorageError) -> ModelStatusError {
    match error {
        StorageError::NotFound => ModelStatusError::NotFound,
        StorageError::Conflict(message) => ModelStatusError::Conflict(message),
        StorageError::Database(message) => ModelStatusError::Infrastructure(message),
    }
}
