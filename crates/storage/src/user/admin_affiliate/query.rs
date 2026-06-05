use sea_orm::{DbBackend, FromQueryResult, Statement, Value};
use types::{
    pagination::{Page, PageSliceRequest},
    user::{
        AdminAffiliateCommissionItem, AdminAffiliateCommissionQuery, AdminAffiliateDailyReportItem, AdminAffiliateOverviewResponse,
        AdminAffiliateReferrerReportItem, AdminAffiliateRelationItem, AdminAffiliateRelationQuery, AdminAffiliateReportQuery, AdminAffiliateReportResponse,
    },
};

use crate::{StorageError, StorageResult};

use super::{
    filter::AffiliateSqlFilter,
    rows::{
        CommissionRow, CountRow, DailyReportRow, OverviewRow, ReferrerReportRow, RelationRow, commission_item, daily_report_item, overview_response,
        referrer_report_item, relation_item,
    },
};

pub(super) async fn overview(store: &super::super::UserStore) -> StorageResult<AdminAffiliateOverviewResponse> {
    let row = OverviewRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, overview_sql(), Vec::new()))
        .one(store.database.connection())
        .await?
        .ok_or_else(|| StorageError::Database("affiliate overview returned no rows".into()))?;
    Ok(overview_response(row))
}

pub(super) async fn relations(
    store: &super::super::UserStore,
    request: PageSliceRequest,
    query: AdminAffiliateRelationQuery,
) -> StorageResult<Page<AdminAffiliateRelationItem>> {
    let filter = AffiliateSqlFilter::relation(&query);
    let total = relation_total(store, &filter).await?;
    let rows = relation_rows(store, request, filter).await?;
    Ok(Page {
        items: rows.into_iter().map(relation_item).collect(),
        total,
        page: request.page,
        page_size: request.page_size,
    })
}

pub(super) async fn commissions(
    store: &super::super::UserStore,
    request: PageSliceRequest,
    query: AdminAffiliateCommissionQuery,
) -> StorageResult<Page<AdminAffiliateCommissionItem>> {
    let filter = AffiliateSqlFilter::commission(&query);
    let total = commission_total(store, &filter).await?;
    let rows = commission_rows(store, request, filter).await?;
    Ok(Page {
        items: rows.into_iter().map(commission_item).collect(),
        total,
        page: request.page,
        page_size: request.page_size,
    })
}

pub(super) async fn report(store: &super::super::UserStore, query: AdminAffiliateReportQuery) -> StorageResult<AdminAffiliateReportResponse> {
    let filter = AffiliateSqlFilter::report(&query);
    let daily_items = daily_report_rows(store, &filter).await?.into_iter().map(daily_report_item).collect();
    let referrer_total = referrer_report_total(store, &filter).await?;
    let referrer_items = referrer_report_rows(store, &query, filter)
        .await?
        .into_iter()
        .map(referrer_report_item)
        .collect();
    Ok(AdminAffiliateReportResponse {
        daily_items,
        referrer_items,
        referrer_total,
        page: query.page,
        page_size: query.page_size,
    })
}

pub(super) async fn export_commissions(
    store: &super::super::UserStore,
    query: AdminAffiliateCommissionQuery,
) -> StorageResult<Vec<AdminAffiliateCommissionItem>> {
    let filter = AffiliateSqlFilter::commission(&query);
    let sql = format!(
        "{} {} {} ORDER BY c.created_at DESC",
        commission_select_sql(),
        commission_join_sql(),
        filter.where_sql()
    );
    Ok(query_rows::<CommissionRow>(store, sql, filter.into_values())
        .await?
        .into_iter()
        .map(commission_item)
        .collect())
}

pub(super) async fn export_daily_report(
    store: &super::super::UserStore,
    query: AdminAffiliateReportQuery,
) -> StorageResult<Vec<AdminAffiliateDailyReportItem>> {
    let filter = AffiliateSqlFilter::report(&query);
    Ok(daily_report_rows(store, &filter).await?.into_iter().map(daily_report_item).collect())
}

