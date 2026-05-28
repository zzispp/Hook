use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase};

use super::{RechargePaymentSettlementInput, RechargeStore};
use crate::{Database, StorageError};

#[tokio::test]
async fn settle_paid_order_requires_provider_trade_no() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[order_record("pending", None)]])
        .into_connection();
    let store = RechargeStore::new(Database::new(connection.clone()));
    let input = RechargePaymentSettlementInput {
        provider_trade_no: None,
        ..settlement_input()
    };

    let error = store.settle_paid_order(input).await.unwrap_err();

    assert!(matches!(error, StorageError::Conflict(message) if message == "provider trade number is required"));
    assert_rolled_back_without_wallet_writes(connection);
}

#[tokio::test]
async fn settle_paid_order_requires_provider_amount() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[order_record("pending", None)]])
        .into_connection();
    let store = RechargeStore::new(Database::new(connection.clone()));
    let input = RechargePaymentSettlementInput {
        payable_amount: None,
        ..settlement_input()
    };

    let error = store.settle_paid_order(input).await.unwrap_err();

    assert!(matches!(error, StorageError::Conflict(message) if message == "payment amount is required"));
    assert_rolled_back_without_wallet_writes(connection);
}

#[tokio::test]
async fn settle_paid_order_rejects_reused_provider_trade_no() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[order_record("pending", None)]])
        .append_query_results([[order_record("paid", Some("trade-1"))]])
        .into_connection();
    let store = RechargeStore::new(Database::new(connection.clone()));

    let error = store.settle_paid_order(settlement_input()).await.unwrap_err();

    assert!(matches!(error, StorageError::Conflict(message) if message == "provider trade number has already been settled"));
    assert_rolled_back_without_wallet_writes(connection);
}

fn assert_rolled_back_without_wallet_writes(connection: sea_orm::DatabaseConnection) {
    let sql = transaction_sql(connection);
    assert!(sql.iter().any(|item| item.contains("BEGIN")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("FROM \"recharge_orders\"")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("FOR UPDATE")), "{sql:?}");
    assert!(sql.iter().any(|item| item.contains("ROLLBACK")), "{sql:?}");
    assert!(!sql.iter().any(|item| item.contains("UPDATE \"wallets\" SET")), "{sql:?}");
    assert!(!sql.iter().any(|item| item.contains("INSERT INTO \"wallet_transactions\"")), "{sql:?}");
    assert!(!sql.iter().any(|item| item.contains("UPDATE \"recharge_orders\" SET")), "{sql:?}");
}

fn transaction_sql(connection: sea_orm::DatabaseConnection) -> Vec<String> {
    connection
        .into_transaction_log()
        .iter()
        .flat_map(|entry| entry.statements())
        .map(|statement| statement.sql.clone())
        .collect()
}

fn settlement_input() -> RechargePaymentSettlementInput {
    RechargePaymentSettlementInput {
        order_no: "R1001".into(),
        payment_channel_code: "epay".into(),
        provider_trade_no: Some("trade-1".into()),
        payment_method: "alipay".into(),
        payable_amount: Some(Decimal::new(10, 0)),
        callback_payload: serde_json::json!({"trade_no": "trade-1"}),
    }
}

fn order_record(status: &str, provider_trade_no: Option<&str>) -> super::RechargeOrderRecord {
    super::RechargeOrderRecord {
        id: format!("order-{status}"),
        order_no: format!("R-{status}"),
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
        provider_trade_no: provider_trade_no.map(str::to_owned),
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

fn now() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::May, 27)
        .unwrap()
        .with_hms(10, 0, 0)
        .unwrap()
        .assume_utc()
}
