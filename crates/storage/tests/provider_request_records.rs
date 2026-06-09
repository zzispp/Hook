use std::collections::BTreeMap;

use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, Value};
use storage::{
    Database, StorageError,
    provider::{
        ProviderStore, RequestBillingRecordPatch, RequestBillingRecordValues, RequestRecordRecordInput, RequestRecordRecordPatch,
        RequestUpstreamCostRecordPatch,
        record::{request_candidates, request_records},
    },
};
use types::model::PatchField;
use types::provider::{ActiveRequestRecordRequest, RequestRecordListRequest, RequestUpstreamCost};

#[tokio::test]
async fn request_record_storage_lists_aggregated_records() {
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[count_row(2)]])
            .append_query_results([list_summaries()])
            .into_connection(),
    );
    let store = ProviderStore::new(database);

    let response = store.list_request_records(RequestRecordListRequest::default()).await.unwrap();
    let success = response.records.iter().find(|record| record.request_id == "req-success").unwrap();
    let streaming = response.records.iter().find(|record| record.request_id == "req-stream").unwrap();

    assert_eq!(response.total, 2);
    assert_eq!(success.status, "success");
    assert_eq!(success.billing_status, "settled");
    assert_eq!(success.client_status_code, Some(200));
    assert_eq!(success.client_error_type, None);
    assert_eq!(success.username.as_deref(), Some("hwnet"));
    assert_eq!(success.provider_name.as_deref(), Some("paid-channel-86"));
    assert_eq!(success.provider_key_name.as_deref(), Some("primary-key"));
    assert_eq!(success.provider_key_preview.as_deref(), Some("***abcd"));
    assert!(success.has_failover);
    assert!(success.has_retry);
    assert_eq!(success.model_name.as_deref(), Some("gpt-5.5"));
    assert_eq!(success.prompt_tokens, Some(12));
    assert_eq!(success.completion_tokens, Some(8));
    assert_eq!(success.total_tokens, Some(20));
    assert_eq!(success.cache_creation_input_tokens, Some(3));
    assert_eq!(success.cache_read_input_tokens, Some(4));
    assert_eq!(success.input_text_tokens, Some(7));
    assert_eq!(success.output_text_tokens, Some(5));
    assert_eq!(success.reasoning_tokens, Some(2));
    assert_eq!(success.usage_source.as_deref(), Some("openai"));
    assert_eq!(success.usage_semantic.as_deref(), Some("openai"));
    assert_eq!(success.created_at, "2026-05-11T11:02:17Z");
    assert_eq!(success.first_byte_time_ms, Some(110));
    assert_eq!(success.total_latency_ms, Some(570));
    assert_eq!(streaming.status, "streaming");
    assert_eq!(streaming.billing_status, "pending");
    assert!(streaming.is_stream);
    assert_eq!(streaming.first_byte_time_ms, Some(120));
    assert_eq!(streaming.total_latency_ms, None);
    assert!(!streaming.has_failover);
    assert!(!streaming.has_retry);
}

#[tokio::test]
async fn request_record_storage_keeps_snapshot_names_without_live_refs() {
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[count_row(1)]])
            .append_query_results([vec![summary("req-success", "success", false, true, true, 2, 2)]])
            .into_connection(),
    );
    let store = ProviderStore::new(database);

    let response = store.list_request_records(RequestRecordListRequest::default()).await.unwrap();
    let record = &response.records[0];

    assert_eq!(response.total, 1);
    assert_eq!(record.user_id.as_deref(), Some("user-1"));
    assert_eq!(record.username.as_deref(), Some("hwnet"));
    assert_eq!(record.token_name.as_deref(), Some("pro池"));
    assert_eq!(record.provider_name.as_deref(), Some("paid-channel-86"));
    assert_eq!(record.provider_key_name.as_deref(), Some("primary-key"));
    assert_eq!(record.provider_key_preview.as_deref(), Some("***abcd"));
}

