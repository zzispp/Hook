use sea_orm::{DbBackend, FromQueryResult, Statement, Value};
use types::{
    pagination::{Page, PageSliceRequest},
    user::{AffiliateCommissionItem, AffiliateCommissionQuery, AffiliateReferralItem, AffiliateReferralQuery, AffiliateSummaryResponse},
};

use crate::{StorageError, StorageResult};

use super::{
    filter::AffiliateSqlFilter,
    rows::{CommissionRow, CountRow, ReferralRow, SummaryRow, commission_item, referral_item, summary_response},
};

pub(super) async fn summary(store: &super::super::UserStore, user_id: &str, affiliate_code: &str) -> StorageResult<AffiliateSummaryResponse> {
    let sql = summary_sql();
    let row = SummaryRow::find_by_statement(statement(sql, vec![user_id.to_owned().into()]))
        .one(store.database.connection())
        .await?
        .ok_or_else(|| StorageError::Database("affiliate summary returned no rows".into()))?;
    Ok(summary_response(row, affiliate_code.to_owned()))
}

pub(super) async fn referrals(
    store: &super::super::UserStore,
    user_id: &str,
    request: PageSliceRequest,
    query: AffiliateReferralQuery,
) -> StorageResult<Page<AffiliateReferralItem>> {
    let filter = AffiliateSqlFilter::referrals(user_id, &query);
    let total = referral_total(store, &filter).await?;
    let rows = referral_rows(store, request, filter).await?;
    Ok(Page {
        items: rows.into_iter().map(referral_item).collect(),
        total,
        page: request.page,
        page_size: request.page_size,
    })
}

pub(super) async fn commissions(
    store: &super::super::UserStore,
    user_id: &str,
    request: PageSliceRequest,
    query: AffiliateCommissionQuery,
) -> StorageResult<Page<AffiliateCommissionItem>> {
    let filter = AffiliateSqlFilter::commissions(user_id, &query);
    let total = commission_total(store, &filter).await?;
    let rows = commission_rows(store, request, filter).await?;
    Ok(Page {
        items: rows.into_iter().map(commission_item).collect(),
        total,
        page: request.page,
        page_size: request.page_size,
    })
}

pub(super) async fn export_commissions(
    store: &super::super::UserStore,
    user_id: &str,
    query: AffiliateCommissionQuery,
) -> StorageResult<Vec<AffiliateCommissionItem>> {
    let filter = AffiliateSqlFilter::commissions(user_id, &query);
    let sql = format!(
        "{} {} {} ORDER BY c.created_at DESC",
        commission_select_sql(),
        commission_join_sql(),
        filter.where_sql(commission_owner_rule(filter.owner_placeholder()))
    );
    Ok(query_rows::<CommissionRow>(store, sql, filter.into_values())
        .await?
        .into_iter()
        .map(commission_item)
        .collect())
}

async fn referral_total(store: &super::super::UserStore, filter: &AffiliateSqlFilter) -> StorageResult<u64> {
    let sql = format!(
        "SELECT COUNT(*)::bigint AS total FROM users u {}",
        filter.where_sql(referral_owner_rule(filter.owner_placeholder()))
    );
    count_query(store, sql, filter.values()).await
}

async fn referral_rows(store: &super::super::UserStore, request: PageSliceRequest, mut filter: AffiliateSqlFilter) -> StorageResult<Vec<ReferralRow>> {
    let limit = filter.push((request.limit as i64).into());
    let offset = filter.push((request.offset as i64).into());
    let sql = format!(
        "{} {} {} GROUP BY u.id, u.username, u.email, u.referred_at ORDER BY u.referred_at DESC LIMIT {limit} OFFSET {offset}",
        referral_select_sql(),
        referral_commission_join_sql(),
        filter.where_sql(referral_owner_rule(filter.owner_placeholder()))
    );
    query_rows(store, sql, filter.into_values()).await
}

async fn commission_total(store: &super::super::UserStore, filter: &AffiliateSqlFilter) -> StorageResult<u64> {
    let sql = format!(
        "SELECT COUNT(*)::bigint AS total FROM affiliate_commissions c {} {}",
        commission_join_sql(),
        filter.where_sql(commission_owner_rule(filter.owner_placeholder()))
    );
    count_query(store, sql, filter.values()).await
}

