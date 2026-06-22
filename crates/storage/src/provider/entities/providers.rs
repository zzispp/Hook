use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::provider::{Provider, ProviderOrigin};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "providers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub name: String,
    pub provider_type: String,
    pub provider_origin: String,
    pub max_retries: Option<i32>,
    pub request_timeout_seconds: Option<f64>,
    pub stream_first_byte_timeout_seconds: Option<f64>,
    pub stream_idle_timeout_seconds: Option<f64>,
    pub priority: i32,
    pub keep_priority_on_conversion: bool,
    pub enable_format_conversion: bool,
    pub is_active: bool,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for Provider {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            provider_type: value.provider_type,
            provider_origin: ProviderOrigin::try_from(value.provider_origin.as_str()).expect("provider_origin must be valid"),
            quick_import_source: None,
            max_retries: value.max_retries,
            request_timeout_seconds: value.request_timeout_seconds,
            stream_first_byte_timeout_seconds: value.stream_first_byte_timeout_seconds,
            stream_idle_timeout_seconds: value.stream_idle_timeout_seconds,
            priority: value.priority,
            keep_priority_on_conversion: value.keep_priority_on_conversion,
            enable_format_conversion: value.enable_format_conversion,
            is_active: value.is_active,
            created_at: format_timestamp(value.created_at),
            updated_at: format_timestamp(value.updated_at),
        }
    }
}

fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("provider timestamp must format as RFC3339")
}
