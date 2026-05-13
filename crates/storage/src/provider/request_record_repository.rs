use types::provider::{ActiveRequestRecordRequest, ActiveRequestRecordResponse, RequestRecordListRequest};

use crate::StorageResult;

use super::ProviderStore;

impl ProviderStore {
    pub async fn create_request_candidate(&self, input: super::RequestCandidateRecordInput) -> StorageResult<types::provider::RequestCandidate> {
        super::request_candidate_query::create_request_candidate(self, input).await
    }

    pub async fn update_request_candidate(&self, input: super::RequestCandidateRecordPatch) -> StorageResult<types::provider::RequestCandidate> {
        super::request_candidate_query::update_request_candidate(self, input).await
    }

    pub async fn mark_available_request_candidates_unused(&self, request_id: &str) -> StorageResult<u64> {
        super::request_candidate_query::mark_available_request_candidates_unused(self, request_id).await
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

    pub async fn list_active_request_records(&self, request: ActiveRequestRecordRequest) -> StorageResult<ActiveRequestRecordResponse> {
        super::request_record_query::list_active_request_records(self, request).await
    }

    pub async fn get_request_record(&self, request_id: &str) -> StorageResult<types::provider::RequestRecordDetail> {
        super::request_record_query::get_request_record(self, request_id).await
    }

    pub async fn delete_request_records_before(&self, cutoff: time::OffsetDateTime) -> StorageResult<u64> {
        super::request_record_cleanup::delete_request_records_before(self, cutoff).await
    }

    pub async fn clear_request_record_payloads_before(&self, cutoff: time::OffsetDateTime) -> StorageResult<u64> {
        super::request_record_cleanup::clear_request_record_payloads_before(self, cutoff).await
    }
}
