use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Schema},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

use super::baseline::seed_domain::{ADMIN_NAMESPACE, TranslationSeed};

const ADDITIVE_VERSION: &str = "m20260609_000004_dashboard_period_i18n";
const MIGRATION_TABLE: &str = "seaql_migrations";
const DASHBOARD_GROUP: &str = "dashboard";
const PERIOD_ITEM_PREFIX: &str = "stats.period.";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    sync_dashboard_period_translations(manager).await?;
    mark_additive_applied(manager).await
}

async fn sync_dashboard_period_translations(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for seed in dashboard_period_translation_seeds()? {
        sync_translation_entry(manager, &seed).await?;
    }
    Ok(())
}

fn dashboard_period_translation_seeds() -> Result<Vec<TranslationSeed>, DbErr> {
    super::baseline::seed_domain::translation_seeds().map(|seeds| seeds.into_iter().filter(is_dashboard_period_seed).collect())
}

fn is_dashboard_period_seed(seed: &TranslationSeed) -> bool {
    seed.namespace == ADMIN_NAMESPACE && seed.group_key == DASHBOARD_GROUP && seed.item_key.starts_with(PERIOD_ITEM_PREFIX)
}

async fn sync_translation_entry(manager: &SchemaManager<'_>, seed: &TranslationSeed) -> Result<(), DbErr> {
    if translation_entry_exists(manager, seed).await? {
        repair_legacy_placeholder(manager, seed).await
    } else {
        insert_translation_entry(manager, seed).await
    }
}

async fn insert_translation_entry(manager: &SchemaManager<'_>, seed: &TranslationSeed) -> Result<(), DbErr> {
    manager
        .execute(
            Query::insert()
                .into_table(TranslationEntries::Table)
                .columns(translation_columns())
                .values_panic(translation_values(seed))
                .to_owned(),
        )
        .await
}

async fn repair_legacy_placeholder(manager: &SchemaManager<'_>, seed: &TranslationSeed) -> Result<(), DbErr> {
    let Some(legacy_value) = legacy_placeholder_value(seed) else {
        return Ok(());
    };
    manager
        .execute(
            Query::update()
                .table(TranslationEntries::Table)
                .value(TranslationEntries::Value, seed.value.clone())
                .value(TranslationEntries::UpdatedAt, Expr::current_timestamp())
                .and_where(seed_filter(seed))
                .and_where(Expr::col(TranslationEntries::Value).eq(legacy_value))
                .to_owned(),
        )
        .await
}

fn legacy_placeholder_value(seed: &TranslationSeed) -> Option<&'static str> {
    match (seed.lang_code, seed.item_key.as_str()) {
        ("cn", "stats.period.upstreamCost") => Some("上游 {{value}}"),
        ("en", "stats.period.upstreamCost") => Some("Upstream {{value}}"),
        _ => None,
    }
}

async fn translation_entry_exists(manager: &SchemaManager<'_>, seed: &TranslationSeed) -> Result<bool, DbErr> {
    let query = Query::select()
        .expr(Expr::val(1))
        .from(TranslationEntries::Table)
        .and_where(seed_filter(seed))
        .limit(1)
        .to_owned();
    manager.get_connection().query_one(&query).await.map(|row| row.is_some())
}

fn seed_filter(seed: &TranslationSeed) -> SimpleExpr {
    Expr::col(TranslationEntries::Namespace)
        .eq(seed.namespace)
        .and(Expr::col(TranslationEntries::GroupKey).eq(seed.group_key.as_str()))
        .and(Expr::col(TranslationEntries::ItemKey).eq(seed.item_key.as_str()))
        .and(Expr::col(TranslationEntries::LangCode).eq(seed.lang_code))
}

fn translation_columns() -> [TranslationEntries; 10] {
    [
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
    ]
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

#[derive(Clone, Copy, DeriveIden)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_period_translation_seeds_include_active_users() {
        let keys: Vec<_> = dashboard_period_translation_seeds()
            .unwrap()
            .into_iter()
            .map(|seed| (seed.lang_code, seed.group_key, seed.item_key, seed.value))
            .collect();

        assert!(keys.contains(&(
            "cn",
            "dashboard".to_owned(),
            "stats.period.activeUsers".to_owned(),
            "{{period}}活跃用户".to_owned(),
        )));
        assert!(keys.contains(&(
            "en",
            "dashboard".to_owned(),
            "stats.period.activeUsers".to_owned(),
            "{{period}} active users".to_owned(),
        )));
    }

    #[test]
    fn legacy_placeholder_value_only_targets_upstream_cost() {
        let mut upstream = dashboard_period_translation_seeds()
            .unwrap()
            .into_iter()
            .find(|seed| seed.lang_code == "cn" && seed.item_key == "stats.period.upstreamCost")
            .unwrap();
        assert_eq!(legacy_placeholder_value(&upstream), Some("上游 {{value}}"));

        upstream.item_key = "stats.period.activeUsers".to_owned();
        assert_eq!(legacy_placeholder_value(&upstream), None);
    }
}
