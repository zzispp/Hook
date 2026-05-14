use rust_decimal::Decimal;
use types::{
    card_code::{
        CARD_CODE_BALANCE_TYPE_GIFT, CARD_CODE_BALANCE_TYPE_RECHARGE, CARD_CODE_STATUS_ACTIVE,
        CARD_CODE_STATUS_DISABLED, CardCodeBatchStatusPayload, CardCodeGeneratePayload,
        CardCodeTypeCreatePayload, CardCodeTypeUpdatePayload,
    },
    pagination::PageRequest,
};

use super::{CardCodeError, CardCodeResult};

const MAX_PAGE_SIZE: u64 = constants::pagination::MAX_PAGE_SIZE;
const MIN_CODE_LENGTH: u8 = 8;
const MAX_CODE_LENGTH: u8 = 64;
const MAX_GENERATE_QUANTITY: u64 = 200;

pub fn validate_page(page: PageRequest) -> CardCodeResult<()> {
    if page.page == 0 {
        return Err(CardCodeError::InvalidInput("page must be greater than 0".into()));
    }
    if page.page_size == 0 || page.page_size > MAX_PAGE_SIZE {
        return Err(CardCodeError::InvalidInput(format!("page_size must be between 1 and {MAX_PAGE_SIZE}")));
    }
    Ok(())
}

pub fn validate_type_create(input: &CardCodeTypeCreatePayload) -> CardCodeResult<()> {
    validate_name(&input.name)?;
    validate_balance_type(&input.balance_type)?;
    validate_status(input.status.as_deref().unwrap_or(CARD_CODE_STATUS_ACTIVE))?;
    Ok(())
}

pub fn validate_type_update(input: &CardCodeTypeUpdatePayload) -> CardCodeResult<()> {
    validate_name(&input.name)?;
    validate_balance_type(&input.balance_type)?;
    validate_status(&input.status)?;
    Ok(())
}

pub fn validate_generate(input: &CardCodeGeneratePayload) -> CardCodeResult<()> {
    if input.type_id.trim().is_empty() {
        return Err(CardCodeError::InvalidInput("type_id is required".into()));
    }
    if input.quantity == 0 || input.quantity > MAX_GENERATE_QUANTITY {
        return Err(CardCodeError::InvalidInput(format!("quantity must be between 1 and {MAX_GENERATE_QUANTITY}")));
    }
    validate_code_length(input.code_length)?;
    validate_amount(input.amount)?;
    validate_status(input.status.as_deref().unwrap_or(CARD_CODE_STATUS_ACTIVE))?;
    validate_optional_expiration(input.expires_at.as_deref())
}

pub fn validate_batch_status(input: &CardCodeBatchStatusPayload) -> CardCodeResult<()> {
    if input.ids.is_empty() {
        return Err(CardCodeError::InvalidInput("ids is required".into()));
    }
    validate_status(&input.status)
}

fn validate_amount(amount: Decimal) -> CardCodeResult<()> {
    if amount <= Decimal::ZERO {
        return Err(CardCodeError::InvalidInput("amount must be greater than 0".into()));
    }
    Ok(())
}

pub fn validate_status(status: &str) -> CardCodeResult<()> {
    if matches!(status, CARD_CODE_STATUS_ACTIVE | CARD_CODE_STATUS_DISABLED) {
        return Ok(());
    }
    Err(CardCodeError::InvalidInput("status must be active or disabled".into()))
}

fn validate_name(name: &str) -> CardCodeResult<()> {
    if name.trim().is_empty() {
        return Err(CardCodeError::InvalidInput("name is required".into()));
    }
    Ok(())
}

fn validate_balance_type(balance_type: &str) -> CardCodeResult<()> {
    if matches!(balance_type, CARD_CODE_BALANCE_TYPE_RECHARGE | CARD_CODE_BALANCE_TYPE_GIFT) {
        return Ok(());
    }
    Err(CardCodeError::InvalidInput("balance_type must be recharge or gift".into()))
}

fn validate_code_length(length: u8) -> CardCodeResult<()> {
    if !(MIN_CODE_LENGTH..=MAX_CODE_LENGTH).contains(&length) {
        return Err(CardCodeError::InvalidInput(format!("code_length must be between {MIN_CODE_LENGTH} and {MAX_CODE_LENGTH}")));
    }
    Ok(())
}

fn validate_optional_expiration(value: Option<&str>) -> CardCodeResult<()> {
    let Some(raw) = value.filter(|item| !item.trim().is_empty()) else {
        return Ok(());
    };
    let expires_at = time::OffsetDateTime::parse(raw, &time::format_description::well_known::Rfc3339)
        .map_err(|_| CardCodeError::InvalidInput("expires_at must be RFC3339".into()))?;
    if expires_at <= time::OffsetDateTime::now_utc() {
        return Err(CardCodeError::InvalidInput("expires_at must be in the future".into()));
    }
    Ok(())
}
