use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Schema},
    seaql_migrations,
};
use std::time::{SystemTime, UNIX_EPOCH};

use super::baseline;

const BASELINE_VERSION: &str = "m20260605_000001_initial_stable_baseline";
const MIGRATION_TABLE: &str = "seaql_migrations";
const ADDITIVE_BASELINE_TABLES: &[&str] = &[
    "provider_groups",
    "provider_group_providers",
    "provider_key_groups",
    "provider_key_group_keys",
    "billing_group_provider_groups",
    "billing_group_provider_key_groups",
];
const BASELINE_TABLES: &[&str] = &[
    "user_groups",
    "users",
    "user_group_memberships",
    "user_identities",
    "affiliate_relation_changes",
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
    "global_models",
    "recharge_packages",
    "recharge_orders",
    "affiliate_commissions",
    "payment_channels",
    "payment_callback_records",
    "billing_groups",
    "providers",
    "provider_endpoints",
    "provider_api_keys",
    "provider_groups",
    "provider_group_providers",
    "provider_key_groups",
    "provider_key_group_keys",
    "provider_models",
    "provider_model_costs",
    "billing_rules",
    "dimension_collectors",
    "provider_cooldowns",
    "provider_cooldown_events",
    "billing_group_providers",
    "billing_group_provider_keys",
    "billing_group_provider_groups",
    "billing_group_provider_key_groups",
    "billing_group_user_groups",
    "system_settings",
    "translation_languages",
    "translation_entries",
    "api_tokens",
    "billing_group_models",
    "request_records",
    "dashboard_cost_analysis_buckets",
    "request_candidates",
    "usage_flush_batches",
    "scheduled_tasks",
    "scheduled_task_runs",
    "performance_monitoring_snapshots",
    "model_status_checks",
    "model_status_check_runs",
    "model_status_check_hourly_stats",
    "dashboard_user_usage_buckets",
    "announcements",
    "support_tickets",
    "support_ticket_messages",
    "support_ticket_email_events",
    "notification_states",
];

pub async fn apply(connection: &DatabaseConnection) -> Result<(), DbErr> {
    let manager = SchemaManager::new(connection);
    match baseline_state(&manager).await? {
        BaselineState::Empty => {
            baseline::apply(&manager).await?;
            mark_baseline_applied(&manager).await
        }
        BaselineState::CompleteWithoutMarker => mark_baseline_applied(&manager).await,
        BaselineState::Applied => super::development_additive::apply(&manager).await,
        BaselineState::Inconsistent { existing_tables, total_tables } => Err(DbErr::Migration(format!(
            "inconsistent baseline state: {existing_tables}/{total_tables} baseline tables exist; run `migration status` and fix the schema before applying migrations"
        ))),
    }
}

pub async fn recreate(connection: &DatabaseConnection) -> Result<(), DbErr> {
    let manager = SchemaManager::new(connection);
    reset(&manager).await?;
    baseline::apply(&manager).await?;
    mark_baseline_applied(&manager).await
}

pub async fn drop(connection: &DatabaseConnection) -> Result<(), DbErr> {
    let manager = SchemaManager::new(connection);
    reset(&manager).await
}

