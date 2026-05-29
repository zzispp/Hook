use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "model_status_checks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub global_model_id: String,
    pub api_format: String,
    pub api_token_id: String,
    pub interval_seconds: i64,
    pub enabled: bool,
    pub next_due_at: TimeDateTimeWithTimeZone,
    pub locked_until: Option<TimeDateTimeWithTimeZone>,
    pub last_status: Option<String>,
    pub last_checked_at: Option<TimeDateTimeWithTimeZone>,
    pub last_latency_ms: Option<i64>,
    pub last_message: Option<String>,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
