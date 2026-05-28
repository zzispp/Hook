use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::PaymentResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentChannelRegistration {
    pub code: String,
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaymentCallbackEndpointKind {
    Notify,
    Return,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentCallbackEndpoint {
    pub kind: PaymentCallbackEndpointKind,
    pub methods: Vec<String>,
    pub path_pattern: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredPaymentCallbackEndpoint {
    pub channel_code: String,
    pub kind: PaymentCallbackEndpointKind,
    pub methods: Vec<String>,
    pub path_pattern: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentMethodOption {
    pub code: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentChannelConfigField {
    pub key: String,
    pub label: String,
    pub secret: bool,
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentChannelConfigSchema {
    pub fields: Vec<PaymentChannelConfigField>,
    pub methods: Vec<PaymentMethodOption>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentChannelConfig {
    pub config: Value,
    pub secret: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentOrderRequest {
    pub order_no: String,
    pub subject: String,
    pub amount: Decimal,
    pub payment_method: String,
    pub notify_url: String,
    pub return_url: String,
    pub config: PaymentChannelConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PaymentOrderAction {
    FormPost {
        action: String,
        method: String,
        fields: BTreeMap<String, String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentCallbackRequest {
    pub params: BTreeMap<String, String>,
    pub config: PaymentChannelConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedPaymentCallback {
    pub order_no: String,
    pub provider_trade_no: Option<String>,
    pub payment_method: String,
    pub amount: Option<Decimal>,
    pub trade_status: PaymentOrderStatus,
    pub raw_payload: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentOrderQueryResult {
    pub status: PaymentOrderStatus,
    pub provider_trade_no: Option<String>,
    pub payment_method: Option<String>,
    pub amount: Option<Decimal>,
    pub raw_payload: Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaymentOrderStatus {
    Pending,
    Paid,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentRefundRequest {
    pub order_no: String,
    pub amount: Decimal,
    pub config: PaymentChannelConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentRefundResult {
    pub provider_refund_no: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentUnsupportedReason {
    pub message: String,
}

/// Implements one payment provider protocol.
///
/// Implementations must validate their provider config, expose unsupported
/// capabilities with explicit errors, and never fabricate successful provider
/// responses.
#[async_trait]
pub trait PaymentChannelProvider: Send + Sync + 'static {
    fn registration(&self) -> PaymentChannelRegistration;
    fn config_schema(&self) -> PaymentChannelConfigSchema;
    fn callback_endpoints(&self) -> Vec<PaymentCallbackEndpoint>;
    async fn create_payment_order(&self, request: PaymentOrderRequest) -> PaymentResult<PaymentOrderAction>;
    async fn query_payment_order(&self, order_no: &str, config: PaymentChannelConfig) -> PaymentResult<PaymentOrderQueryResult>;
    async fn refund_payment_order(&self, request: PaymentRefundRequest) -> PaymentResult<PaymentRefundResult>;
    fn verify_callback(&self, request: PaymentCallbackRequest) -> PaymentResult<VerifiedPaymentCallback>;
}
