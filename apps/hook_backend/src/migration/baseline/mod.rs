use sea_orm_migration::prelude::*;

mod domain_tables;
pub mod iden;
mod indices;
mod seed;
pub mod seed_domain;
mod tables;
mod wallet_tables;

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    create_tables(manager).await?;
    create_indices(manager).await?;
    seed::seed_defaults(manager).await
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

pub async fn drop_tables(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for table in iden::reversed_tables() {
        manager.drop_table(Table::drop().table(table).if_exists().to_owned()).await?;
    }
    Ok(())
}
