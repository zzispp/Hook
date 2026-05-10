use sea_orm_migration::{MigrationName, prelude::*};

mod domain_tables;
mod iden;
mod indices;
mod seed;
mod seed_domain;
mod tables;
mod wallet_tables;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260508_000001_create_baseline"
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
        drop_tables(manager).await
    }
}

async fn create_tables(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for table in tables::baseline_tables() {
        manager.create_table(table).await?;
    }
    Ok(())
}

async fn create_indices(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for index in indices::baseline_indices() {
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
