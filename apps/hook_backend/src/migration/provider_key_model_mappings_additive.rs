use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, DbBackend, EntityTrait, FromQueryResult, QueryFilter, Schema, Statement},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

use storage::{model::provider_models, provider::record::provider_key_model_mappings};

const ADDITIVE_VERSION: &str = "m20260617_000001_provider_key_model_mappings";
const MIGRATION_TABLE: &str = "seaql_migrations";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    super::baseline::apply_schema_without_seed(manager).await?;
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    create_table(manager).await?;
    backfill_legacy_records(manager).await?;
    drop_legacy_table(manager).await?;
    drop_provider_model_mapping_columns(manager).await?;
    mark_additive_applied(manager).await
}

async fn create_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    execute_sql(
        manager,
        "CREATE TABLE IF NOT EXISTS provider_key_model_mappings (
            id VARCHAR(36) PRIMARY KEY,
            provider_id VARCHAR(36) NOT NULL,
            key_id VARCHAR(36) NOT NULL,
            provider_model_id VARCHAR(36) NOT NULL,
            upstream_model_name VARCHAR(200) NOT NULL,
            reasoning_effort VARCHAR(20) NULL,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            CONSTRAINT fk_provider_key_model_mappings_provider FOREIGN KEY (provider_id) REFERENCES providers(id) ON DELETE CASCADE,
            CONSTRAINT fk_provider_key_model_mappings_key FOREIGN KEY (key_id) REFERENCES provider_api_keys(id) ON DELETE CASCADE,
            CONSTRAINT fk_provider_key_model_mappings_provider_model FOREIGN KEY (provider_model_id) REFERENCES provider_models(id) ON DELETE CASCADE
        )",
    )
    .await?;
    execute_sql(
        manager,
        "CREATE UNIQUE INDEX IF NOT EXISTS index_provider_key_model_mappings_unique ON provider_key_model_mappings (key_id, provider_model_id)",
    )
    .await?;
    execute_sql(
        manager,
        "CREATE INDEX IF NOT EXISTS index_provider_key_model_mappings_by_key ON provider_key_model_mappings (key_id)",
    )
    .await?;
    execute_sql(
        manager,
        "CREATE INDEX IF NOT EXISTS index_provider_key_model_mappings_by_provider_model ON provider_key_model_mappings (provider_model_id)",
    )
    .await
}

async fn backfill_legacy_records(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if !manager.has_table("provider_quick_import_key_models").await? {
        return Ok(());
    }
    let records = LegacyQuickImportKeyModelRow::find_by_statement(Statement::from_string(
        DbBackend::Postgres,
        "SELECT provider_id, key_id, global_model_id, upstream_model_id FROM provider_quick_import_key_models".to_owned(),
    ))
    .all(manager.get_connection())
    .await?;
    for record in records {
        let Some(provider_model) = provider_models::Entity::find()
            .filter(provider_models::Column::ProviderId.eq(record.provider_id.clone()))
            .filter(provider_models::Column::GlobalModelId.eq(record.global_model_id.clone()))
            .one(manager.get_connection())
            .await?
        else {
            continue;
        };
        provider_key_model_mappings::ActiveModel {
            id: ActiveValue::Set(uuid::Uuid::now_v7().to_string()),
            provider_id: ActiveValue::Set(record.provider_id),
            key_id: ActiveValue::Set(record.key_id),
            provider_model_id: ActiveValue::Set(provider_model.id),
            upstream_model_name: ActiveValue::Set(record.upstream_model_id),
            reasoning_effort: ActiveValue::Set(None),
            created_at: ActiveValue::Set(time::OffsetDateTime::now_utc()),
            updated_at: ActiveValue::Set(time::OffsetDateTime::now_utc()),
        }
        .insert(manager.get_connection())
        .await?;
    }
    Ok(())
}

#[derive(Debug, FromQueryResult)]
struct LegacyQuickImportKeyModelRow {
    provider_id: String,
    key_id: String,
    global_model_id: String,
    upstream_model_id: String,
}

async fn drop_legacy_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    execute_sql(manager, "DROP TABLE IF EXISTS provider_quick_import_key_models").await
}

async fn drop_provider_model_mapping_columns(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    execute_sql(manager, "ALTER TABLE provider_models DROP COLUMN IF EXISTS provider_model_name").await?;
    execute_sql(manager, "ALTER TABLE provider_models DROP COLUMN IF EXISTS provider_model_mappings").await
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
        applied_at: ActiveValue::Set(current_timestamp()?),
    })
    .exec(manager.get_connection())
    .await?;
    Ok(())
}

async fn create_migration_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let schema = Schema::new(manager.get_database_backend());
    manager
        .create_table(schema.create_table_from_entity(seaql_migrations::Entity).if_not_exists().to_owned())
        .await
}

fn current_timestamp() -> Result<i64, DbErr> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .map_err(|error| DbErr::Migration(format!("system clock before unix epoch: {error}")))
}
