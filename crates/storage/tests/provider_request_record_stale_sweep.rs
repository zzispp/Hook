use std::collections::BTreeMap;

use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, Value};
use storage::{Database, provider::ProviderStore};

macro_rules! request_record {
    ($request_id:expr, $status:expr, $is_stream:expr) => {
        storage::provider::record::request_records::Model {
            request_id: $request_id.into(),
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
            client_api_format: "openai:chat".into(),
            provider_api_format: Some("openai:chat".into()),
            request_type: "chat".into(),
            is_stream: $is_stream,
            has_failover: false,
            has_retry: false,
            status: $status.into(),
            billing_status: billing_status($status),
            client_status_code: client_status_code($status),
            client_error_type: client_error_type($status, $is_stream),
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
            response_headers_time_ms: Some(10),
            first_sse_event_time_ms: Some(11),
            first_token_time_ms: Some(12),
            first_byte_time_ms: Some(12),
            total_latency_ms: Some(42),
            candidate_count: 1,
            request_headers: None,
            request_body: None,
            billing_snapshot: None,
            client_response_headers: None,
            client_response_body: None,
            payload_compressed_at: None,
            created_at: now(),
            started_at: Some(now()),
            finished_at: ($status == "failed").then_some(now()),
            updated_at: now(),
        }
    };
}

#[tokio::test]
async fn request_record_storage_marks_stale_active_records_failed() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![request_record!("req-pending", "pending", false)]])
        .append_exec_results([exec_result(2)])
        .append_query_results([[request_record!("req-pending", "failed", false)]])
        .append_query_results([[sync_state_row("req-pending")]])
        .append_exec_results(exec_results(15))
        .append_query_results([vec![request_record!("req-streaming", "streaming", true)]])
        .append_exec_results([exec_result(3)])
        .append_query_results([[request_record!("req-streaming", "failed", true)]])
        .append_query_results([[sync_state_row("req-streaming")]])
        .append_exec_results(exec_results(15))
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let result = store.mark_stale_request_records_failed(now(), now()).await.unwrap();

    assert_eq!(result.request_records, 2);
    assert_eq!(result.request_candidates, 5);
    let sql = logged_sql(connection);
    assert!(
        sql.iter().any(|item| item.contains("SELECT") && item.contains("\"request_records\"")),
        "{sql:?}"
    );
    assert!(sql.iter().any(|item| item.contains("UPDATE \"request_candidates\" SET")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("\"request_candidates\".\"status\" IN")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("UPDATE \"request_records\" SET")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("\"client_error_type\" = $")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("INSERT INTO dashboard_cost_analysis_buckets")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("INSERT INTO dashboard_request_metric_buckets")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("dashboard_recent_error_snapshots")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("INSERT INTO request_records_partitioned")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("INSERT INTO request_candidates_partitioned")), "{sql:?}");
}

fn logged_sql(connection: sea_orm::DatabaseConnection) -> Vec<String> {
    connection
        .into_transaction_log()
        .iter()
        .flat_map(|entry| entry.statements())
        .map(|statement| statement.sql.clone())
        .collect()
}

fn exec_results(count: usize) -> Vec<MockExecResult> {
    (0..count).map(|_| exec_result(1)).collect()
}

fn exec_result(rows_affected: u64) -> MockExecResult {
    MockExecResult {
        last_insert_id: 0,
        rows_affected,
    }
}

fn sync_state_row(owner_id: &str) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([("owner_id", Value::from(owner_id.to_owned()))])
}

fn billing_status(status: &str) -> String {
    if status == "failed" {
        return "void".into();
    }
    "pending".into()
}

fn client_status_code(status: &str) -> Option<i32> {
    (status == "failed").then_some(504)
}

fn client_error_type(status: &str, is_stream: bool) -> Option<String> {
    if status != "failed" {
        return None;
    }
    Some(if is_stream {
        "stale_streaming_request".into()
    } else {
        "stale_pending_request".into()
    })
}

fn now() -> time::OffsetDateTime {
    time::OffsetDateTime::now_utc()
}
