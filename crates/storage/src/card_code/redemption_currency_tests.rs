use rust_decimal::Decimal;
use types::wallet::{Wallet, WalletId};

use super::*;
use crate::StorageError;

#[test]
fn uses_accounting_card_amounts_without_conversion() {
    let amounts = accounting_redemption_amounts(&card_code(currency::ACCOUNTING_CURRENCY)).unwrap();

    assert_eq!(amounts.recharge, Decimal::new(1000, 2));
    assert_eq!(amounts.gift, Decimal::new(200, 2));
}

#[test]
fn rejects_non_accounting_card_currency() {
    let error = accounting_redemption_amounts(&card_code("CNY")).unwrap_err();

    assert!(matches!(error, StorageError::Conflict(message) if message == "card code currency must be USD"));
}

#[test]
fn rejects_non_accounting_create_currency() {
    let error = ensure_accounting_currency("CNY", "card code currency").unwrap_err();

    assert!(matches!(error, StorageError::Conflict(message) if message == "card code currency must be USD"));
}

#[test]
fn accepts_accounting_wallet_currency() {
    let wallet = wallet_in_accounting_currency(wallet(currency::ACCOUNTING_CURRENCY)).unwrap();

    assert_eq!(wallet.currency, currency::ACCOUNTING_CURRENCY);
}

#[test]
fn rejects_non_accounting_wallet_currency() {
    let error = wallet_in_accounting_currency(wallet("CNY")).unwrap_err();

    assert!(matches!(error, StorageError::Conflict(message) if message == "wallet currency must be USD"));
}

fn card_code(currency: &str) -> CardCodeRecord {
    CardCodeRecord {
        id: "card_1".into(),
        code: "CARD-CODE".into(),
        batch_no: "batch_1".into(),
        type_id: "type_1".into(),
        type_name: "Recharge".into(),
        recharge_amount: Decimal::new(1000, 2),
        gift_amount: Decimal::new(200, 2),
        currency: currency.into(),
        status: "active".into(),
        remark: None,
        expires_at: None,
        created_by_user_id: None,
        created_by_username: None,
        created_ip: None,
        used_by_user_id: None,
        used_by_username: None,
        used_ip: None,
        used_at: None,
        wallet_id: None,
        wallet_transaction_id: None,
        created_at: time::OffsetDateTime::UNIX_EPOCH,
        updated_at: time::OffsetDateTime::UNIX_EPOCH,
    }
}

fn wallet(currency: &str) -> Wallet {
    Wallet {
        id: WalletId("wallet_1".into()),
        user_id: "user_1".into(),
        recharge_balance: Decimal::new(7000, 2),
        gift_balance: Decimal::new(1400, 2),
        currency: currency.into(),
        status: "active".into(),
        limit_mode: "finite".into(),
        total_recharged: Decimal::new(7000, 2),
        total_consumed: Decimal::ZERO,
        total_refunded: Decimal::ZERO,
        total_adjusted: Decimal::new(1400, 2),
        created_at: timestamp(),
        updated_at: timestamp(),
    }
}

fn timestamp() -> String {
    "2026-05-14T00:00:00Z".into()
}
