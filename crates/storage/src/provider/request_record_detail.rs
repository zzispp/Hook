use std::collections::HashMap;

use rust_decimal::Decimal;
use sea_orm::entity::prelude::TimeDateTimeWithTimeZone;
use time::format_description::well_known::Rfc3339;
use types::provider::{RequestCandidateDetail, RequestRecord};

use crate::{StorageResult, json};

use super::{
    record::RequestCandidateRecord,
    request_record_refs::RecordRefs,
    request_record_summary::{DEFAULT_COST_CURRENCY, DEFAULT_REQUEST_TYPE},
};

pub(super) fn aggregate_records(candidates: Vec<RequestCandidateRecord>, refs: &RecordRefs) -> Vec<RequestRecord> {
    let mut groups = HashMap::<String, Vec<RequestCandidateRecord>>::new();
    for candidate in candidates {
        groups.entry(candidate.request_id.clone()).or_default().push(candidate);
    }
    let mut records: Vec<_> = groups.into_values().map(|items| aggregate_record(items, refs)).collect();
    records.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    records
}

pub(super) fn primary_candidate(candidates: &[RequestCandidateRecord]) -> &RequestCandidateRecord {
    candidates
        .iter()
        .find(|item| item.status == "success")
        .or_else(|| candidates.iter().find(|item| item.status != "available"))
        .unwrap_or(&candidates[0])
}

pub(super) fn candidate_detail(candidate: RequestCandidateRecord, refs: &RecordRefs) -> RequestCandidateDetail {
    let provider = candidate.provider_id.as_ref().and_then(|id| refs.providers.get(id));
    let endpoint = candidate.endpoint_id.as_ref().and_then(|id| refs.endpoints.get(id));
    let key = candidate.key_id.as_ref().and_then(|id| refs.keys.get(id));
    let total_tokens = total_tokens(&candidate);
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
        is_stream: candidate.is_stream,
        candidate_index: candidate.candidate_index,
        retry_index: candidate.retry_index,
        status: candidate.status,
        status_code: candidate.status_code,
        prompt_tokens: candidate.prompt_tokens,
        completion_tokens: candidate.completion_tokens,
        total_tokens,
        cache_creation_input_tokens: candidate.cache_creation_input_tokens,
        cache_read_input_tokens: candidate.cache_read_input_tokens,
        token_cost: candidate.token_cost,
        base_cost: candidate.base_cost,
        total_cost: candidate.total_cost,
        billing_multiplier: candidate.billing_multiplier,
        cost_currency: candidate.cost_currency,
        latency_ms: candidate.latency_ms,
        first_byte_time_ms: candidate.first_byte_time_ms,
        error_type: candidate.error_type,
        error_message: candidate.error_message,
        created_at: format_timestamp(candidate.created_at),
        started_at: candidate.started_at.map(format_timestamp),
        finished_at: candidate.finished_at.map(format_timestamp),
    }
}

pub(super) fn detail_payload(value: Option<String>) -> StorageResult<Option<serde_json::Value>> {
    json::decode_optional(value)
}

fn aggregate_record(mut candidates: Vec<RequestCandidateRecord>, refs: &RecordRefs) -> RequestRecord {
    candidates.sort_by(|left, right| (left.candidate_index, left.retry_index).cmp(&(right.candidate_index, right.retry_index)));
    let primary = primary_candidate(&candidates);
    let token = primary.token_id.as_ref().and_then(|id| refs.tokens.get(id));
    let user = token.and_then(|item| item.user_id.as_ref()).and_then(|id| refs.users.get(id));
    let provider = primary.provider_id.as_ref().and_then(|id| refs.providers.get(id));
    let key = primary.key_id.as_ref().and_then(|id| refs.keys.get(id));
    let model = primary.global_model_id.as_ref().and_then(|id| refs.models.get(id));
    RequestRecord {
        request_id: primary.request_id.clone(),
        created_at: format_timestamp(primary.created_at),
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
        provider_key_name: key.map(|item| item.name.clone()),
        provider_key_preview: key.map(|item| masked_key(&item.encrypted_api_key)),
        client_api_format: primary.client_api_format.clone(),
        provider_api_format: primary.provider_api_format.clone(),
        request_type: DEFAULT_REQUEST_TYPE.into(),
        is_stream: candidates.iter().any(|item| item.is_stream),
        has_failover: has_failover(&candidates),
        has_retry: has_retry(&candidates),
        status: request_status(&candidates),
        billing_status: billing_status(&candidates),
        prompt_tokens: primary.prompt_tokens,
        completion_tokens: primary.completion_tokens,
        total_tokens: total_tokens(primary),
        cache_creation_input_tokens: primary.cache_creation_input_tokens,
        cache_read_input_tokens: primary.cache_read_input_tokens,
        total_cost: primary.total_cost.unwrap_or(Decimal::ZERO),
        token_cost: primary.token_cost.unwrap_or(Decimal::ZERO),
        base_cost: primary.base_cost.unwrap_or(Decimal::ZERO),
        billing_multiplier: primary.billing_multiplier.unwrap_or(Decimal::ONE),
        cost_currency: primary.cost_currency.clone().unwrap_or_else(|| DEFAULT_COST_CURRENCY.into()),
        first_byte_time_ms: candidates.iter().filter_map(|item| item.first_byte_time_ms).min(),
        total_latency_ms: total_latency(&candidates),
        candidate_count: candidates.len() as u64,
    }
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

fn total_tokens(candidate: &RequestCandidateRecord) -> Option<i64> {
    candidate.total_tokens.or_else(|| Some(candidate.prompt_tokens? + candidate.completion_tokens?))
}

pub(super) fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("request record timestamp must format as RFC3339")
}

fn masked_key(value: &str) -> String {
    let suffix: String = value.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
    format!("***{suffix}")
}
