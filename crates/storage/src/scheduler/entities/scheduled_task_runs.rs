use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "scheduled_task_runs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub task_code: String,
    pub status: String,
    pub started_at: TimeDateTimeWithTimeZone,
    pub finished_at: Option<TimeDateTimeWithTimeZone>,
    pub duration_ms: Option<i64>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