pub async fn status(connection: &DatabaseConnection) -> Result<BaselineStatus, DbErr> {
    let manager = SchemaManager::new(connection);
    let existing_tables = existing_tables(&manager).await?;
    let baseline_applied = baseline_marker_exists(&manager).await?;
    Ok(BaselineStatus {
        existing_tables,
        total_tables: table_names().len(),
        baseline_applied,
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
    for table_name in BASELINE_TABLES {
        if manager.has_table(table_name).await? {
            existing.push(*table_name);
        }
    }
    Ok(existing)
}

pub fn table_names() -> Vec<&'static str> {
    BASELINE_TABLES.to_vec()
}

async fn baseline_state(manager: &SchemaManager<'_>) -> Result<BaselineState, DbErr> {
    let existing_tables = existing_tables(manager).await?;
    let marker_exists = baseline_marker_exists(manager).await?;
    Ok(classify_baseline_state(
        existing_tables.len(),
        table_names().len(),
        marker_exists,
        &missing_tables(&existing_tables),
    ))
}

fn classify_baseline_state(existing_tables: usize, total_tables: usize, marker_exists: bool, missing_tables: &[&str]) -> BaselineState {
    match (existing_tables, marker_exists) {
        (0, false) => BaselineState::Empty,
        (count, false) if count == total_tables => BaselineState::CompleteWithoutMarker,
        (count, true) if count == total_tables => BaselineState::Applied,
        (_, true) if only_additive_tables_missing(missing_tables) => BaselineState::Applied,
        _ => BaselineState::Inconsistent { existing_tables, total_tables },
    }
}

fn missing_tables(existing_tables: &[&'static str]) -> Vec<&'static str> {
    let existing = existing_tables.iter().copied().collect::<std::collections::BTreeSet<_>>();
    BASELINE_TABLES.iter().copied().filter(|table| !existing.contains(table)).collect()
}

fn only_additive_tables_missing(missing_tables: &[&str]) -> bool {
    !missing_tables.is_empty() && missing_tables.iter().all(|table| ADDITIVE_BASELINE_TABLES.contains(table))
}

async fn baseline_marker_exists(manager: &SchemaManager<'_>) -> Result<bool, DbErr> {
    if !manager.has_table(MIGRATION_TABLE).await? {
        return Ok(false);
    }

    let record = seaql_migrations::Entity::find()
        .filter(seaql_migrations::Column::Version.eq(BASELINE_VERSION))
        .one(manager.get_connection())
        .await?;
    Ok(record.is_some())
}

async fn mark_baseline_applied(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    create_migration_table(manager).await?;
    if baseline_marker_exists(manager).await? {
        return Ok(());
    }

    seaql_migrations::Entity::insert(seaql_migrations::ActiveModel {
        version: ActiveValue::Set(BASELINE_VERSION.to_owned()),
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

pub struct BaselineStatus {
    pub existing_tables: Vec<&'static str>,
    pub total_tables: usize,
    pub baseline_applied: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BaselineState {
    Empty,
    CompleteWithoutMarker,
    Applied,
    Inconsistent { existing_tables: usize, total_tables: usize },
}

#[cfg(test)]
mod tests {
    use super::{BaselineState, classify_baseline_state};

    #[test]
    fn classifies_empty_baseline_without_marker_as_empty() {
        assert_eq!(classify_baseline_state(0, 5, false, &[]), BaselineState::Empty);
    }

    #[test]
    fn classifies_complete_baseline_without_marker_as_marker_pending() {
        assert_eq!(classify_baseline_state(5, 5, false, &[]), BaselineState::CompleteWithoutMarker);
    }

    #[test]
    fn classifies_complete_marked_baseline_as_applied() {
        assert_eq!(classify_baseline_state(5, 5, true, &[]), BaselineState::Applied);
    }

    #[test]
    fn classifies_marked_baseline_missing_only_additive_tables_as_applied() {
        assert_eq!(
            classify_baseline_state(5, 7, true, &["provider_groups", "billing_group_provider_groups"]),
            BaselineState::Applied
        );
    }

    #[test]
    fn classifies_partial_baseline_as_inconsistent() {
        assert_eq!(
            classify_baseline_state(3, 5, false, &["providers", "api_tokens"]),
            BaselineState::Inconsistent {
                existing_tables: 3,
                total_tables: 5,
            }
        );
    }

    #[test]
    fn classifies_marker_without_tables_as_inconsistent() {
        assert_eq!(
            classify_baseline_state(0, 5, true, &["providers", "api_tokens"]),
            BaselineState::Inconsistent {
                existing_tables: 0,
                total_tables: 5,
            }
        );
    }
}
