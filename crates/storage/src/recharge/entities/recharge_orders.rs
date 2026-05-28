use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::recharge::RechargeOrder;

use crate::user::UserEntity as Users;

use super::recharge_packages;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "recharge_orders")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub order_no: String,
    pub user_id: String,
    pub package_id: Option<String>,
    pub package_name: String,
    pub recharge_amount: rust_decimal::Decimal,
    pub gift_amount: rust_decimal::Decimal,
    pub total_arrival_amount: rust_decimal::Decimal,
    pub payable_amount: rust_decimal::Decimal,
    pub status: String,
    pub payment_channel_code: Option<String>,
    pub payment_channel_name: Option<String>,
    pub payment_method: Option<String>,
    pub provider_trade_no: Option<String>,
    pub payment_request_json: Option<String>,
    pub refund_status: Option<String>,
    pub refund_amount: Option<rust_decimal::Decimal>,
    pub paid_at: Option<TimeDateTimeWithTimeZone>,
    pub refunded_at: Option<TimeDateTimeWithTimeZone>,
    pub expires_at: TimeDateTimeWithTimeZone,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "Users", from = "Column::UserId", to = "crate::user::UserColumn::Id")]
    User,
    #[sea_orm(belongs_to = "recharge_packages::Entity", from = "Column::PackageId", to = "recharge_packages::Column::Id")]
    Package,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<Users> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<recharge_packages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Package.def()
    }
}

impl Model {
    pub fn into_response(self, username: String, user_email: String) -> RechargeOrder {
        RechargeOrder {
            id: self.id,
            order_no: self.order_no,
            user_id: self.user_id,
            username,
            user_email,
            package_id: self.package_id,
            package_name: self.package_name,
            recharge_amount: self.recharge_amount,
            gift_amount: self.gift_amount,
            total_arrival_amount: self.total_arrival_amount,
            payable_amount: self.payable_amount,
            status: self.status,
            payment_channel_code: self.payment_channel_code,
            payment_channel_name: self.payment_channel_name,
            payment_method: self.payment_method,
            provider_trade_no: self.provider_trade_no,
            payment_request_json: self.payment_request_json.and_then(|value| serde_json::from_str(&value).ok()),
            refund_status: self.refund_status,
            refund_amount: self.refund_amount,
            paid_at: self.paid_at.map(format_timestamp),
            refunded_at: self.refunded_at.map(format_timestamp),
            expires_at: format_timestamp(self.expires_at),
            created_at: format_timestamp(self.created_at),
            updated_at: format_timestamp(self.updated_at),
        }
    }
}

fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("recharge order timestamp must format as RFC3339")
}
