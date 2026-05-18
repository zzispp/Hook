use sea_orm::{DbBackend, FromQueryResult, Statement};
use types::dashboard::{DashboardFilterOption, DashboardFilterOptionsResponse};

use crate::StorageResult;

use super::{
    DashboardScopeFilter, DashboardStore, DashboardStoreFilterOptionsQuery,
    scope::{SqlParams, add_scope_filter},
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
    let mut filters = vec!["r.token_id IS NOT NULL".into()];
    add_scope_filter(scope, &mut params, &mut filters);
    let where_sql = format!("WHERE {}", filters.join(" AND "));
    let limit = params.push(FILTER_LIMIT);
    let sql = format!(
        "SELECT token_id AS id, COALESCE(MAX(token_name_snapshot), MAX(token_prefix_snapshot), token_id) AS name \
        FROM request_records r {where_sql} \
        GROUP BY token_id \
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
