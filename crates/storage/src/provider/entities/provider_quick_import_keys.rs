use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "provider_quick_import_keys")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub provider_id: String,
    pub source_id: String,
    pub key_id: String,
    pub upstream_token_id: String,
    pub upstream_token_name: String,
    pub upstream_masked_key: String,
    pub upstream_group_id: Option<String>,
    pub upstream_group: Option<String>,
    pub upstream_group_ratio: Decimal,
    pub effective_cost_multiplier: Decimal,
    pub sync_statuses: String,
    pub last_sync_error: Option<String>,
    pub last_synced_at: Option<TimeDateTimeWithTimeZone>,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
