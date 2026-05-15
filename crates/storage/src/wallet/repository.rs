use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait};
use std::collections::BTreeMap;
use types::{
    pagination::{Page, PageSliceRequest},
    wallet::{Wallet, WalletTransaction},
};

use super::{
    WalletLedgerRecordInput, WalletRecord, WalletTransactionRecordInput, wallet_records, wallet_records::ActiveModel as WalletActiveModel,
    wallet_transaction_records, wallet_transaction_records::ActiveModel as WalletTransactionActiveModel,
};
use crate::{Database, StorageError, StorageResult};

const DEFAULT_CURRENCY: &str = currency::DEFAULT_WALLET_CURRENCY;
const DEFAULT_STATUS: &str = "active";
const DEFAULT_LIMIT_MODE: &str = "finite";
#[derive(Clone)]
pub struct WalletStore {
    pub(super) database: Database,
}

impl WalletStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn find_by_user_id(&self, user_id: &str) -> StorageResult<Option<Wallet>> {
        wallet_records::Entity::find()
            .filter(wallet_records::Column::UserId.eq(user_id))
            .one(self.database.connection())
            .await
            .map(|record| record.map(Wallet::from))
            .map_err(StorageError::from)
    }

    pub async fn find_by_user_ids(&self, user_ids: &[String]) -> StorageResult<BTreeMap<String, Wallet>> {
        if user_ids.is_empty() {
            return Ok(BTreeMap::new());
        }
        let records = wallet_records::Entity::find()
            .filter(wallet_records::Column::UserId.is_in(user_ids.iter().cloned()))
            .all(self.database.connection())
            .await?;
        Ok(records.into_iter().map(Wallet::from).map(|wallet| (wallet.user_id.clone(), wallet)).collect())
    }

    pub async fn find_by_id(&self, id: &str) -> StorageResult<Option<Wallet>> {
        wallet_records::Entity::find_by_id(id.to_owned())
            .one(self.database.connection())
            .await
            .map(|record| record.map(Wallet::from))
            .map_err(StorageError::from)
    }

    pub async fn ensure_user_wallet(&self, user_id: &str) -> StorageResult<Wallet> {
        let now = time::OffsetDateTime::now_utc();
        let record = WalletActiveModel {
            id: Set(self.database.next_id()),
            user_id: Set(user_id.to_owned()),
            recharge_balance: Set(Decimal::ZERO),
            gift_balance: Set(Decimal::ZERO),
            currency: Set(DEFAULT_CURRENCY.into()),
            status: Set(DEFAULT_STATUS.into()),
            limit_mode: Set(DEFAULT_LIMIT_MODE.into()),
            total_recharged: Set(Decimal::ZERO),
            total_consumed: Set(Decimal::ZERO),
            total_refunded: Set(Decimal::ZERO),
            total_adjusted: Set(Decimal::ZERO),
            created_at: Set(now),
            updated_at: Set(now),
        };
        let _ = wallet_records::Entity::insert(record)
            .on_conflict_do_nothing_on([wallet_records::Column::UserId])
            .exec_without_returning(self.database.connection())
            .await?;
        self.find_by_user_id(user_id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn grant_initial_balance(&self, user_id: &str, amount: Decimal) -> StorageResult<WalletTransaction> {
        let wallet = self.ensure_user_wallet(user_id).await?;
        let updated = Wallet {
            gift_balance: wallet.gift_balance + amount,
            total_adjusted: wallet.total_adjusted + amount,
            ..wallet.clone()
        };
        let transaction = initial_grant_transaction(&wallet, &updated, amount);
        self.update_balances_with_transaction(WalletLedgerRecordInput { wallet: updated, transaction })
            .await
    }

    pub async fn update_balances(&self, wallet: Wallet) -> StorageResult<Wallet> {
        let record = self.find_record_by_id(&wallet.id.0).await?.ok_or(StorageError::NotFound)?;
        let mut active: WalletActiveModel = record.into();
        set_wallet_balance_fields(&mut active, wallet.clone());
        active.update(self.database.connection()).await?;
        self.find_by_id(&wallet.id.0).await?.ok_or(StorageError::NotFound)
    }

    pub async fn create_transaction(&self, input: WalletTransactionRecordInput) -> StorageResult<WalletTransaction> {
        let record = transaction_active_model(input, self.database.next_id())
            .insert(self.database.connection())
            .await?;
        Ok(record.into())
    }

    pub async fn update_balances_with_transaction(&self, input: WalletLedgerRecordInput) -> StorageResult<WalletTransaction> {
        let tx = self.database.connection().begin().await?;
        let record = self.find_record_by_id_in_tx(&input.wallet.id.0, &tx).await?.ok_or(StorageError::NotFound)?;
        let mut active: WalletActiveModel = record.into();
        set_wallet_balance_fields(&mut active, input.wallet);
        active.update(&tx).await?;
        let transaction = transaction_active_model(input.transaction, self.database.next_id()).insert(&tx).await?;
        tx.commit().await?;
        Ok(transaction.into())
    }

    pub async fn page_transactions(&self, wallet_id: &str, request: PageSliceRequest) -> StorageResult<Page<WalletTransaction>> {
        let query = wallet_transaction_records::Entity::find().filter(wallet_transaction_records::Column::WalletId.eq(wallet_id));
        let total = query.clone().count(self.database.connection()).await?;
        let items = query
            .order_by_desc(wallet_transaction_records::Column::CreatedAt)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?
            .into_iter()
            .map(WalletTransaction::from)
            .collect();

        Ok(Page {
            items,
            total,
            page: request.page,
            page_size: request.page_size,
        })
    }

    async fn find_record_by_id(&self, id: &str) -> StorageResult<Option<WalletRecord>> {
        wallet_records::Entity::find_by_id(id.to_owned())
            .one(self.database.connection())
            .await
            .map_err(StorageError::from)
    }

    async fn find_record_by_id_in_tx(&self, id: &str, tx: &sea_orm::DatabaseTransaction) -> StorageResult<Option<WalletRecord>> {
        wallet_records::Entity::find_by_id(id.to_owned()).one(tx).await.map_err(StorageError::from)
    }
}

pub(super) fn set_wallet_balance_fields(active: &mut WalletActiveModel, wallet: Wallet) {
    active.recharge_balance = Set(wallet.recharge_balance);
    active.gift_balance = Set(wallet.gift_balance);
    active.total_recharged = Set(wallet.total_recharged);
    active.total_consumed = Set(wallet.total_consumed);
    active.total_refunded = Set(wallet.total_refunded);
    active.total_adjusted = Set(wallet.total_adjusted);
    active.updated_at = Set(time::OffsetDateTime::now_utc());
}

pub(super) fn transaction_active_model(input: WalletTransactionRecordInput, id: String) -> WalletTransactionActiveModel {
    WalletTransactionActiveModel {
        id: Set(id),
        wallet_id: Set(input.wallet_id),
        category: Set(input.category),
        reason_code: Set(input.reason_code),
        amount: Set(input.amount),
        balance_before: Set(input.balance_before),
        balance_after: Set(input.balance_after),
        recharge_balance_before: Set(input.recharge_balance_before),
        recharge_balance_after: Set(input.recharge_balance_after),
        gift_balance_before: Set(input.gift_balance_before),
        gift_balance_after: Set(input.gift_balance_after),
        link_type: Set(input.link_type),
        link_id: Set(input.link_id),
        operator_id: Set(input.operator_id),
        description: Set(input.description),
        created_at: Set(time::OffsetDateTime::now_utc()),
    }
}

fn initial_grant_transaction(before: &Wallet, after: &Wallet, amount: Decimal) -> WalletTransactionRecordInput {
    WalletTransactionRecordInput {
        wallet_id: before.id.0.clone(),
        category: "gift".into(),
        reason_code: "gift_initial".into(),
        amount,
        balance_before: before.recharge_balance + before.gift_balance,
        balance_after: after.recharge_balance + after.gift_balance,
        recharge_balance_before: before.recharge_balance,
        recharge_balance_after: after.recharge_balance,
        gift_balance_before: before.gift_balance,
        gift_balance_after: after.gift_balance,
        link_type: Some("system_setting".into()),
        link_id: Some("default_user_grant".into()),
        operator_id: None,
        description: Some("Default initial user grant".into()),
    }
}
