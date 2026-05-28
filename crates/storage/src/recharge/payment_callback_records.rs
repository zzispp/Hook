use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, QueryOrder, QuerySelect, Set};
use types::{pagination::Page, recharge::PaymentCallbackListFilters};

use crate::{StorageError, StorageResult, json};

use super::{PaymentCallbackRecord, PaymentCallbackRecordInput, PaymentCallbackRecordPatch, RechargeStore, query, record::payment_callback_records};

impl RechargeStore {
    pub async fn list_payment_callbacks(
        &self,
        request: types::pagination::PageSliceRequest,
        filters: PaymentCallbackListFilters,
    ) -> StorageResult<Page<types::recharge::PaymentCallbackRecord>> {
        let query = query::filtered_payment_callbacks(filters);
        let total = query.clone().count(self.database.connection()).await?;
        let items = query
            .order_by_desc(payment_callback_records::Column::ReceivedAt)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(page(items, total, request))
    }

    pub async fn create_payment_callback(&self, input: PaymentCallbackRecordInput) -> StorageResult<PaymentCallbackRecord> {
        let now = time::OffsetDateTime::now_utc();
        payment_callback_records::ActiveModel {
            id: Set(self.database.next_id()),
            payment_channel_code: Set(input.payment_channel_code),
            callback_kind: Set(input.callback_kind),
            http_method: Set(input.http_method),
            order_no: Set(None),
            provider_trade_no: Set(None),
            payment_method: Set(None),
            trade_status: Set(None),
            status: Set(types::recharge::PAYMENT_CALLBACK_STATUS_RECEIVED.into()),
            settled: Set(false),
            error_message: Set(None),
            raw_params_json: Set(json::encode_required(&input.raw_params)?),
            received_at: Set(now),
            processed_at: Set(None),
        }
        .insert(self.database.connection())
        .await
        .map_err(Into::into)
    }

    pub async fn update_payment_callback(&self, id: &str, patch: PaymentCallbackRecordPatch) -> StorageResult<PaymentCallbackRecord> {
        let record = payment_callback_records::Entity::find_by_id(id.to_owned())
            .one(self.database.connection())
            .await?
            .ok_or(StorageError::NotFound)?;
        let mut active: payment_callback_records::ActiveModel = record.into();
        active.order_no = Set(patch.order_no);
        active.provider_trade_no = Set(patch.provider_trade_no);
        active.payment_method = Set(patch.payment_method);
        active.trade_status = Set(patch.trade_status);
        active.status = Set(patch.status);
        active.settled = Set(patch.settled);
        active.error_message = Set(patch.error_message);
        active.processed_at = Set(Some(time::OffsetDateTime::now_utc()));
        active.update(self.database.connection()).await.map_err(Into::into)
    }
}

fn page<T>(items: Vec<T>, total: u64, request: types::pagination::PageSliceRequest) -> Page<T> {
    Page {
        items,
        total,
        page: request.page,
        page_size: request.page_size,
    }
}
