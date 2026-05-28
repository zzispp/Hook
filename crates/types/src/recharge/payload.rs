use rust_decimal::Decimal;
use serde::Deserialize;

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

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct PaymentChannelUpdatePayload {
    pub enabled: bool,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct RechargeOrderCreatePayload {
    #[serde(default)]
    pub package_id: Option<String>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub recharge_amount: Option<Decimal>,
    pub payment_channel_code: String,
    pub payment_method: String,
    #[serde(default)]
    pub captcha_token: Option<String>,
}
