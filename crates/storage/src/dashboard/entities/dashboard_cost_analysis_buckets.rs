use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "dashboard_cost_analysis_buckets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub bucket_started_at: TimeDateTimeWithTimeZone,
    pub bucket_ended_at: TimeDateTimeWithTimeZone,
    pub dimension_kind: String,
    pub dimension_id: String,
    pub dimension_name: Option<String>,
    pub shard: i32,
    pub request_count: i64,
    pub success_count: i64,
    pub failed_count: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cache_read_tokens: i64,
    pub total_tokens: i64,
    pub total_cost: Decimal,
    pub upstream_total_cost: Decimal,
    pub cache_read_cost: Decimal,
    pub cache_creation_cost: Decimal,
    pub estimated_full_cost: Decimal,
    pub total_latency_ms: i64,
    pub latency_sample_count: i64,
    pub response_headers_total_ms: i64,
    pub response_headers_sample_count: i64,
    pub first_byte_total_ms: i64,
    pub first_byte_sample_count: i64,
    pub first_sse_event_total_ms: i64,
    pub first_sse_event_sample_count: i64,
    pub first_output_total_ms: i64,
    pub first_output_sample_count: i64,
    pub sse_to_output_total_ms: i64,
    pub sse_to_output_sample_count: i64,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
