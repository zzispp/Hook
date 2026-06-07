use async_trait::async_trait;
use storage::{
    Database, StorageError,
    recharge::{PaymentChannelDefinition, RechargeOrderRecordInput, RechargePackageRecordInput, RechargePackageRecordPatch, RechargeStore},
    setting::SettingStore,
};
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    recharge::{
        PaymentCallbackListFilters, PaymentCallbackRecord, PaymentChannel, RECHARGE_ORDER_STATUS_PENDING, RechargeOrder, RechargeOrderListFilters,
        RechargePackage, RechargePackageCreatePayload, RechargePackageListFilters, RechargePackageUpdatePayload,
    },
    system_setting::SystemSettings,
};

use crate::application::{
    PaymentCallbackCreateRecord, PaymentCallbackUpdateRecord, PaymentChannelRegistration, PaymentChannelStoredConfig, PaymentChannelUpdateRecord,
    RechargeError, RechargeOrderCreateRecord, RechargePaymentSettlementRecord, RechargePaymentSettlementResult, RechargeRepository, RechargeResult,
};

#[derive(Clone)]
pub struct StorageRechargeRepository {
    store: RechargeStore,
    settings: SettingStore,
}

impl StorageRechargeRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: RechargeStore::new(database.clone()),
            settings: SettingStore::new(database),
        }
    }
}

#[async_trait]
impl RechargeRepository for StorageRechargeRepository {
    async fn list_packages(&self, page: PageRequest, filters: RechargePackageListFilters) -> RechargeResult<Page<RechargePackage>> {
        self.store.list_packages(page_slice_request(page), filters).await.map_err(storage_error)
    }

    async fn list_active_packages(&self, page: PageRequest) -> RechargeResult<Page<RechargePackage>> {
        self.store.list_active_packages(page_slice_request(page)).await.map_err(storage_error)
    }

    async fn create_package(&self, input: RechargePackageCreatePayload) -> RechargeResult<RechargePackage> {
        self.store.create_package(package_input(input)).await.map_err(storage_error)
    }

    async fn update_package(&self, id: &str, input: RechargePackageUpdatePayload) -> RechargeResult<RechargePackage> {
        self.store.update_package(id, package_patch(input)).await.map_err(storage_error)
    }

    async fn find_package(&self, id: &str) -> RechargeResult<Option<RechargePackage>> {
        self.store.find_package(id).await.map_err(storage_error)
    }

    async fn list_orders(&self, page: PageRequest, filters: RechargeOrderListFilters) -> RechargeResult<Page<RechargeOrder>> {
        self.store.list_orders(page_slice_request(page), filters).await.map_err(storage_error)
    }

    async fn list_user_orders(&self, user_id: &str, page: PageRequest) -> RechargeResult<Page<RechargeOrder>> {
        self.store.list_user_orders(user_id, page_slice_request(page)).await.map_err(storage_error)
    }

    async fn list_pending_unexpired_orders(&self, limit: u64) -> RechargeResult<Vec<RechargeOrder>> {
        self.store.list_pending_unexpired_orders(limit).await.map_err(storage_error)
    }

    async fn list_payment_callbacks(&self, page: PageRequest, filters: PaymentCallbackListFilters) -> RechargeResult<Page<PaymentCallbackRecord>> {
        self.store
            .list_payment_callbacks(page_slice_request(page), filters)
            .await
            .map_err(storage_error)
    }

    async fn create_payment_callback(&self, input: PaymentCallbackCreateRecord) -> RechargeResult<PaymentCallbackRecord> {
        self.store
            .create_payment_callback(storage_callback_input(input))
            .await
            .map(Into::into)
            .map_err(storage_error)
    }

    async fn update_payment_callback(&self, id: &str, input: PaymentCallbackUpdateRecord) -> RechargeResult<PaymentCallbackRecord> {
        self.store
            .update_payment_callback(id, storage_callback_patch(input))
            .await
            .map(Into::into)
            .map_err(storage_error)
    }

    async fn create_order(&self, input: RechargeOrderCreateRecord, max_unpaid_orders: u64) -> RechargeResult<RechargeOrder> {
        self.store.create_order(order_input(input), max_unpaid_orders).await.map_err(storage_error)
    }

    async fn find_payment_channel(&self, code: &str) -> RechargeResult<Option<PaymentChannel>> {
        self.store
            .payment_channel_record(code)
            .await
            .map(|record| record.map(Into::into))
            .map_err(storage_error)
    }

    async fn payment_channel_config(&self, code: &str) -> RechargeResult<PaymentChannelStoredConfig> {
        let record = self
            .store
            .payment_channel_record(code)
            .await
            .map_err(storage_error)?
            .ok_or(RechargeError::NotFound)?;
        Ok(PaymentChannelStoredConfig {
            config: serde_json::from_str(&record.config_json).map_err(|error| RechargeError::Infrastructure(error.to_string()))?,
            encrypted_secret: Some(record.encrypted_secret).filter(|value| !value.is_empty()),
        })
    }

    async fn list_payment_channels(&self) -> RechargeResult<Vec<PaymentChannel>> {
        self.store.list_payment_channels().await.map_err(storage_error)
    }

