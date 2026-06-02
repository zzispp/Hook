use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    api_token::{ApiToken, ApiTokenType, ModelAccessMode},
    model_status::*,
};

use super::{
    ModelStatusDispatchOptions, ModelStatusProbe, ModelStatusProbeInput, ModelStatusProbeOptions, ModelStatusProbeResult, ModelStatusRepository,
    ModelStatusResult, ModelStatusRunRecord, ModelStatusService, ModelStatusTokenCatalog, ModelStatusUseCase,
};

#[tokio::test]
async fn run_due_checks_defers_throttled_probe() {
    let repository = MemoryRepository::with_due(vec![probe_input()]);
    let service = ModelStatusService::new(repository.clone(), StaticTokenCatalog, DeferredProbe);

    let report = service
        .run_due_checks(ModelStatusDispatchOptions {
            limit: 20,
            concurrency: 4,
            provider_key_min_interval_seconds: 1,
            provider_key_probe_wait_timeout_seconds: 30,
        })
        .await
        .unwrap();

    assert_eq!(report.probed_count, 0);
    assert_eq!(report.deferred_count, 1);
    assert_eq!(report.scanned_count, 1);
    assert_eq!(report.pages_count, 1);
    assert_eq!(repository.recorded_count(), 0);
    assert_eq!(repository.deferred_ids(), vec!["check-1"]);
}

#[derive(Clone, Default)]
struct MemoryRepository {
    due: Arc<Mutex<Vec<ModelStatusProbeInput>>>,
    records: Arc<Mutex<Vec<ModelStatusRunRecord>>>,
    deferred: Arc<Mutex<Vec<String>>>,
}

impl MemoryRepository {
    fn with_due(due: Vec<ModelStatusProbeInput>) -> Self {
        Self {
            due: Arc::new(Mutex::new(due)),
            ..Self::default()
        }
    }

    fn recorded_count(&self) -> usize {
        self.records.lock().unwrap().len()
    }

    fn deferred_ids(&self) -> Vec<String> {
        self.deferred.lock().unwrap().clone()
    }
}

#[async_trait]
impl ModelStatusRepository for MemoryRepository {
    async fn list_public(&self, _request: ModelStatusListRequest) -> ModelStatusResult<ModelStatusCheckListResponse> {
        unimplemented!()
    }

    async fn list_admin(&self, _request: ModelStatusListRequest) -> ModelStatusResult<ModelStatusCheckListResponse> {
        unimplemented!()
    }

    async fn create_check(&self, _input: ModelStatusCheckCreate) -> ModelStatusResult<ModelStatusCheckResponse> {
        unimplemented!()
    }

    async fn batch_create_checks(&self, _input: ModelStatusCheckBatchCreateRequest) -> ModelStatusResult<ModelStatusCheckBatchCreateResponse> {
        unimplemented!()
    }

    async fn update_check(&self, _id: &str, _input: ModelStatusCheckUpdate) -> ModelStatusResult<ModelStatusCheckResponse> {
        unimplemented!()
    }

    async fn delete_check(&self, _id: &str) -> ModelStatusResult<()> {
        unimplemented!()
    }

    async fn batch_update_checks(&self, _input: ModelStatusCheckBatchUpdateRequest) -> ModelStatusResult<ModelStatusCheckBatchUpdateResponse> {
        unimplemented!()
    }

    async fn list_runs(&self, _request: ModelStatusRunListRequest) -> ModelStatusResult<ModelStatusRunListResponse> {
        unimplemented!()
    }

    async fn due_checks(&self, limit: u64, _now: time::OffsetDateTime) -> ModelStatusResult<Vec<ModelStatusProbeInput>> {
        let mut due = self.due.lock().unwrap();
        let count = usize::try_from(limit).unwrap().min(due.len());
        Ok(due.drain(..count).collect())
    }

    async fn record_run(&self, record: ModelStatusRunRecord, _interval_seconds: i64) -> ModelStatusResult<()> {
        self.records.lock().unwrap().push(record);
        Ok(())
    }

    async fn defer_check(&self, check_id: &str, _next_due_at: time::OffsetDateTime) -> ModelStatusResult<()> {
        self.deferred.lock().unwrap().push(check_id.to_owned());
        Ok(())
    }

    async fn token_has_checks(&self, _token_id: &str) -> ModelStatusResult<bool> {
        Ok(false)
    }
}

#[derive(Clone, Copy)]
struct StaticTokenCatalog;

#[async_trait]
impl ModelStatusTokenCatalog for StaticTokenCatalog {
    async fn independent_token(&self, _id: &str) -> ModelStatusResult<Option<ApiToken>> {
        Ok(Some(token()))
    }
}

#[derive(Clone, Copy)]
struct DeferredProbe;

#[async_trait]
impl ModelStatusProbe for DeferredProbe {
    async fn probe(&self, _input: ModelStatusProbeInput, _options: ModelStatusProbeOptions) -> ModelStatusProbeResult {
        ModelStatusProbeResult::Deferred
    }
}

fn probe_input() -> ModelStatusProbeInput {
    ModelStatusProbeInput {
        check_id: "check-1".into(),
        model_name: "gpt-test".into(),
        api_format: "openai:chat".into(),
        interval_seconds: 300,
        token: token(),
    }
}

fn token() -> ApiToken {
    ApiToken {
        id: "token-1".into(),
        user_id: None,
        token_type: ApiTokenType::Independent,
        name: "status token".into(),
        token_value: "hk-test-secret".into(),
        token_hash: "hash".into(),
        token_prefix: "hk-test".into(),
        group_code: constants::billing::DEFAULT_SYSTEM_GROUP_CODE.into(),
        expires_at: None,
        model_access_mode: ModelAccessMode::All,
        allowed_model_ids: Vec::new(),
        rate_limit_rpm: None,
        quota_limit: None,
        used_quota: Decimal::ZERO,
        request_count: 0,
        is_active: true,
        last_used_at: None,
        created_at: "2026-05-29T00:00:00Z".into(),
        updated_at: "2026-05-29T00:00:00Z".into(),
    }
}
