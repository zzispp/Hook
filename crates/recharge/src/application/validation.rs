use rust_decimal::Decimal;
use types::{
    pagination::PageRequest,
    recharge::{
        PAYMENT_CALLBACK_STATUS_FAILED, PAYMENT_CALLBACK_STATUS_IGNORED, PAYMENT_CALLBACK_STATUS_PROCESSED, PAYMENT_CALLBACK_STATUS_RECEIVED,
        PaymentCallbackListFilters, RECHARGE_ORDER_STATUS_CANCELLED, RECHARGE_ORDER_STATUS_EXPIRED, RECHARGE_ORDER_STATUS_FAILED, RECHARGE_ORDER_STATUS_PAID,
        RECHARGE_ORDER_STATUS_PENDING, RECHARGE_PACKAGE_STATUS_ACTIVE, RECHARGE_PACKAGE_STATUS_DISABLED, RechargeOrderListFilters,
        RechargePackageCreatePayload, RechargePackageListFilters, RechargePackageUpdatePayload,
    },
};

use super::{RechargeError, RechargeResult};

const MAX_NAME_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 20_000;
const MAX_PAGE_SIZE: u64 = constants::pagination::MAX_PAGE_SIZE;

pub fn sanitize_create(input: RechargePackageCreatePayload) -> RechargePackageCreatePayload {
    RechargePackageCreatePayload {
        name: input.name.trim().to_owned(),
        description: trim_optional(input.description),
        status: input.status.map(|value| value.trim().to_owned()),
        ..input
    }
}

pub fn sanitize_update(input: RechargePackageUpdatePayload) -> RechargePackageUpdatePayload {
    RechargePackageUpdatePayload {
        name: input.name.trim().to_owned(),
        description: trim_optional(input.description),
        status: input.status.trim().to_owned(),
        ..input
    }
}

pub fn validate_create(input: &RechargePackageCreatePayload) -> RechargeResult<()> {
    validate_name(&input.name)?;
    validate_description(input.description.as_deref())?;
    validate_recharge_amount(input.recharge_amount)?;
    validate_gift_amount(input.gift_amount)?;
    validate_package_status(input.status.as_deref().unwrap_or(RECHARGE_PACKAGE_STATUS_ACTIVE))
}

pub fn validate_update(input: &RechargePackageUpdatePayload) -> RechargeResult<()> {
    validate_name(&input.name)?;
    validate_description(input.description.as_deref())?;
    validate_recharge_amount(input.recharge_amount)?;
    validate_gift_amount(input.gift_amount)?;
    validate_package_status(&input.status)
}

pub fn validate_package_filters(filters: &RechargePackageListFilters) -> RechargeResult<()> {
    validate_optional_package_status(filters.status.as_deref())
}

pub fn validate_order_filters(filters: &RechargeOrderListFilters) -> RechargeResult<()> {
    validate_optional_order_status(filters.status.as_deref())
}

pub fn validate_callback_filters(filters: &PaymentCallbackListFilters) -> RechargeResult<()> {
    validate_optional_callback_status(filters.status.as_deref())
}

pub fn validate_page(page: PageRequest) -> RechargeResult<()> {
    if page.page == 0 {
        return Err(RechargeError::InvalidInput("page must be greater than 0".into()));
    }
    if page.page_size == 0 || page.page_size > MAX_PAGE_SIZE {
        return Err(RechargeError::InvalidInput(format!("page_size must be between 1 and {MAX_PAGE_SIZE}")));
    }
    Ok(())
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned()).filter(|item| !item.is_empty())
}

fn validate_name(value: &str) -> RechargeResult<()> {
    if value.is_empty() || value.len() > MAX_NAME_LENGTH {
        return Err(RechargeError::InvalidInput(format!("name length must be between 1 and {MAX_NAME_LENGTH}")));
    }
    Ok(())
}

fn validate_description(value: Option<&str>) -> RechargeResult<()> {
    if value.is_some_and(|item| item.len() > MAX_DESCRIPTION_LENGTH) {
        return Err(RechargeError::InvalidInput(format!(
            "description length must be at most {MAX_DESCRIPTION_LENGTH}"
        )));
    }
    Ok(())
}

fn validate_recharge_amount(value: Decimal) -> RechargeResult<()> {
    if value <= Decimal::ZERO {
        return Err(RechargeError::InvalidInput("recharge_amount must be greater than 0".into()));
    }
    Ok(())
}

fn validate_gift_amount(value: Decimal) -> RechargeResult<()> {
    if value < Decimal::ZERO {
        return Err(RechargeError::InvalidInput("gift_amount must be greater than or equal to 0".into()));
    }
    Ok(())
}

fn validate_optional_package_status(value: Option<&str>) -> RechargeResult<()> {
    match value.filter(|item| !item.is_empty()) {
        Some(status) => validate_package_status(status),
        None => Ok(()),
    }
}

fn validate_package_status(value: &str) -> RechargeResult<()> {
    if matches!(value, RECHARGE_PACKAGE_STATUS_ACTIVE | RECHARGE_PACKAGE_STATUS_DISABLED) {
        return Ok(());
    }
    Err(RechargeError::InvalidInput("status must be active or disabled".into()))
}

fn validate_optional_order_status(value: Option<&str>) -> RechargeResult<()> {
    match value.filter(|item| !item.is_empty()) {
        Some(status) => validate_order_status(status),
        None => Ok(()),
    }
}

fn validate_order_status(value: &str) -> RechargeResult<()> {
    if matches!(
        value,
        RECHARGE_ORDER_STATUS_PENDING
            | RECHARGE_ORDER_STATUS_EXPIRED
            | RECHARGE_ORDER_STATUS_PAID
            | RECHARGE_ORDER_STATUS_CANCELLED
            | RECHARGE_ORDER_STATUS_FAILED
    ) {
        return Ok(());
    }
    Err(RechargeError::InvalidInput("order status is unsupported".into()))
}

fn validate_optional_callback_status(value: Option<&str>) -> RechargeResult<()> {
    match value.filter(|item| !item.is_empty()) {
        Some(status) => validate_callback_status(status),
        None => Ok(()),
    }
}

fn validate_callback_status(value: &str) -> RechargeResult<()> {
    if matches!(
        value,
        PAYMENT_CALLBACK_STATUS_RECEIVED | PAYMENT_CALLBACK_STATUS_PROCESSED | PAYMENT_CALLBACK_STATUS_IGNORED | PAYMENT_CALLBACK_STATUS_FAILED
    ) {
        return Ok(());
    }
    Err(RechargeError::InvalidInput("payment callback status is unsupported".into()))
}
