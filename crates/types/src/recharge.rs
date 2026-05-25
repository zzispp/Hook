use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::pagination::Page;

pub const RECHARGE_PACKAGE_STATUS_ACTIVE: &str = "active";
pub const RECHARGE_PACKAGE_STATUS_DISABLED: &str = "disabled";
pub const RECHARGE_ORDER_STATUS_PENDING: &str = "pending";
pub const RECHARGE_ORDER_STATUS_EXPIRED: &str = "expired";
pub const RECHARGE_ORDER_STATUS_PAID: &str = "paid";
pub const RECHARGE_ORDER_STATUS_CANCELLED: &str = "cancelled";
pub const RECHARGE_ORDER_STATUS_FAILED: &str = "failed";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RechargePackageListFilters {
    pub search: Option<String>,
    pub status: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RechargeOrderListFilters {
    pub search: Option<String>,
    pub status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RechargePackage {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub recharge_amount: Decimal,
    pub gift_amount: Decimal,
    pub status: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RechargeOrder {
    pub id: String,
    pub order_no: String,
    pub user_id: String,
    pub username: String,
    pub user_email: String,
    pub package_id: Option<String>,
    pub package_name: String,
    pub recharge_amount: Decimal,
    pub gift_amount: Decimal,
    pub total_arrival_amount: Decimal,
    pub payable_amount: Decimal,
    pub status: String,
    pub payment_channel_code: Option<String>,
    pub payment_channel_name: Option<String>,
    pub expires_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentChannel {
    pub code: String,
    pub name: String,
    pub enabled: bool,
    pub registered_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct RechargePackageCreatePayload {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub recharge_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub gift_amount: Decimal,
    #[serde(default)]
    pub status: Option<String>,
    pub sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct RechargePackageUpdatePayload {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub recharge_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub gift_amount: Decimal,
    pub status: String,
    pub sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct PaymentChannelUpdatePayload {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct RechargeOrderCreatePayload {
    pub package_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RechargePackageResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub recharge_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub gift_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_arrival_amount: Decimal,
    pub status: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RechargeOrderResponse {
    pub id: String,
    pub order_no: String,
    pub user_id: String,
    pub username: String,
    pub user_email: String,
    pub package_id: Option<String>,
    pub package_name: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub recharge_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub gift_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_arrival_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub payable_amount: Decimal,
    pub status: String,
    pub payment_channel_code: Option<String>,
    pub payment_channel_name: Option<String>,
    pub expires_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PaymentChannelResponse {
    pub code: String,
    pub name: String,
    pub enabled: bool,
    pub registered_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RechargePackageListResponse {
    pub items: Vec<RechargePackageResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct UserRechargePackageResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub recharge_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub gift_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_arrival_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub estimated_payable_amount: Decimal,
    pub sort_order: i64,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct UserRechargePackageListResponse {
    pub recharge_enabled: bool,
    #[serde(with = "rust_decimal::serde::float")]
    pub arrival_ratio: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub min_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub max_amount: Decimal,
    pub items: Vec<UserRechargePackageResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RechargeOrderListResponse {
    pub items: Vec<RechargeOrderResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

impl From<RechargePackage> for RechargePackageResponse {
    fn from(value: RechargePackage) -> Self {
        let total_arrival_amount = value.recharge_amount + value.gift_amount;
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            recharge_amount: value.recharge_amount,
            gift_amount: value.gift_amount,
            total_arrival_amount,
            status: value.status,
            sort_order: value.sort_order,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<RechargeOrder> for RechargeOrderResponse {
    fn from(value: RechargeOrder) -> Self {
        Self {
            id: value.id,
            order_no: value.order_no,
            user_id: value.user_id,
            username: value.username,
            user_email: value.user_email,
            package_id: value.package_id,
            package_name: value.package_name,
            recharge_amount: value.recharge_amount,
            gift_amount: value.gift_amount,
            total_arrival_amount: value.total_arrival_amount,
            payable_amount: value.payable_amount,
            status: value.status,
            payment_channel_code: value.payment_channel_code,
            payment_channel_name: value.payment_channel_name,
            expires_at: value.expires_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<PaymentChannel> for PaymentChannelResponse {
    fn from(value: PaymentChannel) -> Self {
        Self {
            code: value.code,
            name: value.name,
            enabled: value.enabled,
            registered_at: value.registered_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<Page<RechargePackage>> for RechargePackageListResponse {
    fn from(value: Page<RechargePackage>) -> Self {
        Self {
            items: value.items.into_iter().map(Into::into).collect(),
            total: value.total,
            page: value.page,
            page_size: value.page_size,
        }
    }
}

impl From<Page<RechargeOrder>> for RechargeOrderListResponse {
    fn from(value: Page<RechargeOrder>) -> Self {
        Self {
            items: value.items.into_iter().map(Into::into).collect(),
            total: value.total,
            page: value.page,
            page_size: value.page_size,
        }
    }
}
