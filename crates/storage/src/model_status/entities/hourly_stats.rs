use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "model_status_check_hourly_stats")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub check_id: String,
    pub bucket_started_at: TimeDateTimeWithTimeZone,
    pub total_count: i64,
    pub available_count: i64,
    pub degraded_count: i64,
    pub failed_count: i64,
    pub error_count: i64,
    pub latency_sum_ms: i64,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
