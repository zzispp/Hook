use rust_decimal::Decimal;
use serde::Serialize;

use crate::pagination::Page;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RechargeOrderSummary {
    pub total_payable_amount: Decimal,
    pub order_count: u64,
    pub user_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RechargeOrderUserSummary {
    pub user_id: String,
    pub username: String,
    pub user_email: String,
    pub order_count: u64,
    pub total_payable_amount: Decimal,
    pub last_paid_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RechargeOrderSummaryPage {
    pub summary: RechargeOrderSummary,
    pub users: Page<RechargeOrderUserSummary>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RechargeOrderSummaryResponse {
    pub summary: RechargeOrderSummaryResponseTotals,
    pub items: Vec<RechargeOrderUserSummaryResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RechargeOrderSummaryResponseTotals {
    #[serde(with = "rust_decimal::serde::float")]
    pub total_payable_amount: Decimal,
    pub order_count: u64,
    pub user_count: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RechargeOrderUserSummaryResponse {
    pub user_id: String,
    pub username: String,
    pub user_email: String,
    pub order_count: u64,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_payable_amount: Decimal,
    pub last_paid_at: Option<String>,
}

impl From<RechargeOrderSummaryPage> for RechargeOrderSummaryResponse {
    fn from(value: RechargeOrderSummaryPage) -> Self {
        Self {
            summary: value.summary.into(),
            items: value.users.items.into_iter().map(Into::into).collect(),
            total: value.users.total,
            page: value.users.page,
            page_size: value.users.page_size,
        }
    }
}

impl From<RechargeOrderSummary> for RechargeOrderSummaryResponseTotals {
    fn from(value: RechargeOrderSummary) -> Self {
        Self {
            total_payable_amount: value.total_payable_amount,
            order_count: value.order_count,
            user_count: value.user_count,
        }
    }
}

impl From<RechargeOrderUserSummary> for RechargeOrderUserSummaryResponse {
    fn from(value: RechargeOrderUserSummary) -> Self {
        Self {
            user_id: value.user_id,
            username: value.username,
            user_email: value.user_email,
            order_count: value.order_count,
            total_payable_amount: value.total_payable_amount,
            last_paid_at: value.last_paid_at,
        }
    }
}
