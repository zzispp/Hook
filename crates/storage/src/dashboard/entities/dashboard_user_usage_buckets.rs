use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "dashboard_user_usage_buckets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub bucket_granularity: String,
    pub bucket_started_at: TimeDateTimeWithTimeZone,
    pub bucket_ended_at: TimeDateTimeWithTimeZone,
    pub user_id: String,
    pub username: Option<String>,
    pub request_count: i64,
    pub success_count: i64,
    pub failed_count: i64,
    pub total_tokens: i64,
    pub total_cost: Decimal,
    pub total_latency_ms: i64,
    pub latency_sample_count: i64,
    pub response_headers_total_ms: i64,
    pub response_headers_sample_count: i64,
    pub first_byte_total_ms: i64,
    pub first_byte_sample_count: i64,
    pub first_token_total_ms: i64,
    pub first_token_sample_count: i64,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
