use sea_orm_migration::prelude::*;

mod iden;
mod indices;
mod seed;
mod tables;

#[derive(DeriveMigrationName)]
pub struct Migration;

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
