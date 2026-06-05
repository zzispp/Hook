use sea_orm::entity::prelude::*;

use super::recharge_orders;
use crate::{user::UserEntity as Users, wallet::wallet_transaction_records};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "affiliate_commissions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub referrer_user_id: String,
    pub referred_user_id: String,
    #[sea_orm(unique)]
    pub recharge_order_id: String,
    pub payable_amount: rust_decimal::Decimal,
    pub commission_percent: rust_decimal::Decimal,
    pub commission_amount: rust_decimal::Decimal,
    pub wallet_transaction_id: Option<String>,
    pub status: String,
    pub failure_reason: Option<String>,
    pub created_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "Users", from = "Column::ReferrerUserId", to = "crate::user::UserColumn::Id")]
    Referrer,
    #[sea_orm(belongs_to = "Users", from = "Column::ReferredUserId", to = "crate::user::UserColumn::Id")]
    Referred,
    #[sea_orm(belongs_to = "recharge_orders::Entity", from = "Column::RechargeOrderId", to = "recharge_orders::Column::Id")]
    RechargeOrder,
    #[sea_orm(
        belongs_to = "wallet_transaction_records::Entity",
        from = "Column::WalletTransactionId",
        to = "wallet_transaction_records::Column::Id"
    )]
    WalletTransaction,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<Users> for Entity {
    fn to() -> RelationDef {
        Relation::Referrer.def()
    }
}

impl Related<recharge_orders::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RechargeOrder.def()
    }
}

impl Related<wallet_transaction_records::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WalletTransaction.def()
    }
}