#[tokio::test]
async fn request_record_storage_returns_trace_detail() {
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[summary("req-success", "success", false, true, true, 2, 2)]])
            .append_query_results([success_candidates()])
            .append_query_results([empty_payload_rows()])
            .append_query_results([empty_payload_rows()])
            .append_query_results([empty_payload_rows()])
            .into_connection(),
    );
    let store = ProviderStore::new(database);

    let detail = store.get_request_record("req-success").await.unwrap();
    let failed = &detail.candidates[0];
    let success = &detail.candidates[1];

    assert_eq!(detail.record.request_id, "req-success");
    assert_eq!(detail.record.candidate_count, 2);
    assert_eq!(detail.record.total_cost, Decimal::new(2, 4));
    assert_eq!(detail.record.service_tier.as_deref(), Some("standard"));
    assert_eq!(detail.record.input_cost, Some(Decimal::new(25, 4)));
    assert_eq!(detail.record.output_cost, Some(Decimal::new(30, 4)));
    assert_eq!(detail.record.cache_read_cost, Some(Decimal::new(125, 6)));
    assert_eq!(detail.record.input_price_per_million, Some(Decimal::new(250, 2)));
    assert_eq!(
        detail
            .request_headers
            .as_ref()
            .and_then(|value| value.get("authorization"))
            .and_then(|value| value.as_str()),
        Some("****")
    );
    assert_eq!(
        detail
            .request_body
            .as_ref()
            .and_then(|value| value.get("model"))
            .and_then(|value| value.as_str()),
        Some("gpt-5.5")
    );
    assert_eq!(
        detail
            .client_response_body
            .as_ref()
            .and_then(|value| value.get("id"))
            .and_then(|value| value.as_str()),
        Some("msg-1")
    );
    assert!(detail.client_response_headers.is_none());
    assert_eq!(failed.status, "failed");
    assert_eq!(failed.error_message.as_deref(), Some("rate limit"));
    assert_eq!(failed.error_code.as_deref(), Some("rate_limit"));
    assert_eq!(failed.error_param.as_deref(), Some("model"));
    assert_eq!(
        failed
            .provider_response_body
            .as_ref()
            .and_then(|value| value.get("error"))
            .and_then(|value| value.as_str()),
        Some("rate limit")
    );
    assert_eq!(failed.created_at, "2026-05-11T11:01:17Z");
    assert_eq!(failed.started_at.as_deref(), Some("2026-05-11T11:01:17Z"));
    assert_eq!(failed.finished_at.as_deref(), Some("2026-05-11T11:02:17Z"));
    assert!(!failed.is_stream);
    assert_eq!(success.status_code, Some(200));
    assert_eq!(success.total_tokens, Some(20));
    assert_eq!(success.cache_creation_input_tokens, Some(3));
    assert_eq!(success.cache_read_input_tokens, Some(4));
    assert_eq!(success.cache_creation_5m_input_tokens, Some(1));
    assert_eq!(success.cache_creation_1h_input_tokens, Some(2));
    assert_eq!(success.key_name.as_deref(), Some("primary-key"));
    assert_eq!(success.key_preview.as_deref(), Some("***abcd"));
    assert_eq!(
        success
            .provider_request_body
            .as_ref()
            .and_then(|value| value.get("model"))
            .and_then(|value| value.as_str()),
        Some("gpt-5.5")
    );
}

#[tokio::test]
async fn request_record_storage_lists_active_records_by_ids() {
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([list_summaries()])
            .into_connection(),
    );
    let store = ProviderStore::new(database);

    let response = store
        .list_active_request_records(ActiveRequestRecordRequest {
            ids: vec!["req-success".into(), "req-stream".into()],
        })
        .await
        .unwrap();

    let success = response.records.iter().find(|record| record.request_id == "req-success").unwrap();
    let streaming = response.records.iter().find(|record| record.request_id == "req-stream").unwrap();

    assert_eq!(response.records.len(), 2);
    assert_eq!(success.status, "success");
    assert_eq!(streaming.status, "streaming");
}

