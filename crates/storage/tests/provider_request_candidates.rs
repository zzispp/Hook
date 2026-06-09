use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use storage::{
    Database, StorageError,
    provider::{
        ProviderStore, RequestBillingRecordValues, RequestCandidateRecordInput, RequestCandidateRecordPatch, RequestUpstreamCostRecordPatch,
        record::request_records,
    },
};
use types::model::PatchField;
use types::provider::{RequestCandidateListRequest, RequestUpstreamCost};

#[tokio::test]
async fn request_candidate_storage_creates_success_record() {
    let record = request_candidate_record("record-1", "success");
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([exec_result(1)])
        .append_query_results([[record.clone()]])
        .append_query_results([[record.clone()]])
        .append_query_results([Vec::<request_records::Model>::new()])
        .append_query_results([[summary_record("success")]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let created = store.create_request_candidate(success_input()).await.unwrap();

    assert_eq!(created.request_id, "req-1");
    assert_eq!(created.provider_id.as_deref(), Some("provider-a"));
    assert_eq!(created.status, "success");
    assert_eq!(created.status_code, Some(200));
    assert_eq!(created.input_text_tokens, Some(7));
    assert_eq!(created.output_text_tokens, Some(5));
    assert_eq!(created.reasoning_tokens, Some(2));
    assert_eq!(created.usage_source.as_deref(), Some("openai"));
    assert_eq!(created.usage_semantic.as_deref(), Some("openai"));
    assert_eq!(created.error_type, None);
    assert!(created.started_at.is_some());
    assert!(created.finished_at.is_some());
    assert_partition_sync(&connection.into_transaction_log()[1].statements()[0].sql);
}

#[tokio::test]
async fn request_candidate_storage_rejects_non_accounting_cost_currency() {
    let database = Database::new(MockDatabase::new(DatabaseBackend::Postgres).into_connection());
    let store = ProviderStore::new(database);
    let mut input = success_input();
    input.billing.cost_currency = Some("CNY".into());

    let error = store.create_request_candidate(input).await.unwrap_err();

    assert!(matches!(error, StorageError::Conflict(message) if message == "cost currency must be USD"));
}

#[tokio::test]
async fn request_candidate_storage_lists_failed_and_no_candidate_records() {
    let failed = request_candidate_record("record-1", "failed");
    let no_candidate = request_candidate_record("record-2", "failed");
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[failed.clone(), no_candidate_record(no_candidate)]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

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
    let logs = connection.into_transaction_log();
    let sql = &logs[0].statements()[0].sql;
    assert!(!sql.contains("r.provider_request_headers"), "{sql}");
    assert!(!sql.contains("r.provider_request_body"), "{sql}");
    assert!(!sql.contains("r.provider_response_headers"), "{sql}");
    assert!(!sql.contains("r.provider_response_body"), "{sql}");
}

#[tokio::test]
async fn request_candidate_storage_updates_existing_attempt() {
    let streaming = request_candidate_record("record-1", "streaming");
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[streaming]])
            .append_exec_results([MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results([[request_candidate_record("record-1", "success")]])
            .append_exec_results([exec_result(1)])
            .append_query_results([Vec::<request_records::Model>::new()])
            .append_query_results([[summary_record("success")]])
            .into_connection(),
    );
    let store = ProviderStore::new(database);

    let updated = store.update_request_candidate(success_patch()).await.unwrap();

    assert_eq!(updated.status, "success");
    assert_eq!(updated.status_code, Some(200));
    assert_eq!(updated.cache_creation_5m_input_tokens, Some(1));
    assert_eq!(updated.cache_creation_1h_input_tokens, Some(2));
    assert!(updated.finished_at.is_some());
}

#[tokio::test]
async fn request_candidate_storage_marks_scheduled_records_skipped() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([exec_result(2), exec_result(2)])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let rows = store
        .mark_scheduled_request_candidates_skipped("req-1", "request_terminated_before_attempt")
        .await
        .unwrap();

    assert_eq!(rows, 2);
    let logs = connection.into_transaction_log();
    assert!(!logs.is_empty());
    let sql = &logs[0].statements()[0].sql;
    assert!(sql.contains("UPDATE \"request_candidates\" SET"), "{sql}");
    assert!(sql.contains("\"status\" = $"), "{sql}");
    assert!(sql.contains("\"skip_reason\" = $"), "{sql}");
    assert!(sql.contains("\"finished_at\" = $"), "{sql}");
    assert!(sql.contains("WHERE \"request_candidates\".\"request_id\" = $"), "{sql}");
    assert!(sql.contains("AND \"request_candidates\".\"status\" = $"), "{sql}");
    assert_partition_sync(&logs[1].statements()[0].sql);
}

