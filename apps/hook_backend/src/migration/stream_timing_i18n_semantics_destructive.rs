use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Schema, Value},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const DESTRUCTIVE_VERSION: &str = "m20260630_000004_stream_timing_i18n_semantics";
const MIGRATION_TABLE: &str = "seaql_migrations";
const ADMIN_NAMESPACE: &str = "admin";
const DASHBOARD_GROUP: &str = "dashboard";
const PERFORMANCE_MONITORING_GROUP: &str = "performanceMonitoring";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if destructive_marker_exists(manager).await? {
        return Ok(());
    }
    rename_translation_entry(manager, DASHBOARD_GROUP, "stats.kpi.ttfb", "stats.kpi.firstByte").await?;
    rename_translation_entry(manager, PERFORMANCE_MONITORING_GROUP, "columns.avgTtfb", "columns.avgFirstByte").await?;
    rename_translation_entry(manager, PERFORMANCE_MONITORING_GROUP, "columns.p90P99Ttfb", "columns.p90P99FirstByte").await?;
    rename_translation_entry(manager, PERFORMANCE_MONITORING_GROUP, "columns.latencyTtfb", "columns.latencyStages").await?;
    mark_destructive_applied(manager).await
}

async fn rename_translation_entry(manager: &SchemaManager<'_>, group_key: &str, old_item_key: &str, new_item_key: &str) -> Result<(), DbErr> {
    delete_legacy_duplicates(manager, group_key, old_item_key, new_item_key).await?;
    manager
        .execute(
            Query::update()
                .table(TranslationEntries::Table)
                .value(TranslationEntries::ItemKey, new_item_key)
                .value(TranslationEntries::UpdatedAt, Expr::current_timestamp())
                .and_where(base_filter(group_key, old_item_key))
                .and_where(
                    Expr::cust_with_values(
                        "NOT EXISTS (SELECT 1 FROM translation_entries existing WHERE existing.namespace = $1 AND existing.group_key = $2 AND existing.item_key = $3 AND existing.lang_code = translation_entries.lang_code)",
                        [
                            Value::from(ADMIN_NAMESPACE),
                            Value::from(group_key.to_owned()),
                            Value::from(new_item_key.to_owned()),
                        ],
                    ),
                )
                .to_owned(),
        )
        .await?;
    Ok(())
}

async fn delete_legacy_duplicates(manager: &SchemaManager<'_>, group_key: &str, old_item_key: &str, new_item_key: &str) -> Result<(), DbErr> {
    manager
        .execute(
            Query::delete()
                .from_table(TranslationEntries::Table)
                .and_where(base_filter(group_key, old_item_key))
                .and_where(
                    Expr::cust_with_values(
                        "EXISTS (SELECT 1 FROM translation_entries existing WHERE existing.namespace = $1 AND existing.group_key = $2 AND existing.item_key = $3 AND existing.lang_code = translation_entries.lang_code)",
                        [
                            Value::from(ADMIN_NAMESPACE),
                            Value::from(group_key.to_owned()),
                            Value::from(new_item_key.to_owned()),
                        ],
                    ),
                )
                .to_owned(),
        )
        .await?;
    Ok(())
}

fn base_filter(group_key: &str, item_key: &str) -> SimpleExpr {
    Expr::col(TranslationEntries::Namespace)
        .eq(ADMIN_NAMESPACE)
        .and(Expr::col(TranslationEntries::GroupKey).eq(group_key))
        .and(Expr::col(TranslationEntries::ItemKey).eq(item_key))
}

async fn destructive_marker_exists(manager: &SchemaManager<'_>) -> Result<bool, DbErr> {
    if !manager.has_table(MIGRATION_TABLE).await? {
        return Ok(false);
    }
    seaql_migrations::Entity::find()
        .filter(seaql_migrations::Column::Version.eq(DESTRUCTIVE_VERSION))
        .one(manager.get_connection())
        .await
        .map(|record| record.is_some())
}

async fn mark_destructive_applied(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    create_migration_table(manager).await?;
    if destructive_marker_exists(manager).await? {
        return Ok(());
    }
    seaql_migrations::Entity::insert(seaql_migrations::ActiveModel {
        version: ActiveValue::Set(DESTRUCTIVE_VERSION.to_owned()),
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

#[derive(Clone, Copy, DeriveIden)]
enum TranslationEntries {
    Table,
    Namespace,
    GroupKey,
    ItemKey,
    UpdatedAt,
}

#[cfg(test)]
mod tests {
    use sea_orm_migration::prelude::*;

    use super::{TranslationEntries, base_filter};

    #[test]
    fn base_filter_targets_old_admin_translation_key() {
        let sql = Query::select()
            .expr(Expr::val(1))
            .from(TranslationEntries::Table)
            .and_where(base_filter("performanceMonitoring", "columns.p90P99Ttfb"))
            .to_string(PostgresQueryBuilder);

        assert!(sql.contains("\"namespace\" = 'admin'"));
        assert!(sql.contains("\"group_key\" = 'performanceMonitoring'"));
        assert!(sql.contains("\"item_key\" = 'columns.p90P99Ttfb'"));
    }
}