#[tokio::test]
async fn request_record_storage_filters_summary_before_pagination() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[count_row(0)]])
        .append_query_results([Vec::<request_records::Model>::new()])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let response = store
        .list_request_records(RequestRecordListRequest {
            search: Some("hwnet".into()),
            status: Some("success".into()),
            model_id: Some("model-1".into()),
            provider_id: Some("provider-1".into()),
            api_format: Some("openai:chat".into()),
            type_filter: Some("stream".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(response.total, 0);
    let logs = connection.into_transaction_log();
    let count_sql = &logs[0].statements()[0].sql;
    let list_sql = &logs[1].statements()[0].sql;
    assert!(count_sql.contains("FROM request_records r"), "{count_sql}");
    assert!(count_sql.contains("r.status = $"), "{count_sql}");
    assert!(count_sql.contains("r.global_model_id = $"), "{count_sql}");
    assert!(count_sql.contains("r.provider_id = $"), "{count_sql}");
    assert!(count_sql.contains("r.client_api_format = $"), "{count_sql}");
    assert!(count_sql.contains("r.provider_api_format = $"), "{count_sql}");
    assert!(count_sql.contains("r.is_stream = TRUE"), "{count_sql}");
    assert!(count_sql.contains("LOWER(COALESCE(r.username_snapshot, '')) LIKE $"), "{count_sql}");
    assert!(list_sql.contains("ORDER BY r.created_at DESC"), "{list_sql}");
    assert!(list_sql.contains("LIMIT $"), "{list_sql}");
    assert!(list_sql.contains("OFFSET $"), "{list_sql}");
    assert_summary_query_avoids_legacy_payload_columns(count_sql);
    assert_summary_query_avoids_legacy_payload_columns(list_sql);
}

#[tokio::test]
async fn request_record_storage_lists_user_usage_records_without_upstream_fields() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[count_row(1)]])
        .append_query_results([vec![summary("req-success", "success", false, true, true, 2, 2)]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let response = store
        .list_usage_records(
            "user-1",
            RequestRecordListRequest {
                search: Some("openai".into()),
                model_id: Some("gpt-5.5".into()),
                provider_id: Some("provider-1".into()),
                api_format: Some("openai:cli".into()),
                type_filter: Some("non_stream".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let record_json = serde_json::to_value(&response.records[0]).unwrap();

    assert_eq!(response.total, 1);
    assert_eq!(response.records[0].token_name.as_deref(), Some("pro池"));
    assert_eq!(response.records[0].token_prefix.as_deref(), Some("sk-a0JNVPA"));
    assert_eq!(response.records[0].model_name.as_deref(), Some("gpt-5.5"));
    assert_eq!(response.records[0].client_api_format, "openai:cli");
    assert_eq!(response.records[0].status, "success");
    assert!(record_json.get("provider_id").is_none());
    assert!(record_json.get("provider_name").is_none());
    assert!(record_json.get("provider_key_name").is_none());
    assert!(record_json.get("provider_api_format").is_none());
    assert!(record_json.get("request_id").is_none());

    let logs = connection.into_transaction_log();
    let count_sql = &logs[0].statements()[0].sql;
    assert!(count_sql.contains("r.user_id_snapshot = $"), "{count_sql}");
    assert!(count_sql.contains("(r.global_model_id = $"), "{count_sql}");
    assert!(count_sql.contains("r.model_name_snapshot = $"), "{count_sql}");
    assert!(count_sql.contains("r.client_api_format = $"), "{count_sql}");
    assert!(!count_sql.contains("r.provider_id = $"), "{count_sql}");
    assert!(!count_sql.contains("r.provider_api_format = $"), "{count_sql}");
}

#[tokio::test]
async fn request_record_storage_creates_main_record() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[summary("req-created", "pending", false, false, false, 1, 6)]])
        .append_exec_results([mock_exec_result()])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    store.create_request_record(main_record_input()).await.unwrap();

    let logs = connection.into_transaction_log();
    let sql = &logs[0].statements()[0].sql;
    assert!(sql.contains("INSERT INTO \"request_records\""), "{sql}");
    assert_request_record_partition_sync(&logs[1].statements()[0].sql);
}

#[tokio::test]
async fn request_record_storage_updates_main_record() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[summary("req-success", "pending", false, false, false, 1, 2)]])
        .append_exec_results([
            mock_exec_result(),
            mock_exec_result(),
            mock_exec_result(),
            mock_exec_result(),
            mock_exec_result(),
            mock_exec_result(),
            mock_exec_result(),
        ])
        .append_query_results([[summary("req-success", "success", false, true, true, 1, 2)]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    store.update_request_record(main_record_patch()).await.unwrap();

    let logs = connection.into_transaction_log();
    let sql = &logs[1].statements()[0].sql;
    assert!(sql.contains("UPDATE \"request_records\" SET"), "{sql}");
    assert!(sql.contains("\"status\" = $"), "{sql}");
    assert!(sql.contains("\"client_status_code\" = $"), "{sql}");
    assert!(sql.contains("\"client_response_body\" = $"), "{sql}");
    assert_request_record_partition_sync(&logs.last().unwrap().statements()[0].sql);
}

#[tokio::test]
async fn request_record_storage_syncs_dashboard_tokens_with_cache_context() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[summary("req-success", "pending", false, false, false, 1, 2)]])
        .append_exec_results([
            mock_exec_result(),
            mock_exec_result(),
            mock_exec_result(),
            mock_exec_result(),
            mock_exec_result(),
            mock_exec_result(),
            mock_exec_result(),
        ])
        .append_query_results([[summary("req-success", "success", false, true, true, 1, 2)]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    store.update_request_record(main_record_patch()).await.unwrap();

    let logs = connection.into_transaction_log();
    let user_bucket = logged_statement(&logs, "dashboard_user_usage_buckets");
    let user_bucket_sql = &user_bucket.sql;
    assert!(user_bucket_sql.contains("dashboard_user_usage_buckets"), "{user_bucket_sql}");
    assert_eq!(statement_value(user_bucket.values.as_ref().unwrap(), 9), Value::from(27_i64));
    let cost_bucket = logged_statement(&logs, "dashboard_cost_analysis_buckets");
    let cost_bucket_sql = &cost_bucket.sql;
    assert!(cost_bucket_sql.contains("dashboard_cost_analysis_buckets"), "{cost_bucket_sql}");
    assert_eq!(statement_value(cost_bucket.values.as_ref().unwrap(), 12), Value::from(3_i64));
    assert_eq!(statement_value(cost_bucket.values.as_ref().unwrap(), 14), Value::from(27_i64));
}

fn mock_exec_result() -> MockExecResult {
    MockExecResult {
        last_insert_id: 0,
        rows_affected: 1,
    }
}

fn assert_request_record_partition_sync(sql: &str) {
    assert!(sql.contains("INSERT INTO request_records_partitioned"), "{sql}");
    assert!(sql.contains("ON CONFLICT (created_at, request_id) DO UPDATE"), "{sql}");
    assert!(!sql.contains("request_headers"), "{sql}");
}

fn assert_summary_query_avoids_legacy_payload_columns(sql: &str) {
    assert!(!sql.contains("r.request_headers"), "{sql}");
    assert!(!sql.contains("r.request_body"), "{sql}");
    assert!(!sql.contains("r.client_response_headers"), "{sql}");
    assert!(!sql.contains("r.client_response_body"), "{sql}");
}

fn empty_payload_rows() -> Vec<BTreeMap<&'static str, Value>> {
    Vec::new()
}

