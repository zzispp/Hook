use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::group::BillingGroup;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "billing_groups")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub billing_multiplier: rust_decimal::Decimal,
    pub is_active: bool,
    pub is_system: bool,
    pub sort_order: i64,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for BillingGroup {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            code: value.code,
            name: value.name,
            description: value.description,
            billing_multiplier: value.billing_multiplier,
            allowed_model_ids: Vec::new(),
            allowed_provider_ids: Vec::new(),
            allowed_provider_key_ids: Vec::new(),
            visible_user_group_codes: Vec::new(),
            is_active: value.is_active,
            is_system: value.is_system,
            sort_order: value.sort_order,
            created_at: format_timestamp(value.created_at),
            updated_at: format_timestamp(value.updated_at),
        }
    }
}

fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("billing group timestamp must format as RFC3339")
}
