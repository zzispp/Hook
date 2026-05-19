use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use storage::{
    Database,
    provider::{
        ProviderStore,
        record::{request_candidates, request_records},
    },
};

#[tokio::test]
async fn request_record_storage_compresses_old_payloads() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[summary_record(
            "req-1",
            Some(r#"{"authorization":"****"}"#.into()),
            Some(r#"{"model":"gpt-5.5"}"#.into()),
            Some(r#"{"content-type":"application/json"}"#.into()),
            Some(r#"{"id":"resp-1"}"#.into()),
        )]])
        .append_query_results([[summary_record(
            "req-1",
            Some(r#"{"__hook_compressed_payload__":true,"encoding":"zlib+hex","original_size":24,"data":"789c"}"#.into()),
            Some(r#"{"__hook_compressed_payload__":true,"encoding":"zlib+hex","original_size":19,"data":"789c"}"#.into()),
            Some(r#"{"__hook_compressed_payload__":true,"encoding":"zlib+hex","original_size":33,"data":"789c"}"#.into()),
            Some(r#"{"__hook_compressed_payload__":true,"encoding":"zlib+hex","original_size":15,"data":"789c"}"#.into()),
        )]])
        .append_query_results([[candidate_record(
            "candidate-1",
            "req-1",
            Some(r#"{"authorization":"****"}"#.into()),
            Some(r#"{"model":"gpt-5.5"}"#.into()),
            Some(r#"{"content-type":"application/json"}"#.into()),
            Some(r#"{"id":"resp-1"}"#.into()),
            "success",
        )]])
        .append_query_results([[candidate_record(
            "candidate-1",
            "req-1",
            Some(r#"{"__hook_compressed_payload__":true,"encoding":"zlib+hex","original_size":24,"data":"789c"}"#.into()),
            Some(r#"{"__hook_compressed_payload__":true,"encoding":"zlib+hex","original_size":19,"data":"789c"}"#.into()),
            Some(r#"{"__hook_compressed_payload__":true,"encoding":"zlib+hex","original_size":33,"data":"789c"}"#.into()),
            Some(r#"{"__hook_compressed_payload__":true,"encoding":"zlib+hex","original_size":15,"data":"789c"}"#.into()),
            "success",
        )]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let changed = store.compress_request_record_payloads_before(time::OffsetDateTime::now_utc()).await.unwrap();

    assert_eq!(changed, 2);
    let logs = connection.into_transaction_log();
    assert!(logs[1].statements()[0].sql.contains("UPDATE \"request_records\" SET"));
    assert!(logs[3].statements()[0].sql.contains("UPDATE \"request_candidates\" SET"));
}

#[tokio::test]
async fn request_record_storage_sweeps_stale_pending_requests() {
    let now = time::OffsetDateTime::now_utc();
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[stale_pending_record(now)]])
        .append_query_results([[failed_record(now)]])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let report = store
        .sweep_stale_request_records(now - time::Duration::minutes(15), now - time::Duration::minutes(120))
        .await
        .unwrap();

    assert_eq!(report.pending_records, 1);
    assert_eq!(report.streaming_records, 0);
    assert_eq!(report.failed_candidates, 1);
    assert_eq!(report.skipped_candidates, 1);
    let logs = connection.into_transaction_log();
    assert!(logs[1].statements()[0].sql.contains("UPDATE \"request_records\" SET"));
    assert!(logs[2].statements()[0].sql.contains("UPDATE \"request_candidates\" SET"));
    assert!(logs[3].statements()[0].sql.contains("UPDATE \"request_candidates\" SET"));
}

fn summary_record(
    request_id: &str,
    request_headers: Option<String>,
    request_body: Option<String>,
    client_response_headers: Option<String>,
    client_response_body: Option<String>,
) -> request_records::Model {
    request_records::Model {
        request_id: request_id.into(),
        token_id: Some("token-1".into()),
        user_id_snapshot: Some("user-1".into()),
        username_snapshot: Some("hwnet".into()),
        token_name_snapshot: Some("Token A".into()),
        token_prefix_snapshot: Some("sk-test".into()),
        group_code: Some("default".into()),
        global_model_id: Some("gpt-5.5".into()),
        model_name_snapshot: Some("gpt-5.5".into()),
        provider_id: Some("provider-1".into()),
        provider_name_snapshot: Some("Provider A".into()),
        endpoint_id: Some("endpoint-1".into()),
        key_id: Some("key-1".into()),
        provider_key_name_snapshot: Some("Key A".into()),
        provider_key_preview_snapshot: Some("***test".into()),
        client_api_format: "openai_chat".into(),
        provider_api_format: Some("openai_chat".into()),
        request_type: "chat".into(),
        is_stream: false,
        has_failover: false,
        has_retry: false,
        status: "success".into(),
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
        first_byte_time_ms: Some(12),
        total_latency_ms: Some(42),
        candidate_count: 1,
        request_headers,
        request_body,
        billing_snapshot: None,
        client_response_headers,
        client_response_body,
        created_at: now(),
        started_at: Some(now()),
        finished_at: Some(now()),
        updated_at: now(),
    }
}

fn candidate_record(
    id: &str,
    request_id: &str,
    provider_request_headers: Option<String>,
    provider_request_body: Option<String>,
    provider_response_headers: Option<String>,
    provider_response_body: Option<String>,
    status: &str,
) -> request_candidates::Model {
    request_candidates::Model {
        id: id.into(),
        request_id: request_id.into(),
        token_id: Some("token-1".into()),
        group_code: Some("default".into()),
        global_model_id: Some("gpt-5.5".into()),
        provider_id: Some("provider-1".into()),
        provider_name_snapshot: Some("Provider A".into()),
        endpoint_id: Some("endpoint-1".into()),
        endpoint_name_snapshot: Some("openai_chat".into()),
        key_id: Some("key-1".into()),
        key_name_snapshot: Some("Key A".into()),
        key_preview_snapshot: Some("***test".into()),
        client_api_format: "openai_chat".into(),
        provider_api_format: Some("openai_chat".into()),
        needs_conversion: false,
        is_stream: false,
        is_cached: false,
        provider_request_headers,
        provider_request_body,
        provider_response_headers,
        provider_response_body,
        candidate_index: 0,
        retry_index: 0,
        status: status.into(),
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
        service_tier: Some("standard".into()),
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
        error_type: None,
        error_message: None,
        error_code: None,
        error_param: None,
        created_at: now(),
        started_at: Some(now()),
        finished_at: Some(now()),
    }
}

fn stale_pending_record(now: time::OffsetDateTime) -> request_records::Model {
    request_records::Model {
        request_id: "req-stale".into(),
        token_id: Some("token-1".into()),
        user_id_snapshot: Some("user-1".into()),
        username_snapshot: Some("hwnet".into()),
        token_name_snapshot: Some("Token A".into()),
        token_prefix_snapshot: Some("sk-test".into()),
        group_code: Some("default".into()),
        global_model_id: Some("gpt-5.5".into()),
        model_name_snapshot: Some("gpt-5.5".into()),
        provider_id: Some("provider-1".into()),
        provider_name_snapshot: Some("Provider A".into()),
        endpoint_id: Some("endpoint-1".into()),
        key_id: Some("key-1".into()),
        provider_key_name_snapshot: Some("Key A".into()),
        provider_key_preview_snapshot: Some("***test".into()),
        client_api_format: "openai_chat".into(),
        provider_api_format: Some("openai_chat".into()),
        request_type: "chat".into(),
        is_stream: false,
        has_failover: false,
        has_retry: false,
        status: "pending".into(),
        billing_status: "pending".into(),
        client_status_code: None,
        client_error_type: None,
        client_error_message: None,
        termination_origin: None,
        termination_reason: None,
        stream_end_reason: None,
        prompt_tokens: None,
        completion_tokens: None,
        total_tokens: None,
        cache_creation_input_tokens: None,
        cache_read_input_tokens: None,
        input_text_tokens: None,
        input_audio_tokens: None,
        input_image_tokens: None,
        output_text_tokens: None,
        output_audio_tokens: None,
        output_image_tokens: None,
        reasoning_tokens: None,
        cache_creation_5m_input_tokens: None,
        cache_creation_1h_input_tokens: None,
        usage_source: None,
        usage_semantic: None,
        service_tier: None,
        input_cost: None,
        output_cost: None,
        cache_creation_cost: None,
        cache_read_cost: None,
        request_cost: None,
        input_price_per_million: None,
        output_price_per_million: None,
        cache_creation_price_per_million: None,
        cache_read_price_per_million: None,
        cost_currency: None,
        token_cost: None,
        base_cost: None,
        total_cost: None,
        billing_multiplier: None,
        billing_snapshot: None,
        first_byte_time_ms: None,
        total_latency_ms: None,
        candidate_count: 2,
        request_headers: None,
        request_body: None,
        client_response_headers: None,
        client_response_body: None,
        created_at: now - time::Duration::hours(1),
        started_at: Some(now - time::Duration::hours(1)),
        finished_at: None,
        updated_at: now - time::Duration::hours(1),
    }
}

fn failed_record(now: time::OffsetDateTime) -> request_records::Model {
    let mut record = stale_pending_record(now);
    record.status = "failed".into();
    record.billing_status = "void".into();
    record.client_status_code = Some(504);
    record.client_error_type = Some("stale_pending_request".into());
    record.client_error_message = Some("request remained pending beyond stale sweep threshold".into());
    record.termination_origin = Some("server".into());
    record.termination_reason = Some("pending_timeout".into());
    record.finished_at = Some(now);
    record.updated_at = now;
    record
}

fn now() -> time::OffsetDateTime {
    time::OffsetDateTime::now_utc()
}
