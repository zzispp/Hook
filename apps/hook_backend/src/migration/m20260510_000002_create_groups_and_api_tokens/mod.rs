use sea_orm_migration::{MigrationName, prelude::*};

mod definitions;
mod iden;
mod seed;
mod tables;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260510_000002_create_groups_and_api_tokens"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_tables(manager).await?;
        create_indices(manager).await?;
        seed::seed_defaults(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        seed::remove_defaults(manager).await?;
        drop_tables(manager).await
    }
}

async fn create_tables(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for table in tables::group_token_tables() {
        manager.create_table(table).await?;
    }
    Ok(())
}

async fn create_indices(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for index in tables::group_token_indices() {
        manager.create_index(index).await?;
    }
    Ok(())
}

async fn drop_tables(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for table in iden::reversed_tables() {
        manager.drop_table(Table::drop().table(table).if_exists().to_owned()).await?;
    }
    Ok(())
}