fn assert_partition_sync(sql: &str) {
    assert!(sql.contains("INSERT INTO request_candidates_partitioned"), "{sql}");
    assert!(sql.contains("ON CONFLICT (created_at, id) DO UPDATE"), "{sql}");
    assert!(!sql.contains("provider_request_headers"), "{sql}");
}

fn exec_result(rows_affected: u64) -> MockExecResult {
    MockExecResult {
        last_insert_id: 0,
        rows_affected,
    }
}

fn success_input() -> RequestCandidateRecordInput {
    RequestCandidateRecordInput {
        request_id: "req-1".into(),
        token_id: Some("token-1".into()),
        group_code: Some("default".into()),
        global_model_id: Some("gpt-4o-mini".into()),
        provider_id: Some("provider-a".into()),
        provider_name_snapshot: Some("Provider A".into()),
        endpoint_id: Some("endpoint-a".into()),
        endpoint_name_snapshot: Some("openai:chat".into()),
        key_id: Some("key-a".into()),
        key_name_snapshot: Some("Key A".into()),
        key_preview_snapshot: Some("***test".into()),
        client_api_format: "openai:chat".into(),
        provider_api_format: Some("openai:chat".into()),
        needs_conversion: false,
        is_stream: false,
        is_cached: false,
        provider_request_headers: Some(serde_json::json!({"authorization": "****"})),
        provider_request_body: Some(serde_json::json!({"model": "gpt-4o-mini"})),
        provider_response_headers: Some(serde_json::json!({"content-type": "application/json"})),
        provider_response_body: Some(serde_json::json!({"id": "chatcmpl-1"})),
        candidate_index: 0,
        retry_index: 0,
        status: "success".into(),
        skip_reason: None,
        status_code: Some(200),
        prompt_tokens: Some(12),
        completion_tokens: Some(8),
        total_tokens: Some(20),
        cache_creation_input_tokens: Some(3),
        cache_read_input_tokens: Some(4),
        input_text_tokens: Some(7),
        input_audio_tokens: Some(1),
        input_image_tokens: Some(2),
        output_text_tokens: Some(5),
        output_audio_tokens: Some(1),
        output_image_tokens: Some(2),
        reasoning_tokens: Some(2),
        cache_creation_5m_input_tokens: Some(1),
        cache_creation_1h_input_tokens: Some(2),
        usage_source: Some("openai".into()),
        usage_semantic: Some("openai".into()),
        upstream_cost: RequestUpstreamCost::default(),
        billing: success_billing_values(),
        billing_snapshot: None,
        latency_ms: Some(42),
        first_byte_time_ms: Some(12),
        error_type: None,
        error_message: None,
        error_code: None,
        error_param: None,
        started: true,
        finished: true,
    }
}

