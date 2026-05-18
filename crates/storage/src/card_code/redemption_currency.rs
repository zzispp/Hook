use types::wallet::Wallet;

use crate::{StorageError, StorageResult, card_code::CardCodeRecord};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RedemptionAmounts {
    pub(super) recharge: rust_decimal::Decimal,
    pub(super) gift: rust_decimal::Decimal,
}

impl RedemptionAmounts {
    pub(super) fn total(self) -> rust_decimal::Decimal {
        self.recharge + self.gift
    }
}

pub(super) fn accounting_redemption_amounts(code: &CardCodeRecord) -> StorageResult<RedemptionAmounts> {
    ensure_accounting_currency(&code.currency, "card code currency")?;
    Ok(RedemptionAmounts {
        recharge: code.recharge_amount,
        gift: code.gift_amount,
    })
}

pub(super) fn wallet_in_accounting_currency(wallet: Wallet) -> StorageResult<Wallet> {
    ensure_accounting_currency(&wallet.currency, "wallet currency")?;
    Ok(wallet)
}

pub(super) fn ensure_accounting_currency(value: &str, field: &str) -> StorageResult<()> {
    if value == currency::ACCOUNTING_CURRENCY {
        return Ok(());
    }
    Err(StorageError::Conflict(format!("{field} must be {}", currency::ACCOUNTING_CURRENCY)))
}

#[cfg(test)]
#[path = "redemption_currency_tests.rs"]
mod tests;
