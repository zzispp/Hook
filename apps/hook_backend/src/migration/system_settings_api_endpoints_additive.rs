use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Schema},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const ADDITIVE_VERSION: &str = "m20260622_000001_system_settings_api_endpoints";
const MIGRATION_TABLE: &str = "seaql_migrations";
const ADD_API_ENDPOINTS_SQL: &str = "ALTER TABLE system_settings ADD COLUMN IF NOT EXISTS api_endpoints TEXT NOT NULL DEFAULT '[]'";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    execute_sql(manager, ADD_API_ENDPOINTS_SQL).await?;
    super::translation_seed_sync::seed_missing_translations(manager).await?;
    mark_additive_applied(manager).await
}

async fn execute_sql(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager.get_connection().execute_unprepared(sql).await.map(|_| ())
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
    use super::ADD_API_ENDPOINTS_SQL;

    #[test]
    fn add_api_endpoints_sql_adds_json_text_default() {
        assert!(ADD_API_ENDPOINTS_SQL.contains("ADD COLUMN IF NOT EXISTS api_endpoints TEXT NOT NULL DEFAULT '[]'"));
    }
}
