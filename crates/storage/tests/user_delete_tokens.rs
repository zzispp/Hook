use constants::user_group::DEFAULT_USER_GROUP_CODE;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use storage::{
    Database,
    user::{UserRecord, UserStore},
};
use types::user::UserId;

#[tokio::test]
async fn user_delete_removes_owned_api_tokens_before_soft_delete() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[user_record(false)]])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 2,
        }])
        .append_query_results([[user_record(true)]])
        .into_connection();
    let store = UserStore::new(Database::new(connection.clone()));

    store.delete(UserId("user-1".into())).await.unwrap();

    let logs = connection.into_transaction_log();
    let statements = logs
        .iter()
        .flat_map(|entry| entry.statements())
        .map(|statement| statement.sql.as_str())
        .collect::<Vec<_>>();
    assert!(statements.iter().any(|sql| sql.contains("DELETE FROM \"api_tokens\"")), "{statements:?}");
    assert!(statements.iter().any(|sql| sql.contains("\"api_tokens\".\"user_id\" = $")), "{statements:?}");
    assert!(
        statements.iter().all(|sql| !sql.contains("\"api_tokens\".\"token_type\" = $")),
        "{statements:?}"
    );
    assert!(statements.iter().any(|sql| sql.contains("UPDATE \"users\" SET")), "{statements:?}");
    assert!(statements.iter().any(|sql| sql.contains("\"is_deleted\" = $")), "{statements:?}");
}

fn user_record(is_deleted: bool) -> UserRecord {
    UserRecord {
        id: "user-1".into(),
        username: "hwnet".into(),
        password_hash: Some("hash".into()),
        email: "hwnet@example.test".into(),
        group_code: DEFAULT_USER_GROUP_CODE.into(),
        role: "user".into(),
        is_active: true,
        is_deleted,
        allowed_model_ids: "[]".into(),
        allowed_provider_ids: "[]".into(),
        created_at: now(),
        updated_at: now(),
        last_login_at: None,
        auth_source: "local".into(),
        email_verified: true,
        rate_limit_rpm: None,
        quota_mode: "wallet".into(),
    }
}

fn now() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::May, 14)
        .unwrap()
        .with_hms(10, 30, 0)
        .unwrap()
        .assume_utc()
}
