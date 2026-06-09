use types::provider::{ActiveRequestRecordRequest, ActiveRequestRecordResponse, RequestRecordListRequest, UsageRecordListResponse};

use crate::StorageResult;

use super::{
    ProviderStore, RequestPartitionMaintenanceOptions, RequestPartitionMaintenanceResult, RequestPayloadBackfillOptions, RequestPayloadBackfillResult,
    RequestPayloadKey, RequestPayloadOwner, RequestPayloadPendingInput, RequestPayloadStaleSweepResult, RequestPayloadStoreInput, RequestRecordCleanupOptions,
    RequestRecordCleanupResult, StaleRequestRecordSweepResult, StoredRequestPayload,
};

impl ProviderStore {
    pub async fn create_request_record(&self, input: super::RequestRecordRecordInput) -> StorageResult<()> {
        super::request_record_write::create_request_record(self, input).await
    }

    pub async fn update_request_record(&self, input: super::RequestRecordRecordPatch) -> StorageResult<()> {
        super::request_record_write::update_request_record(self, input).await
    }

    pub async fn create_request_candidate(&self, input: super::RequestCandidateRecordInput) -> StorageResult<types::provider::RequestCandidate> {
        super::request_candidate_query::create_request_candidate(self, input).await
    }

    pub async fn update_request_candidate(&self, input: super::RequestCandidateRecordPatch) -> StorageResult<types::provider::RequestCandidate> {
        super::request_candidate_query::update_request_candidate(self, input).await
    }

    pub async fn mark_scheduled_request_candidates_skipped(&self, request_id: &str, skip_reason: &str) -> StorageResult<u64> {
        super::request_candidate_query::mark_scheduled_request_candidates_skipped(self, request_id, skip_reason).await
    }

    pub async fn list_request_candidates(
        &self,
        request: types::provider::RequestCandidateListRequest,
    ) -> StorageResult<Vec<types::provider::RequestCandidate>> {
        super::request_candidate_list::list_request_candidates(self, request).await
    }

    pub async fn list_request_records(&self, request: RequestRecordListRequest) -> StorageResult<types::provider::RequestRecordListResponse> {
        super::request_record_query::list_request_records(self, request).await
    }

    pub async fn list_usage_records(&self, user_id: &str, request: RequestRecordListRequest) -> StorageResult<UsageRecordListResponse> {
        super::request_record_query::list_usage_records(self, user_id, request).await
    }

    pub async fn list_active_request_records(&self, request: ActiveRequestRecordRequest) -> StorageResult<ActiveRequestRecordResponse> {
        super::request_record_query::list_active_request_records(self, request).await
    }

    pub async fn get_request_record(&self, request_id: &str) -> StorageResult<types::provider::RequestRecordDetail> {
        super::request_record_query::get_request_record(self, request_id).await
    }

    pub async fn cleanup_request_records(&self, options: RequestRecordCleanupOptions) -> StorageResult<RequestRecordCleanupResult> {
        super::request_record_housekeeping::cleanup_request_records(self, options).await
    }

    pub async fn maintain_request_record_partitions(&self, options: RequestPartitionMaintenanceOptions) -> StorageResult<RequestPartitionMaintenanceResult> {
        super::request_record_partitions::maintain_request_partitions(self, options).await
    }

    pub async fn create_pending_request_payload(&self, input: RequestPayloadPendingInput) -> StorageResult<RequestPayloadKey> {
        super::request_record_payload_store::create_pending_payload(self, input).await
    }

    pub async fn store_request_payload(&self, input: RequestPayloadStoreInput) -> StorageResult<()> {
        super::request_record_payload_store::store_payload(self, input).await
    }

    pub async fn mark_request_payload_failed(&self, key: RequestPayloadKey, error: String) -> StorageResult<()> {
        super::request_record_payload_store::mark_payload_failed(self, key, error).await
    }

    pub async fn request_payloads_for_owner(&self, owner: &RequestPayloadOwner) -> StorageResult<Vec<StoredRequestPayload>> {
        super::request_record_payload_store::payloads_for_owner(self, owner).await
    }

    pub async fn mark_stale_request_payloads_failed(&self, cutoff: time::OffsetDateTime) -> StorageResult<RequestPayloadStaleSweepResult> {
        super::request_record_payload_store::mark_stale_payloads_failed(self, cutoff).await
    }

    pub async fn backfill_legacy_request_payloads(&self, options: RequestPayloadBackfillOptions) -> StorageResult<RequestPayloadBackfillResult> {
        super::request_record_payload_backfill::backfill_legacy_payloads(self, options).await
    }

    pub async fn mark_stale_request_records_failed(
        &self,
        pending_cutoff: time::OffsetDateTime,
        streaming_cutoff: time::OffsetDateTime,
    ) -> StorageResult<StaleRequestRecordSweepResult> {
        super::request_record_cleanup::mark_stale_request_records_failed(self, pending_cutoff, streaming_cutoff).await
    }
}
