use ::types::provider::{ROUTING_TIMING_SEMANTICS_COLUMN, ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1, ROUTING_TIMING_SEMANTICS_LEGACY_FIRST_BYTE_V1};
use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Schema, Statement},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const ADDITIVE_VERSION: &str = "m20260630_000001_routing_metric_timing_semantics";
const MIGRATION_TABLE: &str = "seaql_migrations";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    ensure_timing_semantics_columns(manager).await?;
    ensure_timing_semantics_keys(manager).await?;
    recreate_indexes(manager).await?;
    mark_additive_applied(manager).await
}

async fn ensure_timing_semantics_columns(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for sql in timing_semantics_column_sql() {
        execute_sql(manager, sql).await?;
    }
    Ok(())
}

async fn ensure_timing_semantics_keys(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for sql in timing_semantics_key_sql() {
        execute_sql(manager, sql).await?;
    }
    Ok(())
}

async fn recreate_indexes(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for sql in index_sql() {
        execute_sql(manager, sql).await?;
    }
    Ok(())
}

fn timing_semantics_column_sql() -> Vec<String> {
    ["routing_metric_buckets", "routing_route_states", "routing_context_route_states"]
        .into_iter()
        .flat_map(table_timing_semantics_column_sql)
        .collect()
}

fn table_timing_semantics_column_sql(table: &str) -> [String; 4] {
    [
        format!("ALTER TABLE IF EXISTS {table} ADD COLUMN IF NOT EXISTS {ROUTING_TIMING_SEMANTICS_COLUMN} VARCHAR(32) NULL"),
        format!(
            "UPDATE {table} SET {ROUTING_TIMING_SEMANTICS_COLUMN} = '{ROUTING_TIMING_SEMANTICS_LEGACY_FIRST_BYTE_V1}' WHERE {ROUTING_TIMING_SEMANTICS_COLUMN} IS NULL"
        ),
        format!("ALTER TABLE IF EXISTS {table} ALTER COLUMN {ROUTING_TIMING_SEMANTICS_COLUMN} SET DEFAULT '{ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1}'"),
        format!("ALTER TABLE IF EXISTS {table} ALTER COLUMN {ROUTING_TIMING_SEMANTICS_COLUMN} SET NOT NULL"),
    ]
}

fn timing_semantics_key_sql() -> [&'static str; 6] {
    [
        "DROP INDEX IF EXISTS index_routing_metric_buckets_unique",
        "CREATE UNIQUE INDEX IF NOT EXISTS index_routing_metric_buckets_unique ON routing_metric_buckets \
         (bucket_granularity, bucket_started_at, provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream, \
         route_config_fingerprint, price_config_fingerprint, timing_metric_semantics_version)",
        "ALTER TABLE IF EXISTS routing_route_states DROP CONSTRAINT IF EXISTS routing_route_states_pkey",
        "ALTER TABLE IF EXISTS routing_route_states ADD CONSTRAINT routing_route_states_pkey PRIMARY KEY \
         (profile_id, provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream, route_config_fingerprint, price_config_fingerprint, timing_metric_semantics_version)",
        "ALTER TABLE IF EXISTS routing_context_route_states DROP CONSTRAINT IF EXISTS routing_context_route_states_pkey",
        "ALTER TABLE IF EXISTS routing_context_route_states ADD CONSTRAINT routing_context_route_states_pkey PRIMARY KEY \
         (profile_id, context_key, provider_id, key_id, endpoint_id, global_model_id, client_api_format, provider_api_format, is_stream, route_config_fingerprint, price_config_fingerprint, timing_metric_semantics_version)",
    ]
}

fn index_sql() -> [&'static str; 4] {
    [
        "DROP INDEX IF EXISTS index_routing_route_states_by_updated",
        "CREATE INDEX IF NOT EXISTS index_routing_route_states_by_updated ON routing_route_states (profile_id, timing_metric_semantics_version, last_updated_at DESC)",
        "DROP INDEX IF EXISTS index_routing_context_route_states_by_updated",
        "CREATE INDEX IF NOT EXISTS index_routing_context_route_states_by_updated ON routing_context_route_states (profile_id, timing_metric_semantics_version, context_key, last_updated_at DESC)",
    ]
}

async fn execute_sql(manager: &SchemaManager<'_>, sql: impl Into<String>) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_raw(Statement::from_string(manager.get_database_backend(), sql.into()))
        .await?;
    Ok(())
}

async fn additive_marker_exists(manager: &SchemaManager<'_>) -> Result<bool, DbErr> {
    if !manager.has_table(MIGRATION_TABLE).await? {
        return Ok(false);
    }
    seaql_migrations::Entity::find()
        .filter(seaql_migrations::Column::Version.eq(ADDITIVE_VERSION))
        .one(manager.get_connection())
        .await
        .map(|record| record.is_some())
}

async fn mark_additive_applied(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    create_migration_table(manager).await?;
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    seaql_migrations::Entity::insert(seaql_migrations::ActiveModel {
        version: ActiveValue::Set(ADDITIVE_VERSION.to_owned()),
        applied_at: ActiveValue::Set(current_timestamp()?),
    })
    .exec(manager.get_connection())
    .await?;
    Ok(())
}

async fn create_migration_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let schema = Schema::new(manager.get_database_backend());
    let mut statement = schema.create_table_from_entity(seaql_migrations::Entity);
    statement.if_not_exists();
    manager.create_table(statement).await
}

fn current_timestamp() -> Result<i64, DbErr> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .map_err(|error| DbErr::Migration(format!("system time is before UNIX epoch: {error}")))
}

#[cfg(test)]
mod tests {
    use ::types::provider::{ROUTING_TIMING_SEMANTICS_COLUMN, ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1, ROUTING_TIMING_SEMANTICS_LEGACY_FIRST_BYTE_V1};

    use super::{index_sql, timing_semantics_column_sql, timing_semantics_key_sql};

    #[test]
    fn timing_semantics_columns_backfill_legacy_and_default_to_first_token() {
        let sql = timing_semantics_column_sql().join(" ");

        assert!(sql.contains(&format!("ADD COLUMN IF NOT EXISTS {ROUTING_TIMING_SEMANTICS_COLUMN} VARCHAR(32) NULL")));
        assert!(sql.contains(&format!(
            "SET {ROUTING_TIMING_SEMANTICS_COLUMN} = '{ROUTING_TIMING_SEMANTICS_LEGACY_FIRST_BYTE_V1}'"
        )));
        assert!(sql.contains(&format!("SET DEFAULT '{ROUTING_TIMING_SEMANTICS_FIRST_TOKEN_V1}'")));
        assert!(sql.contains(&format!("ALTER COLUMN {ROUTING_TIMING_SEMANTICS_COLUMN} SET NOT NULL")));
    }

    #[test]
    fn timing_semantics_keys_extend_bucket_and_state_identity() {
        let sql = timing_semantics_key_sql().join(" ");

        assert!(sql.contains("timing_metric_semantics_version)"));
        assert!(sql.contains("route_config_fingerprint, price_config_fingerprint, timing_metric_semantics_version)"));
    }

    #[test]
    fn timing_semantics_indexes_lead_with_profile_and_semantics() {
        let sql = index_sql().join(" ");

        assert!(sql.contains("routing_route_states (profile_id, timing_metric_semantics_version, last_updated_at DESC)"));
        assert!(sql.contains("routing_context_route_states (profile_id, timing_metric_semantics_version, context_key, last_updated_at DESC)"));
    }
}
