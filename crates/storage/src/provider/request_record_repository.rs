use types::provider::{ActiveRequestRecordRequest, ActiveRequestRecordResponse, RequestRecordListRequest, UsageRecordListResponse};

use crate::StorageResult;

use super::{ProviderStore, RequestRecordCleanupOptions, RequestRecordCleanupResult, StaleRequestRecordSweepResult};

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
        super::request_candidate_query::list_request_candidates(self, request).await
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

    pub async fn mark_stale_request_records_failed(
        &self,
        pending_cutoff: time::OffsetDateTime,
        streaming_cutoff: time::OffsetDateTime,
    ) -> StorageResult<StaleRequestRecordSweepResult> {
        super::request_record_cleanup::mark_stale_request_records_failed(self, pending_cutoff, streaming_cutoff).await
    }
}