fn statement_value(values: &sea_orm::Values, index: usize) -> Value {
    values.iter().nth(index).cloned().expect("statement value must exist")
}

fn logged_statement<'a>(logs: &'a [sea_orm::Transaction], pattern: &str) -> &'a sea_orm::Statement {
    logs.iter()
        .flat_map(|entry| entry.statements())
        .find(|statement| statement.sql.contains(pattern))
        .expect("statement must be logged")
}

#[tokio::test]
async fn request_record_storage_rejects_non_accounting_cost_currency_on_create() {
    let database = Database::new(MockDatabase::new(DatabaseBackend::Postgres).into_connection());
    let store = ProviderStore::new(database);
    let mut input = main_record_input();
    input.billing.cost_currency = Some("CNY".into());

    let error = store.create_request_record(input).await.unwrap_err();

    assert!(matches!(error, StorageError::Conflict(message) if message == "cost currency must be USD"));
}

#[tokio::test]
async fn request_record_storage_rejects_non_accounting_cost_currency_on_update() {
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[summary("req-success", "pending", false, false, false, 1, 2)]])
            .into_connection(),
    );
    let store = ProviderStore::new(database);
    let mut input = main_record_patch();
    input.billing.cost_currency = PatchField::Value("CNY".into());

    let error = store.update_request_record(input).await.unwrap_err();

    assert!(matches!(error, StorageError::Conflict(message) if message == "cost currency must be USD"));
}

