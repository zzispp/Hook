mod billing_runtime;
mod event;
mod record_billing;
mod records;
mod routing_observability;
#[cfg(test)]
mod tests;
mod upstream_cost;

use storage::{
    StorageError,
    api_token::ApiTokenUsageRecord,
    model::GlobalModelUsageRecord,
    provider::{
        KIND_CLIENT_RESPONSE_BODY, KIND_CLIENT_RESPONSE_HEADERS, KIND_PROVIDER_REQUEST_BODY, KIND_PROVIDER_REQUEST_HEADERS, KIND_PROVIDER_RESPONSE_BODY,
        KIND_PROVIDER_RESPONSE_HEADERS, KIND_REQUEST_BODY, KIND_REQUEST_HEADERS, OWNER_REQUEST_CANDIDATE, OWNER_REQUEST_RECORD, ProviderStore,
        RequestPayloadOwner,
    },
};
use time::OffsetDateTime;
use types::model::PatchField;

pub(crate) use self::billing_runtime::{BillingAttempt, model_usage_record, request_billing_status};
use self::billing_runtime::{attempt_billing, token_usage_record, wallet_settlement_input};
use self::event::AuditEvent;
pub(crate) use self::event::{AttemptAuditInput, AuditCandidate};
pub use self::event::{AttemptRecordInput, TokenUsage};
use super::{
    LlmProxyError, LlmProxyState,
    billing::{WalletSettlementInput, settle_wallet_usage},
    candidate::CandidateSelection,
    proxy::capture::RequestCapture,
    request_payload_writer,
    request_record_policy::RequestRecordPolicies,
};

pub const SKIP_REASON_REQUEST_TERMINATED: &str = "request_terminated_before_attempt";

pub async fn record_scheduled_candidates(state: &LlmProxyState, selection: &CandidateSelection, capture: &RequestCapture) -> Result<(), LlmProxyError> {
    persist_event(state, AuditEvent::ScheduledCandidates { selection, capture }).await
}

pub fn record_attempt<'a>(
    state: &'a LlmProxyState,
    request_id: &'a str,
    input: AttemptRecordInput<'_>,
) -> impl Future<Output = Result<(), LlmProxyError>> + 'a {
    persist_attempt(state, request_id, AttemptAuditInput::from(input))
}

pub async fn record_skipped_candidates(state: &LlmProxyState, request_id: &str, skip_reason: &str) -> Result<(), LlmProxyError> {
    persist_event(state, AuditEvent::SkippedCandidates { request_id, skip_reason }).await
}

async fn persist_event(state: &LlmProxyState, event: AuditEvent<'_>) -> Result<(), LlmProxyError> {
    match event {
        AuditEvent::ScheduledCandidates { selection, capture } => persist_scheduled_candidates(state, selection, capture).await,
        AuditEvent::SkippedCandidates { request_id, skip_reason } => persist_skipped_candidates(state, request_id, skip_reason).await,
    }
}

async fn persist_scheduled_candidates(state: &LlmProxyState, selection: &CandidateSelection, capture: &RequestCapture) -> Result<(), LlmProxyError> {
    let policies = request_record_policies(state).await?;
    let store = ProviderStore::new(state.database.clone());
    let record_input = records::request_record_input(selection, capture, &policies)?;
    let request_payloads = request_record_payload_jobs(&selection.request_id, &record_input);
    store.create_request_record(record_input).await?;
    enqueue_payload_jobs(state, request_payloads).await?;
    for candidate in &selection.candidates {
        store
            .create_request_candidate(records::scheduled_input(&selection.request_id, &candidate.trace, 0))
            .await?;
    }
    routing_observability::record_decision_sample(&store, selection).await?;
    Ok(())
}

async fn persist_attempt(state: &LlmProxyState, request_id: &str, input: AttemptAuditInput) -> Result<(), LlmProxyError> {
    let store = ProviderStore::new(state.database.clone());
    let billing = attempt_billing(&store, request_id, &input).await?;
    let upstream_cost = upstream_cost::request_upstream_cost(&store, &input).await?;
    let policies = request_record_policies(state).await?;
    let candidate_patch = records::attempt_patch(request_id, &input, billing.as_ref(), &upstream_cost, &policies)?;
    let candidate_payload_seeds = candidate_patch_payload_jobs(&candidate_patch);
    match store.update_request_candidate(candidate_patch).await {
        Ok(candidate) => enqueue_payload_jobs(state, candidate_payloads(&candidate.id, candidate_payload_seeds)).await?,
        Err(StorageError::NotFound) => create_missing_attempt(state, &store, request_id, &input, billing.as_ref(), &upstream_cost, &policies).await?,
        Err(error) => return Err(error.into()),
    }
    let request_patch = records::request_record_patch(request_id, &input, billing.as_ref(), &upstream_cost, &policies)?;
    let request_payloads = request_patch_payload_jobs(request_id, &request_patch);
    store.update_request_record(request_patch).await?;
    enqueue_payload_jobs(state, request_payloads).await?;
    routing_observability::record_finished_attempt(state, &store, &input, &upstream_cost).await?;
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
    state: &LlmProxyState,
    store: &ProviderStore,
    request_id: &str,
    input: &AttemptAuditInput,
    billing: Option<&BillingAttempt>,
    upstream_cost: &types::provider::RequestUpstreamCost,
    policies: &RequestRecordPolicies,
) -> Result<(), LlmProxyError> {
    let record = records::attempt_input(request_id, input, billing, upstream_cost, policies)?;
    let payloads = candidate_input_payload_jobs(&record);
    let candidate = store.create_request_candidate(record).await?;
    enqueue_payload_jobs(state, candidate_payloads(&candidate.id, payloads)).await?;
    Ok(())
}

