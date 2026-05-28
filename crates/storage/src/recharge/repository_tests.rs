use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, Value};
use std::collections::BTreeMap;

use super::{RechargeOrderRecordInput, RechargeStore};
use crate::{Database, StorageError, user::UserRecord};

#[tokio::test]
async fn create_order_locks_user_counts_pending_orders_then_inserts() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[user_record()]])
        .append_query_results([[count_row(0)]])
        .append_query_results([[order_record("pending")]])
        .into_connection();
    let store = RechargeStore::new(Database::new(connection.clone()));

    let order = store.create_order(order_input(), 5).await.unwrap();

    assert_eq!(order.order_no, "R1001");
    let sql = transaction_sql(connection);
    assert!(sql.iter().any(|item| item.contains("BEGIN")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("FROM \"users\"")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("FOR UPDATE")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("COUNT")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("INSERT INTO \"recharge_orders\"")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("COMMIT")), "{sql:?}");
}

#[tokio::test]
async fn create_order_rejects_when_unexpired_pending_count_reaches_limit() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[user_record()]])
        .append_query_results([[count_row(5)]])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let store = RechargeStore::new(Database::new(connection.clone()));

    let error = store.create_order(order_input(), 5).await.unwrap_err();

    assert!(matches!(error, StorageError::Conflict(message) if message == "unpaid recharge order limit reached: 5"));
    let sql = transaction_sql(connection);
    assert!(sql.iter().any(|item| item.contains("ROLLBACK")), "{sql:?}");
    assert!(!sql.iter().any(|item| item.contains("INSERT INTO \"recharge_orders\"")), "{sql:?}");
}

fn transaction_sql(connection: sea_orm::DatabaseConnection) -> Vec<String> {
    connection
        .into_transaction_log()
        .iter()
        .flat_map(|entry| entry.statements())
        .map(|statement| statement.sql.clone())
        .collect()
}

fn order_input() -> RechargeOrderRecordInput {
    RechargeOrderRecordInput {
        order_no: "R1001".into(),
        user_id: "user-1".into(),
        package_id: Some("package-1".into()),
        package_name: "Starter".into(),
        recharge_amount: Decimal::new(10, 0),
        gift_amount: Decimal::new(2, 0),
        total_arrival_amount: Decimal::new(12, 0),
        payable_amount: Decimal::new(10, 0),
        status: "pending".into(),
        payment_channel_code: Some("epay".into()),
        payment_channel_name: Some("易支付".into()),
        payment_method: Some("alipay".into()),
        payment_request_json: Some(serde_json::json!({"kind": "form_post"})),
        expires_at: now() + time::Duration::minutes(30),
    }
}

fn user_record() -> UserRecord {
    UserRecord {
        id: "user-1".into(),
        username: "alice".into(),
        password_hash: "hash".into(),
        email: "alice@example.com".into(),
        role: "user".into(),
        group_code: "default".into(),
        is_active: true,
        is_deleted: false,
        allowed_model_ids: "[]".into(),
        allowed_provider_ids: "[]".into(),
        created_at: now(),
        updated_at: now(),
        last_login_at: None,
        auth_source: UserRecord::local_auth_source(),
        email_verified: true,
        rate_limit_rpm: None,
        quota_mode: UserRecord::default_quota_mode(),
    }
}

fn order_record(status: &str) -> super::RechargeOrderRecord {
    super::RechargeOrderRecord {
        id: "order-1".into(),
        order_no: "R1001".into(),
        user_id: "user-1".into(),
        package_id: Some("package-1".into()),
        package_name: "Starter".into(),
        recharge_amount: Decimal::new(10, 0),
        gift_amount: Decimal::new(2, 0),
        total_arrival_amount: Decimal::new(12, 0),
        payable_amount: Decimal::new(10, 0),
        status: status.into(),
        payment_channel_code: Some("epay".into()),
        payment_channel_name: Some("易支付".into()),
        payment_method: Some("alipay".into()),
        provider_trade_no: None,
        payment_request_json: None,
        refund_status: None,
        refund_amount: None,
        paid_at: None,
        refunded_at: None,
        expires_at: now() + time::Duration::minutes(30),
        created_at: now(),
        updated_at: now(),
    }
}

fn count_row(total: i64) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([("num_items", total.into())])
}

fn now() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::May, 27)
        .unwrap()
        .with_hms(10, 0, 0)
        .unwrap()
        .assume_utc()
}
