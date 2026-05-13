use std::collections::BTreeMap;

use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, Value};
use storage::{
    Database,
    provider::{
        ProviderStore,
        record::{provider_api_keys, provider_endpoints, providers, request_candidates, request_records},
    },
};
use types::provider::{ActiveRequestRecordRequest, RequestRecordListRequest};

#[tokio::test]
async fn request_record_storage_lists_aggregated_records() {
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[count_row(2)]])
            .append_query_results([list_summaries()])
            .append_query_results([provider_records()])
            .append_query_results([endpoint_records()])
            .append_query_results([key_records()])
            .append_query_results([token_records()])
            .append_query_results([user_records()])
            .append_query_results([model_records()])
            .into_connection(),
    );
    let store = ProviderStore::new(database);

    let response = store.list_request_records(RequestRecordListRequest::default()).await.unwrap();
    let success = response.records.iter().find(|record| record.request_id == "req-success").unwrap();
    let streaming = response.records.iter().find(|record| record.request_id == "req-stream").unwrap();

    assert_eq!(response.total, 2);
    assert_eq!(success.status, "success");
    assert_eq!(success.billing_status, "settled");
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
async fn request_record_storage_returns_trace_detail() {
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([success_candidates()])
            .append_query_results([provider_records()])
            .append_query_results([endpoint_records()])
            .append_query_results([key_records()])
            .append_query_results([token_records()])
            .append_query_results([user_records()])
            .append_query_results([model_records()])
            .into_connection(),
    );
    let store = ProviderStore::new(database);

    let detail = store.get_request_record("req-success").await.unwrap();
    let failed = &detail.candidates[0];
    let success = &detail.candidates[1];

    assert_eq!(detail.record.request_id, "req-success");
    assert_eq!(detail.record.candidate_count, 2);
    assert_eq!(detail.record.total_cost, Decimal::new(2, 4));
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
        detail.response_body.as_ref().and_then(|value| value.get("id")).and_then(|value| value.as_str()),
        Some("msg-1")
    );
    assert_eq!(failed.status, "failed");
    assert_eq!(failed.error_message.as_deref(), Some("rate limit"));
    assert_eq!(failed.created_at, "2026-05-11T11:01:17Z");
    assert_eq!(failed.started_at.as_deref(), Some("2026-05-11T11:01:17Z"));
    assert_eq!(failed.finished_at.as_deref(), Some("2026-05-11T11:02:17Z"));
    assert!(!failed.is_stream);
    assert_eq!(success.status_code, Some(200));
    assert_eq!(success.total_tokens, Some(20));
    assert_eq!(success.cache_creation_input_tokens, Some(3));
    assert_eq!(success.cache_read_input_tokens, Some(4));
    assert_eq!(success.key_name.as_deref(), Some("primary-key"));
    assert_eq!(success.key_preview.as_deref(), Some("***abcd"));
}

