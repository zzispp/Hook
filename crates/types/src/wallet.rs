mod admin;
mod ledger;

pub use admin::*;
pub use ledger::*;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalletId(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wallet {
    pub id: WalletId,
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
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalletTransaction {
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
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WalletAdjustment {
    pub wallet_id: String,
    pub amount: Decimal,
    pub balance_type: WalletBalanceType,
    pub adjustment_type: WalletAdjustmentType,
    pub operator_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WalletRecharge {
    pub wallet_id: String,
    pub amount: Decimal,
    pub operator_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WalletAdjustmentType {
    Increase,
    Deduct,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WalletBalanceType {
    Recharge,
    Gift,
}
