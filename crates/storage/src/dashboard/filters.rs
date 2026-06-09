use sea_orm::{DbBackend, FromQueryResult, Statement};
use types::dashboard::{DashboardFilterOption, DashboardFilterOptionsResponse};

use crate::StorageResult;

use super::{
    DashboardScopeFilter, DashboardStore, DashboardStoreFilterOptionsQuery,
    scope::{SqlParams, add_metric_scope_filter},
};

const FILTER_LIMIT: i64 = 100;

pub(super) async fn filter_options(store: &DashboardStore, query: DashboardStoreFilterOptionsQuery) -> StorageResult<DashboardFilterOptionsResponse> {
    Ok(DashboardFilterOptionsResponse {
        users: user_options(store, &query.scope).await?,
        tokens: token_options(store, &query.scope).await?,
    })
}

async fn user_options(store: &DashboardStore, scope: &DashboardScopeFilter) -> StorageResult<Vec<DashboardFilterOption>> {
    if !matches!(scope, DashboardScopeFilter::Global) {
        return Ok(Vec::new());
    }
    option_rows(
        store,
        "SELECT id, username AS name FROM users WHERE is_deleted = FALSE ORDER BY username ASC LIMIT $1",
        vec![sea_orm::Value::from(FILTER_LIMIT)],
    )
    .await
}

async fn token_options(store: &DashboardStore, scope: &DashboardScopeFilter) -> StorageResult<Vec<DashboardFilterOption>> {
    let mut params = SqlParams::new();
    let mut filters = vec!["b.source_type = 'request'".into(), "b.token_id IS NOT NULL".into()];
    add_metric_scope_filter(scope, &mut params, &mut filters);
    let where_sql = format!("WHERE {}", filters.join(" AND "));
    let limit = params.push(FILTER_LIMIT);
    let sql = format!(
        "SELECT b.token_id AS id, COALESCE(MAX(b.token_name), MAX(b.token_prefix), b.token_id) AS name \
        FROM dashboard_request_metric_buckets b {where_sql} \
        GROUP BY b.token_id \
        HAVING COALESCE(SUM(b.request_count), 0) > 0 \
        ORDER BY name ASC \
        LIMIT {limit}"
    );
    option_rows(store, &sql, params.values).await
}

async fn option_rows(store: &DashboardStore, sql: &str, values: Vec<sea_orm::Value>) -> StorageResult<Vec<DashboardFilterOption>> {
    let rows = OptionRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql.to_owned(), values))
        .all(store.database().connection())
        .await?;
    Ok(rows.into_iter().map(|row| DashboardFilterOption { id: row.id, name: row.name }).collect())
}

#[derive(Debug, FromQueryResult)]
struct OptionRow {
    id: String,
    name: String,
}
