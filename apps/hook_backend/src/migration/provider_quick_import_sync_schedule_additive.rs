use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Schema, Set},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};
use storage::scheduler::entities::scheduled_tasks;

const ADDITIVE_VERSION: &str = "m20260622_000002_provider_quick_import_sync_schedule";
const MIGRATION_TABLE: &str = "seaql_migrations";
const TASK_CODE: &str = "provider_quick_import_sync";
const INTERVAL_SECONDS: i64 = 600;
const LEASE_SECONDS: i64 = 1800;

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    update_task_schedule(manager).await?;
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    mark_additive_applied(manager).await
}

async fn update_task_schedule(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let Some(record) = scheduled_tasks::Entity::find_by_id(TASK_CODE.to_owned()).one(manager.get_connection()).await? else {
        return Ok(());
    };
    if record.interval_seconds == INTERVAL_SECONDS && record.lease_seconds == LEASE_SECONDS {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let mut active: scheduled_tasks::ActiveModel = record.into();
    active.interval_seconds = Set(INTERVAL_SECONDS);
    active.lease_seconds = Set(LEASE_SECONDS);
    active.next_run_at = Set(now + time::Duration::seconds(INTERVAL_SECONDS));
    active.locked_until = Set(None);
    active.locked_by = Set(None);
    active.updated_at = Set(now);
    active.update(manager.get_connection()).await?;
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
