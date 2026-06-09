use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement};

use crate::{StorageResult, dashboard::scope::SqlParams};

#[derive(Debug, FromQueryResult)]
struct SyncStateRow {
    owner_id: String,
}

pub(super) async fn lock_exists<C>(connection: &C, owner_type: &str, owner_id: &str) -> StorageResult<bool>
where
    C: ConnectionTrait,
{
    let mut params = SqlParams::new();
    let sql = format!(
        "SELECT owner_id FROM dashboard_request_metric_sync_states \
        WHERE owner_type = {} AND owner_id = {} FOR UPDATE",
        params.push(owner_type.to_owned()),
        params.push(owner_id.to_owned())
    );
    let row = SyncStateRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .one(connection)
        .await?;
    Ok(row.map(|item| !item.owner_id.is_empty()).unwrap_or(false))
}

pub(super) async fn upsert<C>(connection: &C, owner_type: &str, owner_id: &str) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let now = time::OffsetDateTime::now_utc();
    let mut params = SqlParams::new();
    let sql = format!(
        "INSERT INTO dashboard_request_metric_sync_states (owner_type, owner_id, created_at, updated_at) \
        VALUES ({}, {}, {}, {}) \
        ON CONFLICT (owner_type, owner_id) DO UPDATE SET updated_at = EXCLUDED.updated_at",
        params.push(owner_type.to_owned()),
        params.push(owner_id.to_owned()),
        params.push(now),
        params.push(now)
    );
    connection
        .execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .await?;
    Ok(())
}

pub(super) async fn delete<C>(connection: &C, owner_type: &str, owner_id: &str) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let mut params = SqlParams::new();
    let sql = format!(
        "DELETE FROM dashboard_request_metric_sync_states WHERE owner_type = {} AND owner_id = {}",
        params.push(owner_type.to_owned()),
        params.push(owner_id.to_owned())
    );
    connection
        .execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql, params.values))
        .await?;
    Ok(())
}
