use types::provider::{RequestCandidateDetail, RequestPayloadMeta, RequestPayloadSource, RequestPayloadStatus};

use crate::{
    StorageResult,
    provider::record::{RequestCandidateRecord, RequestRecordSummaryRecord},
};

use super::{
    ProviderStore, RequestPayloadOwner, StoredRequestPayload,
    request_record_detail::{candidate_detail, detail_payload, format_timestamp},
    request_record_payload_store::{
        KIND_CLIENT_RESPONSE_BODY, KIND_CLIENT_RESPONSE_HEADERS, KIND_PROVIDER_REQUEST_BODY, KIND_PROVIDER_REQUEST_HEADERS, KIND_PROVIDER_RESPONSE_BODY,
        KIND_PROVIDER_RESPONSE_HEADERS, KIND_REQUEST_BODY, KIND_REQUEST_HEADERS, OWNER_REQUEST_CANDIDATE, OWNER_REQUEST_RECORD,
    },
};

pub(super) struct RecordDetailPayloads {
    pub payloads: Vec<RequestPayloadMeta>,
    pub request_headers: Option<serde_json::Value>,
    pub request_body: Option<serde_json::Value>,
    pub client_response_headers: Option<serde_json::Value>,
    pub client_response_body: Option<serde_json::Value>,
}

pub(super) async fn record_payloads(store: &ProviderStore, summary: &RequestRecordSummaryRecord) -> StorageResult<RecordDetailPayloads> {
    let payloads = owner_payloads(store, OWNER_REQUEST_RECORD, &summary.request_id).await?;
    Ok(RecordDetailPayloads {
        request_headers: detail_payload_value(&payloads, KIND_REQUEST_HEADERS, summary.request_headers.clone())?,
        request_body: detail_payload_value(&payloads, KIND_REQUEST_BODY, summary.request_body.clone())?,
        client_response_headers: detail_payload_value(&payloads, KIND_CLIENT_RESPONSE_HEADERS, summary.client_response_headers.clone())?,
        client_response_body: detail_payload_value(&payloads, KIND_CLIENT_RESPONSE_BODY, summary.client_response_body.clone())?,
        payloads: merge_legacy_record_payloads(payload_metas(&payloads), summary),
    })
}

pub(super) async fn candidate_details_with_payloads(
    store: &ProviderStore,
    candidates: Vec<RequestCandidateRecord>,
) -> StorageResult<Vec<RequestCandidateDetail>> {
    let mut details = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        details.push(candidate_detail_with_payloads(store, candidate).await?);
    }
    Ok(details)
}

async fn candidate_detail_with_payloads(store: &ProviderStore, candidate: RequestCandidateRecord) -> StorageResult<RequestCandidateDetail> {
    let payloads = owner_payloads(store, OWNER_REQUEST_CANDIDATE, &candidate.id).await?;
    let mut detail = candidate_detail(candidate.clone())?;
    detail.provider_request_headers = detail_payload_value(&payloads, KIND_PROVIDER_REQUEST_HEADERS, candidate.provider_request_headers.clone())?;
    detail.provider_request_body = detail_payload_value(&payloads, KIND_PROVIDER_REQUEST_BODY, candidate.provider_request_body.clone())?;
    detail.provider_response_headers = detail_payload_value(&payloads, KIND_PROVIDER_RESPONSE_HEADERS, candidate.provider_response_headers.clone())?;
    detail.provider_response_body = detail_payload_value(&payloads, KIND_PROVIDER_RESPONSE_BODY, candidate.provider_response_body.clone())?;
    detail.payloads = merge_legacy_candidate_payloads(payload_metas(&payloads), &candidate);
    Ok(detail)
}

async fn owner_payloads(store: &ProviderStore, owner_type: &str, owner_id: &str) -> StorageResult<Vec<StoredRequestPayload>> {
    store
        .request_payloads_for_owner(&RequestPayloadOwner {
            owner_type: owner_type.to_owned(),
            owner_id: owner_id.to_owned(),
        })
        .await
}

