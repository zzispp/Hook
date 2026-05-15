mod event;
mod records;

use provider::application::billing::{RequestBillingAmount, RequestBillingInput, calculate_request_billing};
use storage::{StorageError, api_token::ApiTokenUsageRecord, model::GlobalModelUsageRecord, provider::ProviderStore};
use time::OffsetDateTime;

use self::event::AuditEvent;
pub use self::event::{AttemptRecordInput, TokenUsage};
use super::{
    LlmProxyError, LlmProxyState,
    billing::{WalletSettlementInput, settle_wallet_usage},
    candidate::CandidateSelection,
    proxy::capture::RequestCapture,
    request_record_policy::RequestRecordPolicy,
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
    let policy = request_record_policy(state).await?;
    let store = ProviderStore::new(state.database.clone());
    store.create_request_record(records::request_record_input(selection, capture, &policy)?).await?;
    for candidate in &selection.candidates {
        store
            .create_request_candidate(records::scheduled_input(&selection.request_id, &candidate.trace, 0))
            .await?;
    }
    Ok(())
}

async fn persist_attempt(state: &LlmProxyState, request_id: &str, input: AttemptRecordInput<'_>) -> Result<(), LlmProxyError> {
    let model_usage_record = model_usage_record(&input);
    let usage_record = token_usage_record(request_id, &input, OffsetDateTime::now_utc())?;
    let settlement = wallet_settlement_input(request_id, &input)?;
    let policy = request_record_policy(state).await?;
    let store = ProviderStore::new(state.database.clone());
    match store.update_request_candidate(records::attempt_patch(request_id, &input, &policy)?).await {
        Ok(_) => {}
        Err(StorageError::NotFound) => create_missing_attempt(&store, request_id, &input, &policy).await?,
        Err(error) => return Err(error.into()),
    }
    store.update_request_record(records::request_record_patch(request_id, &input, &policy)?).await?;
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
    policy: &RequestRecordPolicy,
) -> Result<(), LlmProxyError> {
    store.create_request_candidate(records::attempt_input(request_id, input, policy)?).await?;
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

async fn request_record_policy(state: &LlmProxyState) -> Result<RequestRecordPolicy, LlmProxyError> {
    let snapshot = state.scheduling_snapshot().await?;
    RequestRecordPolicy::from_snapshot(&snapshot).map_err(LlmProxyError::Infrastructure)
}

fn attempt_billing(input: &AttemptRecordInput<'_>) -> Option<RequestBillingAmount> {
    if input.status != "success" {
        return None;
    }
    let usage = input.usage?;
    Some(calculate_request_billing(RequestBillingInput {
        prompt_tokens: usage.prompt_tokens.unwrap_or(0),
        completion_tokens: usage.completion_tokens.unwrap_or(0),
        cache_creation_input_tokens: usage.cache_creation_input_tokens.unwrap_or(0),
        cache_read_input_tokens: usage.cache_read_input_tokens.unwrap_or(0),
        price_per_request: input.candidate.price_per_request,
        tiered_pricing: input.candidate.tiered_pricing.clone(),
        billing_multiplier: input.candidate.billing_multiplier,
    }))
}

fn token_usage_record(request_id: &str, input: &AttemptRecordInput<'_>, used_at: OffsetDateTime) -> Result<Option<ApiTokenUsageRecord>, LlmProxyError> {
    if !should_record_successful_usage(input) {
        return Ok(None);
    }
    let Some(token_id) = input.candidate.trace.token_id.clone() else {
        return Ok(None);
    };
    let cost = attempt_billing(input)
        .map(|amount| amount.total_cost)
        .ok_or_else(|| LlmProxyError::Infrastructure(format!("successful token request missing billing usage: {request_id}/{token_id}")))?;
    Ok(Some(ApiTokenUsageRecord {
        cost,
        token_id,
        request_count: 1,
        used_at,
    }))
}

fn model_usage_record(input: &AttemptRecordInput<'_>) -> Option<GlobalModelUsageRecord> {
    if !should_record_successful_usage(input) {
        return None;
    }
    Some(GlobalModelUsageRecord {
        count: 1,
        model_id: input.candidate.trace.global_model_id.clone(),
    })
}

fn wallet_settlement_input<'a>(request_id: &'a str, input: &'a AttemptRecordInput<'a>) -> Result<Option<WalletSettlementInput<'a>>, LlmProxyError> {
    if !should_record_successful_usage(input) {
        return Ok(None);
    }
    let usage = input
        .usage
        .ok_or_else(|| LlmProxyError::Infrastructure(format!("successful wallet settlement missing usage: {request_id}")))?;
    let amount =
        attempt_billing(input).ok_or_else(|| LlmProxyError::Infrastructure(format!("successful wallet settlement missing billing amount: {request_id}")))?;
    Ok(Some(WalletSettlementInput {
        request_id,
        candidate: input.candidate,
        usage,
        amount,
    }))
}

fn should_record_successful_usage(input: &AttemptRecordInput<'_>) -> bool {
    input.status == "success" && input.finished
}

fn billing_status(status: &str) -> &'static str {
    match status {
        "success" => "settled",
        "failed" | "cancelled" => "void",
        _ => "pending",
    }
}

fn total_tokens(usage: Option<TokenUsage>) -> Option<i64> {
    usage.and_then(|item| item.total_tokens.or_else(|| Some(item.prompt_tokens? + item.completion_tokens?)))
}
