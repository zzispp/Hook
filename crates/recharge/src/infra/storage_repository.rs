use async_trait::async_trait;
use storage::{
    Database, StorageError,
    recharge::{PaymentChannelDefinition, RechargeOrderRecordInput, RechargePackageRecordInput, RechargePackageRecordPatch, RechargeStore},
    setting::SettingStore,
};
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    recharge::{
        PaymentChannel, PaymentChannelUpdatePayload, RECHARGE_ORDER_STATUS_PENDING, RechargeOrder, RechargeOrderListFilters, RechargePackage,
        RechargePackageCreatePayload, RechargePackageListFilters, RechargePackageUpdatePayload,
    },
    system_setting::SystemSettings,
};

use crate::application::{PaymentChannelRegistration, RechargeError, RechargeOrderCreateRecord, RechargeRepository, RechargeResult};

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

    async fn create_order(&self, input: RechargeOrderCreateRecord) -> RechargeResult<RechargeOrder> {
        self.store.create_order(order_input(input)).await.map_err(storage_error)
    }

    async fn list_payment_channels(&self) -> RechargeResult<Vec<PaymentChannel>> {
        self.store.list_payment_channels().await.map_err(storage_error)
    }

    async fn update_payment_channel(&self, code: &str, input: PaymentChannelUpdatePayload) -> RechargeResult<PaymentChannel> {
        self.store.update_payment_channel(code, input.enabled).await.map_err(storage_error)
    }

    async fn sync_payment_channels(&self, registrations: &[PaymentChannelRegistration]) -> RechargeResult<()> {
        let definitions = registrations.iter().map(channel_definition).collect::<Vec<_>>();
        self.store.sync_payment_channels(&definitions).await.map_err(storage_error)
    }

    async fn get_system_settings(&self) -> RechargeResult<SystemSettings> {
        self.settings.get_system_settings().await.map_err(storage_error)
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
        user_id: input.user_id,
        package_id: input.package_id,
        package_name: input.package_name,
        recharge_amount: input.recharge_amount,
        gift_amount: input.gift_amount,
        total_arrival_amount: input.total_arrival_amount,
        payable_amount: input.payable_amount,
        status: RECHARGE_ORDER_STATUS_PENDING.into(),
        payment_channel_code: None,
        payment_channel_name: None,
        expires_at: input.expires_at,
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
