use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::card_code::CardCode;

use super::card_code_types;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "card_codes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub code: String,
    pub batch_no: String,
    pub type_id: String,
    pub type_name: String,
    pub recharge_amount: rust_decimal::Decimal,
    pub gift_amount: rust_decimal::Decimal,
    pub currency: String,
    pub status: String,
    pub remark: Option<String>,
    pub expires_at: Option<TimeDateTimeWithTimeZone>,
    pub created_by_user_id: Option<String>,
    pub created_by_username: Option<String>,
    pub created_ip: Option<String>,
    pub used_by_user_id: Option<String>,
    pub used_by_username: Option<String>,
    pub used_ip: Option<String>,
    pub used_at: Option<TimeDateTimeWithTimeZone>,
    pub wallet_id: Option<String>,
    pub wallet_transaction_id: Option<String>,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "card_code_types::Entity", from = "Column::TypeId", to = "card_code_types::Column::Id")]
    Type,
}

impl Related<card_code_types::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Type.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for CardCode {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            code: value.code,
            batch_no: value.batch_no,
            type_id: value.type_id,
            type_name: value.type_name,
            recharge_amount: value.recharge_amount,
            gift_amount: value.gift_amount,
            currency: value.currency,
            status: value.status,
            remark: value.remark,
            expires_at: value.expires_at.map(format_timestamp),
            created_at: format_timestamp(value.created_at),
            updated_at: format_timestamp(value.updated_at),
            created_by_user_id: value.created_by_user_id,
            created_by_username: value.created_by_username,
            created_ip: value.created_ip,
            used_by_user_id: value.used_by_user_id,
            used_by_username: value.used_by_username,
            used_ip: value.used_ip,
            used_at: value.used_at.map(format_timestamp),
            wallet_id: value.wallet_id,
            wallet_transaction_id: value.wallet_transaction_id,
        }
    }
}

fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("card code timestamp must format as RFC3339")
}
