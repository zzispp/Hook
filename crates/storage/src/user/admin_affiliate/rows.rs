use rust_decimal::Decimal;
use sea_orm::FromQueryResult;
use types::user::{
    AdminAffiliateCommissionItem, AdminAffiliateDailyReportItem, AdminAffiliateOverviewResponse, AdminAffiliateReferrerReportItem,
    AdminAffiliateRelationChangeItem, AdminAffiliateRelationItem, AdminAffiliateUserSummary,
};

pub(super) fn overview_response(row: OverviewRow) -> AdminAffiliateOverviewResponse {
    AdminAffiliateOverviewResponse {
        total_referred_users: row.total_referred_users.unwrap_or_default() as u64,
        active_referrer_count: row.active_referrer_count.unwrap_or_default() as u64,
        total_commission_amount: row.total_commission_amount.unwrap_or(Decimal::ZERO),
        today_commission_amount: row.today_commission_amount.unwrap_or(Decimal::ZERO),
        month_commission_amount: row.month_commission_amount.unwrap_or(Decimal::ZERO),
        affiliate_commission_percent: row.affiliate_commission_percent.unwrap_or(Decimal::ZERO),
    }
}

pub(super) fn relation_item(row: RelationRow) -> AdminAffiliateRelationItem {
    let referrer = relation_referrer_summary(&row);
    AdminAffiliateRelationItem {
        user: user_summary(row.user_id, row.username, row.email, row.affiliate_code),
        referrer,
        referred_at: row.referred_at,
        referred_recharge_amount: row.referred_recharge_amount.unwrap_or(Decimal::ZERO),
        commission_amount: row.commission_amount.unwrap_or(Decimal::ZERO),
        last_commission_at: row.last_commission_at,
    }
}

pub(super) fn relation_change_item(row: RelationChangeRow) -> AdminAffiliateRelationChangeItem {
    AdminAffiliateRelationChangeItem {
        id: row.id,
        user: user_summary(row.user_id, row.username, row.email, row.affiliate_code),
        old_referrer: optional_user_summary(
            row.old_referrer_id,
            row.old_referrer_username,
            row.old_referrer_email,
            row.old_referrer_affiliate_code,
        ),
        new_referrer: optional_user_summary(
            row.new_referrer_id,
            row.new_referrer_username,
            row.new_referrer_email,
            row.new_referrer_affiliate_code,
        ),
        operator: optional_user_summary(row.operator_id.clone(), row.operator_username, row.operator_email, row.operator_affiliate_code),
        operator_user_id: row.operator_id,
        reason: row.reason,
        created_at: row.created_at,
    }
}

