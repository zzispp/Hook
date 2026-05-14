use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::card_code::CardCodeType;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "card_code_types")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub name: String,
    pub balance_type: String,
    pub status: String,
    pub remark: Option<String>,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for CardCodeType {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            balance_type: value.balance_type,
            status: value.status,
            remark: value.remark,
            created_at: format_timestamp(value.created_at),
            updated_at: format_timestamp(value.updated_at),
        }
    }
}

fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("card code type timestamp must format as RFC3339")
}
