use rust_decimal::Decimal;
use types::wallet::Wallet;

#[derive(Clone, Debug, PartialEq)]
pub struct WalletLedgerRecordInput {
    pub wallet: Wallet,
    pub transaction: WalletTransactionRecordInput,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WalletTransactionRecordInput {
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
}
