use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Schema, Statement},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const ADDITIVE_VERSION: &str = "m20260612_000002_global_model_user_usage_counts";
const MIGRATION_TABLE: &str = "seaql_migrations";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    create_usage_table(manager).await?;
    backfill_usage_counts(manager).await?;
    mark_additive_applied(manager).await
}

async fn create_usage_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    execute_sql(
        manager,
        "CREATE TABLE IF NOT EXISTS global_model_user_usage_counts (\
         user_id VARCHAR(36) NOT NULL, \
         global_model_id VARCHAR(36) NOT NULL REFERENCES global_models(id) ON DELETE CASCADE, \
         usage_count BIGINT NOT NULL DEFAULT 0, \
         created_at TIMESTAMPTZ NOT NULL, \
         updated_at TIMESTAMPTZ NOT NULL, \
         PRIMARY KEY (user_id, global_model_id))",
    )
    .await
}

async fn backfill_usage_counts(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    execute_sql(manager, backfill_usage_counts_sql()).await
}

fn backfill_usage_counts_sql() -> &'static str {
    "WITH retained AS (\
         SELECT DISTINCT ON (request_id) request_id, user_id_snapshot, global_model_id, status, billing_status \
         FROM (\
             SELECT request_id, user_id_snapshot, global_model_id, status, billing_status, 0 AS source_priority FROM request_records_partitioned \
             UNION ALL \
             SELECT request_id, user_id_snapshot, global_model_id, status, billing_status, 1 AS source_priority FROM request_records\
         ) source_records \
         ORDER BY request_id, source_priority\
     ) \
     INSERT INTO global_model_user_usage_counts (user_id, global_model_id, usage_count, created_at, updated_at) \
     SELECT retained.user_id_snapshot, retained.global_model_id, COUNT(*)::BIGINT, NOW(), NOW() \
     FROM retained \
     INNER JOIN global_models gm ON gm.id = retained.global_model_id \
     WHERE retained.user_id_snapshot IS NOT NULL \
       AND retained.global_model_id IS NOT NULL \
       AND retained.status = 'success' \
       AND retained.billing_status = 'settled' \
     GROUP BY retained.user_id_snapshot, retained.global_model_id \
     ON CONFLICT (user_id, global_model_id) DO UPDATE SET \
     usage_count = EXCLUDED.usage_count, updated_at = EXCLUDED.updated_at"
}

async fn execute_sql(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_raw(Statement::from_string(manager.get_database_backend(), sql.to_owned()))
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
        applied_at: ActiveValue::Set(unix_timestamp()),
    })
    .exec(manager.get_connection())
    .await?;
    Ok(())
}

async fn create_migration_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if manager.has_table(MIGRATION_TABLE).await? {
        return Ok(());
    }
    let schema = Schema::new(manager.get_database_backend());
    manager.create_table(schema.create_table_from_entity(seaql_migrations::Entity)).await
}

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX_EPOCH")
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::{ADDITIVE_VERSION, backfill_usage_counts_sql};

    #[test]
    fn additive_version_is_unique_for_user_model_usage() {
        assert_eq!(ADDITIVE_VERSION, "m20260612_000002_global_model_user_usage_counts");
    }

    #[test]
    fn backfill_counts_successful_settled_user_model_records_once() {
        let sql = backfill_usage_counts_sql();

        assert!(sql.contains("status = 'success'"), "{sql}");
        assert!(sql.contains("billing_status = 'settled'"), "{sql}");
        assert!(sql.contains("user_id_snapshot IS NOT NULL"), "{sql}");
        assert!(sql.contains("global_model_id IS NOT NULL"), "{sql}");
        assert!(sql.contains("INNER JOIN global_models"), "{sql}");
        assert!(sql.contains("request_records_partitioned"), "{sql}");
        assert!(sql.contains("DISTINCT ON (request_id)"), "{sql}");
        assert!(sql.contains("ORDER BY request_id, source_priority"), "{sql}");
    }
}
