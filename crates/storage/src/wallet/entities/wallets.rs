use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use types::wallet::{Wallet, WalletId};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "wallets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub user_id: String,
    pub recharge_balance: Decimal,
    pub gift_balance: Decimal,
    pub currency: String,
    pub status: String,
    pub limit_mode: String,
    pub total_recharged: Decimal,
    pub total_consumed: Decimal,
    pub total_refunded: Decimal,
    pub total_adjusted: Decimal,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for Wallet {
    fn from(value: Model) -> Self {
        Self {
            id: WalletId(value.id),
            user_id: value.user_id,
            recharge_balance: value.recharge_balance,
            gift_balance: value.gift_balance,
            currency: value.currency,
            status: value.status,
            limit_mode: value.limit_mode,
            total_recharged: value.total_recharged,
            total_consumed: value.total_consumed,
            total_refunded: value.total_refunded,
            total_adjusted: value.total_adjusted,
            created_at: value.created_at.to_string(),
            updated_at: value.updated_at.to_string(),
        }
    }
}