pub(super) fn commission_item(row: CommissionRow) -> AdminAffiliateCommissionItem {
    AdminAffiliateCommissionItem {
        id: row.id,
        referrer: user_summary(row.referrer_id, row.referrer_username, row.referrer_email, row.referrer_affiliate_code),
        referred: user_summary(row.referred_id, row.referred_username, row.referred_email, row.referred_affiliate_code),
        recharge_order_id: row.recharge_order_id,
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

pub(super) fn daily_report_item(row: DailyReportRow) -> AdminAffiliateDailyReportItem {
    AdminAffiliateDailyReportItem {
        date: row.date,
        commission_order_count: row.commission_order_count.unwrap_or_default() as u64,
        referred_payer_count: row.referred_payer_count.unwrap_or_default() as u64,
        payable_amount: row.payable_amount.unwrap_or(Decimal::ZERO),
        commission_amount: row.commission_amount.unwrap_or(Decimal::ZERO),
    }
}

pub(super) fn referrer_report_item(row: ReferrerReportRow) -> AdminAffiliateReferrerReportItem {
    AdminAffiliateReferrerReportItem {
        referrer: user_summary(row.referrer_id, row.referrer_username, row.referrer_email, row.referrer_affiliate_code),
        referred_user_count: row.referred_user_count.unwrap_or_default() as u64,
        commission_order_count: row.commission_order_count.unwrap_or_default() as u64,
        payable_amount: row.payable_amount.unwrap_or(Decimal::ZERO),
        commission_amount: row.commission_amount.unwrap_or(Decimal::ZERO),
    }
}

fn relation_referrer_summary(row: &RelationRow) -> Option<AdminAffiliateUserSummary> {
    optional_user_summary(
        row.referrer_id.clone(),
        row.referrer_username.clone(),
        row.referrer_email.clone(),
        row.referrer_affiliate_code.clone(),
    )
}

fn optional_user_summary(
    id: Option<String>,
    username: Option<String>,
    email: Option<String>,
    affiliate_code: Option<String>,
) -> Option<AdminAffiliateUserSummary> {
    Some(user_summary(
        id?,
        username.unwrap_or_default(),
        email.unwrap_or_default(),
        affiliate_code.unwrap_or_default(),
    ))
}

fn user_summary(id: String, username: String, email: String, affiliate_code: String) -> AdminAffiliateUserSummary {
    AdminAffiliateUserSummary {
        id,
        username,
        email,
        affiliate_code,
    }
}

#[derive(Debug, FromQueryResult)]
pub(super) struct CountRow {
    pub(super) total: Option<i64>,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct OverviewRow {
    pub(super) total_referred_users: Option<i64>,
    pub(super) active_referrer_count: Option<i64>,
    pub(super) total_commission_amount: Option<Decimal>,
    pub(super) today_commission_amount: Option<Decimal>,
    pub(super) month_commission_amount: Option<Decimal>,
    pub(super) affiliate_commission_percent: Option<Decimal>,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct RelationRow {
    pub(super) user_id: String,
    pub(super) username: String,
    pub(super) email: String,
    pub(super) affiliate_code: String,
    pub(super) referrer_id: Option<String>,
    pub(super) referrer_username: Option<String>,
    pub(super) referrer_email: Option<String>,
    pub(super) referrer_affiliate_code: Option<String>,
    pub(super) referred_at: Option<String>,
    pub(super) referred_recharge_amount: Option<Decimal>,
    pub(super) commission_amount: Option<Decimal>,
    pub(super) last_commission_at: Option<String>,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct RelationChangeRow {
    pub(super) id: String,
    pub(super) user_id: String,
    pub(super) username: String,
    pub(super) email: String,
    pub(super) affiliate_code: String,
    pub(super) old_referrer_id: Option<String>,
    pub(super) old_referrer_username: Option<String>,
    pub(super) old_referrer_email: Option<String>,
    pub(super) old_referrer_affiliate_code: Option<String>,
    pub(super) new_referrer_id: Option<String>,
    pub(super) new_referrer_username: Option<String>,
    pub(super) new_referrer_email: Option<String>,
    pub(super) new_referrer_affiliate_code: Option<String>,
    pub(super) operator_id: Option<String>,
    pub(super) operator_username: Option<String>,
    pub(super) operator_email: Option<String>,
    pub(super) operator_affiliate_code: Option<String>,
    pub(super) reason: String,
    pub(super) created_at: String,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct CommissionRow {
    pub(super) id: String,
    pub(super) referrer_id: String,
    pub(super) referrer_username: String,
    pub(super) referrer_email: String,
    pub(super) referrer_affiliate_code: String,
    pub(super) referred_id: String,
    pub(super) referred_username: String,
    pub(super) referred_email: String,
    pub(super) referred_affiliate_code: String,
    pub(super) recharge_order_id: String,
    pub(super) recharge_order_no: String,
    pub(super) payable_amount: Decimal,
    pub(super) commission_percent: Decimal,
    pub(super) commission_amount: Decimal,
    pub(super) wallet_transaction_id: Option<String>,
    pub(super) status: String,
    pub(super) failure_reason: Option<String>,
    pub(super) created_at: String,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct DailyReportRow {
    pub(super) date: String,
    pub(super) commission_order_count: Option<i64>,
    pub(super) referred_payer_count: Option<i64>,
    pub(super) payable_amount: Option<Decimal>,
    pub(super) commission_amount: Option<Decimal>,
}

#[derive(Debug, FromQueryResult)]
pub(super) struct ReferrerReportRow {
    pub(super) referrer_id: String,
    pub(super) referrer_username: String,
    pub(super) referrer_email: String,
    pub(super) referrer_affiliate_code: String,
    pub(super) referred_user_count: Option<i64>,
    pub(super) commission_order_count: Option<i64>,
    pub(super) payable_amount: Option<Decimal>,
    pub(super) commission_amount: Option<Decimal>,
}
