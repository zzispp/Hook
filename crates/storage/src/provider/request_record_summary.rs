use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use time::OffsetDateTime;

use crate::StorageResult;

use super::{
    record::{RequestCandidateRecord, request_candidates, request_records},
    repository::ProviderStore,
};

pub(super) const DEFAULT_REQUEST_TYPE: &str = "chat";
pub(super) const DEFAULT_COST_CURRENCY: &str = "USD";

pub async fn sync_request_record(store: &ProviderStore, request_id: &str) -> StorageResult<()> {
    let candidates = request_candidates::Entity::find()
        .filter(request_candidates::Column::RequestId.eq(request_id))
        .all(store.connection())
        .await?;
    if candidates.is_empty() {
        delete_request_record(store, request_id).await?;
        return Ok(());
    }
    upsert_request_record(store, summary_from_candidates(&candidates)).await
}

async fn delete_request_record(store: &ProviderStore, request_id: &str) -> StorageResult<()> {
    request_records::Entity::delete_by_id(request_id.to_owned()).exec(store.connection()).await?;
    Ok(())
}

async fn upsert_request_record(store: &ProviderStore, summary: RequestRecordSummary) -> StorageResult<()> {
    let existing = request_records::Entity::find_by_id(summary.request_id.clone()).one(store.connection()).await?;
    match existing {
        Some(record) => update_summary(store, record, summary).await,
        None => insert_summary(store, summary).await,
    }
}

async fn insert_summary(store: &ProviderStore, summary: RequestRecordSummary) -> StorageResult<()> {
    active_model(summary).insert(store.connection()).await?;
    Ok(())
}

async fn update_summary(store: &ProviderStore, record: request_records::Model, summary: RequestRecordSummary) -> StorageResult<()> {
    let mut active = active_model(summary);
    active.request_id = Set(record.request_id);
    active.update(store.connection()).await?;
    Ok(())
}

fn active_model(summary: RequestRecordSummary) -> request_records::ActiveModel {
    request_records::ActiveModel {
        request_id: Set(summary.request_id),
        token_id: Set(summary.token_id),
        group_code: Set(summary.group_code),
        global_model_id: Set(summary.global_model_id),
        provider_id: Set(summary.provider_id),
        endpoint_id: Set(summary.endpoint_id),
        key_id: Set(summary.key_id),
        client_api_format: Set(summary.client_api_format),
        provider_api_format: Set(summary.provider_api_format),
        request_type: Set(DEFAULT_REQUEST_TYPE.into()),
        is_stream: Set(summary.is_stream),
        has_failover: Set(summary.has_failover),
        has_retry: Set(summary.has_retry),
        status: Set(summary.status),
        billing_status: Set(summary.billing_status),
        prompt_tokens: Set(summary.prompt_tokens),
        completion_tokens: Set(summary.completion_tokens),
        total_tokens: Set(summary.total_tokens),
        cache_creation_input_tokens: Set(summary.cache_creation_input_tokens),
        cache_read_input_tokens: Set(summary.cache_read_input_tokens),
        cost_currency: Set(summary.cost_currency),
        token_cost: Set(summary.token_cost),
        base_cost: Set(summary.base_cost),
        total_cost: Set(summary.total_cost),
        billing_multiplier: Set(summary.billing_multiplier),
        first_byte_time_ms: Set(summary.first_byte_time_ms),
        total_latency_ms: Set(summary.total_latency_ms),
        candidate_count: Set(summary.candidate_count),
        created_at: Set(summary.created_at),
        started_at: Set(summary.started_at),
        finished_at: Set(summary.finished_at),
        updated_at: Set(OffsetDateTime::now_utc()),
    }
}

