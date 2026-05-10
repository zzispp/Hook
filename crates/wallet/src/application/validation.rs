use rust_decimal::Decimal;
use types::pagination::PageRequest;

use super::{WalletError, WalletResult};

const MAX_PAGE_SIZE: u64 = constants::pagination::MAX_PAGE_SIZE;

pub fn validate_user_id(user_id: &str) -> WalletResult<()> {
    if user_id.trim().is_empty() {
        return Err(WalletError::InvalidInput("user_id is required".into()));
    }
    Ok(())
}

pub fn validate_wallet_id(wallet_id: &str) -> WalletResult<()> {
    if wallet_id.trim().is_empty() {
        return Err(WalletError::InvalidInput("wallet_id is required".into()));
    }
    Ok(())
}

pub fn validate_positive_amount(field: &str, amount: Decimal) -> WalletResult<()> {
    if amount <= Decimal::ZERO {
        return Err(WalletError::InvalidInput(format!("{field} must be greater than 0")));
    }
    Ok(())
}

pub fn validate_page(page: PageRequest) -> WalletResult<()> {
    if page.page == 0 {
        return Err(WalletError::InvalidInput("page must be greater than 0".into()));
    }
    if page.page_size == 0 || page.page_size > MAX_PAGE_SIZE {
        return Err(WalletError::InvalidInput(format!("page_size must be between 1 and {MAX_PAGE_SIZE}")));
    }
    Ok(())
}
