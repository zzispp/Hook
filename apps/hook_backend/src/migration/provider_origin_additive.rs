use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Schema, Statement},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

use super::baseline::seed_domain::TranslationSeed;

const ADDITIVE_VERSION: &str = "m20260610_000001_provider_origin";
const MIGRATION_TABLE: &str = "seaql_migrations";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    seed_missing_translations(manager).await?;
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    add_provider_origin_column(manager).await?;
    backfill_quick_import_origins(manager).await?;
    mark_additive_applied(manager).await
}

async fn add_provider_origin_column(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    execute_sql(
        manager,
        "ALTER TABLE providers ADD COLUMN IF NOT EXISTS provider_origin VARCHAR(32) NOT NULL DEFAULT 'manual'",
    )
    .await
}

async fn backfill_quick_import_origins(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    execute_sql(
        manager,
        "UPDATE providers \
         SET provider_origin = 'quick_import' \
         WHERE id IN ( \
             SELECT DISTINCT provider_id \
             FROM provider_api_keys \
             WHERE note LIKE 'Imported from newapi group:%' \
         )",
    )
    .await
}

async fn seed_missing_translations(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for seed in super::baseline::seed_domain::translation_seeds()? {
        ensure_translation_entry(manager, &seed).await?;
    }
    Ok(())
}

async fn ensure_translation_entry(manager: &SchemaManager<'_>, seed: &TranslationSeed) -> Result<(), DbErr> {
    if translation_entry_exists(manager, seed).await? {
        return Ok(());
    }
    manager
        .execute(
            Query::insert()
                .into_table(TranslationEntries::Table)
                .columns([
                    TranslationEntries::Id,
                    TranslationEntries::Namespace,
                    TranslationEntries::GroupKey,
                    TranslationEntries::ItemKey,
                    TranslationEntries::LangCode,
                    TranslationEntries::Value,
                    TranslationEntries::Description,
                    TranslationEntries::Enabled,
                    TranslationEntries::CreatedAt,
                    TranslationEntries::UpdatedAt,
                ])
                .values_panic(translation_values(seed))
                .to_owned(),
        )
        .await
}

async fn translation_entry_exists(manager: &SchemaManager<'_>, seed: &TranslationSeed) -> Result<bool, DbErr> {
    let query = Query::select()
        .expr(Expr::val(1))
        .from(TranslationEntries::Table)
        .and_where(Expr::col(TranslationEntries::Namespace).eq(seed.namespace))
        .and_where(Expr::col(TranslationEntries::GroupKey).eq(seed.group_key.as_str()))
        .and_where(Expr::col(TranslationEntries::ItemKey).eq(seed.item_key.as_str()))
        .and_where(Expr::col(TranslationEntries::LangCode).eq(seed.lang_code))
        .limit(1)
        .to_owned();
    manager.get_connection().query_one(&query).await.map(|row| row.is_some())
}

fn translation_values(seed: &TranslationSeed) -> [Expr; 10] {
    [
        new_id().into(),
        seed.namespace.into(),
        seed.group_key.clone().into(),
        seed.item_key.clone().into(),
        seed.lang_code.into(),
        seed.value.clone().into(),
        Option::<String>::None.into(),
        true.into(),
        Expr::current_timestamp(),
        Expr::current_timestamp(),
    ]
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

fn new_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

#[derive(DeriveIden)]
enum TranslationEntries {
    Table,
    Id,
    Namespace,
    GroupKey,
    ItemKey,
    LangCode,
    Value,
    Description,
    Enabled,
    CreatedAt,
    UpdatedAt,
}
