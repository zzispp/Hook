use types::{
    pagination::{Page, PageSliceRequest},
    user::{AdminAffiliateRelationChangeItem, AdminAffiliateRelationChangeQuery},
};

use crate::StorageResult;

use super::{
    filter::AffiliateSqlFilter,
    query::{count_query, query_rows},
    rows::{RelationChangeRow, relation_change_item},
};

pub(super) async fn relation_changes(
    store: &super::super::UserStore,
    request: PageSliceRequest,
    query: AdminAffiliateRelationChangeQuery,
) -> StorageResult<Page<AdminAffiliateRelationChangeItem>> {
    let filter = AffiliateSqlFilter::relation_change(&query);
    let total = relation_change_total(store, &filter).await?;
    let rows = relation_change_rows(store, request, filter).await?;
    Ok(Page {
        items: rows.into_iter().map(relation_change_item).collect(),
        total,
        page: request.page,
        page_size: request.page_size,
    })
}

async fn relation_change_total(store: &super::super::UserStore, filter: &AffiliateSqlFilter) -> StorageResult<u64> {
    let sql = format!(
        "SELECT COUNT(*)::bigint AS total FROM affiliate_relation_changes ch {} {}",
        relation_change_join_sql(),
        filter.where_sql()
    );
    count_query(store, sql, filter.values()).await
}

async fn relation_change_rows(
    store: &super::super::UserStore,
    request: PageSliceRequest,
    mut filter: AffiliateSqlFilter,
) -> StorageResult<Vec<RelationChangeRow>> {
    let limit = filter.push((request.limit as i64).into());
    let offset = filter.push((request.offset as i64).into());
    let sql = format!(
        "{} {} {} ORDER BY ch.created_at DESC LIMIT {limit} OFFSET {offset}",
        relation_change_select_sql(),
        relation_change_join_sql(),
        filter.where_sql()
    );
    query_rows(store, sql, filter.into_values()).await
}

fn relation_change_select_sql() -> &'static str {
    "SELECT ch.id, ch.user_id, u.username, u.email, u.affiliate_code, \
    old_ref.id AS old_referrer_id, old_ref.username AS old_referrer_username, old_ref.email AS old_referrer_email, \
    old_ref.affiliate_code AS old_referrer_affiliate_code, new_ref.id AS new_referrer_id, \
    new_ref.username AS new_referrer_username, new_ref.email AS new_referrer_email, \
    new_ref.affiliate_code AS new_referrer_affiliate_code, op.id AS operator_id, \
    op.username AS operator_username, op.email AS operator_email, op.affiliate_code AS operator_affiliate_code, \
    ch.reason, to_char(ch.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at \
    FROM affiliate_relation_changes ch"
}

fn relation_change_join_sql() -> &'static str {
    "JOIN users u ON u.id = ch.user_id \
    LEFT JOIN users old_ref ON old_ref.id = ch.old_referrer_user_id \
    LEFT JOIN users new_ref ON new_ref.id = ch.new_referrer_user_id \
    LEFT JOIN users op ON op.id = ch.operator_user_id"
}
