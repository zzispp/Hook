use sea_orm::{DatabaseBackend, MockDatabase};
use storage::{
    Database,
    provider::{ProviderStore, RequestCandidateRecordInput},
};
use types::provider::RequestCandidateListRequest;

#[tokio::test]
async fn request_candidate_storage_creates_success_record() {
    let record = request_candidate_record("record-1", "success");
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[record.clone()]])
            .into_connection(),
    );
    let store = ProviderStore::new(database);

    let created = store.create_request_candidate(success_input()).await.unwrap();

    assert_eq!(created.request_id, "req-1");
    assert_eq!(created.provider_id.as_deref(), Some("provider-a"));
    assert_eq!(created.status, "success");
    assert_eq!(created.status_code, Some(200));
    assert_eq!(created.error_type, None);
    assert!(created.started_at.is_some());
    assert!(created.finished_at.is_some());
}

#[tokio::test]
async fn request_candidate_storage_lists_failed_and_no_candidate_records() {
    let failed = request_candidate_record("record-1", "failed");
    let no_candidate = request_candidate_record("record-2", "failed");
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[failed.clone(), no_candidate_record(no_candidate)]])
            .into_connection(),
    );
    let store = ProviderStore::new(database);

    let records = store
        .list_request_candidates(RequestCandidateListRequest {
            request_id: Some("req-1".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].error_type.as_deref(), Some("upstream_error"));
    assert_eq!(records[1].provider_id, None);
    assert_eq!(records[1].error_type.as_deref(), Some("no_candidate"));
}

fn success_input() -> RequestCandidateRecordInput {
    RequestCandidateRecordInput {
        request_id: "req-1".into(),
        token_id: Some("token-1".into()),
        group_code: Some("default".into()),
        global_model_id: Some("gpt-4o-mini".into()),
        provider_id: Some("provider-a".into()),
        endpoint_id: Some("endpoint-a".into()),
        key_id: Some("key-a".into()),
        client_api_format: "openai_chat".into(),
        provider_api_format: Some("openai_chat".into()),
        needs_conversion: false,
        is_stream: false,
        candidate_index: 0,
        retry_index: 0,
        status: "success".into(),
        status_code: Some(200),
        latency_ms: Some(42),
        first_byte_time_ms: Some(12),
        error_type: None,
        error_message: None,
        started: true,
        finished: true,
    }
}

fn request_candidate_record(id: &str, status: &str) -> storage::provider::record::request_candidates::Model {
    storage::provider::record::request_candidates::Model {
        id: id.into(),
        request_id: "req-1".into(),
        token_id: Some("token-1".into()),
        group_code: Some("default".into()),
        global_model_id: Some("gpt-4o-mini".into()),
        provider_id: Some("provider-a".into()),
        endpoint_id: Some("endpoint-a".into()),
        key_id: Some("key-a".into()),
        client_api_format: "openai_chat".into(),
        provider_api_format: Some("openai_chat".into()),
        needs_conversion: false,
        is_stream: false,
        candidate_index: 0,
        retry_index: 0,
        status: status.into(),
        status_code: Some(200),
        latency_ms: Some(42),
        first_byte_time_ms: Some(12),
        error_type: failed_error_type(status),
        error_message: failed_error_message(status),
        created_at: now(),
        started_at: Some(now()),
        finished_at: Some(now()),
    }
}

fn no_candidate_record(mut record: storage::provider::record::request_candidates::Model) -> storage::provider::record::request_candidates::Model {
    record.provider_id = None;
    record.endpoint_id = None;
    record.key_id = None;
    record.error_type = Some("no_candidate".into());
    record.error_message = Some("该分组下暂无 missing-model 模型可用".into());
    record
}

fn failed_error_type(status: &str) -> Option<String> {
    (status == "failed").then(|| "upstream_error".into())
}

fn failed_error_message(status: &str) -> Option<String> {
    (status == "failed").then(|| "rate limit".into())
}

fn now() -> time::OffsetDateTime {
    time::OffsetDateTime::now_utc()
}