fn summary_from_candidates(candidates: &[RequestCandidateRecord]) -> RequestRecordSummary {
    let primary = primary_candidate(candidates);
    RequestRecordSummary {
        request_id: primary.request_id.clone(),
        token_id: primary.token_id.clone(),
        group_code: primary.group_code.clone(),
        global_model_id: primary.global_model_id.clone(),
        provider_id: primary.provider_id.clone(),
        endpoint_id: primary.endpoint_id.clone(),
        key_id: primary.key_id.clone(),
        client_api_format: primary.client_api_format.clone(),
        provider_api_format: primary.provider_api_format.clone(),
        is_stream: candidates.iter().any(|item| item.is_stream),
        has_failover: has_failover(candidates),
        has_retry: has_retry(candidates),
        status: request_status(candidates),
        billing_status: billing_status(candidates),
        prompt_tokens: primary.prompt_tokens,
        completion_tokens: primary.completion_tokens,
        total_tokens: total_tokens(primary),
        cache_creation_input_tokens: primary.cache_creation_input_tokens,
        cache_read_input_tokens: primary.cache_read_input_tokens,
        cost_currency: primary.cost_currency.clone(),
        token_cost: primary.token_cost,
        base_cost: primary.base_cost,
        total_cost: primary.total_cost,
        billing_multiplier: primary.billing_multiplier,
        first_byte_time_ms: candidates.iter().filter_map(|item| item.first_byte_time_ms).min(),
        total_latency_ms: total_latency(candidates),
        candidate_count: candidate_count(candidates),
        created_at: primary.created_at,
        started_at: candidates.iter().filter_map(|item| item.started_at).min(),
        finished_at: finished_at(candidates),
    }
}

fn primary_candidate(candidates: &[RequestCandidateRecord]) -> &RequestCandidateRecord {
    candidates
        .iter()
        .find(|item| item.status == "success")
        .or_else(|| candidates.iter().find(|item| item.status != "available"))
        .unwrap_or(&candidates[0])
}

fn request_status(candidates: &[RequestCandidateRecord]) -> String {
    if candidates.iter().any(|item| item.status == "success") {
        return "success".into();
    }
    if candidates.iter().any(|item| item.status == "streaming") {
        return "streaming".into();
    }
    if candidates.iter().any(|item| item.status == "pending") {
        return "pending".into();
    }
    if candidates.iter().all(|item| item.status == "available") {
        return "pending".into();
    }
    "failed".into()
}

fn billing_status(candidates: &[RequestCandidateRecord]) -> String {
    match request_status(candidates).as_str() {
        "success" => "settled".into(),
        "failed" => "void".into(),
        _ => "pending".into(),
    }
}

fn finished_at(candidates: &[RequestCandidateRecord]) -> Option<OffsetDateTime> {
    candidates.iter().filter_map(|item| item.finished_at).max()
}

fn has_failover(candidates: &[RequestCandidateRecord]) -> bool {
    let mut indexes = candidates.iter().filter(|item| executed_candidate(item)).map(|item| item.candidate_index);
    let Some(first) = indexes.next() else {
        return false;
    };
    indexes.any(|index| index != first)
}

fn has_retry(candidates: &[RequestCandidateRecord]) -> bool {
    candidates.iter().any(|item| executed_candidate(item) && item.retry_index > 0)
}

fn executed_candidate(candidate: &RequestCandidateRecord) -> bool {
    matches!(candidate.status.as_str(), "success" | "failed")
}

fn total_latency(candidates: &[RequestCandidateRecord]) -> Option<i64> {
    let values: Vec<_> = candidates.iter().filter_map(|item| item.latency_ms).collect();
    (!values.is_empty()).then(|| values.into_iter().sum())
}

fn total_tokens(candidate: &RequestCandidateRecord) -> Option<i64> {
    candidate.total_tokens.or_else(|| Some(candidate.prompt_tokens? + candidate.completion_tokens?))
}

fn candidate_count(candidates: &[RequestCandidateRecord]) -> i64 {
    candidates.len().try_into().unwrap_or(i64::MAX)
}

struct RequestRecordSummary {
    request_id: String,
    token_id: Option<String>,
    group_code: Option<String>,
    global_model_id: Option<String>,
    provider_id: Option<String>,
    endpoint_id: Option<String>,
    key_id: Option<String>,
    client_api_format: String,
    provider_api_format: Option<String>,
    is_stream: bool,
    has_failover: bool,
    has_retry: bool,
    status: String,
    billing_status: String,
    prompt_tokens: Option<i64>,
    completion_tokens: Option<i64>,
    total_tokens: Option<i64>,
    cache_creation_input_tokens: Option<i64>,
    cache_read_input_tokens: Option<i64>,
    cost_currency: Option<String>,
    token_cost: Option<Decimal>,
    base_cost: Option<Decimal>,
    total_cost: Option<Decimal>,
    billing_multiplier: Option<Decimal>,
    first_byte_time_ms: Option<i64>,
    total_latency_ms: Option<i64>,
    candidate_count: i64,
    created_at: OffsetDateTime,
    started_at: Option<OffsetDateTime>,
    finished_at: Option<OffsetDateTime>,
}
