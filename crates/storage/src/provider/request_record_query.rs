use std::collections::HashMap;

use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use types::provider::{
    ActiveRequestRecordRequest, ActiveRequestRecordResponse, RequestCandidateDetail, RequestRecord, RequestRecordDetail, RequestRecordListRequest,
    RequestRecordListResponse,
};

use crate::{StorageError, StorageResult, provider::record::request_candidates};

use super::{
    record::RequestCandidateRecord,
    request_record_refs::{RecordRefs, load_refs},
};

const CANDIDATE_FETCH_LIMIT: u64 = 1000;
const DEFAULT_REQUEST_TYPE: &str = "chat";

pub async fn list_request_records(store: &super::ProviderStore, request: RequestRecordListRequest) -> StorageResult<RequestRecordListResponse> {
    let candidates = recent_candidates(store).await?;
    let refs = load_refs(store, &candidates).await?;
    let mut records = aggregate_records(candidates, &refs);
    filter_records(&mut records, &request);
    let total = records.len() as u64;
    let records = records.into_iter().skip(request.skip as usize).take(request.limit as usize).collect();
    Ok(RequestRecordListResponse { records, total })
}

pub async fn list_active_request_records(store: &super::ProviderStore, request: ActiveRequestRecordRequest) -> StorageResult<ActiveRequestRecordResponse> {
    let candidates = active_candidates(store, &request.ids).await?;
    let refs = load_refs(store, &candidates).await?;
    let records = aggregate_records(candidates, &refs)
        .into_iter()
        .filter(|record| active_status(&record.status))
        .collect();
    Ok(ActiveRequestRecordResponse { records })
}

pub async fn get_request_record(store: &super::ProviderStore, request_id: &str) -> StorageResult<RequestRecordDetail> {
    let candidates = request_candidates::Entity::find()
        .filter(request_candidates::Column::RequestId.eq(request_id))
        .order_by_asc(request_candidates::Column::CandidateIndex)
        .order_by_asc(request_candidates::Column::RetryIndex)
        .all(store.connection())
        .await?;
    if candidates.is_empty() {
        return Err(StorageError::NotFound);
    }
    let refs = load_refs(store, &candidates).await?;
    let mut records = aggregate_records(candidates.clone(), &refs);
    let record = records.pop().ok_or(StorageError::NotFound)?;
    let details = candidates.into_iter().map(|candidate| candidate_detail(candidate, &refs)).collect();
    Ok(RequestRecordDetail {
        record,
        candidates: details,
        request_body: None,
    })
}

async fn active_candidates(store: &super::ProviderStore, ids: &[String]) -> StorageResult<Vec<RequestCandidateRecord>> {
    if ids.is_empty() {
        return recent_candidates(store).await;
    }
    request_candidates::Entity::find()
        .filter(request_candidates::Column::RequestId.is_in(ids.iter().cloned()))
        .order_by_desc(request_candidates::Column::CreatedAt)
        .all(store.connection())
        .await
        .map_err(StorageError::from)
}

async fn recent_candidates(store: &super::ProviderStore) -> StorageResult<Vec<RequestCandidateRecord>> {
    request_candidates::Entity::find()
        .order_by_desc(request_candidates::Column::CreatedAt)
        .limit(CANDIDATE_FETCH_LIMIT)
        .all(store.connection())
        .await
        .map_err(StorageError::from)
}

fn active_status(status: &str) -> bool {
    matches!(status, "pending" | "streaming")
}

fn aggregate_records(candidates: Vec<RequestCandidateRecord>, refs: &RecordRefs) -> Vec<RequestRecord> {
    let mut groups = HashMap::<String, Vec<RequestCandidateRecord>>::new();
    for candidate in candidates {
        groups.entry(candidate.request_id.clone()).or_default().push(candidate);
    }
    let mut records: Vec<_> = groups.into_values().map(|items| aggregate_record(items, refs)).collect();
    records.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    records
}

