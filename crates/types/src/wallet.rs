use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AdminWalletListFilters {
    pub search: Option<String>,
    pub status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WalletSummaryResponse {
    pub id: String,
    pub user_id: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub balance: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub recharge_balance: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub gift_balance: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub refundable_balance: Decimal,
    pub currency: String,
    pub status: String,
    pub limit_mode: String,
    pub unlimited: bool,
    pub created_at: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_recharged: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_consumed: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_refunded: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_adjusted: Decimal,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WalletBalanceResponse {
    pub wallet: WalletSummaryResponse,
    pub unlimited: bool,
    pub limit_mode: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub balance: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub recharge_balance: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub gift_balance: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub refundable_balance: Decimal,
    pub currency: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WalletTransactionResponse {
    pub id: String,
    pub category: String,
    pub reason_code: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub balance_before: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub balance_after: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub recharge_balance_before: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub recharge_balance_after: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub gift_balance_before: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub gift_balance_after: Decimal,
    pub link_type: Option<String>,
    pub link_id: Option<String>,
    pub operator_id: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WalletTransactionsResponse {
    pub wallet: WalletSummaryResponse,
    pub items: Vec<WalletTransactionResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminWalletResponse {
    pub id: String,
    pub user_id: String,
    pub owner_name: String,
    pub owner_email: String,
    pub owner_type: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub balance: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub recharge_balance: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub gift_balance: Decimal,
    pub currency: String,
    pub status: String,
    pub limit_mode: String,
    pub unlimited: bool,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_recharged: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_consumed: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_refunded: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_adjusted: Decimal,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminWalletListResponse {
    pub items: Vec<AdminWalletResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminWalletTransactionsResponse {
    pub wallet: AdminWalletResponse,
    pub items: Vec<WalletTransactionResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct AdminWalletAdjustmentPayload {
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    pub balance_type: WalletBalanceTypePayload,
    pub adjustment_type: WalletAdjustmentTypePayload,
    pub description: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletBalanceTypePayload {
    Recharge,
    Gift,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletAdjustmentTypePayload {
    Increase,
    Deduct,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdminWalletAdjustmentResponse {
    pub transaction: WalletTransactionResponse,
}

impl From<Wallet> for WalletSummaryResponse {
    fn from(value: Wallet) -> Self {
        let recharge_balance = value.recharge_balance;
        let gift_balance = value.gift_balance;
        let balance = recharge_balance + gift_balance;
        let unlimited = value.limit_mode == "unlimited";
        Self {
            id: value.id.0,
            user_id: value.user_id,
            balance,
            recharge_balance,
            gift_balance,
            refundable_balance: recharge_balance,
            currency: value.currency,
            status: value.status,
            limit_mode: value.limit_mode,
            unlimited,
            created_at: value.created_at,
            total_recharged: value.total_recharged,
            total_consumed: value.total_consumed,
            total_refunded: value.total_refunded,
            total_adjusted: value.total_adjusted,
            updated_at: value.updated_at,
        }
    }
}

impl From<WalletSummaryResponse> for WalletBalanceResponse {
    fn from(wallet: WalletSummaryResponse) -> Self {
        Self {
            unlimited: wallet.unlimited,
            limit_mode: wallet.limit_mode.clone(),
            balance: wallet.balance,
            recharge_balance: wallet.recharge_balance,
            gift_balance: wallet.gift_balance,
            refundable_balance: wallet.refundable_balance,
            currency: wallet.currency.clone(),
            wallet,
        }
    }
}

impl From<WalletTransaction> for WalletTransactionResponse {
    fn from(value: WalletTransaction) -> Self {
        Self {
            id: value.id,
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
            created_at: value.created_at,
        }
    }
}

impl From<WalletBalanceTypePayload> for WalletBalanceType {
    fn from(value: WalletBalanceTypePayload) -> Self {
        match value {
            WalletBalanceTypePayload::Recharge => Self::Recharge,
            WalletBalanceTypePayload::Gift => Self::Gift,
        }
    }
}

impl From<WalletAdjustmentTypePayload> for WalletAdjustmentType {
    fn from(value: WalletAdjustmentTypePayload) -> Self {
        match value {
            WalletAdjustmentTypePayload::Increase => Self::Increase,
            WalletAdjustmentTypePayload::Deduct => Self::Deduct,
        }
    }
}
