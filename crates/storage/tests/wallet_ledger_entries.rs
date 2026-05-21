use std::collections::BTreeMap;

use rust_decimal::Decimal;
use sea_orm::{DatabaseBackend, MockDatabase, Value};
use storage::{Database, wallet::WalletStore};
use types::{
    pagination::PageSliceRequest,
    wallet::{WalletDailyUsageDetailRequest, WalletLedgerEntryFilters, WalletLedgerEntryKind},
};

#[tokio::test]
async fn wallet_ledger_entries_aggregate_model_usage_by_local_day() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[count_row(1)]])
        .append_query_results([vec![daily_entry_row("2026-05-21", 3)]])
        .into_connection();
    let store = WalletStore::new(Database::new(connection.clone()));

    let page = store
        .page_ledger_entries("wallet-1", page_request(), WalletLedgerEntryFilters::default(), 480)
        .await
        .unwrap();

    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].entry_kind, WalletLedgerEntryKind::DailyModelUsage);
    assert_eq!(page.items[0].local_date.as_deref(), Some("2026-05-21"));
    assert_eq!(page.items[0].transaction_count, 3);
    let logs = connection.into_transaction_log();
    let count_sql = &logs[0].statements()[0].sql;
    assert!(count_sql.contains("GROUP BY f.wallet_id, f.local_date"), "{count_sql}");
    assert!(count_sql.contains("INTERVAL '1 minute'"), "{count_sql}");
}

#[tokio::test]
async fn admin_ledger_entries_group_by_wallet_and_day() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[count_row(2)]])
        .append_query_results([vec![admin_daily_entry_row("wallet-1"), admin_daily_entry_row("wallet-2")]])
        .into_connection();
    let store = WalletStore::new(Database::new(connection.clone()));

    let page = store
        .page_admin_ledger_entries(page_request(), WalletLedgerEntryFilters::default(), 480)
        .await
        .unwrap();

    assert_eq!(page.total, 2);
    assert_eq!(page.items[0].entry.entry_kind, WalletLedgerEntryKind::DailyModelUsage);
    assert_eq!(page.items[0].owner_name, "owner-wallet-1");
    let logs = connection.into_transaction_log();
    let list_sql = &logs[1].statements()[0].sql;
    assert!(list_sql.contains("JOIN wallets w ON w.id = t.wallet_id"), "{list_sql}");
    assert!(list_sql.contains("GROUP BY f.wallet_id, f.local_date"), "{list_sql}");
}

#[tokio::test]
async fn daily_usage_transactions_filter_by_local_date_and_paginate() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[count_row(5)]])
        .append_query_results([vec![transaction_row("tx-1")]])
        .into_connection();
    let store = WalletStore::new(Database::new(connection.clone()));

    let page = store
        .page_daily_usage_transactions(
            "wallet-1",
            page_request(),
            WalletDailyUsageDetailRequest {
                local_date: "2026-05-21".into(),
                tz_offset_minutes: 480,
            },
        )
        .await
        .unwrap();

    assert_eq!(page.total, 5);
    assert_eq!(page.items[0].id, "tx-1");
    let logs = connection.into_transaction_log();
    let count_sql = &logs[0].statements()[0].sql;
    let list_sql = &logs[1].statements()[0].sql;
    assert!(count_sql.contains("t.category = $"), "{count_sql}");
    assert!(count_sql.contains("to_char(date_trunc('day'"), "{count_sql}");
    assert!(list_sql.contains("LIMIT $"), "{list_sql}");
    assert!(list_sql.contains("OFFSET $"), "{list_sql}");
}

fn page_request() -> PageSliceRequest {
    PageSliceRequest {
        offset: 0,
        limit: 20,
        page: 1,
        page_size: 20,
    }
}

fn count_row(total: i64) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([("total", total.into())])
}

fn daily_entry_row(local_date: &'static str, count: i64) -> BTreeMap<&'static str, Value> {
    base_entry_row("daily_model_usage", "daily_model_usage:wallet-1:2026-05-21", "wallet-1", local_date, count)
}

fn admin_daily_entry_row(wallet_id: &'static str) -> BTreeMap<&'static str, Value> {
    let mut row = base_entry_row("daily_model_usage", "daily", wallet_id, "2026-05-21", 2);
    row.extend([
        ("currency", Value::from(currency::DEFAULT_WALLET_CURRENCY)),
        ("owner_name", Value::from(format!("owner-{wallet_id}"))),
        ("owner_email", Value::from(format!("{wallet_id}@example.com"))),
        ("owner_type", Value::from("user")),
        ("wallet_status", Value::from("active")),
    ]);
    row
}

fn transaction_row(id: &'static str) -> storage::wallet::record::wallet_transactions::Model {
    storage::wallet::record::wallet_transactions::Model {
        id: id.into(),
        wallet_id: "wallet-1".into(),
        category: "consume".into(),
        reason_code: "llm_model_usage".into(),
        amount: Decimal::new(-1, 0),
        balance_before: Decimal::new(10, 0),
        balance_after: Decimal::new(9, 0),
        recharge_balance_before: Decimal::new(10, 0),
        recharge_balance_after: Decimal::new(9, 0),
        gift_balance_before: Decimal::ZERO,
        gift_balance_after: Decimal::ZERO,
        link_type: Some("llm_request_record".into()),
        link_id: Some("request-1".into()),
        operator_id: None,
        description: Some("usage".into()),
        created_at: ts(0),
    }
}

fn base_entry_row(
    kind: &'static str,
    id: &'static str,
    wallet_id: &'static str,
    local_date: &'static str,
    count: i64,
) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([
        ("entry_kind", Value::from(kind)),
        ("id", Value::from(id)),
        ("wallet_id", Value::from(wallet_id)),
        ("category", Value::from("consume")),
        ("reason_code", Value::from("llm_model_usage")),
        ("amount", Value::from(Decimal::new(-3, 0))),
        ("balance_before", Value::from(Decimal::new(10, 0))),
        ("balance_after", Value::from(Decimal::new(7, 0))),
        ("recharge_balance_before", Value::from(Decimal::new(10, 0))),
        ("recharge_balance_after", Value::from(Decimal::new(7, 0))),
        ("gift_balance_before", Value::from(Decimal::ZERO)),
        ("gift_balance_after", Value::from(Decimal::ZERO)),
        ("link_type", Value::from("llm_request_record")),
        ("link_id", Value::String(None)),
        ("operator_id", Value::String(None)),
        ("description", Value::String(None)),
        ("created_at", Value::from(ts(60))),
        ("local_date", Value::from(local_date)),
        ("transaction_count", Value::from(count)),
        ("first_created_at", Value::from(ts(0))),
        ("last_created_at", Value::from(ts(60))),
    ])
}

fn ts(minutes: i64) -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::May, 21)
        .unwrap()
        .with_hms(0, 0, 0)
        .unwrap()
        .assume_utc()
        + time::Duration::minutes(minutes)
}