async fn commission_rows(store: &super::super::UserStore, request: PageSliceRequest, mut filter: AffiliateSqlFilter) -> StorageResult<Vec<CommissionRow>> {
    let limit = filter.push((request.limit as i64).into());
    let offset = filter.push((request.offset as i64).into());
    let sql = format!(
        "{} {} {} ORDER BY c.created_at DESC LIMIT {limit} OFFSET {offset}",
        commission_select_sql(),
        commission_join_sql(),
        filter.where_sql(commission_owner_rule(filter.owner_placeholder()))
    );
    query_rows(store, sql, filter.into_values()).await
}

async fn count_query(store: &super::super::UserStore, sql: String, values: Vec<Value>) -> StorageResult<u64> {
    CountRow::find_by_statement(statement(sql, values))
        .one(store.database.connection())
        .await?
        .map(|row| row.total.unwrap_or_default() as u64)
        .ok_or_else(|| StorageError::Database("affiliate count query returned no rows".into()))
}

async fn query_rows<T>(store: &super::super::UserStore, sql: String, values: Vec<Value>) -> StorageResult<Vec<T>>
where
    T: FromQueryResult + Send + Sync + 'static,
{
    T::find_by_statement(statement(sql, values))
        .all(store.database.connection())
        .await
        .map_err(Into::into)
}

fn statement(sql: String, values: Vec<Value>) -> Statement {
    Statement::from_sql_and_values(DbBackend::Postgres, sql, values)
}

fn summary_sql() -> String {
    "SELECT \
    (SELECT COUNT(*)::bigint FROM users WHERE is_deleted = FALSE AND referred_by_user_id = $1) AS referred_user_count, \
    COALESCE((SELECT SUM(payable_amount) FROM affiliate_commissions WHERE referrer_user_id = $1), 0) AS total_referred_recharge_amount, \
    COALESCE((SELECT SUM(commission_amount) FROM affiliate_commissions WHERE referrer_user_id = $1 AND status = 'success'), 0) AS total_commission_amount, \
    COALESCE((SELECT SUM(commission_amount) FROM affiliate_commissions WHERE referrer_user_id = $1 AND status = 'success' AND created_at >= date_trunc('day', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'), 0) AS today_commission_amount, \
    COALESCE((SELECT SUM(commission_amount) FROM affiliate_commissions WHERE referrer_user_id = $1 AND status = 'success' AND created_at >= date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'), 0) AS month_commission_amount, \
    COALESCE((SELECT affiliate_enabled FROM system_settings WHERE id = 'global'), FALSE) AS affiliate_enabled, \
    COALESCE((SELECT affiliate_commission_percent FROM system_settings WHERE id = 'global'), 0) AS affiliate_commission_percent, \
    (SELECT to_char(MAX(created_at), 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') FROM affiliate_commissions WHERE referrer_user_id = $1 AND status = 'success') AS last_commission_at"
        .into()
}

fn referral_select_sql() -> &'static str {
    "SELECT u.id AS referred_user_id, u.username, u.email, to_char(u.referred_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS referred_at, \
    COALESCE(SUM(c.payable_amount), 0) AS referred_recharge_amount, \
    COALESCE(SUM(CASE WHEN c.status = 'success' THEN c.commission_amount ELSE 0 END), 0) AS commission_amount, \
    to_char(MAX(c.created_at) FILTER (WHERE c.status = 'success'), 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS last_commission_at FROM users u"
}

fn referral_commission_join_sql() -> &'static str {
    "LEFT JOIN affiliate_commissions c ON c.referred_user_id = u.id AND c.referrer_user_id = u.referred_by_user_id"
}

fn commission_select_sql() -> &'static str {
    "SELECT c.id, u.id AS referred_user_id, u.username AS referred_username, u.email AS referred_email, o.order_no AS recharge_order_no, \
    c.payable_amount, c.commission_percent, c.commission_amount, c.wallet_transaction_id, c.status, c.failure_reason, \
    to_char(c.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at FROM affiliate_commissions c"
}

fn commission_join_sql() -> &'static str {
    "JOIN users u ON u.id = c.referred_user_id JOIN recharge_orders o ON o.id = c.recharge_order_id"
}

fn referral_owner_rule(owner_placeholder: &str) -> String {
    format!("u.is_deleted = FALSE AND u.referred_by_user_id = {owner_placeholder}")
}

fn commission_owner_rule(owner_placeholder: &str) -> String {
    format!("c.referrer_user_id = {owner_placeholder} AND u.is_deleted = FALSE")
}
