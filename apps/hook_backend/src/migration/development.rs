use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DatabaseConnection;

use super::baseline;

const MIGRATION_TABLE: &str = "seaql_migrations";

pub async fn apply(connection: &DatabaseConnection) -> Result<(), DbErr> {
    let manager = SchemaManager::new(connection);
    reset(&manager).await?;
    baseline::apply(&manager).await
}

pub async fn drop(connection: &DatabaseConnection) -> Result<(), DbErr> {
    let manager = SchemaManager::new(connection);
    reset(&manager).await
}

pub async fn status(connection: &DatabaseConnection) -> Result<BaselineStatus, DbErr> {
    let manager = SchemaManager::new(connection);
    let existing_tables = existing_tables(&manager).await?;
    Ok(BaselineStatus {
        existing_tables,
        total_tables: table_names().len(),
    })
}

async fn reset(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    baseline::drop_tables(manager).await?;
    manager
        .drop_table(Table::drop().table(Alias::new(MIGRATION_TABLE)).if_exists().to_owned())
        .await
}

async fn existing_tables(manager: &SchemaManager<'_>) -> Result<Vec<&'static str>, DbErr> {
    let mut existing = Vec::new();
    for table_name in table_names() {
        if manager.has_table(table_name).await? {
            existing.push(table_name);
        }
    }
    Ok(existing)
}

pub fn table_names() -> Vec<&'static str> {
    vec![
        "users",
        "user_group_memberships",
        "user_identities",
        "user_password_reset_tokens",
        "roles",
        "api_permissions",
        "menu_sections",
        "menu_items",
        "menu_api_permissions",
        "role_menu_permissions",
        "role_api_permissions",
        "wallets",
        "wallet_transactions",
        "card_code_types",
        "card_codes",
        "recharge_packages",
        "recharge_orders",
        "payment_channels",
        "global_models",
        "providers",
        "provider_endpoints",
        "provider_api_keys",
        "provider_models",
        "billing_rules",
        "dimension_collectors",
        "provider_cooldowns",
        "billing_group_providers",
        "billing_groups",
        "system_settings",
        "translation_languages",
        "translation_entries",
        "api_tokens",
        "billing_group_models",
        "request_records",
        "request_candidates",
        "scheduled_tasks",
        "scheduled_task_runs",
        "performance_monitoring_snapshots",
        "announcements",
        "support_tickets",
        "support_ticket_messages",
        "support_ticket_email_events",
        "notification_states",
        "usage_flush_batches",
        "model_status_checks",
        "model_status_check_runs",
        "model_status_check_hourly_stats",
    ]
}

pub struct BaselineStatus {
    pub existing_tables: Vec<&'static str>,
    pub total_tables: usize,
}
