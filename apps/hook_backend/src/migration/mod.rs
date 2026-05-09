use sea_orm_migration::prelude::*;

mod m20260508_000001_create_baseline;
mod m20260510_000002_create_groups_and_api_tokens;
mod m20260510_000003_extend_api_tokens;
mod m20260510_000004_seed_admin_tokens;
mod m20260510_000005_system_group_and_model_group_pricing;
mod m20260510_000006_create_billing_group_model_bindings;
mod m20260510_000007_create_system_settings;

mod defaults;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260508_000001_create_baseline::Migration),
            Box::new(m20260510_000002_create_groups_and_api_tokens::Migration),
            Box::new(m20260510_000003_extend_api_tokens::Migration),
            Box::new(m20260510_000004_seed_admin_tokens::Migration),
            Box::new(m20260510_000005_system_group_and_model_group_pricing::Migration),
            Box::new(m20260510_000006_create_billing_group_model_bindings::Migration),
            Box::new(m20260510_000007_create_system_settings::Migration),
        ]
    }
}
