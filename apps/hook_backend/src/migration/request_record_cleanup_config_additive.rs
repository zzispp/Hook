use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Schema, Set},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};
use storage::scheduler::entities::scheduled_tasks;

const ADDITIVE_VERSION: &str = "m20260609_000001_request_record_cleanup_config";
const MIGRATION_TABLE: &str = "seaql_migrations";
const TASK_CODE: &str = "request_record_cleanup";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    super::translation_seed_sync::seed_missing_translations(manager).await?;
    merge_request_record_cleanup_config(manager).await?;
    mark_additive_applied(manager).await
}

async fn merge_request_record_cleanup_config(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let Some(record) = scheduled_tasks::Entity::find_by_id(TASK_CODE.to_owned()).one(manager.get_connection()).await? else {
        return Ok(());
    };
    let Some(config) = merged_config(&record.config)? else {
        return Ok(());
    };
    let mut active: scheduled_tasks::ActiveModel = record.into();
    active.config = Set(config);
    active.updated_at = Set(time::OffsetDateTime::now_utc());
    active.update(manager.get_connection()).await?;
    Ok(())
}

fn merged_config(raw: &str) -> Result<Option<String>, DbErr> {
    let mut value: serde_json::Value =
        serde_json::from_str(raw).map_err(|error| DbErr::Migration(format!("invalid request_record_cleanup config JSON: {error}")))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| DbErr::Migration("request_record_cleanup config must be a JSON object".into()))?;
    let mut changed = false;
    for (key, value) in cleanup_defaults() {
        if !object.contains_key(key) {
            object.insert(key.to_owned(), serde_json::json!(value));
            changed = true;
        }
    }
    if changed {
        serde_json::to_string(&value)
            .map(Some)
            .map_err(|error| DbErr::Migration(format!("encode request_record_cleanup config failed: {error}")))
    } else {
        Ok(None)
    }
}

fn cleanup_defaults() -> [(&'static str, i64); 6] {
    [
        ("delete_batch_size", 200),
        ("compress_batch_size", 50),
        ("max_runtime_seconds", 120),
        ("batch_sleep_ms", 100),
        ("statement_timeout_seconds", 15),
        ("lock_timeout_seconds", 2),
    ]
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
    use super::merged_config;

    #[test]
    fn merged_config_preserves_existing_retention_fields() {
        let config = merged_config(r#"{"record_retention_days":3,"payload_retention_days":1}"#).unwrap().unwrap();
        let value: serde_json::Value = serde_json::from_str(&config).unwrap();

        assert_eq!(value["record_retention_days"], 3);
        assert_eq!(value["payload_retention_days"], 1);
        assert_eq!(value["delete_batch_size"], 200);
        assert_eq!(value["lock_timeout_seconds"], 2);
    }

    #[test]
    fn merged_config_keeps_existing_batch_fields() {
        let config = merged_config(
            r#"{"delete_batch_size":10,"compress_batch_size":5,"max_runtime_seconds":30,"batch_sleep_ms":0,"statement_timeout_seconds":3,"lock_timeout_seconds":1}"#,
        );

        assert_eq!(config.unwrap(), None);
    }
}
