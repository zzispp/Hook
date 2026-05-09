use sea_orm_migration::{MigrationName, prelude::*};

mod definitions;
mod iden;
mod seed;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260510_000004_seed_admin_tokens"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        seed::seed_defaults(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        seed::remove_defaults(manager).await
    }
}
