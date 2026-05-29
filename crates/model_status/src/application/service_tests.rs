use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    api_token::{ApiToken, ApiTokenType, ModelAccessMode},
    model_status::{
        ModelStatusAvailability, ModelStatusCheckCreate, ModelStatusCheckListResponse, ModelStatusCheckResponse, ModelStatusCheckUpdate,
        ModelStatusListRequest, ModelStatusRunListRequest, ModelStatusRunListResponse,
    },
};

use super::{
    ModelStatusError, ModelStatusProbe, ModelStatusProbeInput, ModelStatusProbeOutput, ModelStatusRepository, ModelStatusResult, ModelStatusRunRecord,
    ModelStatusRunStatus, ModelStatusService, ModelStatusTokenCatalog, ModelStatusUseCase,
};

#[tokio::test]
async fn create_rejects_user_token() {
    let service = ModelStatusService::new(MemoryRepository::default(), StaticTokenCatalog::new(user_token()), StaticProbe);

    let result = service.create_check(create_input()).await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: model status checks require an active independent token"
    );
}

#[tokio::test]
async fn create_accepts_independent_token() {
    let repository = MemoryRepository::default();
    let service = ModelStatusService::new(repository.clone(), StaticTokenCatalog::new(independent_token()), StaticProbe);

    service.create_check(create_input()).await.unwrap();

    assert_eq!(repository.created_count(), 1);
}

#[tokio::test]
async fn run_due_checks_records_probe_result() {
    let repository = MemoryRepository::with_due(vec![ModelStatusProbeInput {
        check_id: "check-1".into(),
        model_name: "gpt-test".into(),
        api_format: "openai:chat".into(),
        interval_seconds: 300,
        token: independent_token(),
    }]);
    let service = ModelStatusService::new(repository.clone(), StaticTokenCatalog::new(independent_token()), StaticProbe);

    let dispatched = service.run_due_checks(20, 4).await.unwrap();

    assert_eq!(dispatched, 1);
    assert_eq!(repository.records().len(), 1);
    assert_eq!(repository.records()[0].status, ModelStatusRunStatus::Operational);
}

#[tokio::test]
async fn batch_delete_rejects_blank_ids() {
    let service = ModelStatusService::new(MemoryRepository::default(), StaticTokenCatalog::new(independent_token()), StaticProbe);

    let result = service.batch_delete_checks(vec![" ".into()]).await;

    assert_eq!(result.unwrap_err().to_string(), "invalid input: ids cannot contain blank values");
}

#[tokio::test]
async fn batch_delete_reports_success_and_failed_items() {
    let repository = MemoryRepository::default();
    let service = ModelStatusService::new(repository.clone(), StaticTokenCatalog::new(independent_token()), StaticProbe);

    let result = service.batch_delete_checks(vec!["check-1".into(), "missing".into()]).await.unwrap();

    assert_eq!(result.success_count, 1);
    assert_eq!(result.failed.len(), 1);
    assert_eq!(result.failed[0].id, "missing");
    assert_eq!(repository.deleted(), vec!["check-1".to_owned()]);
}

#[derive(Clone, Default)]
struct MemoryRepository {
    due: Arc<Vec<ModelStatusProbeInput>>,
    created: Arc<Mutex<u64>>,
    records: Arc<Mutex<Vec<ModelStatusRunRecord>>>,
    deleted: Arc<Mutex<Vec<String>>>,
}

impl MemoryRepository {
    fn with_due(due: Vec<ModelStatusProbeInput>) -> Self {
        Self {
            due: Arc::new(due),
            ..Self::default()
        }
    }

    fn created_count(&self) -> u64 {
        *self.created.lock().unwrap()
    }

    fn records(&self) -> Vec<ModelStatusRunRecord> {
        self.records.lock().unwrap().clone()
    }

    fn deleted(&self) -> Vec<String> {
        self.deleted.lock().unwrap().clone()
    }
}

#[async_trait]
impl ModelStatusRepository for MemoryRepository {
    async fn list_public(&self, _request: ModelStatusListRequest) -> ModelStatusResult<ModelStatusCheckListResponse> {
        unimplemented!("not needed for service tests")
    }