type PayloadJobSeed = (&'static str, serde_json::Value);

fn request_record_payload_jobs(request_id: &str, record: &storage::provider::RequestRecordRecordInput) -> Vec<request_payload_writer::RequestPayloadJob> {
    let owner = payload_owner(OWNER_REQUEST_RECORD, request_id);
    let seeds = [
        optional_payload(KIND_REQUEST_HEADERS, record.request_headers.clone()),
        optional_payload(KIND_REQUEST_BODY, record.request_body.clone()),
    ];
    payload_jobs(owner, seeds.into_iter().flatten())
}

fn request_patch_payload_jobs(request_id: &str, patch: &storage::provider::RequestRecordRecordPatch) -> Vec<request_payload_writer::RequestPayloadJob> {
    let owner = payload_owner(OWNER_REQUEST_RECORD, request_id);
    let seeds = [
        patch_payload(KIND_CLIENT_RESPONSE_HEADERS, &patch.client_response_headers),
        patch_payload(KIND_CLIENT_RESPONSE_BODY, &patch.client_response_body),
    ];
    payload_jobs(owner, seeds.into_iter().flatten())
}

fn candidate_input_payload_jobs(record: &storage::provider::RequestCandidateRecordInput) -> Vec<PayloadJobSeed> {
    [
        optional_payload(KIND_PROVIDER_REQUEST_HEADERS, record.provider_request_headers.clone()),
        optional_payload(KIND_PROVIDER_REQUEST_BODY, record.provider_request_body.clone()),
        optional_payload(KIND_PROVIDER_RESPONSE_HEADERS, record.provider_response_headers.clone()),
        optional_payload(KIND_PROVIDER_RESPONSE_BODY, record.provider_response_body.clone()),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn candidate_patch_payload_jobs(patch: &storage::provider::RequestCandidateRecordPatch) -> Vec<PayloadJobSeed> {
    [
        patch_payload(KIND_PROVIDER_REQUEST_HEADERS, &patch.provider_request_headers),
        patch_payload(KIND_PROVIDER_REQUEST_BODY, &patch.provider_request_body),
        patch_payload(KIND_PROVIDER_RESPONSE_HEADERS, &patch.provider_response_headers),
        patch_payload(KIND_PROVIDER_RESPONSE_BODY, &patch.provider_response_body),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn candidate_payloads(candidate_id: &str, seeds: Vec<PayloadJobSeed>) -> Vec<request_payload_writer::RequestPayloadJob> {
    payload_jobs(payload_owner(OWNER_REQUEST_CANDIDATE, candidate_id), seeds.into_iter())
}

fn optional_payload(kind: &'static str, value: Option<serde_json::Value>) -> Option<PayloadJobSeed> {
    value.map(|payload| (kind, payload))
}

fn patch_payload(kind: &'static str, value: &PatchField<serde_json::Value>) -> Option<PayloadJobSeed> {
    match value {
        PatchField::Value(payload) => Some((kind, payload.clone())),
        PatchField::Null | PatchField::Missing => None,
    }
}

fn payload_jobs(owner: RequestPayloadOwner, seeds: impl Iterator<Item = PayloadJobSeed>) -> Vec<request_payload_writer::RequestPayloadJob> {
    seeds
        .map(|(kind, payload)| request_payload_writer::payload_job(owner.clone(), kind, payload))
        .collect()
}

fn payload_owner(owner_type: &str, owner_id: &str) -> RequestPayloadOwner {
    RequestPayloadOwner {
        owner_type: owner_type.to_owned(),
        owner_id: owner_id.to_owned(),
    }
}

async fn enqueue_payload_jobs(state: &LlmProxyState, jobs: Vec<request_payload_writer::RequestPayloadJob>) -> Result<(), LlmProxyError> {
    for job in jobs {
        state.enqueue_request_payload(job).await?;
    }
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