fn list_summaries() -> Vec<request_records::Model> {
    vec![
        summary("req-stream", "streaming", true, false, false, 1, 6),
        summary("req-success", "success", false, true, true, 2, 2),
    ]
}

fn main_record_input() -> RequestRecordRecordInput {
    RequestRecordRecordInput {
        request_id: "req-created".into(),
        token_id: Some("token-1".into()),
        user_id_snapshot: Some("user-1".into()),
        username_snapshot: Some("hwnet".into()),
        token_name_snapshot: Some("pro池".into()),
        token_prefix_snapshot: Some("sk-a0JNVPA".into()),
        group_code: Some("default".into()),
        global_model_id: Some("gpt-5.5".into()),
        model_name_snapshot: Some("gpt-5.5".into()),
        provider_id: Some("provider-1".into()),
        provider_name_snapshot: Some("paid-channel-86".into()),
        endpoint_id: Some("endpoint-1".into()),
        key_id: Some("key-1".into()),
        provider_key_name_snapshot: Some("primary-key".into()),
        provider_key_preview_snapshot: Some("***abcd".into()),
        client_api_format: "openai:cli".into(),
        provider_api_format: Some("claude:chat".into()),
        request_type: "chat".into(),
        is_stream: false,
        has_failover: false,
        has_retry: false,
        status: "pending".into(),
        billing_status: "pending".into(),
        upstream_cost: RequestUpstreamCost::default(),
        billing: RequestBillingRecordValues {
            service_tier: Some("standard".into()),
            ..RequestBillingRecordValues::default()
        },
        billing_snapshot: None,
        candidate_count: 1,
        request_headers: Some(serde_json::json!({"authorization": "****"})),
        request_body: Some(serde_json::json!({"model": "gpt-5.5"})),
    }
}

fn main_record_patch() -> RequestRecordRecordPatch {
    RequestRecordRecordPatch {
        request_id: "req-success".into(),
        provider_id: Some("provider-1".into()),
        provider_name_snapshot: Some("paid-channel-86".into()),
        endpoint_id: Some("endpoint-1".into()),
        key_id: Some("key-1".into()),
        provider_key_name_snapshot: Some("primary-key".into()),
        provider_key_preview_snapshot: Some("***abcd".into()),
        provider_api_format: Some("claude:chat".into()),
        is_stream: Some(false),
        has_failover: Some(true),
        has_retry: Some(true),
        status: "success".into(),
        billing_status: "settled".into(),
        client_status_code: PatchField::Value(200),
        client_error_type: PatchField::Null,
        client_error_message: PatchField::Null,
        termination_origin: PatchField::Null,
        termination_reason: PatchField::Null,
        stream_end_reason: PatchField::Null,
        prompt_tokens: PatchField::Value(12),
        completion_tokens: PatchField::Value(8),
        total_tokens: PatchField::Value(20),
        cache_creation_input_tokens: PatchField::Value(3),
        cache_read_input_tokens: PatchField::Value(4),
        input_text_tokens: PatchField::Value(7),
        input_audio_tokens: PatchField::Value(1),
        input_image_tokens: PatchField::Value(2),
        output_text_tokens: PatchField::Value(5),
        output_audio_tokens: PatchField::Value(1),
        output_image_tokens: PatchField::Value(2),
        reasoning_tokens: PatchField::Value(2),
        cache_creation_5m_input_tokens: PatchField::Value(1),
        cache_creation_1h_input_tokens: PatchField::Value(2),
        usage_source: PatchField::Value("openai".into()),
        usage_semantic: PatchField::Value("openai".into()),
        upstream_cost: RequestUpstreamCostRecordPatch::default(),
        billing: success_billing_patch(),
        billing_snapshot: PatchField::Missing,
        first_byte_time_ms: PatchField::Value(110),
        total_latency_ms: PatchField::Value(570),
        client_response_headers: PatchField::Value(serde_json::json!({"content-type": "application/json"})),
        client_response_body: PatchField::Value(serde_json::json!({"id": "msg-1"})),
        started: true,
        finished: true,
    }
}

