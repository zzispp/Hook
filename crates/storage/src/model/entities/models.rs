use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "models")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub provider_id: String,
    pub global_model_id: String,
    pub provider_model_name: String,
    pub provider_model_mappings: Option<String>,
    pub price_per_request: Option<Decimal>,
    pub tiered_pricing: Option<String>,
    pub supports_vision: Option<bool>,
    pub supports_function_calling: Option<bool>,
    pub supports_streaming: Option<bool>,
    pub supports_extended_thinking: Option<bool>,
    pub supports_image_generation: Option<bool>,
    pub is_active: bool,
    pub is_available: bool,
    pub config: Option<String>,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
