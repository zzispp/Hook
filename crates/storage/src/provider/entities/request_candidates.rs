use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "request_candidates")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub request_id: String,
    pub token_id: Option<String>,
    pub group_code: Option<String>,
    pub global_model_id: Option<String>,
    pub provider_id: Option<String>,
    pub provider_name_snapshot: Option<String>,
    pub endpoint_id: Option<String>,
    pub endpoint_name_snapshot: Option<String>,
    pub key_id: Option<String>,
    pub key_name_snapshot: Option<String>,
    pub key_preview_snapshot: Option<String>,
    pub client_api_format: String,
    pub provider_api_format: Option<String>,
    pub needs_conversion: bool,
    pub is_stream: bool,
    pub provider_request_headers: Option<String>,
    pub provider_request_body: Option<String>,
    pub provider_response_headers: Option<String>,
    pub provider_response_body: Option<String>,
    pub candidate_index: i32,
    pub retry_index: i32,
    pub status: String,
    pub skip_reason: Option<String>,
    pub status_code: Option<i32>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
    pub service_tier: Option<String>,
    pub input_cost: Option<Decimal>,
    pub output_cost: Option<Decimal>,
    pub cache_creation_cost: Option<Decimal>,
    pub cache_read_cost: Option<Decimal>,
    pub request_cost: Option<Decimal>,
    pub input_price_per_million: Option<Decimal>,
    pub output_price_per_million: Option<Decimal>,
    pub cache_creation_price_per_million: Option<Decimal>,
    pub cache_read_price_per_million: Option<Decimal>,
    pub cost_currency: Option<String>,
    pub token_cost: Option<Decimal>,
    pub base_cost: Option<Decimal>,
    pub total_cost: Option<Decimal>,
    pub billing_multiplier: Option<Decimal>,
    pub latency_ms: Option<i64>,
    pub first_byte_time_ms: Option<i64>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
    pub error_param: Option<String>,
    pub created_at: TimeDateTimeWithTimeZone,
    pub started_at: Option<TimeDateTimeWithTimeZone>,
    pub finished_at: Option<TimeDateTimeWithTimeZone>,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
