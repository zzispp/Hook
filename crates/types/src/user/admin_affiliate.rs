use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AdminAffiliateUserSummary {
    pub id: String,
    pub username: String,
    pub email: String,
    pub affiliate_code: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminAffiliateOverviewResponse {
    pub total_referred_users: u64,
    pub active_referrer_count: u64,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_commission_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub today_commission_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub month_commission_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub affiliate_commission_percent: Decimal,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct AdminAffiliateRelationQuery {
    pub page: u64,
    pub page_size: u64,
    #[serde(default)]
    pub user_search: Option<String>,
    #[serde(default)]
    pub referrer_search: Option<String>,
    #[serde(default)]
    pub has_referrer: Option<bool>,
    #[serde(default)]
    pub referred_start: Option<String>,
    #[serde(default)]
    pub referred_end: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminAffiliateRelationItem {
    pub user: AdminAffiliateUserSummary,
    pub referrer: Option<AdminAffiliateUserSummary>,
    pub referred_at: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub referred_recharge_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub commission_amount: Decimal,
    pub last_commission_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminAffiliateRelationListResponse {
    pub items: Vec<AdminAffiliateRelationItem>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct AdminAffiliateRelationChangeQuery {
    pub page: u64,
    pub page_size: u64,
    #[serde(default)]
    pub user_search: Option<String>,
    #[serde(default)]
    pub operator_search: Option<String>,
    #[serde(default)]
    pub start_at: Option<String>,
    #[serde(default)]
    pub end_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminAffiliateRelationChangeItem {
    pub id: String,
    pub user: AdminAffiliateUserSummary,
    pub old_referrer: Option<AdminAffiliateUserSummary>,
    pub new_referrer: Option<AdminAffiliateUserSummary>,
    pub operator: Option<AdminAffiliateUserSummary>,
    pub operator_user_id: Option<String>,
    pub reason: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminAffiliateRelationChangeListResponse {
    pub items: Vec<AdminAffiliateRelationChangeItem>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct AdminAffiliateRelationUpdateRequest {
    #[serde(default)]
    pub referrer_aff_code: Option<String>,
    #[serde(default)]
    pub clear_referrer: bool,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AdminAffiliateCommissionQuery {
    pub page: u64,
    pub page_size: u64,
    #[serde(default)]
    pub referrer_search: Option<String>,
    #[serde(default)]
    pub referred_search: Option<String>,
    #[serde(default)]
    pub recharge_order_id: Option<String>,
    #[serde(default)]
    pub start_at: Option<String>,
    #[serde(default)]
    pub end_at: Option<String>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub min_commission_amount: Option<Decimal>,
    #[serde(default, with = "rust_decimal::serde::float_option")]
    pub max_commission_amount: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminAffiliateCommissionItem {
    pub id: String,
    pub referrer: AdminAffiliateUserSummary,
    pub referred: AdminAffiliateUserSummary,
    pub recharge_order_id: String,
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
pub struct AdminAffiliateCommissionListResponse {
    pub items: Vec<AdminAffiliateCommissionItem>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct AdminAffiliateReportQuery {
    pub page: u64,
    pub page_size: u64,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub referrer_search: Option<String>,
    #[serde(default)]
    pub referred_search: Option<String>,
    #[serde(default)]
    pub export_type: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminAffiliateDailyReportItem {
    pub date: String,
    pub commission_order_count: u64,
    pub referred_payer_count: u64,
    #[serde(with = "rust_decimal::serde::float")]
    pub payable_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub commission_amount: Decimal,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminAffiliateReferrerReportItem {
    pub referrer: AdminAffiliateUserSummary,
    pub referred_user_count: u64,
    pub commission_order_count: u64,
    #[serde(with = "rust_decimal::serde::float")]
    pub payable_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub commission_amount: Decimal,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminAffiliateReportResponse {
    pub daily_items: Vec<AdminAffiliateDailyReportItem>,
    pub referrer_items: Vec<AdminAffiliateReferrerReportItem>,
    pub referrer_total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AffiliateRelationChangeRecord {
    pub id: String,
    pub user_id: String,
    pub old_referrer_user_id: Option<String>,
    pub new_referrer_user_id: Option<String>,
    pub operator_user_id: Option<String>,
    pub reason: String,
    pub created_at: String,
}
