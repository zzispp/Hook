use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "provider_quick_import_sources")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub provider_id: String,
    pub source_kind: String,
    pub base_url: String,
    pub encrypted_system_access_token: String,
    pub email: String,
    pub encrypted_password: String,
    pub encrypted_auth_token: String,
    pub encrypted_refresh_token: String,
    pub token_expires_at: Option<TimeDateTimeWithTimeZone>,
    pub user_id: String,
    pub recharge_multiplier: Decimal,
    pub auto_sync_enabled: bool,
    pub cost_sync_mode: String,
    pub upstream_anomaly_action: String,
    pub token_deleted_action: String,
    pub token_disabled_action: String,
    pub group_removed_action: String,
    pub group_changed_action: String,
    pub key_unavailable_action: String,
    pub model_removed_action: String,
    pub fetch_failure_action: String,
    pub fetch_failure_disable_threshold: i32,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    pub last_synced_at: Option<TimeDateTimeWithTimeZone>,
    pub consecutive_failures: i32,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
