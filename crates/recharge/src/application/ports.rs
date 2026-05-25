use async_trait::async_trait;
use types::{
    pagination::{Page, PageRequest},
    recharge::{
        PaymentChannelUpdatePayload, RechargeOrder, RechargeOrderCreatePayload, RechargeOrderListFilters, RechargeOrderListResponse, RechargePackage,
        RechargePackageCreatePayload, RechargePackageListFilters, RechargePackageListResponse, RechargePackageUpdatePayload, UserRechargePackageListResponse,
    },
    system_setting::SystemSettings,
};

use super::RechargeResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentChannelRegistration {
    pub code: String,
    pub name: String,
}

pub trait RegisteredPaymentChannel: Send + Sync + 'static {
    fn code(&self) -> &'static str;
    fn name(&self) -> &'static str;
}

#[derive(Clone, Default)]
pub struct PaymentChannelRegistry {
    channels: Vec<PaymentChannelRegistration>,
}

impl PaymentChannelRegistry {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn with_channels(channels: Vec<PaymentChannelRegistration>) -> Self {
        Self { channels }
    }

    pub fn registrations(&self) -> &[PaymentChannelRegistration] {
        &self.channels
    }
}

#[async_trait]
pub trait RechargeRepository: Send + Sync + 'static {
    async fn list_packages(&self, page: PageRequest, filters: RechargePackageListFilters) -> RechargeResult<Page<RechargePackage>>;
    async fn list_active_packages(&self, page: PageRequest) -> RechargeResult<Page<RechargePackage>>;
    async fn create_package(&self, input: RechargePackageCreatePayload) -> RechargeResult<RechargePackage>;
    async fn update_package(&self, id: &str, input: RechargePackageUpdatePayload) -> RechargeResult<RechargePackage>;
    async fn find_package(&self, id: &str) -> RechargeResult<Option<RechargePackage>>;
    async fn list_orders(&self, page: PageRequest, filters: RechargeOrderListFilters) -> RechargeResult<Page<RechargeOrder>>;
    async fn list_user_orders(&self, user_id: &str, page: PageRequest) -> RechargeResult<Page<RechargeOrder>>;
    async fn create_order(&self, input: RechargeOrderCreateRecord) -> RechargeResult<RechargeOrder>;
    async fn list_payment_channels(&self) -> RechargeResult<Vec<types::recharge::PaymentChannel>>;
    async fn update_payment_channel(&self, code: &str, input: PaymentChannelUpdatePayload) -> RechargeResult<types::recharge::PaymentChannel>;
    async fn sync_payment_channels(&self, registrations: &[PaymentChannelRegistration]) -> RechargeResult<()>;
    async fn get_system_settings(&self) -> RechargeResult<SystemSettings>;
}

#[async_trait]
pub trait RechargeUseCase: Send + Sync + 'static {
    async fn list_packages(&self, page: PageRequest, filters: RechargePackageListFilters) -> RechargeResult<RechargePackageListResponse>;
    async fn list_user_packages(&self, page: PageRequest) -> RechargeResult<UserRechargePackageListResponse>;
    async fn create_package(&self, input: RechargePackageCreatePayload) -> RechargeResult<RechargePackage>;
    async fn update_package(&self, id: &str, input: RechargePackageUpdatePayload) -> RechargeResult<RechargePackage>;
    async fn list_orders(&self, page: PageRequest, filters: RechargeOrderListFilters) -> RechargeResult<RechargeOrderListResponse>;
    async fn list_user_orders(&self, user_id: &str, page: PageRequest) -> RechargeResult<RechargeOrderListResponse>;
    async fn create_user_order(&self, user_id: &str, input: RechargeOrderCreatePayload) -> RechargeResult<RechargeOrder>;
    async fn list_payment_channels(&self) -> RechargeResult<Vec<types::recharge::PaymentChannel>>;
    async fn update_payment_channel(&self, code: &str, input: PaymentChannelUpdatePayload) -> RechargeResult<types::recharge::PaymentChannel>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RechargeOrderCreateRecord {
    pub user_id: String,
    pub package_id: Option<String>,
    pub package_name: String,
    pub recharge_amount: rust_decimal::Decimal,
    pub gift_amount: rust_decimal::Decimal,
    pub total_arrival_amount: rust_decimal::Decimal,
    pub payable_amount: rust_decimal::Decimal,
    pub expires_at: time::OffsetDateTime,
}
