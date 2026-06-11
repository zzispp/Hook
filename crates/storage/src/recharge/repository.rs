use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait};
use types::{
    pagination::{Page, PageSliceRequest},
    recharge::{PaymentChannel, RechargeOrder, RechargeOrderListFilters, RechargeOrderSummaryPage, RechargePackage, RechargePackageListFilters},
};

use crate::{Database, StorageError, StorageResult, json};

use super::{
    PaymentChannelDefinition, RechargeOrderRecord, RechargeOrderRecordInput, RechargePackageRecord, RechargePackageRecordInput, RechargePackageRecordPatch,
    query,
    record::{payment_channels as payment_channel_records, recharge_orders as recharge_order_records, recharge_packages as recharge_package_records},
};

#[derive(Clone)]
pub struct RechargeStore {
    pub(super) database: Database,
}

impl RechargeStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn list_packages(&self, request: PageSliceRequest, filters: RechargePackageListFilters) -> StorageResult<Page<RechargePackage>> {
        let query = query::filtered_packages(filters);
        let total = query.clone().count(self.database.connection()).await?;
        let items = query
            .order_by_asc(recharge_package_records::Column::SortOrder)
            .order_by_desc(recharge_package_records::Column::UpdatedAt)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(page(items, total, request))
    }

    pub async fn list_active_packages(&self, request: PageSliceRequest) -> StorageResult<Page<RechargePackage>> {
        let query = query::active_packages();
        let total = query.clone().count(self.database.connection()).await?;
        let items = query
            .order_by_asc(recharge_package_records::Column::SortOrder)
            .order_by_desc(recharge_package_records::Column::UpdatedAt)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(page(items, total, request))
    }

    pub async fn create_package(&self, input: RechargePackageRecordInput) -> StorageResult<RechargePackage> {
        let now = time::OffsetDateTime::now_utc();
        let record = recharge_package_records::ActiveModel {
            id: Set(self.database.next_id()),
            name: Set(input.name),
            description: Set(input.description),
            recharge_amount: Set(input.recharge_amount),
            gift_amount: Set(input.gift_amount),
            status: Set(input.status),
            sort_order: Set(input.sort_order),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(self.database.connection())
        .await?;
        Ok(record.into())
    }

    pub async fn update_package(&self, id: &str, input: RechargePackageRecordPatch) -> StorageResult<RechargePackage> {
        let record = self.package_record(id).await?.ok_or(StorageError::NotFound)?;
        let mut active: recharge_package_records::ActiveModel = record.into();
        active.name = Set(input.name);
        active.description = Set(input.description);
        active.recharge_amount = Set(input.recharge_amount);
        active.gift_amount = Set(input.gift_amount);
        active.status = Set(input.status);
        active.sort_order = Set(input.sort_order);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        Ok(active.update(self.database.connection()).await?.into())
    }

    pub async fn find_package(&self, id: &str) -> StorageResult<Option<RechargePackage>> {
        self.package_record(id).await.map(|record| record.map(Into::into))
    }

    pub async fn list_orders(&self, request: PageSliceRequest, filters: RechargeOrderListFilters) -> StorageResult<Page<RechargeOrder>> {
        let query = query::filtered_orders(filters);
        let total = query.clone().count(self.database.connection()).await?;
        let items = query
            .order_by_desc(recharge_order_records::Column::CreatedAt)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?
            .into_iter()
            .map(order_response)
            .collect();
        Ok(page(items, total, request))
    }

    pub async fn list_order_summary(&self, request: PageSliceRequest, filters: RechargeOrderListFilters) -> StorageResult<RechargeOrderSummaryPage> {
        super::summary::list_order_summary(self, request, filters).await
    }

    pub async fn list_user_orders(&self, user_id: &str, request: PageSliceRequest) -> StorageResult<Page<RechargeOrder>> {
        let query = query::user_orders(user_id);
        let total = query.clone().count(self.database.connection()).await?;
        let items = query
            .order_by_desc(recharge_order_records::Column::CreatedAt)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?
            .into_iter()
            .map(user_order_response)
            .collect();
        Ok(page(items, total, request))
    }

    pub async fn list_pending_unexpired_orders(&self, limit: u64) -> StorageResult<Vec<RechargeOrder>> {
        recharge_order_records::Entity::find()
            .filter(recharge_order_records::Column::Status.eq(types::recharge::RECHARGE_ORDER_STATUS_PENDING))
            .filter(recharge_order_records::Column::ExpiresAt.gt(time::OffsetDateTime::now_utc()))
            .order_by_asc(recharge_order_records::Column::CreatedAt)
            .limit(limit)
            .all(self.database.connection())
            .await
            .map(|orders| orders.into_iter().map(user_order_response).collect())
            .map_err(Into::into)
    }

    pub async fn create_order(&self, input: RechargeOrderRecordInput, max_unpaid_orders: u64) -> StorageResult<RechargeOrder> {
        let tx = self.database.connection().begin().await?;
        lock_user_in_tx(&input.user_id, &tx).await?;
        ensure_unpaid_order_capacity(&input.user_id, max_unpaid_orders, &tx).await?;
        let record = insert_order_in_tx(self, input, &tx).await?;
        tx.commit().await?;
        Ok(user_order_response(record))
    }

    async fn package_record(&self, id: &str) -> StorageResult<Option<RechargePackageRecord>> {
        recharge_package_records::Entity::find_by_id(id.to_owned())
            .one(self.database.connection())
            .await
            .map_err(Into::into)
    }
}