fn success_patch() -> RequestCandidateRecordPatch {
    RequestCandidateRecordPatch {
        request_id: "req-1".into(),
        candidate_index: 0,
        retry_index: 0,
        status: "success".into(),
        skip_reason: None,
        status_code: Some(200),
        prompt_tokens: Some(12),
        completion_tokens: Some(8),
        total_tokens: Some(20),
        cache_creation_input_tokens: Some(3),
        cache_read_input_tokens: Some(4),
        input_text_tokens: Some(7),
        input_audio_tokens: Some(1),
        input_image_tokens: Some(2),
        output_text_tokens: Some(5),
        output_audio_tokens: Some(1),
        output_image_tokens: Some(2),
        reasoning_tokens: Some(2),
        cache_creation_5m_input_tokens: Some(1),
        cache_creation_1h_input_tokens: Some(2),
        usage_source: Some("openai".into()),
        usage_semantic: Some("openai".into()),
        upstream_cost: RequestUpstreamCostRecordPatch::default(),
        billing: success_billing_values(),
        billing_snapshot: PatchField::Missing,
        latency_ms: Some(42),
        first_byte_time_ms: Some(12),
        error_type: None,
        error_message: None,
        error_code: None,
        error_param: None,
        provider_request_headers: PatchField::Missing,
        provider_request_body: PatchField::Missing,
        provider_response_headers: PatchField::Missing,
        provider_response_body: PatchField::Value(serde_json::json!({"id": "chatcmpl-1"})),
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
        provider_name_snapshot: Some("Provider A".into()),
        endpoint_id: Some("endpoint-a".into()),
        endpoint_name_snapshot: Some("openai:chat".into()),
        key_id: Some("key-a".into()),
        key_name_snapshot: Some("Key A".into()),
        key_preview_snapshot: Some("***test".into()),
        client_api_format: "openai:chat".into(),
        provider_api_format: Some("openai:chat".into()),
        needs_conversion: false,
        is_stream: false,
        is_cached: false,
        provider_request_headers: None,
        provider_request_body: None,
        provider_response_headers: None,
        provider_response_body: None,
        payload_compressed_at: None,
        candidate_index: 0,
        retry_index: 0,
        status: status.into(),
        skip_reason: skipped_reason(status),
        status_code: Some(200),
        prompt_tokens: Some(12),
        completion_tokens: Some(8),
        total_tokens: Some(20),
        cache_creation_input_tokens: Some(3),
        cache_read_input_tokens: Some(4),
        input_text_tokens: Some(7),
        input_audio_tokens: Some(1),
        input_image_tokens: Some(2),
        output_text_tokens: Some(5),
        output_audio_tokens: Some(1),
        output_image_tokens: Some(2),
        reasoning_tokens: Some(2),
        cache_creation_5m_input_tokens: Some(1),
        cache_creation_1h_input_tokens: Some(2),
        usage_source: Some("openai".into()),
        usage_semantic: Some("openai".into()),
        service_tier: Some("standard".into()),
        upstream_cost_mode: None,
        upstream_cost_source: None,
        upstream_price_per_request: None,
        upstream_input_price_per_million: None,
        upstream_output_price_per_million: None,
        upstream_cache_creation_price_per_million: None,
        upstream_cache_read_price_per_million: None,
        upstream_request_cost: None,
        upstream_input_cost: None,
        upstream_output_cost: None,
        upstream_cache_creation_cost: None,
        upstream_cache_read_cost: None,
        upstream_total_cost: None,
        input_cost: Some(Decimal::new(25, 4)),
        output_cost: Some(Decimal::new(30, 4)),
        cache_creation_cost: Some(Decimal::new(125, 5)),
        cache_read_cost: Some(Decimal::new(125, 6)),
        request_cost: Some(Decimal::new(1, 2)),
        input_price_per_million: Some(Decimal::new(250, 2)),
        output_price_per_million: Some(Decimal::new(1500, 2)),
        cache_creation_price_per_million: Some(Decimal::new(125, 2)),
        cache_read_price_per_million: Some(Decimal::new(25, 2)),
        cost_currency: Some(currency::ACCOUNTING_CURRENCY.into()),
        token_cost: Some(Decimal::new(1, 4)),
        base_cost: Some(Decimal::new(1, 5)),
        total_cost: Some(Decimal::new(2, 4)),
        billing_multiplier: Some(Decimal::new(2, 0)),
        billing_snapshot: None,
        latency_ms: Some(42),
        first_byte_time_ms: Some(12),
        error_type: failed_error_type(status),
        error_message: failed_error_message(status),
        error_code: failed_error_code(status),
        error_param: failed_error_param(status),
        created_at: now(),
        started_at: Some(now()),
        finished_at: Some(now()),
    }
}

fn no_candidate_record(mut record: storage::provider::record::request_candidates::Model) -> storage::provider::record::request_candidates::Model {
    record.provider_id = None;
    record.provider_name_snapshot = None;
    record.endpoint_id = None;
    record.endpoint_name_snapshot = None;
    record.key_id = None;
    record.key_name_snapshot = None;
    record.key_preview_snapshot = None;
    record.error_type = Some("no_candidate".into());
    record.error_message = Some("该分组下暂无 missing-model 模型可用".into());
    record
}

fn success_billing_values() -> RequestBillingRecordValues {
    RequestBillingRecordValues {
        service_tier: Some("standard".into()),
        cost_currency: Some(currency::ACCOUNTING_CURRENCY.into()),
        input_cost: Some(Decimal::new(25, 4)),
        output_cost: Some(Decimal::new(30, 4)),
        cache_creation_cost: Some(Decimal::new(125, 5)),
        cache_read_cost: Some(Decimal::new(125, 6)),
        request_cost: Some(Decimal::new(1, 2)),
        token_cost: Some(Decimal::new(1, 4)),
        base_cost: Some(Decimal::new(1, 5)),
        total_cost: Some(Decimal::new(2, 4)),
        billing_multiplier: Some(Decimal::new(2, 0)),
        input_price_per_million: Some(Decimal::new(250, 2)),
        output_price_per_million: Some(Decimal::new(1500, 2)),
        cache_creation_price_per_million: Some(Decimal::new(125, 2)),
        cache_read_price_per_million: Some(Decimal::new(25, 2)),
    }
}

