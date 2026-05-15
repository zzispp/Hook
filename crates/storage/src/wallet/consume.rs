use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, TransactionTrait};
use types::wallet::{Wallet, WalletTransaction};

use super::{
    WALLET_CONSUME_INSUFFICIENT_BALANCE, WalletConsumeRecordInput, WalletLedgerRecordInput, WalletRecord, WalletStore, WalletTransactionRecordInput,
    record::wallets as wallet_records,
    repository::{set_wallet_balance_fields, transaction_active_model},
    wallet_records::ActiveModel as WalletActiveModel,
};
use crate::{StorageError, StorageResult};

const WALLET_STATUS_ACTIVE: &str = "active";
const WALLET_LIMIT_UNLIMITED: &str = "unlimited";

impl WalletStore {
    pub async fn consume_with_transaction(&self, input: WalletConsumeRecordInput) -> StorageResult<Option<WalletTransaction>> {
        ensure_positive_consume_amount(input.amount)?;
        let tx = self.database.connection().begin().await?;
        let record = lock_record_by_user_id(&input.user_id, &tx).await?.ok_or(StorageError::NotFound)?;
        let wallet = Wallet::from(record.clone());
        let Some(ledger) = consume_ledger(wallet, input)? else {
            tx.commit().await?;
            return Ok(None);
        };
        let mut active: WalletActiveModel = record.into();
        set_wallet_balance_fields(&mut active, ledger.wallet);
        active.update(&tx).await?;
        let transaction = transaction_active_model(ledger.transaction, self.database.next_id()).insert(&tx).await?;
        tx.commit().await?;
        Ok(Some(transaction.into()))
    }
}

async fn lock_record_by_user_id(user_id: &str, tx: &sea_orm::DatabaseTransaction) -> StorageResult<Option<WalletRecord>> {
    wallet_records::Entity::find()
        .filter(wallet_records::Column::UserId.eq(user_id))
        .lock_exclusive()
        .one(tx)
        .await
        .map_err(StorageError::from)
}

fn ensure_positive_consume_amount(amount: Decimal) -> StorageResult<()> {
    if amount <= Decimal::ZERO {
        return Err(StorageError::Conflict("wallet consume amount must be positive".into()));
    }
    Ok(())
}

fn consume_ledger(wallet: Wallet, input: WalletConsumeRecordInput) -> StorageResult<Option<WalletLedgerRecordInput>> {
    if wallet.status != WALLET_STATUS_ACTIVE {
        return Err(StorageError::Conflict("wallet is not active".into()));
    }
    if wallet.limit_mode == WALLET_LIMIT_UNLIMITED {
        return Ok(None);
    }
    let wallet_id = wallet.id.0.clone();
    let change = consume_balance_change(&wallet, input.amount)?;
    Ok(Some(WalletLedgerRecordInput {
        wallet: consumed_wallet(wallet, input.amount, change),
        transaction: consume_transaction_input(wallet_id, input, change),
    }))
}

fn consume_balance_change(wallet: &Wallet, amount: Decimal) -> StorageResult<ConsumeBalanceChange> {
    let gift_charge = wallet.gift_balance.min(amount);
    let recharge_charge = amount - gift_charge;
    if wallet.recharge_balance < recharge_charge {
        return Err(StorageError::Conflict(WALLET_CONSUME_INSUFFICIENT_BALANCE.into()));
    }
    Ok(ConsumeBalanceChange {
        before_recharge: wallet.recharge_balance,
        after_recharge: wallet.recharge_balance - recharge_charge,
        before_gift: wallet.gift_balance,
        after_gift: wallet.gift_balance - gift_charge,
    })
}

fn consumed_wallet(wallet: Wallet, amount: Decimal, change: ConsumeBalanceChange) -> Wallet {
    Wallet {
        recharge_balance: change.after_recharge,
        gift_balance: change.after_gift,
        total_consumed: wallet.total_consumed + amount,
        ..wallet
    }
}

fn consume_transaction_input(wallet_id: String, input: WalletConsumeRecordInput, change: ConsumeBalanceChange) -> WalletTransactionRecordInput {
    WalletTransactionRecordInput {
        wallet_id,
        category: input.category,
        reason_code: input.reason_code,
        amount: -input.amount,
        balance_before: change.before_total(),
        balance_after: change.after_total(),
        recharge_balance_before: change.before_recharge,
        recharge_balance_after: change.after_recharge,
        gift_balance_before: change.before_gift,
        gift_balance_after: change.after_gift,
        link_type: input.link_type,
        link_id: input.link_id,
        operator_id: input.operator_id,
        description: input.description,
    }
}

#[derive(Clone, Copy)]
struct ConsumeBalanceChange {
    before_recharge: Decimal,
    after_recharge: Decimal,
    before_gift: Decimal,
    after_gift: Decimal,
}

impl ConsumeBalanceChange {
    fn before_total(&self) -> Decimal {
        self.before_recharge + self.before_gift
    }

    fn after_total(&self) -> Decimal {
        self.after_recharge + self.after_gift
    }
}