async fn lock_user_in_tx(user_id: &str, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    crate::user::UserEntity::find_by_id(user_id.to_owned())
        .lock_exclusive()
        .one(tx)
        .await?
        .ok_or(StorageError::NotFound)?;
    Ok(())
}

async fn ensure_unpaid_order_capacity(user_id: &str, max_unpaid_orders: u64, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    let total = recharge_order_records::Entity::find()
        .filter(recharge_order_records::Column::UserId.eq(user_id.to_owned()))
        .filter(recharge_order_records::Column::Status.eq(types::recharge::RECHARGE_ORDER_STATUS_PENDING))
        .filter(recharge_order_records::Column::ExpiresAt.gt(time::OffsetDateTime::now_utc()))
        .count(tx)
        .await?;
    if total >= max_unpaid_orders {
        return Err(StorageError::Conflict(format!("unpaid recharge order limit reached: {max_unpaid_orders}")));
    }
    Ok(())
}

async fn insert_order_in_tx(store: &RechargeStore, input: RechargeOrderRecordInput, tx: &sea_orm::DatabaseTransaction) -> StorageResult<RechargeOrderRecord> {
    insert_order_active_model(store, input)?.insert(tx).await.map_err(Into::into)
}

fn insert_order_active_model(store: &RechargeStore, input: RechargeOrderRecordInput) -> StorageResult<recharge_order_records::ActiveModel> {
    let now = time::OffsetDateTime::now_utc();
    Ok(recharge_order_records::ActiveModel {
        order_no: Set(input.order_no),
        id: Set(store.database.next_id()),
        user_id: Set(input.user_id),
        package_id: Set(input.package_id),
        package_name: Set(input.package_name),
        recharge_amount: Set(input.recharge_amount),
        gift_amount: Set(input.gift_amount),
        total_arrival_amount: Set(input.total_arrival_amount),
        payable_amount: Set(input.payable_amount),
        status: Set(input.status),
        payment_channel_code: Set(input.payment_channel_code),
        payment_channel_name: Set(input.payment_channel_name),
        payment_method: Set(input.payment_method),
        provider_trade_no: Set(None),
        payment_request_json: Set(json::encode_optional(&input.payment_request_json)?),
        refund_status: Set(None),
        refund_amount: Set(None),
        paid_at: Set(None),
        refunded_at: Set(None),
        expires_at: Set(input.expires_at),
        created_at: Set(now),
        updated_at: Set(now),
    })
}

impl RechargeStore {
    pub async fn list_payment_channels(&self) -> StorageResult<Vec<PaymentChannel>> {
        payment_channel_records::Entity::find()
            .order_by_asc(payment_channel_records::Column::Name)
            .all(self.database.connection())
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    pub async fn sync_payment_channels(&self, definitions: &[PaymentChannelDefinition]) -> StorageResult<()> {
        for definition in definitions {
            self.sync_payment_channel(definition).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "repository_tests.rs"]
mod tests;

fn order_response(value: (RechargeOrderRecord, Option<crate::user::UserRecord>)) -> RechargeOrder {
    let username = value.1.as_ref().map(|user| user.username.clone()).unwrap_or_default();
    let user_email = value.1.map(|user| user.email).unwrap_or_default();
    value.0.into_response(username, user_email)
}

fn user_order_response(value: RechargeOrderRecord) -> RechargeOrder {
    value.into_response(String::new(), String::new())
}

fn page<T>(items: Vec<T>, total: u64, request: PageSliceRequest) -> Page<T> {
    Page {
        items,
        total,
        page: request.page,
        page_size: request.page_size,
    }
}
