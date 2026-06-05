use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AffiliateSummaryResponse {
    pub affiliate_code: String,
    pub affiliate_link: String,
    pub affiliate_enabled: bool,
    pub referred_user_count: u64,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_referred_recharge_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_commission_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub today_commission_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub month_commission_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub affiliate_commission_percent: Decimal,
    pub last_commission_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct AffiliateReferralQuery {
    pub page: u64,
    pub page_size: u64,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub referred_start: Option<String>,
    #[serde(default)]
    pub referred_end: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AffiliateReferralItem {
    pub referred_user_id: String,
    pub username: String,
    pub masked_email: String,
    pub referred_at: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub referred_recharge_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub commission_amount: Decimal,
    pub last_commission_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AffiliateReferralListResponse {
    pub items: Vec<AffiliateReferralItem>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AffiliateCommissionQuery {
    pub page: u64,
    pub page_size: u64,
    #[serde(default)]
    pub referred_search: Option<String>,
    #[serde(default)]
    pub recharge_order_no: Option<String>,
    #[serde(default)]
    pub start_at: Option<String>,
    #[serde(default)]
    pub end_at: Option<String>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub min_commission_amount: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub max_commission_amount: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AffiliateReferredUserSummary {
    pub referred_user_id: String,
    pub username: String,
    pub masked_email: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AffiliateCommissionItem {
    pub id: String,
    pub referred: AffiliateReferredUserSummary,
    pub recharge_order_no: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub payable_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub commission_percent: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub commission_amount: Decimal,
    pub wallet_transaction_id: Option<String>,
    pub status: String,
    pub failure_reason: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AffiliateCommissionListResponse {
    pub items: Vec<AffiliateCommissionItem>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}
