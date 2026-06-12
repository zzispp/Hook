use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Schema},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const ADDITIVE_VERSION: &str = "m20260612_000001_auth_layout_site_name_i18n";
const MIGRATION_TABLE: &str = "seaql_migrations";
const AUTH_NAMESPACE: &str = "auth";
const LAYOUT_GROUP: &str = "layout";
const SECTION_TITLE_ITEM: &str = "sectionTitle";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    super::translation_seed_sync::seed_missing_translations(manager).await?;
    repair_auth_section_title(manager).await?;
    mark_additive_applied(manager).await
}

async fn repair_auth_section_title(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for update in section_title_updates() {
        manager
            .execute(
                Query::update()
                    .table(TranslationEntries::Table)
                    .value(TranslationEntries::Value, update.next)
                    .value(TranslationEntries::UpdatedAt, Expr::current_timestamp())
                    .and_where(section_title_filter(update.lang_code))
                    .and_where(Expr::col(TranslationEntries::Value).eq(update.previous))
                    .to_owned(),
            )
            .await?;
    }
    Ok(())
}

fn section_title_filter(lang_code: &'static str) -> SimpleExpr {
    Expr::col(TranslationEntries::Namespace)
        .eq(AUTH_NAMESPACE)
        .and(Expr::col(TranslationEntries::GroupKey).eq(LAYOUT_GROUP))
        .and(Expr::col(TranslationEntries::ItemKey).eq(SECTION_TITLE_ITEM))
        .and(Expr::col(TranslationEntries::LangCode).eq(lang_code))
}

fn section_title_updates() -> [SectionTitleUpdate; 2] {
    [
        SectionTitleUpdate {
            lang_code: "cn",
            previous: "安全访问 Hook",
            next: "安全访问 {{siteName}}",
        },
        SectionTitleUpdate {
            lang_code: "en",
            previous: "Secure access to Hook",
            next: "Secure access to {{siteName}}",
        },
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

struct SectionTitleUpdate {
    lang_code: &'static str,
    previous: &'static str,
    next: &'static str,
}

#[derive(Clone, Copy, DeriveIden)]
enum TranslationEntries {
    Table,
    Namespace,
    GroupKey,
    ItemKey,
    LangCode,
    Value,
    UpdatedAt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_title_updates_only_target_legacy_auth_copy() {
        let updates = section_title_updates();

        assert_eq!(updates[0].previous, "安全访问 Hook");
        assert_eq!(updates[0].next, "安全访问 {{siteName}}");
        assert_eq!(updates[1].previous, "Secure access to Hook");
        assert_eq!(updates[1].next, "Secure access to {{siteName}}");
    }
}
