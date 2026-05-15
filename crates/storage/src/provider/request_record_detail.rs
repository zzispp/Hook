use sea_orm::entity::prelude::TimeDateTimeWithTimeZone;
use time::format_description::well_known::Rfc3339;
use types::provider::RequestCandidateDetail;

use crate::StorageResult;

use super::{record::RequestCandidateRecord, request_record_payload_codec};

pub(super) fn candidate_detail(candidate: RequestCandidateRecord) -> StorageResult<RequestCandidateDetail> {
    let total_tokens = total_tokens(&candidate);
    Ok(RequestCandidateDetail {
        id: candidate.id,
        request_id: candidate.request_id,
        provider_id: candidate.provider_id,
        provider_name: candidate.provider_name_snapshot,
        endpoint_id: candidate.endpoint_id,
        endpoint_name: candidate.endpoint_name_snapshot,
        key_id: candidate.key_id,
        key_name: candidate.key_name_snapshot,
        key_preview: candidate.key_preview_snapshot,
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
        service_tier: candidate.service_tier,
        input_cost: candidate.input_cost,
        output_cost: candidate.output_cost,
        cache_creation_cost: candidate.cache_creation_cost,
        cache_read_cost: candidate.cache_read_cost,
        request_cost: candidate.request_cost,
        input_price_per_million: candidate.input_price_per_million,
        output_price_per_million: candidate.output_price_per_million,
        cache_creation_price_per_million: candidate.cache_creation_price_per_million,
        cache_read_price_per_million: candidate.cache_read_price_per_million,
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
