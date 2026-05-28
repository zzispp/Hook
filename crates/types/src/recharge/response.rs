use rust_decimal::Decimal;
use serde::Serialize;

use crate::pagination::Page;

use super::{PaymentCallbackRecord, PaymentChannel, RechargeOrder, RechargePackage};

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
    pub payment_method: Option<String>,
    pub provider_trade_no: Option<String>,
    pub refund_status: Option<String>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub refund_amount: Option<Decimal>,
    pub paid_at: Option<String>,
    pub refunded_at: Option<String>,
    pub expires_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PaymentChannelResponse {
    pub code: String,
    pub name: String,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub secret_set: bool,
    pub config_schema: Option<serde_json::Value>,
    pub registered_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PublicPaymentChannelResponse {
    pub code: String,
    pub name: String,
    pub methods: Vec<payment::PaymentMethodOption>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PaymentCallbackRecordResponse {
    pub id: String,
    pub payment_channel_code: String,
    pub callback_kind: String,
    pub http_method: String,
    pub order_no: Option<String>,
    pub provider_trade_no: Option<String>,
    pub payment_method: Option<String>,
    pub trade_status: Option<String>,
    pub status: String,
    pub settled: bool,
    pub error_message: Option<String>,
    pub raw_params: serde_json::Value,
    pub received_at: String,
    pub processed_at: Option<String>,
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

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PaymentCallbackRecordListResponse {
    pub items: Vec<PaymentCallbackRecordResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RechargeOrderCreateResponse {
    pub order: RechargeOrderResponse,
    pub payment: payment::PaymentOrderAction,
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
            payment_method: value.payment_method,
            provider_trade_no: value.provider_trade_no,
            refund_status: value.refund_status,
            refund_amount: value.refund_amount,
            paid_at: value.paid_at,
            refunded_at: value.refunded_at,
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
            config: value.config,
            secret_set: value.secret_set,
            config_schema: value.config_schema,
            registered_at: value.registered_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<PaymentCallbackRecord> for PaymentCallbackRecordResponse {
    fn from(value: PaymentCallbackRecord) -> Self {
        Self {
            id: value.id,
            payment_channel_code: value.payment_channel_code,
            callback_kind: value.callback_kind,
            http_method: value.http_method,
            order_no: value.order_no,
            provider_trade_no: value.provider_trade_no,
            payment_method: value.payment_method,
            trade_status: value.trade_status,
            status: value.status,
            settled: value.settled,
            error_message: value.error_message,
            raw_params: value.raw_params,
            received_at: value.received_at,
            processed_at: value.processed_at,
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

impl From<Page<PaymentCallbackRecord>> for PaymentCallbackRecordListResponse {
    fn from(value: Page<PaymentCallbackRecord>) -> Self {
        Self {
            items: value.items.into_iter().map(Into::into).collect(),
            total: value.total,
            page: value.page,
            page_size: value.page_size,
        }
    }
}