    async fn update_payment_channel(&self, code: &str, input: PaymentChannelUpdateRecord) -> RechargeResult<PaymentChannel> {
        self.store
            .update_payment_channel(code, payment_channel_patch(input))
            .await
            .map_err(storage_error)
    }

    async fn sync_payment_channels(&self, registrations: &[PaymentChannelRegistration]) -> RechargeResult<()> {
        let definitions = registrations.iter().map(channel_definition).collect::<Vec<_>>();
        self.store.sync_payment_channels(&definitions).await.map_err(storage_error)
    }

    async fn get_system_settings(&self) -> RechargeResult<SystemSettings> {
        self.settings.get_system_settings().await.map_err(storage_error)
    }

    async fn settle_paid_order(&self, input: RechargePaymentSettlementRecord) -> RechargeResult<RechargePaymentSettlementResult> {
        self.store
            .settle_paid_order(storage_settlement_input(input))
            .await
            .map(|record| RechargePaymentSettlementResult {
                order: record.order,
                settled: record.settled,
            })
            .map_err(storage_error)
    }

    async fn expire_pending_orders(&self, now: time::OffsetDateTime) -> RechargeResult<u64> {
        self.store.expire_pending_orders(now).await.map_err(storage_error)
    }
}

fn package_input(input: RechargePackageCreatePayload) -> RechargePackageRecordInput {
    RechargePackageRecordInput {
        name: input.name,
        description: input.description,
        recharge_amount: input.recharge_amount,
        gift_amount: input.gift_amount,
        status: input.status.unwrap_or_else(|| types::recharge::RECHARGE_PACKAGE_STATUS_ACTIVE.into()),
        sort_order: input.sort_order,
    }
}

fn package_patch(input: RechargePackageUpdatePayload) -> RechargePackageRecordPatch {
    RechargePackageRecordPatch {
        name: input.name,
        description: input.description,
        recharge_amount: input.recharge_amount,
        gift_amount: input.gift_amount,
        status: input.status,
        sort_order: input.sort_order,
    }
}

fn order_input(input: RechargeOrderCreateRecord) -> RechargeOrderRecordInput {
    RechargeOrderRecordInput {
        order_no: input.order_no,
        user_id: input.user_id,
        package_id: input.package_id,
        package_name: input.package_name,
        recharge_amount: input.recharge_amount,
        gift_amount: input.gift_amount,
        total_arrival_amount: input.total_arrival_amount,
        payable_amount: input.payable_amount,
        status: RECHARGE_ORDER_STATUS_PENDING.into(),
        payment_channel_code: Some(input.payment_channel_code),
        payment_channel_name: Some(input.payment_channel_name),
        payment_method: Some(input.payment_method),
        payment_request_json: Some(input.payment_request_json),
        expires_at: input.expires_at,
    }
}

fn payment_channel_patch(input: PaymentChannelUpdateRecord) -> storage::recharge::PaymentChannelRecordPatch {
    storage::recharge::PaymentChannelRecordPatch {
        enabled: input.enabled,
        config: input.config,
        encrypted_secret: input.encrypted_secret,
    }
}

fn storage_callback_input(input: PaymentCallbackCreateRecord) -> storage::recharge::PaymentCallbackRecordInput {
    storage::recharge::PaymentCallbackRecordInput {
        payment_channel_code: input.payment_channel_code,
        callback_kind: input.callback_kind,
        http_method: input.http_method,
        raw_params: input.raw_params,
    }
}

fn storage_callback_patch(input: PaymentCallbackUpdateRecord) -> storage::recharge::PaymentCallbackRecordPatch {
    storage::recharge::PaymentCallbackRecordPatch {
        order_no: input.order_no,
        provider_trade_no: input.provider_trade_no,
        payment_method: input.payment_method,
        trade_status: input.trade_status,
        status: input.status,
        settled: input.settled,
        error_message: input.error_message,
    }
}

fn storage_settlement_input(input: RechargePaymentSettlementRecord) -> storage::recharge::RechargePaymentSettlementInput {
    storage::recharge::RechargePaymentSettlementInput {
        order_no: input.order_no,
        payment_channel_code: input.payment_channel_code,
        provider_trade_no: input.provider_trade_no,
        payment_method: input.payment_method,
        payable_amount: input.payable_amount,
        callback_payload: input.callback_payload,
    }
}

fn channel_definition(input: &PaymentChannelRegistration) -> PaymentChannelDefinition {
    PaymentChannelDefinition {
        code: input.code.clone(),
        name: input.name.clone(),
    }
}

fn page_slice_request(page: PageRequest) -> PageSliceRequest {
    PageSliceRequest {
        offset: (page.page - 1) * page.page_size,
        limit: page.page_size,
        page: page.page,
        page_size: page.page_size,
    }
}

fn storage_error(error: StorageError) -> RechargeError {
    match error {
        StorageError::NotFound => RechargeError::NotFound,
        StorageError::Conflict(message) => RechargeError::Conflict(message),
        StorageError::Database(message) => RechargeError::Infrastructure(message),
    }
}
