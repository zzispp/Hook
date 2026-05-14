use rust_decimal::Decimal;
use types::{card_code::CardCodeRedeemInput, wallet::Wallet};

use crate::{StorageError, StorageResult, card_code::CardCodeRecord};

pub(super) const CURRENCY_CNY: &str = "CNY";
pub(super) const CURRENCY_USD: &str = "USD";
const MONEY_SCALE: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RedemptionAmounts {
    pub(super) recharge: Decimal,
    pub(super) gift: Decimal,
}

impl RedemptionAmounts {
    pub(super) fn total(self) -> Decimal {
        self.recharge + self.gift
    }
}

pub(super) fn redemption_amounts(code: &CardCodeRecord, input: &CardCodeRedeemInput) -> StorageResult<RedemptionAmounts> {
    Ok(RedemptionAmounts {
        recharge: convert_amount(code.recharge_amount, &code.currency, input)?,
        gift: convert_amount(code.gift_amount, &code.currency, input)?,
    })
}

pub(super) fn wallet_in_target_currency(wallet: Wallet, input: &CardCodeRedeemInput) -> StorageResult<Wallet> {
    if wallet.currency == input.target_currency {
        return Ok(wallet);
    }
    let source_currency = wallet.currency.clone();
    Ok(Wallet {
        recharge_balance: convert_amount(wallet.recharge_balance, &source_currency, input)?,
        gift_balance: convert_amount(wallet.gift_balance, &source_currency, input)?,
        currency: input.target_currency.clone(),
        total_recharged: convert_amount(wallet.total_recharged, &source_currency, input)?,
        total_consumed: convert_amount(wallet.total_consumed, &source_currency, input)?,
        total_refunded: convert_amount(wallet.total_refunded, &source_currency, input)?,
        total_adjusted: convert_amount(wallet.total_adjusted, &source_currency, input)?,
        ..wallet
    })
}

fn convert_amount(value: Decimal, source_currency: &str, input: &CardCodeRedeemInput) -> StorageResult<Decimal> {
    if source_currency == input.target_currency {
        return Ok(value);
    }
    let rate = required_usd_cny_rate(input)?;
    match (source_currency, input.target_currency.as_str()) {
        (CURRENCY_USD, CURRENCY_CNY) => Ok((value * rate).round_dp(MONEY_SCALE)),
        (CURRENCY_CNY, CURRENCY_USD) => Ok((value / rate).round_dp(MONEY_SCALE)),
        _ => Err(StorageError::Database(format!(
            "unsupported card code currency conversion: {source_currency} -> {}",
            input.target_currency
        ))),
    }
}

fn required_usd_cny_rate(input: &CardCodeRedeemInput) -> StorageResult<Decimal> {
    let rate = input
        .usd_cny_rate
        .ok_or_else(|| StorageError::Database("USD/CNY exchange rate is required for card code redemption".into()))?;
    if rate <= Decimal::ZERO {
        return Err(StorageError::Database("USD/CNY exchange rate must be greater than 0".into()));
    }
    Ok(rate)
}

#[cfg(test)]
#[path = "redemption_currency_tests.rs"]
mod tests;
