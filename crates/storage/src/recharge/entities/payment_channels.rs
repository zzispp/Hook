use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::recharge::PaymentChannel;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "payment_channels")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub code: String,
    pub name: String,
    pub enabled: bool,
    pub registered_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for PaymentChannel {
    fn from(value: Model) -> Self {
        Self {
            code: value.code,
            name: value.name,
            enabled: value.enabled,
            registered_at: format_timestamp(value.registered_at),
            updated_at: format_timestamp(value.updated_at),
        }
    }
}

fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("payment channel timestamp must format as RFC3339")
}
