use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::recharge::RechargePackage;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "recharge_packages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub recharge_amount: rust_decimal::Decimal,
    pub gift_amount: rust_decimal::Decimal,
    pub status: String,
    pub sort_order: i64,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for RechargePackage {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            recharge_amount: value.recharge_amount,
            gift_amount: value.gift_amount,
            status: value.status,
            sort_order: value.sort_order,
            created_at: format_timestamp(value.created_at),
            updated_at: format_timestamp(value.updated_at),
        }
    }
}

fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("recharge package timestamp must format as RFC3339")
}
