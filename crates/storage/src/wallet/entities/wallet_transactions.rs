use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use types::wallet::WalletTransaction;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "wallet_transactions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub wallet_id: String,
    pub category: String,
    pub reason_code: String,
    pub amount: Decimal,
    pub balance_before: Decimal,
    pub balance_after: Decimal,
    pub recharge_balance_before: Decimal,
    pub recharge_balance_after: Decimal,
    pub gift_balance_before: Decimal,
    pub gift_balance_after: Decimal,
    pub link_type: Option<String>,
    pub link_id: Option<String>,
    pub operator_id: Option<String>,
    pub description: Option<String>,
    pub created_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for WalletTransaction {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            wallet_id: value.wallet_id,
            category: value.category,
            reason_code: value.reason_code,
            amount: value.amount,
            balance_before: value.balance_before,
            balance_after: value.balance_after,
            recharge_balance_before: value.recharge_balance_before,
            recharge_balance_after: value.recharge_balance_after,
            gift_balance_before: value.gift_balance_before,
            gift_balance_after: value.gift_balance_after,
            link_type: value.link_type,
            link_id: value.link_id,
            operator_id: value.operator_id,
            description: value.description,
            created_at: value.created_at.to_string(),
        }
    }
}
