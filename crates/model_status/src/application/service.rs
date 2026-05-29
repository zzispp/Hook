use async_trait::async_trait;
use futures_util::{StreamExt, stream};
use types::model_status::*;

use super::{ModelStatusError, ModelStatusProbe, ModelStatusRepository, ModelStatusResult, ModelStatusRunRecord, ModelStatusTokenCatalog, ModelStatusUseCase};
use crate::application::validation::{
    validate_batch_create, validate_batch_delete, validate_batch_update, validate_create, validate_dispatch_options, validate_run_list, validate_update,
};

pub struct ModelStatusService<R, T, P> {
    repository: R,
    tokens: T,
    probe: P,
}

impl<R, T, P> ModelStatusService<R, T, P>
where
    R: ModelStatusRepository,
    T: ModelStatusTokenCatalog,
    P: ModelStatusProbe,
{
    pub const fn new(repository: R, tokens: T, probe: P) -> Self {
        Self { repository, tokens, probe }
    }
}

#[async_trait]
impl<R, T, P> ModelStatusUseCase for ModelStatusService<R, T, P>
where
    R: ModelStatusRepository,
    T: ModelStatusTokenCatalog,
    P: ModelStatusProbe,
{
    async fn list_public(&self, request: ModelStatusListRequest) -> ModelStatusResult<ModelStatusCheckListResponse> {
        self.repository.list_public(request).await
    }

    async fn list_admin(&self, request: ModelStatusListRequest) -> ModelStatusResult<ModelStatusCheckListResponse> {
        self.repository.list_admin(request).await
    }

    async fn create_check(&self, input: ModelStatusCheckCreate) -> ModelStatusResult<ModelStatusCheckResponse> {
        validate_create(&input)?;
        self.ensure_independent_token(&input.api_token_id).await?;
        self.repository.create_check(input).await
    }

    async fn batch_create_checks(&self, input: ModelStatusCheckBatchCreateRequest) -> ModelStatusResult<ModelStatusCheckBatchCreateResponse> {
        validate_batch_create(&input)?;
        self.ensure_independent_token(&input.api_token_id).await?;
        self.repository.batch_create_checks(input).await
    }

    async fn update_check(&self, id: &str, input: ModelStatusCheckUpdate) -> ModelStatusResult<ModelStatusCheckResponse> {
        validate_update(&input)?;
        if let Some(token_id) = input.api_token_id.as_deref() {
            self.ensure_independent_token(token_id).await?;
        }
        self.repository.update_check(id, input).await
    }

    async fn delete_check(&self, id: &str) -> ModelStatusResult<()> {
        self.repository.delete_check(id).await
    }

    async fn batch_delete_checks(&self, ids: Vec<String>) -> ModelStatusResult<ModelStatusCheckBatchDeleteResponse> {
        validate_batch_delete(&ids)?;
        let mut success_count = 0;
        let mut failed = Vec::new();
        for id in ids {
            match self.repository.delete_check(&id).await {
                Ok(()) => success_count += 1,
                Err(error) => failed.push(ModelStatusCheckBatchDeleteFailure { id, error: error.to_string() }),
            }
        }
        Ok(ModelStatusCheckBatchDeleteResponse { success_count, failed })
    }

    async fn batch_update_checks(&self, input: ModelStatusCheckBatchUpdateRequest) -> ModelStatusResult<ModelStatusCheckBatchUpdateResponse> {
        validate_batch_update(&input)?;
        if let Some(token_id) = input.api_token_id.as_deref() {
            self.ensure_independent_token(token_id).await?;
        }
        self.repository.batch_update_checks(input).await
    }

    async fn list_runs(&self, request: ModelStatusRunListRequest) -> ModelStatusResult<ModelStatusRunListResponse> {
        validate_run_list(&request)?;
        self.repository.list_runs(request).await
    }

    async fn run_due_checks(&self, limit: u64, concurrency: usize) -> ModelStatusResult<u64> {
        validate_dispatch_options(limit, concurrency)?;
        let checks = self.repository.due_checks(limit, time::OffsetDateTime::now_utc()).await?;
        let total = checks.len() as u64;
        stream::iter(checks)
            .map(|input| async move {
                let interval = input.interval_seconds;
                let output = self.probe.probe(input.clone()).await;
                self.repository
                    .record_run(
                        ModelStatusRunRecord {
                            check_id: input.check_id,
                            status: output.status,
                            latency_ms: output.latency_ms,
                            status_code: output.status_code,
                            message: output.message,
                            checked_at: time::OffsetDateTime::now_utc(),
                        },
                        interval,
                    )
                    .await
            })
            .buffer_unordered(concurrency)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(total)
    }

    async fn token_has_checks(&self, token_id: &str) -> ModelStatusResult<bool> {
        self.repository.token_has_checks(token_id).await
    }
}

impl<R, T, P> ModelStatusService<R, T, P>
where
    R: ModelStatusRepository,
    T: ModelStatusTokenCatalog,
    P: ModelStatusProbe,
{
    async fn ensure_independent_token(&self, id: &str) -> ModelStatusResult<()> {
        self.tokens
            .independent_token(id)
            .await?
            .ok_or_else(|| ModelStatusError::InvalidInput("model status checks require an active independent token".into()))
            .map(|_| ())
    }
}
