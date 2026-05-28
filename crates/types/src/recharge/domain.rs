use rust_decimal::Decimal;

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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaymentCallbackListFilters {
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

#[derive(Clone, Debug, PartialEq)]
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
    pub payment_method: Option<String>,
    pub provider_trade_no: Option<String>,
    pub payment_request_json: Option<serde_json::Value>,
    pub refund_status: Option<String>,
    pub refund_amount: Option<Decimal>,
    pub paid_at: Option<String>,
    pub refunded_at: Option<String>,
    pub expires_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentChannel {
    pub code: String,
    pub name: String,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub secret_set: bool,
    pub config_schema: Option<serde_json::Value>,
    pub registered_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentCallbackRecord {
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
