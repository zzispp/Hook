use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RechargePackageRecordInput {
    pub name: String,
    pub description: Option<String>,
    pub recharge_amount: Decimal,
    pub gift_amount: Decimal,
    pub status: String,
    pub sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RechargePackageRecordPatch {
    pub name: String,
    pub description: Option<String>,
    pub recharge_amount: Decimal,
    pub gift_amount: Decimal,
    pub status: String,
    pub sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RechargeOrderRecordInput {
    pub order_no: String,
    pub user_id: String,
    pub package_id: Option<String>,
    pub package_name: String,
    pub recharge_amount: Decimal,
    pub gift_amount: Decimal,
    pub total_arrival_amount: Decimal,
    pub payable_amount: Decimal,
    pub status: String,
    pub payment_channel_code: Option<String>,
    pub payment_channel_name: Option<String>,
    pub payment_method: Option<String>,
    pub payment_request_json: Option<serde_json::Value>,
    pub expires_at: time::OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentChannelDefinition {
    pub code: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentChannelRecordPatch {
    pub enabled: bool,
    pub config: Option<serde_json::Value>,
    pub encrypted_secret: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentCallbackRecordInput {
    pub payment_channel_code: String,
    pub callback_kind: String,
    pub http_method: String,
    pub raw_params: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentCallbackRecordPatch {
    pub order_no: Option<String>,
    pub provider_trade_no: Option<String>,
    pub payment_method: Option<String>,
    pub trade_status: Option<String>,
    pub status: String,
    pub settled: bool,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RechargePaymentSettlementInput {
    pub order_no: String,
    pub payment_channel_code: String,
    pub provider_trade_no: Option<String>,
    pub payment_method: String,
    pub payable_amount: Option<Decimal>,
    pub callback_payload: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RechargePaymentSettlementRecord {
    pub order: types::recharge::RechargeOrder,
    pub settled: bool,
}
