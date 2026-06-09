use sea_orm::{ConnectionTrait, DbBackend, Statement, Value};

use crate::StorageResult;

use super::{
    ProviderStore,
    request_record_partition_columns::{
        REQUEST_CANDIDATE_METADATA_COLUMNS, REQUEST_CANDIDATE_PARTITION_TABLE, REQUEST_RECORD_METADATA_COLUMNS, REQUEST_RECORD_PARTITION_TABLE,
    },
};

pub async fn sync_request_record(store: &ProviderStore, request_id: &str) -> StorageResult<()> {
    execute_sync(store, sync_request_record_sql(), [Value::from(request_id.to_owned())]).await
}

pub async fn sync_request_candidate(store: &ProviderStore, candidate_id: &str) -> StorageResult<()> {
    execute_sync(store, sync_request_candidate_sql(), [Value::from(candidate_id.to_owned())]).await
}

pub async fn sync_request_candidates_for_request(store: &ProviderStore, request_id: &str) -> StorageResult<()> {
    execute_sync(store, sync_request_candidates_for_request_sql(), [Value::from(request_id.to_owned())]).await
}

fn sync_request_record_sql() -> String {
    partition_sync_sql(SyncSqlInput {
        target: REQUEST_RECORD_PARTITION_TABLE,
        source: "request_records",
        columns: REQUEST_RECORD_METADATA_COLUMNS,
        conflict_columns: &["created_at", "request_id"],
        where_clause: "request_id = $1",
    })
}

fn sync_request_candidate_sql() -> String {
    partition_sync_sql(SyncSqlInput {
        target: REQUEST_CANDIDATE_PARTITION_TABLE,
        source: "request_candidates",
        columns: REQUEST_CANDIDATE_METADATA_COLUMNS,
        conflict_columns: &["created_at", "id"],
        where_clause: "id = $1",
    })
}

fn sync_request_candidates_for_request_sql() -> String {
    partition_sync_sql(SyncSqlInput {
        target: REQUEST_CANDIDATE_PARTITION_TABLE,
        source: "request_candidates",
        columns: REQUEST_CANDIDATE_METADATA_COLUMNS,
        conflict_columns: &["created_at", "id"],
        where_clause: "request_id = $1",
    })
}

struct SyncSqlInput<'a> {
    target: &'a str,
    source: &'a str,
    columns: &'a str,
    conflict_columns: &'a [&'a str],
    where_clause: &'a str,
}

fn partition_sync_sql(input: SyncSqlInput<'_>) -> String {
    format!(
        "INSERT INTO {} ({}) SELECT {} FROM {} WHERE {} ON CONFLICT ({}) DO UPDATE SET {}",
        input.target,
        input.columns,
        input.columns,
        input.source,
        input.where_clause,
        input.conflict_columns.join(", "),
        update_assignments(input.columns, input.conflict_columns)
    )
}

fn update_assignments(columns: &str, conflict_columns: &[&str]) -> String {
    column_names(columns)
        .into_iter()
        .filter(|column| !conflict_columns.contains(&column.as_str()))
        .map(|column| format!("{column} = EXCLUDED.{column}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn column_names(columns: &str) -> Vec<String> {
    columns
        .split(',')
        .map(str::trim)
        .filter(|column| !column.is_empty())
        .map(str::to_owned)
        .collect()
}

async fn execute_sync<const N: usize>(store: &ProviderStore, sql: String, values: [Value; N]) -> StorageResult<()> {
    store
        .connection()
        .execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::update_assignments;

    #[test]
    fn update_assignments_exclude_composite_key() {
        let assignments = update_assignments("created_at, request_id, status", &["created_at", "request_id"]);

        assert_eq!(assignments, "status = EXCLUDED.status");
    }
}
