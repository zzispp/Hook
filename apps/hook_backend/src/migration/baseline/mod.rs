use sea_orm_migration::prelude::*;

mod card_code_tables;
mod domain_tables;
pub mod iden;
mod indices;
mod model_status_tables;
mod operations_tables;
mod performance_monitoring_tables;
mod provider_key_group_indices;
mod provider_key_group_tables;
mod recharge_tables;
mod request_candidate_tables;
mod scheduler_tables;
mod seed;
pub mod seed_domain;
mod setting_seed;
mod setting_tables;
mod tables;
mod translation_tables;
mod wallet_tables;

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    apply_schema_without_seed(manager).await?;
    seed::seed_defaults(manager).await
}

pub async fn apply_schema_without_seed(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    create_tables(manager).await?;
    create_indices(manager).await
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