fn aggregate_record(mut candidates: Vec<RequestCandidateRecord>, refs: &RecordRefs) -> RequestRecord {
    candidates.sort_by(|left, right| (left.candidate_index, left.retry_index).cmp(&(right.candidate_index, right.retry_index)));
    let primary = primary_candidate(&candidates);
    let token = primary.token_id.as_ref().and_then(|id| refs.tokens.get(id));
    let user = token.and_then(|item| item.user_id.as_ref()).and_then(|id| refs.users.get(id));
    let provider = primary.provider_id.as_ref().and_then(|id| refs.providers.get(id));
    let model = primary.global_model_id.as_ref().and_then(|id| refs.models.get(id));
    RequestRecord {
        request_id: primary.request_id.clone(),
        created_at: primary.created_at.to_string(),
        user_id: token.and_then(|item| item.user_id.clone()),
        username: user.map(|item| item.username.clone()),
        token_id: primary.token_id.clone(),
        token_name: token.map(|item| item.name.clone()),
        token_prefix: token.map(|item| item.token_prefix.clone()),
        group_code: primary.group_code.clone(),
        global_model_id: primary.global_model_id.clone(),
        model_name: model.map(|item| item.name.clone()).or_else(|| primary.global_model_id.clone()),
        provider_id: primary.provider_id.clone(),
        provider_name: provider.map(|item| item.name.clone()),
        client_api_format: primary.client_api_format.clone(),
        provider_api_format: primary.provider_api_format.clone(),
        request_type: DEFAULT_REQUEST_TYPE.into(),
        is_stream: candidates.iter().any(|item| item.is_stream),
        status: request_status(&candidates),
        billing_status: billing_status(&candidates),
        prompt_tokens: None,
        completion_tokens: None,
        total_tokens: None,
        total_cost: Decimal::ZERO,
        first_byte_time_ms: candidates.iter().filter_map(|item| item.first_byte_time_ms).min(),
        total_latency_ms: total_latency(&candidates),
        candidate_count: candidates.len() as u64,
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

fn total_latency(candidates: &[RequestCandidateRecord]) -> Option<i64> {
    let values: Vec<_> = candidates.iter().filter_map(|item| item.latency_ms).collect();
    (!values.is_empty()).then(|| values.into_iter().sum())
}

fn candidate_detail(candidate: RequestCandidateRecord, refs: &RecordRefs) -> RequestCandidateDetail {
    let provider = candidate.provider_id.as_ref().and_then(|id| refs.providers.get(id));
    let endpoint = candidate.endpoint_id.as_ref().and_then(|id| refs.endpoints.get(id));
    let key = candidate.key_id.as_ref().and_then(|id| refs.keys.get(id));
    RequestCandidateDetail {
        id: candidate.id,
        request_id: candidate.request_id,
        provider_id: candidate.provider_id,
        provider_name: provider.map(|item| item.name.clone()),
        endpoint_id: candidate.endpoint_id,
        endpoint_name: endpoint.map(|item| item.api_format.clone()),
        key_id: candidate.key_id,
        key_name: key.map(|item| item.name.clone()),
        key_preview: key.map(|item| masked_key(&item.encrypted_api_key)),
        client_api_format: candidate.client_api_format,
        provider_api_format: candidate.provider_api_format,
        needs_conversion: candidate.needs_conversion,
        candidate_index: candidate.candidate_index,
        retry_index: candidate.retry_index,
        status: candidate.status,
        status_code: candidate.status_code,
        latency_ms: candidate.latency_ms,
        first_byte_time_ms: candidate.first_byte_time_ms,
        error_type: candidate.error_type,
        error_message: candidate.error_message,
        created_at: candidate.created_at.to_string(),
        started_at: candidate.started_at.map(|value| value.to_string()),
        finished_at: candidate.finished_at.map(|value| value.to_string()),
    }
}

fn filter_records(records: &mut Vec<RequestRecord>, request: &RequestRecordListRequest) {
    if let Some(status) = request.status.as_ref().filter(|value| !value.is_empty()) {
        records.retain(|record| record.status == *status);
    }
    if let Some(search) = request
        .search
        .as_ref()
        .map(|value| value.to_ascii_lowercase())
        .filter(|value| !value.is_empty())
    {
        records.retain(|record| record_matches(record, &search));
    }
}

fn record_matches(record: &RequestRecord, search: &str) -> bool {
    [
        Some(record.request_id.as_str()),
        record.username.as_deref(),
        record.model_name.as_deref(),
        record.provider_name.as_deref(),
        record.token_name.as_deref(),
        record.token_prefix.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|value| value.to_ascii_lowercase().contains(search))
}

fn masked_key(value: &str) -> String {
    let suffix: String = value.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
    format!("***{suffix}")
}
