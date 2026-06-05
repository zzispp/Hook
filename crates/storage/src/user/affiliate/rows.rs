use rust_decimal::Decimal;
use sea_orm::FromQueryResult;
use types::user::{AffiliateCommissionItem, AffiliateReferralItem, AffiliateReferredUserSummary, AffiliateSummaryResponse};

pub(super) fn summary_response(row: SummaryRow, affiliate_code: String) -> AffiliateSummaryResponse {
    AffiliateSummaryResponse {
        affiliate_link: format!("/auth/sign-up?aff={affiliate_code}"),
        affiliate_code,
        affiliate_enabled: row.affiliate_enabled.unwrap_or(false),
        referred_user_count: row.referred_user_count.unwrap_or_default() as u64,
        total_referred_recharge_amount: row.total_referred_recharge_amount.unwrap_or(Decimal::ZERO),
        total_commission_amount: row.total_commission_amount.unwrap_or(Decimal::ZERO),
        today_commission_amount: row.today_commission_amount.unwrap_or(Decimal::ZERO),
        month_commission_amount: row.month_commission_amount.unwrap_or(Decimal::ZERO),
        affiliate_commission_percent: row.affiliate_commission_percent.unwrap_or(Decimal::ZERO),
        last_commission_at: row.last_commission_at,
    }
}

pub(super) fn referral_item(row: ReferralRow) -> AffiliateReferralItem {
    AffiliateReferralItem {
        referred_user_id: row.referred_user_id,
        username: row.username,
        masked_email: mask_email(&row.email),
        referred_at: row.referred_at,
        referred_recharge_amount: row.referred_recharge_amount.unwrap_or(Decimal::ZERO),
        commission_amount: row.commission_amount.unwrap_or(Decimal::ZERO),
        last_commission_at: row.last_commission_at,
    }
}

pub(super) fn commission_item(row: CommissionRow) -> AffiliateCommissionItem {
    AffiliateCommissionItem {
        id: row.id,
        referred: AffiliateReferredUserSummary {
            referred_user_id: row.referred_user_id,
            username: row.referred_username,
            masked_email: mask_email(&row.referred_email),
        },
        recharge_order_no: row.recharge_order_no,
        payable_amount: row.payable_amount,
        commission_percent: row.commission_percent,
        commission_amount: row.commission_amount,
        wallet_transaction_id: row.wallet_transaction_id,
        status: row.status,
        failure_reason: row.failure_reason,
        created_at: row.created_at,
    }
}

fn mask_email(email: &str) -> String {
    let Some((local, domain)) = email.split_once('@') else {
        return mask_local(email);
    };
    format!("{}@{}", mask_local(local), domain)
}

fn mask_local(local: &str) -> String {
    let visible: String = local.chars().take(if local.chars().count() < 2 { 1 } else { 2 }).collect();
    format!("{visible}***")
}

#[derive(Debug, FromQueryResult)]
pub(super) struct CountRow {
    pub(super) total: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct SummaryRow {
    pub(super) referred_user_count: Option<i64>,
    pub(super) total_referred_recharge_amount: Option<Decimal>,
    pub(super) total_commission_amount: Option<Decimal>,
    pub(super) today_commission_amount: Option<Decimal>,
    pub(super) month_commission_amount: Option<Decimal>,
    pub(super) affiliate_enabled: Option<bool>,
    pub(super) affiliate_commission_percent: Option<Decimal>,
    pub(super) last_commission_at: Option<String>,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct ReferralRow {
    pub(super) referred_user_id: String,
    pub(super) username: String,
    pub(super) email: String,
    pub(super) referred_at: Option<String>,
    pub(super) referred_recharge_amount: Option<Decimal>,
    pub(super) commission_amount: Option<Decimal>,
    pub(super) last_commission_at: Option<String>,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct CommissionRow {
    pub(super) id: String,
    pub(super) referred_user_id: String,
    pub(super) referred_username: String,
    pub(super) referred_email: String,
    pub(super) recharge_order_no: String,
    pub(super) payable_amount: Decimal,
    pub(super) commission_percent: Decimal,
    pub(super) commission_amount: Decimal,
    pub(super) wallet_transaction_id: Option<String>,
    pub(super) status: String,
    pub(super) failure_reason: Option<String>,
    pub(super) created_at: String,
}
