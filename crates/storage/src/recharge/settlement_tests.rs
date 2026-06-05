use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase};

use super::{RechargePaymentSettlementInput, RechargeStore};
use crate::{Database, StorageError, user::UserRecord, wallet::wallet_records};

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

#[test]
fn affiliate_commission_uses_payable_amount_as_base() {
    let order = order_record("pending", None);
    let context = super::AffiliateCommissionContext::new(&order, "referrer-1".into(), commission_settings(15, 0));

    assert_eq!(context.payable_amount, Decimal::new(10, 0));
    assert_eq!(context.commission_amount, Decimal::new(15, 1));
}

#[test]
fn affiliate_commission_below_minimum_fails_without_wallet_credit() {
    let order = order_record("pending", None);
    let context = super::AffiliateCommissionContext::new(&order, "referrer-1".into(), commission_settings(10, 2));

    assert_eq!(context.commission_amount, Decimal::new(1, 0));
    assert!(context.commission_below_minimum());
}

#[test]
fn affiliate_commission_equal_to_minimum_still_succeeds() {
    let order = order_record("pending", None);
    let context = super::AffiliateCommissionContext::new(&order, "referrer-1".into(), commission_settings(10, 1));

    assert_eq!(context.commission_amount, Decimal::new(1, 0));
    assert!(!context.commission_below_minimum());
}

#[test]
fn affiliate_referrer_rejects_self_reference() {
    let order = order_record("pending", None);
    let user = referred_user_record(order.user_id.clone(), Some(order.user_id.clone()));

    assert_eq!(super::valid_referrer_user_id(&user, &order), None);
}

#[test]
fn affiliate_disabled_setting_disables_commission_percent() {
    assert_eq!(super::active_affiliate_commission_settings(false, Decimal::new(10, 0), Decimal::ONE), None);
}

#[test]
fn affiliate_enabled_setting_returns_commission_percent() {
    assert_eq!(
        super::active_affiliate_commission_settings(true, Decimal::new(10, 0), Decimal::ONE),
        Some(super::AffiliateCommissionSettings {
            percent: Decimal::new(10, 0),
            min_amount: Decimal::ONE,
        })
    );
}

#[test]
fn affiliate_transaction_credits_gift_balance_only() {
    let order = order_record("pending", None);
    let wallet = types::wallet::Wallet::from(wallet_record("wallet-1", "referrer-1"));
    let context = super::AffiliateCommissionContext::new(&order, "referrer-1".into(), commission_settings(10, 0));

    let transaction = super::affiliate_transaction("tx-1".into(), &wallet, &context, now());

    assert_eq!(transaction.category.unwrap(), "gift");
    assert_eq!(transaction.reason_code.unwrap(), "affiliate_commission");
    assert_eq!(transaction.amount.unwrap(), Decimal::new(1, 0));
    assert_eq!(transaction.recharge_balance_after.unwrap(), wallet.recharge_balance);
    assert_eq!(transaction.gift_balance_after.unwrap(), wallet.gift_balance + Decimal::new(1, 0));
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

fn commission_settings(percent: i64, min_amount: i64) -> super::AffiliateCommissionSettings {
    super::AffiliateCommissionSettings {
        percent: Decimal::new(percent, 0),
        min_amount: Decimal::new(min_amount, 0),
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

fn referred_user_record(id: String, referred_by_user_id: Option<String>) -> UserRecord {
    UserRecord {
        id,
        username: "alice".into(),
        password_hash: Some("hash".into()),
        email: "alice@example.com".into(),
        role: "user".into(),
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
        affiliate_code: "alice-aff".into(),
        referred_by_user_id,
        referred_at: Some(now()),
    }
}

fn wallet_record(id: &str, user_id: &str) -> wallet_records::Model {
    wallet_records::Model {
        id: id.into(),
        user_id: user_id.into(),
        recharge_balance: Decimal::new(5, 0),
        gift_balance: Decimal::new(2, 0),
        currency: currency::ACCOUNTING_CURRENCY.into(),
        status: "active".into(),
        limit_mode: "finite".into(),
        total_recharged: Decimal::ZERO,
        total_consumed: Decimal::ZERO,
        total_refunded: Decimal::ZERO,
        total_adjusted: Decimal::ZERO,
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
