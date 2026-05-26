mod billing_runtime;
mod event;
mod record_billing;
mod records;
#[cfg(test)]
mod tests;
mod upstream_cost;

use storage::{StorageError, api_token::ApiTokenUsageRecord, model::GlobalModelUsageRecord, provider::ProviderStore};
use time::OffsetDateTime;

pub(crate) use self::billing_runtime::{BillingAttempt, request_billing_status};
use self::billing_runtime::{attempt_billing, model_usage_record, token_usage_record, wallet_settlement_input};
use self::event::AuditEvent;
pub use self::event::{AttemptRecordInput, TokenUsage};
use super::{
    LlmProxyError, LlmProxyState,
    billing::{WalletSettlementInput, settle_wallet_usage},
    candidate::CandidateSelection,
    proxy::capture::RequestCapture,
    request_record_policy::RequestRecordPolicies,
};

pub const SKIP_REASON_REQUEST_TERMINATED: &str = "request_terminated_before_attempt";

pub async fn record_scheduled_candidates(state: &LlmProxyState, selection: &CandidateSelection, capture: &RequestCapture) -> Result<(), LlmProxyError> {
    persist_event(state, AuditEvent::ScheduledCandidates { selection, capture }).await
}

pub async fn record_attempt(state: &LlmProxyState, request_id: &str, input: AttemptRecordInput<'_>) -> Result<(), LlmProxyError> {
    persist_event(
        state,
        AuditEvent::Attempt {
            request_id,
            input: Box::new(input),
        },
    )
    .await
}

pub async fn record_skipped_candidates(state: &LlmProxyState, request_id: &str, skip_reason: &str) -> Result<(), LlmProxyError> {
    persist_event(state, AuditEvent::SkippedCandidates { request_id, skip_reason }).await
}

async fn persist_event(state: &LlmProxyState, event: AuditEvent<'_>) -> Result<(), LlmProxyError> {
    match event {
        AuditEvent::ScheduledCandidates { selection, capture } => persist_scheduled_candidates(state, selection, capture).await,
        AuditEvent::Attempt { request_id, input } => persist_attempt(state, request_id, *input).await,
        AuditEvent::SkippedCandidates { request_id, skip_reason } => persist_skipped_candidates(state, request_id, skip_reason).await,
    }
}

async fn persist_scheduled_candidates(state: &LlmProxyState, selection: &CandidateSelection, capture: &RequestCapture) -> Result<(), LlmProxyError> {
    let policies = request_record_policies(state).await?;
    let store = ProviderStore::new(state.database.clone());
    store
        .create_request_record(records::request_record_input(selection, capture, &policies)?)
        .await?;
    for candidate in &selection.candidates {
        store
            .create_request_candidate(records::scheduled_input(&selection.request_id, &candidate.trace, 0))
            .await?;
    }
    Ok(())
}

async fn persist_attempt(state: &LlmProxyState, request_id: &str, input: AttemptRecordInput<'_>) -> Result<(), LlmProxyError> {
    let store = ProviderStore::new(state.database.clone());
    let billing = attempt_billing(&store, request_id, &input).await?;
    let upstream_cost = upstream_cost::request_upstream_cost(&store, &input).await?;
    let policies = request_record_policies(state).await?;
    match store
        .update_request_candidate(records::attempt_patch(request_id, &input, billing.as_ref(), &upstream_cost, &policies)?)
        .await
    {
        Ok(_) => {}
        Err(StorageError::NotFound) => create_missing_attempt(&store, request_id, &input, billing.as_ref(), &upstream_cost, &policies).await?,
        Err(error) => return Err(error.into()),
    }
    store
        .update_request_record(records::request_record_patch(request_id, &input, billing.as_ref(), &upstream_cost, &policies)?)
        .await?;
    let model_usage_record = model_usage_record(&input, billing.as_ref());
    let usage_record = token_usage_record(request_id, &input, billing.as_ref(), OffsetDateTime::now_utc())?;
    let settlement = wallet_settlement_input(request_id, &input, billing.as_ref())?;
    record_success_usage(state, usage_record, settlement, model_usage_record).await
}

async fn persist_skipped_candidates(state: &LlmProxyState, request_id: &str, skip_reason: &str) -> Result<(), LlmProxyError> {
    ProviderStore::new(state.database.clone())
        .mark_scheduled_request_candidates_skipped(request_id, skip_reason)
        .await?;
    Ok(())
}

async fn create_missing_attempt(
    store: &ProviderStore,
    request_id: &str,
    input: &AttemptRecordInput<'_>,
    billing: Option<&BillingAttempt>,
    upstream_cost: &types::provider::RequestUpstreamCost,
    policies: &RequestRecordPolicies,
) -> Result<(), LlmProxyError> {
    store
        .create_request_candidate(records::attempt_input(request_id, input, billing, upstream_cost, policies)?)
        .await?;
    Ok(())
}

async fn record_success_usage(
    state: &LlmProxyState,
    usage_record: Option<ApiTokenUsageRecord>,
    settlement: Option<WalletSettlementInput<'_>>,
    model_usage_record: Option<GlobalModelUsageRecord>,
) -> Result<(), LlmProxyError> {
    if let Some(record) = usage_record {
        let token_id = record.token_id.clone();
        let cost = record.cost;
        let used_at = record.used_at;
        state.cache.record_token_usage(&token_id, cost, used_at).await?;
        state.cache.enqueue_token_usage_persist(&record).await?;
    }
    if let Some(settlement) = settlement {
        settle_wallet_usage(state, settlement).await?;
    }
    if let Some(record) = model_usage_record {
        state.cache.enqueue_model_usage_persist(&record).await?;
    }
    Ok(())
}

async fn request_record_policies(state: &LlmProxyState) -> Result<RequestRecordPolicies, LlmProxyError> {
    let snapshot = state.scheduling_snapshot().await?;
    RequestRecordPolicies::from_snapshot(&snapshot).map_err(LlmProxyError::Infrastructure)
}