pub(super) async fn export_referrer_report(
    store: &super::super::UserStore,
    query: AdminAffiliateReportQuery,
) -> StorageResult<Vec<AdminAffiliateReferrerReportItem>> {
    let filter = AffiliateSqlFilter::report(&query);
    let sql = format!(
        "{} {} {} GROUP BY r.id, r.username, r.email, r.affiliate_code ORDER BY commission_amount DESC",
        referrer_report_select_sql(),
        commission_join_sql(),
        filter.where_sql()
    );
    Ok(query_rows::<ReferrerReportRow>(store, sql, filter.into_values())
        .await?
        .into_iter()
        .map(referrer_report_item)
        .collect())
}

async fn relation_total(store: &super::super::UserStore, filter: &AffiliateSqlFilter) -> StorageResult<u64> {
    let sql = format!(
        "SELECT COUNT(*)::bigint AS total FROM users u {} {}",
        relation_referrer_join(),
        filter.where_sql()
    );
    count_query(store, sql, filter.values()).await
}

async fn relation_rows(store: &super::super::UserStore, request: PageSliceRequest, mut filter: AffiliateSqlFilter) -> StorageResult<Vec<RelationRow>> {
    let limit = filter.push((request.limit as i64).into());
    let offset = filter.push((request.offset as i64).into());
    let sql = format!(
        "{} {} {} {} LIMIT {limit} OFFSET {offset}",
        relation_select_sql(),
        relation_referrer_join(),
        filter.where_sql(),
        relation_group_sql()
    );
    query_rows(store, sql, filter.into_values()).await
}

async fn commission_total(store: &super::super::UserStore, filter: &AffiliateSqlFilter) -> StorageResult<u64> {
    let sql = format!(
        "SELECT COUNT(*)::bigint AS total FROM affiliate_commissions c {} {}",
        commission_join_sql(),
        filter.where_sql()
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
        filter.where_sql()
    );
    query_rows(store, sql, filter.into_values()).await
}

async fn daily_report_rows(store: &super::super::UserStore, filter: &AffiliateSqlFilter) -> StorageResult<Vec<DailyReportRow>> {
    let sql = format!(
        "{} {} {} GROUP BY date ORDER BY date DESC",
        daily_report_select_sql(),
        commission_join_sql(),
        filter.where_sql()
    );
    query_rows(store, sql, filter.values()).await
}

async fn referrer_report_total(store: &super::super::UserStore, filter: &AffiliateSqlFilter) -> StorageResult<u64> {
    let sql = format!(
        "SELECT COUNT(*)::bigint AS total FROM (SELECT c.referrer_user_id FROM affiliate_commissions c {} {} GROUP BY c.referrer_user_id) ranked",
        commission_join_sql(),
        filter.where_sql()
    );
    count_query(store, sql, filter.values()).await
}

async fn referrer_report_rows(
    store: &super::super::UserStore,
    query: &AdminAffiliateReportQuery,
    mut filter: AffiliateSqlFilter,
) -> StorageResult<Vec<ReferrerReportRow>> {
    let limit = filter.push((query.page_size as i64).into());
    let offset = filter.push(((query.page.saturating_sub(1) * query.page_size) as i64).into());
    let sql = format!(
        "{} {} {} GROUP BY r.id, r.username, r.email, r.affiliate_code ORDER BY commission_amount DESC LIMIT {limit} OFFSET {offset}",
        referrer_report_select_sql(),
        commission_join_sql(),
        filter.where_sql()
    );
    query_rows(store, sql, filter.into_values()).await
}

pub(super) async fn count_query(store: &super::super::UserStore, sql: String, values: Vec<Value>) -> StorageResult<u64> {
    CountRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
        .one(store.database.connection())
        .await?
        .map(|row| row.total.unwrap_or_default() as u64)
        .ok_or_else(|| StorageError::Database("affiliate count query returned no rows".into()))
}

pub(super) async fn query_rows<T>(store: &super::super::UserStore, sql: String, values: Vec<Value>) -> StorageResult<Vec<T>>
where
    T: FromQueryResult + Send + Sync + 'static,
{
    T::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
        .all(store.database.connection())
        .await
        .map_err(Into::into)
}

