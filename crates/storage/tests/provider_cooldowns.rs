use sea_orm::{DatabaseBackend, MockDatabase};
use storage::{
    Database,
    provider::{
        ProviderCooldownRecordInput, ProviderStore,
        record::{provider_cooldown_events, provider_cooldowns},
    },
};

#[tokio::test]
async fn provider_cooldown_event_is_appended() {
    let event = event_record("event-1");
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[event.clone()]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    store.create_provider_cooldown_event(input()).await.unwrap();

    let logs = connection.into_transaction_log();
    let sql = &logs[0].statements()[0].sql;
    assert!(sql.contains("INSERT INTO \"provider_cooldown_events\""), "{sql}");
    assert!(sql.contains("\"triggered_at\""), "{sql}");
}

#[tokio::test]
async fn active_provider_cooldowns_for_restore_reads_unreleased_active_records() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![cooldown_record()]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let cooldowns = store.active_provider_cooldowns_for_restore().await.unwrap();

    assert_eq!(cooldowns.len(), 1);
    assert_eq!(cooldowns[0].provider_id, "provider-a");
    let logs = connection.into_transaction_log();
    let sql = &logs[0].statements()[0].sql;
    assert!(sql.contains("\"released_at\" IS NULL"), "{sql}");
    assert!(sql.contains("\"cooldown_until\" >"), "{sql}");
}

fn input() -> ProviderCooldownRecordInput {
    ProviderCooldownRecordInput {
        provider_id: "provider-a".into(),
        provider_name_snapshot: "Provider A".into(),
        status_code: 429,
        observed_count: 3,
        threshold_count: 3,
        window_seconds: 60,
        cooldown_seconds: 300,
        triggered_at: ts(10),
        cooldown_until: ts(310),
        request_id: "req-1".into(),
        candidate_index: 0,
        retry_index: 1,
        endpoint_id: Some("endpoint-a".into()),
        endpoint_name_snapshot: Some("Endpoint A".into()),
        key_id: Some("key-a".into()),
        key_name_snapshot: Some("Key A".into()),
        error_type: Some("rate_limit".into()),
        error_message: Some("limited".into()),
        error_code: Some("rate_limit_exceeded".into()),
        error_param: None,
    }
}

fn cooldown_record() -> provider_cooldowns::Model {
    let input = input();
    provider_cooldowns::Model {
        provider_id: input.provider_id,
        provider_name_snapshot: input.provider_name_snapshot,
        status_code: input.status_code,
        observed_count: input.observed_count,
        threshold_count: input.threshold_count,
        window_seconds: input.window_seconds,
        cooldown_seconds: input.cooldown_seconds,
        triggered_at: input.triggered_at,
        cooldown_until: time::OffsetDateTime::now_utc() + time::Duration::minutes(5),
        released_at: None,
        request_id: input.request_id,
        candidate_index: input.candidate_index,
        retry_index: input.retry_index,
        endpoint_id: input.endpoint_id,
        endpoint_name_snapshot: input.endpoint_name_snapshot,
        key_id: input.key_id,
        key_name_snapshot: input.key_name_snapshot,
        error_type: input.error_type,
        error_message: input.error_message,
        error_code: input.error_code,
        error_param: input.error_param,
        created_at: ts(10),
        updated_at: ts(20),
    }
}

fn event_record(id: &str) -> provider_cooldown_events::Model {
    let input = input();
    provider_cooldown_events::Model {
        id: id.into(),
        provider_id: input.provider_id,
        provider_name_snapshot: input.provider_name_snapshot,
        status_code: input.status_code,
        observed_count: input.observed_count,
        threshold_count: input.threshold_count,
        window_seconds: input.window_seconds,
        cooldown_seconds: input.cooldown_seconds,
        triggered_at: input.triggered_at,
        cooldown_until: input.cooldown_until,
        request_id: input.request_id,
        candidate_index: input.candidate_index,
        retry_index: input.retry_index,
        endpoint_id: input.endpoint_id,
        endpoint_name_snapshot: input.endpoint_name_snapshot,
        key_id: input.key_id,
        key_name_snapshot: input.key_name_snapshot,
        error_type: input.error_type,
        error_message: input.error_message,
        error_code: input.error_code,
        error_param: input.error_param,
        created_at: ts(20),
    }
}

fn ts(seconds: i64) -> time::OffsetDateTime {
    time::OffsetDateTime::from_unix_timestamp(seconds).unwrap()
}