#[tokio::test]
async fn request_record_storage_lists_active_records_by_ids() {
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([list_summaries()])
            .append_query_results([provider_records()])
            .append_query_results([endpoint_records()])
            .append_query_results([key_records()])
            .append_query_results([token_records()])
            .append_query_results([user_records()])
            .append_query_results([model_records()])
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
        .append_query_results([Vec::<providers::Model>::new()])
        .append_query_results([Vec::<provider_endpoints::Model>::new()])
        .append_query_results([Vec::<provider_api_keys::Model>::new()])
        .append_query_results([Vec::<storage::api_token::api_token_records::Model>::new()])
        .append_query_results([Vec::<storage::user::UserRecord>::new()])
        .append_query_results([Vec::<storage::model::global_models::Model>::new()])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let response = store
        .list_request_records(RequestRecordListRequest {
            search: Some("hwnet".into()),
            status: Some("success".into()),
            model_id: Some("model-1".into()),
            provider_id: Some("provider-1".into()),
            api_format: Some("openai_chat".into()),
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
    assert!(count_sql.contains("LOWER(COALESCE(u.username, '')) LIKE $"), "{count_sql}");
    assert!(list_sql.contains("ORDER BY r.created_at DESC"), "{list_sql}");
    assert!(list_sql.contains("LIMIT $"), "{list_sql}");
    assert!(list_sql.contains("OFFSET $"), "{list_sql}");
}

fn list_summaries() -> Vec<request_records::Model> {
    vec![
        summary("req-stream", "streaming", true, false, false, 1, 6),
        summary("req-success", "success", false, true, true, 2, 2),
    ]
}

fn summary(request_id: &str, status: &str, is_stream: bool, has_failover: bool, has_retry: bool, candidate_count: i64, minute: u8) -> request_records::Model {
    request_records::Model {
        request_id: request_id.into(),
        token_id: Some("token-1".into()),
        group_code: Some("default".into()),
        global_model_id: Some("gpt-5.5".into()),
        provider_id: Some("provider-1".into()),
        endpoint_id: Some("endpoint-1".into()),
        key_id: Some("key-1".into()),
        client_api_format: "openai_cli".into(),
        provider_api_format: Some("claude_chat".into()),
        request_type: "chat".into(),
        is_stream,
        has_failover,
        has_retry,
        status: status.into(),
        billing_status: billing_status(status).into(),
        prompt_tokens: (status == "success").then_some(12),
        completion_tokens: (status == "success").then_some(8),
        total_tokens: (status == "success").then_some(20),
        cache_creation_input_tokens: (status == "success").then_some(3),
        cache_read_input_tokens: (status == "success").then_some(4),
        cost_currency: (status == "success").then(|| "USD".into()),
        token_cost: (status == "success").then_some(Decimal::new(1, 4)),
        base_cost: (status == "success").then_some(Decimal::new(1, 5)),
        total_cost: (status == "success").then_some(Decimal::new(2, 4)),
        billing_multiplier: (status == "success").then_some(Decimal::new(2, 0)),
        first_byte_time_ms: first_byte_time_ms(status),
        total_latency_ms: (status == "success").then_some(570),
        candidate_count,
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
        "failed" => "void",
        _ => "pending",
    }
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
        endpoint_id: Some("endpoint-1".into()),
        key_id: Some("key-1".into()),
        client_api_format: "openai_cli".into(),
        provider_api_format: Some("claude_chat".into()),
        needs_conversion: true,
        is_stream: status == "streaming",
        request_headers: request_headers(status),
        request_body: request_body(status),
        response_body: response_body(status),
        candidate_index,
        retry_index,
        status: status.into(),
        status_code: (status == "success").then_some(200),
        prompt_tokens: (status == "success").then_some(12),
        completion_tokens: (status == "success").then_some(8),
        total_tokens: (status == "success").then_some(20),
        cache_creation_input_tokens: (status == "success").then_some(3),
        cache_read_input_tokens: (status == "success").then_some(4),
        cost_currency: (status == "success").then(|| "USD".into()),
        token_cost: (status == "success").then_some(Decimal::new(1, 4)),
        base_cost: (status == "success").then_some(Decimal::new(1, 5)),
        total_cost: (status == "success").then_some(Decimal::new(2, 4)),
        billing_multiplier: (status == "success").then_some(Decimal::new(2, 0)),
        latency_ms: latency_ms(status),
        first_byte_time_ms: first_byte_time_ms(status),
        error_type: (status == "failed").then(|| "upstream_error".into()),
        error_message: (status == "failed").then(|| "rate limit".into()),
        created_at: at_minute(minute),
        started_at: Some(at_minute(minute)),
        finished_at: (status != "streaming").then(|| at_minute(minute + 1)),
    }
}

fn provider_records() -> Vec<providers::Model> {
    vec![providers::Model {
        id: "provider-1".into(),
        name: "paid-channel-86".into(),
        provider_type: "custom".into(),
        max_retries: Some(2),
        request_timeout_seconds: Some(60.0),
        stream_first_byte_timeout_seconds: Some(15.0),
        priority: 10,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
        created_at: at_minute(0),
        updated_at: at_minute(0),
    }]
}

fn endpoint_records() -> Vec<provider_endpoints::Model> {
    vec![provider_endpoints::Model {
        id: "endpoint-1".into(),
        provider_id: "provider-1".into(),
        api_format: "claude_chat".into(),
        base_url: "https://example.test".into(),
        custom_path: None,
        max_retries: None,
        is_active: true,
        format_acceptance_config: None,
        header_rules: None,
        body_rules: None,
        created_at: at_minute(0),
        updated_at: at_minute(0),
    }]
}

fn key_records() -> Vec<provider_api_keys::Model> {
    vec![provider_api_keys::Model {
        id: "key-1".into(),
        provider_id: "provider-1".into(),
        name: "primary-key".into(),
        encrypted_api_key: "sk-provider-abcd".into(),
        note: None,
        internal_priority: 10,
        rpm_limit: None,
        learned_rpm_limit: None,
        cache_ttl_minutes: 5,
        max_probe_interval_minutes: 32,
        time_range_enabled: false,
        time_range_start: None,
        time_range_end: None,
        health_by_format: None,
        circuit_breaker_by_format: None,
        is_active: true,
        created_at: at_minute(0),
        updated_at: at_minute(0),
    }]
}

fn token_records() -> Vec<storage::api_token::api_token_records::Model> {
    vec![storage::api_token::api_token_records::Model {
        id: "token-1".into(),
        user_id: Some("user-1".into()),
        token_type: "user".into(),
        name: "pro池".into(),
        token_value: "sk-token".into(),
        token_hash: "hash".into(),
        token_prefix: "sk-a0JNVPA".into(),
        group_code: "default".into(),
        expires_at: None,
        model_access_mode: "all".into(),
        allowed_model_ids: "[]".into(),
        rate_limit_rpm: None,
        quota_limit: None,
        used_quota: Decimal::ZERO,
        request_count: 0,
        is_active: true,
        last_used_at: None,
        created_at: at_minute(0),
        updated_at: at_minute(0),
    }]
}

fn user_records() -> Vec<storage::user::UserRecord> {
    vec![storage::user::UserRecord {
        id: "user-1".into(),
        username: "hwnet".into(),
        password_hash: "hash".into(),
        email: "hwnet@example.test".into(),
        role: "user".into(),
        is_active: true,
        is_deleted: false,
        allowed_model_ids: "[]".into(),
        allowed_provider_ids: "[]".into(),
        created_at: at_minute(0),
        updated_at: at_minute(0),
        last_login_at: None,
        auth_source: "local".into(),
        email_verified: true,
        rate_limit_rpm: None,
        quota_mode: "wallet".into(),
    }]
}

fn model_records() -> Vec<storage::model::global_models::Model> {
    vec![storage::model::global_models::Model {
        id: "gpt-5.5".into(),
        name: "gpt-5.5".into(),
        display_name: "GPT-5.5".into(),
        default_price_per_request: None,
        default_tiered_pricing: "{}".into(),
        supported_capabilities: None,
        config: None,
        is_active: true,
        usage_count: 0,
        created_at: at_minute(0),
        updated_at: at_minute(0),
    }]
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
    (status == "success").then(|| r#"{"id":"msg-1"}"#.into())
}

fn at_minute(minute: u8) -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::May, 11)
        .unwrap()
        .with_hms(11, minute, 17)
        .unwrap()
        .assume_utc()
}
