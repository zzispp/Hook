use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "provider_cooldown_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub provider_id: String,
    pub provider_name_snapshot: String,
    pub status_code: i32,
    pub observed_count: i64,
    pub threshold_count: i64,
    pub window_seconds: i64,
    pub cooldown_seconds: i64,
    pub triggered_at: TimeDateTimeWithTimeZone,
    pub cooldown_until: TimeDateTimeWithTimeZone,
    pub request_id: String,
    pub candidate_index: i32,
    pub retry_index: i32,
    pub endpoint_id: Option<String>,
    pub endpoint_name_snapshot: Option<String>,
    pub key_id: Option<String>,
    pub key_name_snapshot: Option<String>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
    pub error_param: Option<String>,
    pub created_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