fn summary(request_id: &str, status: &str, is_stream: bool, has_failover: bool, has_retry: bool, candidate_count: i64, minute: u8) -> request_records::Model {
    request_records::Model {
        request_id: request_id.into(),
        token_id: Some("token-1".into()),
        user_id_snapshot: Some("user-1".into()),
        username_snapshot: Some("hwnet".into()),
        token_name_snapshot: Some("pro池".into()),
        token_prefix_snapshot: Some("sk-a0JNVPA".into()),
        group_code: Some("default".into()),
        global_model_id: Some("gpt-5.5".into()),
        model_name_snapshot: Some("gpt-5.5".into()),
        provider_id: Some("provider-1".into()),
        provider_name_snapshot: Some("paid-channel-86".into()),
        endpoint_id: Some("endpoint-1".into()),
        key_id: Some("key-1".into()),
        provider_key_name_snapshot: Some("primary-key".into()),
        provider_key_preview_snapshot: Some("***abcd".into()),
        client_api_format: "openai:cli".into(),
        provider_api_format: Some("claude:chat".into()),
        request_type: "chat".into(),
        is_stream,
        has_failover,
        has_retry,
        status: status.into(),
        billing_status: billing_status(status).into(),
        client_status_code: client_status_code(status),
        client_error_type: client_error_type(status),
        client_error_message: client_error_message(status),
        termination_origin: (status == "cancelled").then(|| "client".into()),
        termination_reason: (status == "cancelled").then(|| "disconnected".into()),
        stream_end_reason: (status == "cancelled").then(|| "client_disconnected".into()),
        prompt_tokens: (status == "success").then_some(12),
        completion_tokens: (status == "success").then_some(8),
        total_tokens: (status == "success").then_some(20),
        cache_creation_input_tokens: (status == "success").then_some(3),
        cache_read_input_tokens: (status == "success").then_some(4),
        input_text_tokens: (status == "success").then_some(7),
        input_audio_tokens: (status == "success").then_some(1),
        input_image_tokens: (status == "success").then_some(2),
        output_text_tokens: (status == "success").then_some(5),
        output_audio_tokens: (status == "success").then_some(1),
        output_image_tokens: (status == "success").then_some(2),
        reasoning_tokens: (status == "success").then_some(2),
        cache_creation_5m_input_tokens: (status == "success").then_some(1),
        cache_creation_1h_input_tokens: (status == "success").then_some(2),
        usage_source: (status == "success").then(|| "openai".into()),
        usage_semantic: (status == "success").then(|| "openai".into()),
        service_tier: (status == "success").then(|| "standard".into()),
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
        input_cost: (status == "success").then_some(Decimal::new(25, 4)),
        output_cost: (status == "success").then_some(Decimal::new(30, 4)),
        cache_creation_cost: (status == "success").then_some(Decimal::new(125, 5)),
        cache_read_cost: (status == "success").then_some(Decimal::new(125, 6)),
        request_cost: (status == "success").then_some(Decimal::new(1, 2)),
        input_price_per_million: (status == "success").then_some(Decimal::new(250, 2)),
        output_price_per_million: (status == "success").then_some(Decimal::new(1500, 2)),
        cache_creation_price_per_million: (status == "success").then_some(Decimal::new(125, 2)),
        cache_read_price_per_million: (status == "success").then_some(Decimal::new(25, 2)),
        cost_currency: (status == "success").then(|| currency::ACCOUNTING_CURRENCY.into()),
        token_cost: (status == "success").then_some(Decimal::new(1, 4)),
        base_cost: (status == "success").then_some(Decimal::new(1, 5)),
        total_cost: (status == "success").then_some(Decimal::new(2, 4)),
        billing_multiplier: (status == "success").then_some(Decimal::new(2, 0)),
        billing_snapshot: None,
        first_byte_time_ms: first_byte_time_ms(status),
        total_latency_ms: (status == "success").then_some(570),
        candidate_count,
        request_headers: request_headers(status),
        request_body: request_body(status),
        client_response_headers: None,
        client_response_body: response_body(status),
        payload_compressed_at: None,
        created_at: at_minute(minute),
        started_at: Some(at_minute(minute)),
        finished_at: (status != "streaming").then(|| at_minute(minute + 1)),
        updated_at: at_minute(minute + 1),
    }
}

