use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, Value};
use std::{collections::BTreeMap, time::Duration};
use storage::{
    Database,
    provider::{
        ProviderStore, RequestRecordCleanupOptions,
        record::{request_candidates, request_records},
    },
};

const PLAIN_HEADERS: &str = r#"{"authorization":"****"}"#;
const PLAIN_BODY: &str = r#"{"model":"gpt-5.5"}"#;
const PLAIN_RESPONSE_HEADERS: &str = r#"{"content-type":"application/json"}"#;
const PLAIN_RESPONSE_BODY: &str = r#"{"id":"resp-1"}"#;
const COMPRESSED_HEADERS: &str = r#"{"__hook_compressed_payload__":true,"encoding":"zlib+hex","original_size":24,"data":"789c"}"#;
const COMPRESSED_BODY: &str = r#"{"__hook_compressed_payload__":true,"encoding":"zlib+hex","original_size":19,"data":"789c"}"#;
const COMPRESSED_RESPONSE_HEADERS: &str = r#"{"__hook_compressed_payload__":true,"encoding":"zlib+hex","original_size":33,"data":"789c"}"#;
const COMPRESSED_RESPONSE_BODY: &str = r#"{"__hook_compressed_payload__":true,"encoding":"zlib+hex","original_size":15,"data":"789c"}"#;

macro_rules! summary_model {
    ($request_id:expr, $payloads:expr) => {{
        let payloads = $payloads;
        request_records::Model {
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
            first_byte_time_ms: Some(12),
            total_latency_ms: Some(42),
            candidate_count: 1,
            request_headers: payloads.headers,
            request_body: payloads.body,
            billing_snapshot: None,
            client_response_headers: payloads.response_headers,
            client_response_body: payloads.response_body,
            created_at: now(),
            started_at: Some(now()),
            finished_at: Some(now()),
            updated_at: now(),
        }
    }};
}

macro_rules! candidate_model {
    ($id:expr, $request_id:expr, $payloads:expr) => {{
        let payloads = $payloads;
        request_candidates::Model {
            id: $id.into(),
            request_id: $request_id.into(),
            token_id: Some("token-1".into()),
            group_code: Some("default".into()),
            global_model_id: Some("gpt-5.5".into()),
            provider_id: Some("provider-1".into()),
            provider_name_snapshot: Some("Provider A".into()),
            endpoint_id: Some("endpoint-1".into()),
            endpoint_name_snapshot: Some("openai:chat".into()),
            key_id: Some("key-1".into()),
            key_name_snapshot: Some("Key A".into()),
            key_preview_snapshot: Some("***test".into()),
            client_api_format: "openai:chat".into(),
            provider_api_format: Some("openai:chat".into()),
            needs_conversion: false,
            is_stream: false,
            is_cached: false,
            provider_request_headers: payloads.headers,
            provider_request_body: payloads.body,
            provider_response_headers: payloads.response_headers,
            provider_response_body: payloads.response_body,
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
            error_type: None,
            error_message: None,
            error_code: None,
            error_param: None,
            created_at: now(),
            started_at: Some(now()),
            finished_at: Some(now()),
        }
    }};
}

#[tokio::test]
async fn request_record_storage_compresses_old_payloads() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results(timeout_exec_results(10))
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([[orphan_candidate_counts(0)]])
        .append_query_results([[summary_record("req-1", plain_payloads())]])
        .append_query_results([[summary_record("req-1", compressed_payloads())]])
        .append_query_results([[candidate_record("candidate-1", "req-1", plain_payloads())]])
        .append_query_results([[candidate_record("candidate-1", "req-1", compressed_payloads())]])
        .append_query_results([Vec::<BTreeMap<&'static str, Value>>::new()])
        .append_query_results([[orphan_candidate_counts(0)]])
        .append_query_results([Vec::<request_records::Model>::new()])
        .append_query_results([Vec::<request_candidates::Model>::new()])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let result = store.cleanup_request_records(cleanup_options()).await.unwrap();

    assert_eq!(result.compressed_records, 1);
    assert_eq!(result.compressed_candidates, 1);
    let sql = logged_sql(connection);
    assert!(sql.iter().any(|item| item.contains("UPDATE \"request_records\" SET")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("UPDATE \"request_candidates\" SET")), "{sql:?}");
}

fn cleanup_options() -> RequestRecordCleanupOptions {
    RequestRecordCleanupOptions {
        record_cutoff: now(),
        payload_cutoff: now(),
        delete_batch_size: 200,
        compress_batch_size: 50,
        max_runtime: Duration::from_secs(60),
        batch_sleep: Duration::ZERO,
        statement_timeout_seconds: 15,
        lock_timeout_seconds: 2,
    }
}

fn orphan_candidate_counts(deleted_candidates: i64) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([("deleted_candidates", Value::from(deleted_candidates))])
}

fn timeout_exec_results(count: usize) -> Vec<MockExecResult> {
    vec![MockExecResult::default(); count]
}

fn logged_sql(connection: sea_orm::DatabaseConnection) -> Vec<String> {
    connection
        .into_transaction_log()
        .iter()
        .flat_map(|entry| entry.statements())
        .map(|statement| statement.sql.clone())
        .collect()
}

fn summary_record(request_id: &str, payloads: Payloads) -> request_records::Model {
    summary_model!(request_id, payloads)
}

fn candidate_record(id: &str, request_id: &str, payloads: Payloads) -> request_candidates::Model {
    candidate_model!(id, request_id, payloads)
}

fn now() -> time::OffsetDateTime {
    time::OffsetDateTime::now_utc()
}

fn plain_payloads() -> Payloads {
    Payloads::new(PLAIN_HEADERS, PLAIN_BODY, PLAIN_RESPONSE_HEADERS, PLAIN_RESPONSE_BODY)
}

fn compressed_payloads() -> Payloads {
    Payloads::new(COMPRESSED_HEADERS, COMPRESSED_BODY, COMPRESSED_RESPONSE_HEADERS, COMPRESSED_RESPONSE_BODY)
}

struct Payloads {
    headers: Option<String>,
    body: Option<String>,
    response_headers: Option<String>,
    response_body: Option<String>,
}

impl Payloads {
    fn new(headers: &str, body: &str, response_headers: &str, response_body: &str) -> Self {
        Self {
            headers: Some(headers.into()),
            body: Some(body.into()),
            response_headers: Some(response_headers.into()),
            response_body: Some(response_body.into()),
        }
    }
}
