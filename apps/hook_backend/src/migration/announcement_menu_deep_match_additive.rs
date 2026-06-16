use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Schema},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

const ADDITIVE_VERSION: &str = "m20260616_000001_announcement_menu_deep_match";
const MIGRATION_TABLE: &str = "seaql_migrations";
const ANNOUNCEMENTS_MENU_CODE: &str = "announcements";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    sync_announcement_menu(manager).await?;
    mark_additive_applied(manager).await
}

async fn sync_announcement_menu(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let update = announcement_menu_update();
    manager
        .execute(
            Query::update()
                .table(MenuItems::Table)
                .value(MenuItems::DeepMatch, update.deep_match)
                .value(MenuItems::UpdatedAt, Expr::current_timestamp())
                .and_where(Expr::col(MenuItems::Code).eq(update.code))
                .and_where(Expr::col(MenuItems::DeepMatch).ne(update.deep_match))
                .to_owned(),
        )
        .await?;
    Ok(())
}

fn announcement_menu_update() -> MenuUpdate {
    MenuUpdate {
        code: ANNOUNCEMENTS_MENU_CODE,
        deep_match: true,
    }
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

struct MenuUpdate {
    code: &'static str,
    deep_match: bool,
}

#[derive(Clone, Copy, DeriveIden)]
enum MenuItems {
    Table,
    Code,
    DeepMatch,
    UpdatedAt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_update_targets_user_announcement_deep_match() {
        let update = announcement_menu_update();

        assert_eq!(update.code, "announcements");
        assert!(update.deep_match);
    }
}
