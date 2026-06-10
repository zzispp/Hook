use sea_orm::{DatabaseBackend, MockDatabase};
use storage::{
    Database,
    provider::{
        ProviderRecordInput, ProviderStore,
        record::{provider_group_providers, providers},
    },
};
use types::provider::ProviderOrigin;

#[tokio::test]
async fn create_provider_with_group_inserts_membership_in_one_transaction() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[provider_record("provider-a")]])
        .append_query_results([[provider_group_member_record("group-a", "provider-a")]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let created = store.create_provider(provider_input(Some("group-a"))).await.unwrap();

    assert_eq!(created.id, "provider-a");
    let statements = sql_statements(connection);
    assert_eq!(statements.iter().filter(|sql| sql.contains("BEGIN")).count(), 1);
    assert_eq!(statements.iter().filter(|sql| sql.contains("COMMIT")).count(), 1);
    assert!(statements.iter().any(|sql| sql.contains("INSERT INTO \"providers\"")), "{statements:?}");
    assert!(
        statements.iter().any(|sql| sql.contains("INSERT INTO \"provider_group_providers\"")),
        "{statements:?}"
    );
}

#[tokio::test]
async fn create_provider_without_group_keeps_single_provider_insert() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[provider_record("provider-a")]])
        .into_connection();
    let store = ProviderStore::new(Database::new(connection.clone()));

    let created = store.create_provider(provider_input(None)).await.unwrap();

    assert_eq!(created.id, "provider-a");
    let statements = sql_statements(connection);
    assert_eq!(statements.iter().filter(|sql| sql.contains("BEGIN")).count(), 0);
    assert!(statements.iter().any(|sql| sql.contains("INSERT INTO \"providers\"")), "{statements:?}");
    assert!(
        !statements.iter().any(|sql| sql.contains("INSERT INTO \"provider_group_providers\"")),
        "{statements:?}"
    );
}

fn provider_input(provider_group_id: Option<&str>) -> ProviderRecordInput {
    ProviderRecordInput {
        name: "Provider A".into(),
        provider_type: "custom".into(),
        provider_origin: ProviderOrigin::Manual,
        provider_group_id: provider_group_id.map(str::to_owned),
        max_retries: Some(2),
        request_timeout_seconds: Some(300.0),
        stream_first_byte_timeout_seconds: Some(60.0),
        stream_idle_timeout_seconds: Some(300.0),
        priority: 100,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
    }
}

fn provider_record(id: &str) -> providers::Model {
    providers::Model {
        id: id.into(),
        name: "Provider A".into(),
        provider_type: "custom".into(),
        provider_origin: "manual".into(),
        max_retries: Some(2),
        request_timeout_seconds: Some(300.0),
        stream_first_byte_timeout_seconds: Some(60.0),
        stream_idle_timeout_seconds: Some(300.0),
        priority: 100,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
        created_at: now(),
        updated_at: now(),
    }
}

fn provider_group_member_record(group_id: &str, provider_id: &str) -> provider_group_providers::Model {
    provider_group_providers::Model {
        id: "membership-a".into(),
        provider_group_id: group_id.into(),
        provider_id: provider_id.into(),
        priority: 100,
        created_at: now(),
        updated_at: now(),
    }
}

fn sql_statements(connection: sea_orm::DatabaseConnection) -> Vec<String> {
    connection
        .into_transaction_log()
        .into_iter()
        .flat_map(|transaction| transaction.statements().iter().map(|statement| statement.sql.clone()).collect::<Vec<_>>())
        .collect()
}

fn now() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::May, 11)
        .unwrap()
        .with_hms(12, 0, 0)
        .unwrap()
        .assume_utc()
}
