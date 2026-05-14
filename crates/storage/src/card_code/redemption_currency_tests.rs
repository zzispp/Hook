use rust_decimal::Decimal;
use types::{
    card_code::CardCodeRedeemInput,
    wallet::{Wallet, WalletId},
};

use super::*;
use crate::StorageError;

#[test]
fn converts_usd_card_amount_to_cny_target_currency() {
    let input = redeem_input(CURRENCY_CNY, Some(Decimal::new(720, 2)));

    let amount = convert_amount(Decimal::new(1000, 2), CURRENCY_USD, &input).unwrap();

    assert_eq!(amount, Decimal::new(7200, 2));
}

#[test]
fn converts_cny_card_amount_to_usd_target_currency() {
    let input = redeem_input(CURRENCY_USD, Some(Decimal::new(720, 2)));

    let amount = convert_amount(Decimal::new(7200, 2), CURRENCY_CNY, &input).unwrap();

    assert_eq!(amount, Decimal::new(1000, 2));
}

#[test]
fn same_currency_conversion_does_not_require_exchange_rate() {
    let input = redeem_input(CURRENCY_CNY, None);

    let amount = convert_amount(Decimal::new(1000, 2), CURRENCY_CNY, &input).unwrap();

    assert_eq!(amount, Decimal::new(1000, 2));
}

#[test]
fn wallet_balances_are_normalized_before_crediting_target_currency() {
    let input = redeem_input(CURRENCY_USD, Some(Decimal::new(700, 2)));

    let wallet = wallet_in_target_currency(wallet(CURRENCY_CNY), &input).unwrap();

    assert_eq!(wallet.currency, CURRENCY_USD);
    assert_eq!(wallet.recharge_balance, Decimal::new(1000, 2));
    assert_eq!(wallet.gift_balance, Decimal::new(200, 2));
}

#[test]
fn currency_mismatch_exposes_missing_exchange_rate() {
    let input = redeem_input(CURRENCY_USD, None);

    let error = convert_amount(Decimal::new(1000, 2), CURRENCY_CNY, &input).unwrap_err();

    assert!(matches!(error, StorageError::Database(message) if message.contains("exchange rate")));
}

fn redeem_input(target_currency: &str, usd_cny_rate: Option<Decimal>) -> CardCodeRedeemInput {
    CardCodeRedeemInput {
        code: "CARD-CODE".into(),
        user_id: "user_1".into(),
        username: "alice".into(),
        client_ip: None,
        target_currency: target_currency.into(),
        usd_cny_rate,
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
