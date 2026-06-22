use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "scheduled_tasks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub code: String,
    pub enabled: bool,
    pub interval_seconds: i64,
    pub lease_seconds: i64,
    pub config: String,
    pub next_run_at: TimeDateTimeWithTimeZone,
    pub locked_until: Option<TimeDateTimeWithTimeZone>,
    pub locked_by: Option<String>,
    pub last_started_at: Option<TimeDateTimeWithTimeZone>,
    pub last_finished_at: Option<TimeDateTimeWithTimeZone>,
    pub last_status: Option<String>,
    pub last_duration_ms: Option<i64>,
    pub last_error: Option<String>,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
