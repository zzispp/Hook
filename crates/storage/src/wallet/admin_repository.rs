use std::collections::HashMap;

use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, SelectTwo};
use types::{
    pagination::{Page, PageSliceRequest},
    wallet::{
        AdminWalletLedgerFilters, AdminWalletLedgerTransactionResponse, AdminWalletListFilters, AdminWalletResponse, WalletSummaryResponse,
        WalletTransactionResponse,
    },
};

use crate::StorageResult;
use crate::user::{UserColumn, UserEntity as Users, UserRecord};

use super::WalletStore;
use super::{AdminWalletLedgerRecord, AdminWalletRecord, WalletRecord, WalletTransactionRecord, wallet_records, wallet_transaction_records};

impl WalletStore {
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

    pub async fn page_admin_wallets(&self, request: PageSliceRequest, filters: AdminWalletListFilters) -> StorageResult<Page<AdminWalletResponse>> {
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

    pub async fn page_admin_ledger(
        &self,
        request: PageSliceRequest,
        filters: AdminWalletLedgerFilters,
    ) -> StorageResult<Page<AdminWalletLedgerTransactionResponse>> {
        let query = filtered_admin_ledger(filters);
        let total = query.clone().count(self.database.connection()).await?;
        let records = query
            .order_by_desc(wallet_transaction_records::Column::CreatedAt)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?;
        let users = admin_ledger_user_map(&records, self.database.connection()).await?;
        Ok(Page {
            items: records
                .into_iter()
                .map(|record| admin_ledger_record(record, &users))
                .map(admin_ledger_response)
                .collect(),
            total,
            page: request.page,
            page_size: request.page_size,
        })
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

fn filtered_admin_ledger(filters: AdminWalletLedgerFilters) -> SelectTwo<wallet_transaction_records::Entity, wallet_records::Entity> {
    let mut query = wallet_transaction_records::Entity::find().find_also_related(wallet_records::Entity);
    if let Some(category) = filters.category.filter(|value| !value.is_empty()) {
        query = query.filter(wallet_transaction_records::Column::Category.eq(category));
    }
    if let Some(reason) = filters.reason_code.filter(|value| !value.is_empty()) {
        query = query.filter(wallet_transaction_records::Column::ReasonCode.eq(reason));
    }
    if filters.owner_type.as_deref() == Some("user") {
        query = query.filter(wallet_records::Column::UserId.is_not_null());
    }
    query
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

fn admin_ledger_response(record: AdminWalletLedgerRecord) -> AdminWalletLedgerTransactionResponse {
    AdminWalletLedgerTransactionResponse {
        transaction: WalletTransactionResponse::from(record.transaction),
        currency: record.wallet.currency,
        owner_name: record.wallet.owner_name,
        owner_email: record.wallet.owner_email,
        owner_type: record.wallet.owner_type,
        wallet_status: record.wallet.status,
    }
}

impl From<(WalletRecord, Option<UserRecord>)> for AdminWalletRecord {
    fn from(value: (WalletRecord, Option<UserRecord>)) -> Self {
        admin_wallet_record(value.0, value.1)
    }
}

fn admin_wallet_record(wallet: WalletRecord, user: Option<UserRecord>) -> AdminWalletRecord {
    let fallback_owner = wallet.user_id.clone();
    AdminWalletRecord {
        wallet: wallet.into(),
        owner_name: user.as_ref().map(|item| item.username.clone()).unwrap_or_else(|| fallback_owner.clone()),
        owner_email: user.map(|item| item.email).unwrap_or_default(),
    }
}

fn admin_ledger_record(value: (WalletTransactionRecord, Option<WalletRecord>), users: &HashMap<String, UserRecord>) -> AdminWalletLedgerRecord {
    let wallet = value.1.expect("wallet transaction must have wallet");
    let user = users.get(&wallet.user_id).cloned();
    AdminWalletLedgerRecord {
        transaction: value.0.into(),
        wallet: admin_wallet_response(admin_wallet_record(wallet, user)),
    }
}

async fn admin_ledger_user_map(
    records: &[(WalletTransactionRecord, Option<WalletRecord>)],
    connection: &DatabaseConnection,
) -> StorageResult<HashMap<String, UserRecord>> {
    let user_ids: Vec<_> = records
        .iter()
        .filter_map(|(_, wallet)| wallet.as_ref().map(|item| item.user_id.clone()))
        .collect();
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }
    Ok(Users::find()
        .filter(UserColumn::Id.is_in(user_ids))
        .all(connection)
        .await?
        .into_iter()
        .map(|user| (user.id.clone(), user))
        .collect())
}