    async fn list_admin(&self, _request: ModelStatusListRequest) -> ModelStatusResult<ModelStatusCheckListResponse> {
        unimplemented!("not needed for service tests")
    }

    async fn create_check(&self, _input: ModelStatusCheckCreate) -> ModelStatusResult<ModelStatusCheckResponse> {
        *self.created.lock().unwrap() += 1;
        Ok(check_response())
    }

    async fn update_check(&self, _id: &str, _input: ModelStatusCheckUpdate) -> ModelStatusResult<ModelStatusCheckResponse> {
        unimplemented!("not needed for service tests")
    }

    async fn delete_check(&self, id: &str) -> ModelStatusResult<()> {
        if id == "missing" {
            return Err(ModelStatusError::NotFound);
        }
        self.deleted.lock().unwrap().push(id.to_owned());
        Ok(())
    }

    async fn list_runs(&self, _request: ModelStatusRunListRequest) -> ModelStatusResult<ModelStatusRunListResponse> {
        unimplemented!("not needed for service tests")
    }

    async fn due_checks(&self, _limit: u64, _now: time::OffsetDateTime) -> ModelStatusResult<Vec<ModelStatusProbeInput>> {
        Ok((*self.due).clone())
    }

    async fn record_run(&self, record: ModelStatusRunRecord, _interval_seconds: i64) -> ModelStatusResult<()> {
        self.records.lock().unwrap().push(record);
        Ok(())
    }

    async fn token_has_checks(&self, _token_id: &str) -> ModelStatusResult<bool> {
        Ok(false)
    }
}

#[derive(Clone)]
struct StaticTokenCatalog {
    token: ApiToken,
}

impl StaticTokenCatalog {
    const fn new(token: ApiToken) -> Self {
        Self { token }
    }
}

#[async_trait]
impl ModelStatusTokenCatalog for StaticTokenCatalog {
    async fn independent_token(&self, _id: &str) -> ModelStatusResult<Option<ApiToken>> {
        Ok((self.token.token_type == ApiTokenType::Independent).then(|| self.token.clone()))
    }
}

#[derive(Clone, Copy)]
struct StaticProbe;

#[async_trait]
impl ModelStatusProbe for StaticProbe {
    async fn probe(&self, _input: ModelStatusProbeInput) -> ModelStatusProbeOutput {
        ModelStatusProbeOutput {
            status: ModelStatusRunStatus::Operational,
            latency_ms: Some(42),
            status_code: Some(200),
            message: None,
        }
    }
}

fn create_input() -> ModelStatusCheckCreate {
    ModelStatusCheckCreate {
        name: "OpenAI".into(),
        global_model_id: "model-1".into(),
        api_format: "openai:chat".into(),
        api_token_id: "token-1".into(),
        interval_seconds: 300,
        enabled: Some(true),
    }
}

fn check_response() -> ModelStatusCheckResponse {
    ModelStatusCheckResponse {
        id: "check-1".into(),
        name: "OpenAI".into(),
        global_model_id: "model-1".into(),
        model_name: "gpt-test".into(),
        api_format: "openai:chat".into(),
        api_token_id: "token-1".into(),
        api_token_name: "status token".into(),
        interval_seconds: 300,
        enabled: true,
        next_due_at: "2026-05-29T00:00:00Z".into(),
        last_status: None,
        last_checked_at: None,
        last_latency_ms: None,
        last_message: None,
        availability: ModelStatusAvailability {
            total_checks: 0,
            available_checks: 0,
            availability_pct: None,
        },
        timeline: Vec::new(),
        created_at: "2026-05-29T00:00:00Z".into(),
        updated_at: "2026-05-29T00:00:00Z".into(),
    }
}

fn independent_token() -> ApiToken {
    token(ApiTokenType::Independent)
}

fn user_token() -> ApiToken {
    token(ApiTokenType::User)
}

fn token(token_type: ApiTokenType) -> ApiToken {
    ApiToken {
        id: "token-1".into(),
        user_id: Some("user-1".into()),
        token_type,
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
