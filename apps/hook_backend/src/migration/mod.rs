use sea_orm_migration::prelude::*;

mod m20260508_000001_create_baseline;
mod m20260508_000002_add_rbac_timestamps;
mod m20260508_000003_add_wallets;
mod m20260508_000004_wallet_user_only_access;

mod defaults;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260508_000001_create_baseline::Migration),
            Box::new(m20260508_000002_add_rbac_timestamps::Migration),
            Box::new(m20260508_000003_add_wallets::Migration),
            Box::new(m20260508_000004_wallet_user_only_access::Migration),
        ]
    }
}
