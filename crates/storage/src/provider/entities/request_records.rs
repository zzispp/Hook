use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "request_records")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub request_id: String,
    pub token_id: Option<String>,
    pub user_id_snapshot: Option<String>,
    pub username_snapshot: Option<String>,
    pub token_name_snapshot: Option<String>,
    pub token_prefix_snapshot: Option<String>,
    pub group_code: Option<String>,
    pub global_model_id: Option<String>,
    pub model_name_snapshot: Option<String>,
    pub provider_id: Option<String>,
    pub provider_name_snapshot: Option<String>,
    pub endpoint_id: Option<String>,
    pub key_id: Option<String>,
    pub provider_key_name_snapshot: Option<String>,
    pub provider_key_preview_snapshot: Option<String>,
    pub client_api_format: String,
    pub provider_api_format: Option<String>,
    pub request_type: String,
    pub is_stream: bool,
    pub has_failover: bool,
    pub has_retry: bool,
    pub status: String,
    pub billing_status: String,
    pub client_status_code: Option<i32>,
    pub client_error_type: Option<String>,
    pub client_error_message: Option<String>,
    pub termination_origin: Option<String>,
    pub termination_reason: Option<String>,
    pub stream_end_reason: Option<String>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
    pub cost_currency: Option<String>,
    pub token_cost: Option<Decimal>,
    pub base_cost: Option<Decimal>,
    pub total_cost: Option<Decimal>,
    pub billing_multiplier: Option<Decimal>,
    pub first_byte_time_ms: Option<i64>,
    pub total_latency_ms: Option<i64>,
    pub candidate_count: i64,
    pub request_headers: Option<String>,
    pub request_body: Option<String>,
    pub client_response_headers: Option<String>,
    pub client_response_body: Option<String>,
    pub created_at: TimeDateTimeWithTimeZone,
    pub started_at: Option<TimeDateTimeWithTimeZone>,
    pub finished_at: Option<TimeDateTimeWithTimeZone>,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