fn summary_record(status: &str) -> request_records::Model {
    request_records::Model {
        request_id: "req-1".into(),
        token_id: Some("token-1".into()),
        user_id_snapshot: Some("user-1".into()),
        username_snapshot: Some("hwnet".into()),
        token_name_snapshot: Some("Token A".into()),
        token_prefix_snapshot: Some("sk-test".into()),
        group_code: Some("default".into()),
        global_model_id: Some("gpt-4o-mini".into()),
        model_name_snapshot: Some("gpt-4o-mini".into()),
        provider_id: Some("provider-a".into()),
        provider_name_snapshot: Some("Provider A".into()),
        endpoint_id: Some("endpoint-a".into()),
        key_id: Some("key-a".into()),
        provider_key_name_snapshot: Some("Key A".into()),
        provider_key_preview_snapshot: Some("***test".into()),
        client_api_format: "openai:chat".into(),
        provider_api_format: Some("openai:chat".into()),
        request_type: "chat".into(),
        is_stream: false,
        has_failover: false,
        has_retry: false,
        status: status.into(),
        billing_status: "settled".into(),
        client_status_code: Some(200),
        client_error_type: None,
        client_error_message: None,
        termination_origin: None,
        termination_reason: None,
        stream_end_reason: None,
        prompt_tokens: Some(12),
        completion_tokens: Some(8),
        total_tokens: Some(20),
        cache_creation_input_tokens: Some(3),
        cache_read_input_tokens: Some(4),
        input_text_tokens: Some(7),
        input_audio_tokens: Some(1),
        input_image_tokens: Some(2),
        output_text_tokens: Some(5),
        output_audio_tokens: Some(1),
        output_image_tokens: Some(2),
        reasoning_tokens: Some(2),
        cache_creation_5m_input_tokens: Some(1),
        cache_creation_1h_input_tokens: Some(2),
        usage_source: Some("openai".into()),
        usage_semantic: Some("openai".into()),
        service_tier: Some("standard".into()),
        upstream_cost_mode: None,
        upstream_cost_source: None,
        upstream_price_per_request: None,
        upstream_input_price_per_million: None,
        upstream_output_price_per_million: None,
        upstream_cache_creation_price_per_million: None,
        upstream_cache_read_price_per_million: None,
        upstream_request_cost: None,
        upstream_input_cost: None,
        upstream_output_cost: None,
        upstream_cache_creation_cost: None,
        upstream_cache_read_cost: None,
        upstream_total_cost: None,
        input_cost: Some(Decimal::new(25, 4)),
        output_cost: Some(Decimal::new(30, 4)),
        cache_creation_cost: Some(Decimal::new(125, 5)),
        cache_read_cost: Some(Decimal::new(125, 6)),
        request_cost: Some(Decimal::new(1, 2)),
        input_price_per_million: Some(Decimal::new(250, 2)),
        output_price_per_million: Some(Decimal::new(1500, 2)),
        cache_creation_price_per_million: Some(Decimal::new(125, 2)),
        cache_read_price_per_million: Some(Decimal::new(25, 2)),
        cost_currency: Some(currency::ACCOUNTING_CURRENCY.into()),
        token_cost: Some(Decimal::new(1, 4)),
        base_cost: Some(Decimal::new(1, 5)),
        total_cost: Some(Decimal::new(2, 4)),
        billing_multiplier: Some(Decimal::new(2, 0)),
        billing_snapshot: None,
        first_byte_time_ms: Some(12),
        total_latency_ms: Some(42),
        candidate_count: 1,
        request_headers: None,
        request_body: None,
        client_response_headers: None,
        client_response_body: None,
        payload_compressed_at: None,
        created_at: now(),
        started_at: Some(now()),
        finished_at: Some(now()),
        updated_at: now(),
    }
}

fn failed_error_type(status: &str) -> Option<String> {
    (status == "failed").then(|| "upstream_error".into())
}

fn failed_error_message(status: &str) -> Option<String> {
    (status == "failed").then(|| "rate limit".into())
}

fn failed_error_code(status: &str) -> Option<String> {
    (status == "failed").then(|| "rate_limit".into())
}

fn failed_error_param(status: &str) -> Option<String> {
    (status == "failed").then(|| "model".into())
}

fn skipped_reason(status: &str) -> Option<String> {
    (status == "skipped").then(|| "request_terminated_before_attempt".into())
}

fn now() -> time::OffsetDateTime {
    time::OffsetDateTime::now_utc()
}