fn count_row(total: i64) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([("total", total.into())])
}

fn billing_status(status: &str) -> &'static str {
    match status {
        "success" => "settled",
        "cancelled" => "void",
        "failed" => "void",
        _ => "pending",
    }
}

fn success_billing_patch() -> RequestBillingRecordPatch {
    RequestBillingRecordPatch {
        service_tier: PatchField::Value("standard".into()),
        cost_currency: PatchField::Value(currency::ACCOUNTING_CURRENCY.into()),
        input_cost: PatchField::Value(Decimal::new(25, 4)),
        output_cost: PatchField::Value(Decimal::new(30, 4)),
        cache_creation_cost: PatchField::Value(Decimal::new(125, 5)),
        cache_read_cost: PatchField::Value(Decimal::new(125, 6)),
        request_cost: PatchField::Value(Decimal::new(1, 2)),
        token_cost: PatchField::Value(Decimal::new(1, 4)),
        base_cost: PatchField::Value(Decimal::new(1, 5)),
        total_cost: PatchField::Value(Decimal::new(2, 4)),
        billing_multiplier: PatchField::Value(Decimal::new(2, 0)),
        input_price_per_million: PatchField::Value(Decimal::new(250, 2)),
        output_price_per_million: PatchField::Value(Decimal::new(1500, 2)),
        cache_creation_price_per_million: PatchField::Value(Decimal::new(125, 2)),
        cache_read_price_per_million: PatchField::Value(Decimal::new(25, 2)),
    }
}

fn client_status_code(status: &str) -> Option<i32> {
    match status {
        "success" => Some(200),
        "cancelled" => Some(499),
        _ => None,
    }
}

fn client_error_type(status: &str) -> Option<String> {
    (status == "cancelled").then(|| "client_disconnected".into())
}

fn client_error_message(status: &str) -> Option<String> {
    (status == "cancelled").then(|| "client disconnected".into())
}

fn success_candidates() -> Vec<request_candidates::Model> {
    vec![
        candidate("req-success", "success-1", "failed", 0, 1, 1),
        candidate("req-success", "success-2", "success", 1, 0, 2),
    ]
}