fn overview_sql() -> String {
    "SELECT \
    (SELECT COUNT(*)::bigint FROM users WHERE is_deleted = FALSE AND referred_by_user_id IS NOT NULL) AS total_referred_users, \
    (SELECT COUNT(DISTINCT referred_by_user_id)::bigint FROM users WHERE is_deleted = FALSE AND referred_by_user_id IS NOT NULL) AS active_referrer_count, \
    COALESCE((SELECT SUM(commission_amount) FROM affiliate_commissions WHERE status = 'success'), 0) AS total_commission_amount, \
    COALESCE((SELECT SUM(commission_amount) FROM affiliate_commissions WHERE status = 'success' AND created_at >= date_trunc('day', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'), 0) AS today_commission_amount, \
    COALESCE((SELECT SUM(commission_amount) FROM affiliate_commissions WHERE status = 'success' AND created_at >= date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'), 0) AS month_commission_amount, \
    COALESCE((SELECT affiliate_commission_percent FROM system_settings WHERE id = 'global'), 0) AS affiliate_commission_percent"
        .into()
}

fn relation_select_sql() -> &'static str {
    "SELECT u.id AS user_id, u.username, u.email, u.affiliate_code, ref.id AS referrer_id, ref.username AS referrer_username, \
    ref.email AS referrer_email, ref.affiliate_code AS referrer_affiliate_code, to_char(u.referred_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS referred_at, \
    COALESCE(SUM(c.payable_amount), 0) AS referred_recharge_amount, \
    COALESCE(SUM(CASE WHEN c.status = 'success' THEN c.commission_amount ELSE 0 END), 0) AS commission_amount, \
    to_char(MAX(c.created_at) FILTER (WHERE c.status = 'success'), 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS last_commission_at FROM users u"
}

fn relation_referrer_join() -> &'static str {
    "LEFT JOIN users ref ON ref.id = u.referred_by_user_id LEFT JOIN affiliate_commissions c ON c.referred_user_id = u.id"
}

fn relation_group_sql() -> &'static str {
    "GROUP BY u.id, u.username, u.email, u.affiliate_code, ref.id, ref.username, ref.email, ref.affiliate_code, u.referred_at ORDER BY u.created_at DESC"
}

fn commission_select_sql() -> &'static str {
    "SELECT c.id, c.recharge_order_id, o.order_no AS recharge_order_no, c.payable_amount, c.commission_percent, c.commission_amount, \
    c.wallet_transaction_id, c.status, c.failure_reason, to_char(c.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at, \
    r.id AS referrer_id, r.username AS referrer_username, r.email AS referrer_email, r.affiliate_code AS referrer_affiliate_code, \
    u.id AS referred_id, u.username AS referred_username, u.email AS referred_email, u.affiliate_code AS referred_affiliate_code FROM affiliate_commissions c"
}

fn commission_join_sql() -> &'static str {
    "JOIN users r ON r.id = c.referrer_user_id JOIN users u ON u.id = c.referred_user_id JOIN recharge_orders o ON o.id = c.recharge_order_id"
}

fn daily_report_select_sql() -> &'static str {
    "SELECT to_char(date_trunc('day', c.created_at AT TIME ZONE 'UTC'), 'YYYY-MM-DD') AS date, COUNT(*)::bigint AS commission_order_count, \
    COUNT(DISTINCT c.referred_user_id)::bigint AS referred_payer_count, COALESCE(SUM(c.payable_amount), 0) AS payable_amount, \
    COALESCE(SUM(c.commission_amount), 0) AS commission_amount FROM affiliate_commissions c"
}

fn referrer_report_select_sql() -> &'static str {
    "SELECT r.id AS referrer_id, r.username AS referrer_username, r.email AS referrer_email, r.affiliate_code AS referrer_affiliate_code, \
    COUNT(DISTINCT c.referred_user_id)::bigint AS referred_user_count, COUNT(*)::bigint AS commission_order_count, \
    COALESCE(SUM(c.payable_amount), 0) AS payable_amount, COALESCE(SUM(c.commission_amount), 0) AS commission_amount FROM affiliate_commissions c"
}
