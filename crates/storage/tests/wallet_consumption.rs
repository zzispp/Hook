use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use storage::{
    Database, StorageError,
    wallet::{WalletConsumeRecordInput, WalletStore, record::wallets},
};

#[tokio::test]
async fn ensure_user_wallet_creates_usd_wallet() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .append_query_results([[wallet_record()]])
        .into_connection();
    let store = WalletStore::new(Database::new(connection));

    let wallet = store.ensure_user_wallet("user-1").await.unwrap();

    assert_eq!(wallet.currency, currency::DEFAULT_WALLET_CURRENCY);
}

#[tokio::test]
async fn wallet_consumption_locks_wallet_and_recomputes_balances_in_transaction() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[wallet_record()]])
        .append_query_results([[updated_wallet_record()]])
        .append_query_results([[transaction_record()]])
        .into_connection();
    let store = WalletStore::new(Database::new(connection.clone()));

    let transaction = store.consume_with_transaction(consume_input()).await.unwrap().unwrap();

    assert_eq!(transaction.balance_before, Decimal::new(15, 0));
    assert_eq!(transaction.balance_after, Decimal::new(8, 0));
    assert_eq!(transaction.recharge_balance_before, Decimal::new(10, 0));
    assert_eq!(transaction.recharge_balance_after, Decimal::new(8, 0));
    assert_eq!(transaction.gift_balance_before, Decimal::new(5, 0));
    assert_eq!(transaction.gift_balance_after, Decimal::ZERO);
    let logs = connection.into_transaction_log();
    let statements = logs
        .iter()
        .flat_map(|entry| entry.statements())
        .map(|statement| statement.sql.as_str())
        .collect::<Vec<_>>();
    assert!(statements.iter().any(|sql| sql.contains("BEGIN")), "{statements:?}");
    assert!(statements.iter().any(|sql| sql.contains("FOR UPDATE")), "{statements:?}");
    assert!(statements.iter().any(|sql| sql.contains("UPDATE \"wallets\" SET")), "{statements:?}");
    assert!(
        statements.iter().any(|sql| sql.contains("INSERT INTO \"wallet_transactions\"")),
        "{statements:?}"
    );
    assert!(statements.iter().any(|sql| sql.contains("COMMIT")), "{statements:?}");
}

#[tokio::test]
async fn wallet_consumption_rejects_insufficient_recharge_after_gift_is_spent() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[wallet_record()]])
        .into_connection();
    let store = WalletStore::new(Database::new(connection));

    let error = store
        .consume_with_transaction(WalletConsumeRecordInput {
            amount: Decimal::new(16, 0),
            ..consume_input()
        })
        .await
        .unwrap_err();

    assert!(matches!(error, StorageError::Conflict(message) if message == "insufficient wallet balance"));
}

fn consume_input() -> WalletConsumeRecordInput {
    WalletConsumeRecordInput {
        user_id: "user-1".into(),
        amount: Decimal::new(7, 0),
        category: "consume".into(),
        reason_code: "llm_model_usage".into(),
        link_type: Some("llm_request_record".into()),
        link_id: Some("request-1".into()),
        operator_id: None,
        description: Some("LLM request usage".into()),
    }
}

fn wallet_record() -> wallets::Model {
    wallets::Model {
        id: "wallet-1".into(),
        user_id: "user-1".into(),
        recharge_balance: Decimal::new(10, 0),
        gift_balance: Decimal::new(5, 0),
        currency: "USD".into(),
        status: "active".into(),
        limit_mode: "finite".into(),
        total_recharged: Decimal::new(10, 0),
        total_consumed: Decimal::ZERO,
        total_refunded: Decimal::ZERO,
        total_adjusted: Decimal::new(5, 0),
        created_at: now(),
        updated_at: now(),
    }
}

fn updated_wallet_record() -> wallets::Model {
    wallets::Model {
        recharge_balance: Decimal::new(8, 0),
        gift_balance: Decimal::ZERO,
        total_consumed: Decimal::new(7, 0),
        ..wallet_record()
    }
}

fn transaction_record() -> storage::wallet::record::wallet_transactions::Model {
    storage::wallet::record::wallet_transactions::Model {
        id: "transaction-1".into(),
        wallet_id: "wallet-1".into(),
        category: "consume".into(),
        reason_code: "llm_model_usage".into(),
        amount: -Decimal::new(7, 0),
        balance_before: Decimal::new(15, 0),
        balance_after: Decimal::new(8, 0),
        recharge_balance_before: Decimal::new(10, 0),
        recharge_balance_after: Decimal::new(8, 0),
        gift_balance_before: Decimal::new(5, 0),
        gift_balance_after: Decimal::ZERO,
        link_type: Some("llm_request_record".into()),
        link_id: Some("request-1".into()),
        operator_id: None,
        description: Some("LLM request usage".into()),
        created_at: now(),
    }
}

fn now() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::May, 15)
        .unwrap()
        .with_hms(14, 0, 0)
        .unwrap()
        .assume_utc()
}
