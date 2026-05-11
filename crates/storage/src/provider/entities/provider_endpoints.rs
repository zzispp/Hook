use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "provider_endpoints")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub provider_id: String,
    pub api_format: String,
    pub base_url: String,
    pub custom_path: Option<String>,
    pub max_retries: Option<i32>,
    pub is_active: bool,
    pub format_acceptance_config: Option<String>,
    pub header_rules: Option<String>,
    pub body_rules: Option<String>,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
