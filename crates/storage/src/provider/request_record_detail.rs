use sea_orm::entity::prelude::TimeDateTimeWithTimeZone;
use time::format_description::well_known::Rfc3339;
use types::provider::RequestCandidateDetail;

use crate::StorageResult;

use super::{record::RequestCandidateRecord, request_record_payload_codec, request_record_refs::RecordRefs};

pub(super) fn candidate_detail(candidate: RequestCandidateRecord, refs: &RecordRefs) -> StorageResult<RequestCandidateDetail> {
    let provider = candidate.provider_id.as_ref().and_then(|id| refs.providers.get(id));
    let endpoint = candidate.endpoint_id.as_ref().and_then(|id| refs.endpoints.get(id));
    let key = candidate.key_id.as_ref().and_then(|id| refs.keys.get(id));
    let total_tokens = total_tokens(&candidate);
    Ok(RequestCandidateDetail {
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
        skip_reason: candidate.skip_reason,
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
        error_code: candidate.error_code,
        error_param: candidate.error_param,
        provider_request_headers: detail_payload(candidate.provider_request_headers)?,
        provider_request_body: detail_payload(candidate.provider_request_body)?,
        provider_response_headers: detail_payload(candidate.provider_response_headers)?,
        provider_response_body: detail_payload(candidate.provider_response_body)?,
        created_at: format_timestamp(candidate.created_at),
        started_at: candidate.started_at.map(format_timestamp),
        finished_at: candidate.finished_at.map(format_timestamp),
    })
}

pub(super) fn detail_payload(value: Option<String>) -> StorageResult<Option<serde_json::Value>> {
    request_record_payload_codec::decode_payload(value)
}

pub(super) fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("request record timestamp must format as RFC3339")
}

fn total_tokens(candidate: &RequestCandidateRecord) -> Option<i64> {
    candidate.total_tokens.or_else(|| Some(candidate.prompt_tokens? + candidate.completion_tokens?))
}

fn masked_key(value: &str) -> String {
    let suffix: String = value.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
    format!("***{suffix}")
}
