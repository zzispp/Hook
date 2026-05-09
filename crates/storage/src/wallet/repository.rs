use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, SelectTwo, Set, TransactionTrait};
use types::{
    pagination::{Page, PageSliceRequest},
    wallet::{AdminWalletListFilters, AdminWalletResponse, Wallet, WalletSummaryResponse, WalletTransaction},
};

use crate::{Database, StorageError, StorageResult};
use crate::user::{UserColumn, UserEntity as Users, UserRecord};

use super::{
    AdminWalletRecord, WalletLedgerRecordInput, WalletRecord, WalletTransactionRecordInput, wallet_records, wallet_records::ActiveModel as WalletActiveModel,
    wallet_transaction_records, wallet_transaction_records::ActiveModel as WalletTransactionActiveModel,
};

const DEFAULT_CURRENCY: &str = "CNY";
const DEFAULT_STATUS: &str = "active";
const DEFAULT_LIMIT_MODE: &str = "finite";

#[derive(Clone)]
pub struct WalletStore {
    database: Database,
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

    pub async fn find_admin_wallet_by_id(&self, id: &str) -> StorageResult<Option<AdminWalletResponse>> {
        let Some(wallet) = self.find_by_id(id).await? else {
            return Ok(None);
        };
        let Some(user) = Users::find_by_id(wallet.user_id.clone()).one(self.database.connection()).await? else {
            return Ok(None);
        };
        Ok(Some(admin_wallet_response(AdminWalletRecord {
            wallet,
            owner_name: user.username,
            owner_email: user.email,
        })))
    }

    pub async fn page_admin_wallets(
        &self,
        request: PageSliceRequest,
        filters: AdminWalletListFilters,
    ) -> StorageResult<Page<AdminWalletResponse>> {
        let query = filtered_admin_wallets(filters);
        let total = query.clone().count(self.database.connection()).await?;
        let records = query
            .order_by_desc(wallet_records::Column::CreatedAt)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?;
        Ok(Page {
            items: records.into_iter().map(AdminWalletRecord::from).map(admin_wallet_response).collect(),
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

fn filtered_admin_wallets(filters: AdminWalletListFilters) -> SelectTwo<wallet_records::Entity, Users> {
    let mut query = wallet_records::Entity::find().find_also_related(Users);
    if let Some(status) = filters.status.filter(|value| !value.is_empty()) {
        query = query.filter(wallet_records::Column::Status.eq(status));
    }
    match filters.search {
        Some(search) if !search.is_empty() => query.filter(admin_wallet_search_condition(&search)),
        _ => query,
    }
}

fn admin_wallet_search_condition(search: &str) -> Condition {
    Condition::any()
        .add(wallet_records::Column::Id.contains(search))
        .add(wallet_records::Column::UserId.contains(search))
        .add(UserColumn::Username.contains(search))
        .add(UserColumn::Email.contains(search))
}

fn admin_wallet_response(record: AdminWalletRecord) -> AdminWalletResponse {
    let wallet = record.wallet.clone();
    let summary = WalletSummaryResponse::from(wallet);
    AdminWalletResponse {
        id: summary.id,
        user_id: summary.user_id,
        owner_name: record.owner_name,
        owner_email: record.owner_email,
        owner_type: "user".into(),
        balance: summary.balance,
        recharge_balance: summary.recharge_balance,
        gift_balance: summary.gift_balance,
        currency: summary.currency,
        status: summary.status,
        limit_mode: summary.limit_mode,
        unlimited: summary.unlimited,
        total_recharged: summary.total_recharged,
        total_consumed: summary.total_consumed,
        total_refunded: summary.total_refunded,
        total_adjusted: summary.total_adjusted,
        created_at: record.wallet.created_at,
        updated_at: summary.updated_at,
    }
}

impl From<(WalletRecord, Option<UserRecord>)> for AdminWalletRecord {
    fn from(value: (WalletRecord, Option<UserRecord>)) -> Self {
        let user = value.1.expect("wallet owner user must exist");
        Self {
            wallet: value.0.into(),
            owner_name: user.username,
            owner_email: user.email,
        }
    }
}

fn set_wallet_balance_fields(active: &mut WalletActiveModel, wallet: Wallet) {
    active.recharge_balance = Set(wallet.recharge_balance);
    active.gift_balance = Set(wallet.gift_balance);
    active.total_recharged = Set(wallet.total_recharged);
    active.total_consumed = Set(wallet.total_consumed);
    active.total_refunded = Set(wallet.total_refunded);
    active.total_adjusted = Set(wallet.total_adjusted);
    active.updated_at = Set(time::OffsetDateTime::now_utc());
}

fn transaction_active_model(input: WalletTransactionRecordInput, id: String) -> WalletTransactionActiveModel {
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