fn detail_payload_value(payloads: &[StoredRequestPayload], kind: &str, legacy: Option<String>) -> StorageResult<Option<serde_json::Value>> {
    if let Some(value) = payload_value(payloads, kind) {
        return Ok(Some(value));
    }
    detail_payload(legacy)
}

fn payload_value(payloads: &[StoredRequestPayload], kind: &str) -> Option<serde_json::Value> {
    payloads
        .iter()
        .find(|payload| payload.meta.kind == kind && payload.payload.is_some())
        .and_then(|payload| payload.payload.clone())
}

fn payload_metas(payloads: &[StoredRequestPayload]) -> Vec<RequestPayloadMeta> {
    payloads.iter().map(|payload| payload.meta.clone()).collect()
}

fn merge_legacy_record_payloads(mut payloads: Vec<RequestPayloadMeta>, record: &RequestRecordSummaryRecord) -> Vec<RequestPayloadMeta> {
    push_legacy_payload(
        &mut payloads,
        OWNER_REQUEST_RECORD,
        &record.request_id,
        KIND_REQUEST_HEADERS,
        record.request_headers.as_ref(),
        record.updated_at,
    );
    push_legacy_payload(
        &mut payloads,
        OWNER_REQUEST_RECORD,
        &record.request_id,
        KIND_REQUEST_BODY,
        record.request_body.as_ref(),
        record.updated_at,
    );
    push_legacy_payload(
        &mut payloads,
        OWNER_REQUEST_RECORD,
        &record.request_id,
        KIND_CLIENT_RESPONSE_HEADERS,
        record.client_response_headers.as_ref(),
        record.updated_at,
    );
    push_legacy_payload(
        &mut payloads,
        OWNER_REQUEST_RECORD,
        &record.request_id,
        KIND_CLIENT_RESPONSE_BODY,
        record.client_response_body.as_ref(),
        record.updated_at,
    );
    payloads
}

fn merge_legacy_candidate_payloads(mut payloads: Vec<RequestPayloadMeta>, candidate: &RequestCandidateRecord) -> Vec<RequestPayloadMeta> {
    push_legacy_payload(
        &mut payloads,
        OWNER_REQUEST_CANDIDATE,
        &candidate.id,
        KIND_PROVIDER_REQUEST_HEADERS,
        candidate.provider_request_headers.as_ref(),
        candidate.created_at,
    );
    push_legacy_payload(
        &mut payloads,
        OWNER_REQUEST_CANDIDATE,
        &candidate.id,
        KIND_PROVIDER_REQUEST_BODY,
        candidate.provider_request_body.as_ref(),
        candidate.created_at,
    );
    push_legacy_payload(
        &mut payloads,
        OWNER_REQUEST_CANDIDATE,
        &candidate.id,
        KIND_PROVIDER_RESPONSE_HEADERS,
        candidate.provider_response_headers.as_ref(),
        candidate.created_at,
    );
    push_legacy_payload(
        &mut payloads,
        OWNER_REQUEST_CANDIDATE,
        &candidate.id,
        KIND_PROVIDER_RESPONSE_BODY,
        candidate.provider_response_body.as_ref(),
        candidate.created_at,
    );
    payloads
}

fn push_legacy_payload(
    payloads: &mut Vec<RequestPayloadMeta>,
    owner_type: &str,
    owner_id: &str,
    kind: &str,
    value: Option<&String>,
    updated_at: time::OffsetDateTime,
) {
    if value.is_none() || payloads.iter().any(|payload| payload.kind == kind) {
        return;
    }
    payloads.push(RequestPayloadMeta {
        owner_type: owner_type.to_owned(),
        owner_id: owner_id.to_owned(),
        kind: kind.to_owned(),
        status: RequestPayloadStatus::Stored,
        source: RequestPayloadSource::Legacy,
        original_size: value.map(|item| item.len().try_into().unwrap_or(i64::MAX)),
        compressed_size: None,
        sha256: None,
        error_message: None,
        updated_at: format_timestamp(updated_at),
    });
}