fn candidate(request_id: &str, id: &str, status: &str, candidate_index: i32, retry_index: i32, minute: u8) -> request_candidates::Model {
    request_candidates::Model {
        id: id.into(),
        request_id: request_id.into(),
        token_id: Some("token-1".into()),
        group_code: Some("default".into()),
        global_model_id: Some("gpt-5.5".into()),
        provider_id: Some("provider-1".into()),
        provider_name_snapshot: Some("paid-channel-86".into()),
        endpoint_id: Some("endpoint-1".into()),
        endpoint_name_snapshot: Some("claude:chat".into()),
        key_id: Some("key-1".into()),
        key_name_snapshot: Some("primary-key".into()),
        key_preview_snapshot: Some("***abcd".into()),
        client_api_format: "openai:cli".into(),
        provider_api_format: Some("claude:chat".into()),
        needs_conversion: true,
        is_stream: status == "streaming",
        is_cached: false,
        provider_request_headers: request_headers(status),
        provider_request_body: request_body(status),
        provider_response_headers: response_headers(status),
        provider_response_body: response_body(status),
        payload_compressed_at: None,
        candidate_index,
        retry_index,
        status: status.into(),
        skip_reason: (status == "skipped").then(|| "request_terminated_before_attempt".into()),
        status_code: (status == "success").then_some(200),
        prompt_tokens: (status == "success").then_some(12),
        completion_tokens: (status == "success").then_some(8),
        total_tokens: (status == "success").then_some(20),
        cache_creation_input_tokens: (status == "success").then_some(3),
        cache_read_input_tokens: (status == "success").then_some(4),
        input_text_tokens: (status == "success").then_some(7),
        input_audio_tokens: (status == "success").then_some(1),
        input_image_tokens: (status == "success").then_some(2),
        output_text_tokens: (status == "success").then_some(5),
        output_audio_tokens: (status == "success").then_some(1),
        output_image_tokens: (status == "success").then_some(2),
        reasoning_tokens: (status == "success").then_some(2),
        cache_creation_5m_input_tokens: (status == "success").then_some(1),
        cache_creation_1h_input_tokens: (status == "success").then_some(2),
        usage_source: (status == "success").then(|| "openai".into()),
        usage_semantic: (status == "success").then(|| "openai".into()),
        service_tier: (status == "success").then(|| "standard".into()),
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
        input_cost: (status == "success").then_some(Decimal::new(25, 4)),
        output_cost: (status == "success").then_some(Decimal::new(30, 4)),
        cache_creation_cost: (status == "success").then_some(Decimal::new(125, 5)),
        cache_read_cost: (status == "success").then_some(Decimal::new(125, 6)),
        request_cost: (status == "success").then_some(Decimal::new(1, 2)),
        input_price_per_million: (status == "success").then_some(Decimal::new(250, 2)),
        output_price_per_million: (status == "success").then_some(Decimal::new(1500, 2)),
        cache_creation_price_per_million: (status == "success").then_some(Decimal::new(125, 2)),
        cache_read_price_per_million: (status == "success").then_some(Decimal::new(25, 2)),
        cost_currency: (status == "success").then(|| currency::ACCOUNTING_CURRENCY.into()),
        token_cost: (status == "success").then_some(Decimal::new(1, 4)),
        base_cost: (status == "success").then_some(Decimal::new(1, 5)),
        total_cost: (status == "success").then_some(Decimal::new(2, 4)),
        billing_multiplier: (status == "success").then_some(Decimal::new(2, 0)),
        billing_snapshot: None,
        latency_ms: latency_ms(status),
        first_byte_time_ms: first_byte_time_ms(status),
        error_type: (status == "failed").then(|| "upstream_error".into()),
        error_message: (status == "failed").then(|| "rate limit".into()),
        error_code: (status == "failed").then(|| "rate_limit".into()),
        error_param: (status == "failed").then(|| "model".into()),
        created_at: at_minute(minute),
        started_at: Some(at_minute(minute)),
        finished_at: (status != "streaming").then(|| at_minute(minute + 1)),
    }
}

fn latency_ms(status: &str) -> Option<i64> {
    match status {
        "failed" => Some(250),
        "success" => Some(320),
        _ => None,
    }
}

fn first_byte_time_ms(status: &str) -> Option<i64> {
    match status {
        "success" => Some(110),
        "streaming" => Some(120),
        _ => None,
    }
}

fn request_headers(status: &str) -> Option<String> {
    (status == "success").then(|| r#"{"authorization":"****"}"#.into())
}

fn request_body(status: &str) -> Option<String> {
    (status == "success").then(|| r#"{"model":"gpt-5.5"}"#.into())
}

fn response_body(status: &str) -> Option<String> {
    match status {
        "success" => Some(r#"{"id":"msg-1"}"#.into()),
        "failed" => Some(r#"{"error":"rate limit"}"#.into()),
        _ => None,
    }
}

fn response_headers(status: &str) -> Option<String> {
    (status == "success" || status == "failed").then(|| r#"{"content-type":"application/json"}"#.into())
}

fn at_minute(minute: u8) -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::May, 11)
        .unwrap()
        .with_hms(11, minute, 17)
        .unwrap()
        .assume_utc()
}
