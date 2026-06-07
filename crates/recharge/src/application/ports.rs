use async_trait::async_trait;
use payment::{PaymentCallbackRequest, PaymentChannelRegistration};
use types::{
    pagination::{Page, PageRequest},
    recharge::{
        PaymentCallbackListFilters, PaymentCallbackRecord, PaymentCallbackRecordListResponse, PaymentChannelUpdatePayload, PublicPaymentChannelResponse,
        RechargeOrder, RechargeOrderCreatePayload, RechargeOrderCreateResponse, RechargeOrderListFilters, RechargeOrderListResponse, RechargePackage,
        RechargePackageCreatePayload, RechargePackageListFilters, RechargePackageListResponse, RechargePackageUpdatePayload, UserRechargePackageListResponse,
    },
    system_setting::SystemSettings,
};

use super::RechargeResult;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaymentCallbackKind {
    Notify,
    Return,
}

impl PaymentCallbackKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Notify => types::recharge::PAYMENT_CALLBACK_KIND_NOTIFY,
            Self::Return => types::recharge::PAYMENT_CALLBACK_KIND_RETURN,
        }
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
    async fn list_pending_unexpired_orders(&self, limit: u64) -> RechargeResult<Vec<RechargeOrder>>;
    async fn list_payment_callbacks(&self, page: PageRequest, filters: PaymentCallbackListFilters) -> RechargeResult<Page<PaymentCallbackRecord>>;
    async fn create_payment_callback(&self, input: PaymentCallbackCreateRecord) -> RechargeResult<PaymentCallbackRecord>;
    async fn update_payment_callback(&self, id: &str, input: PaymentCallbackUpdateRecord) -> RechargeResult<PaymentCallbackRecord>;
    async fn create_order(&self, input: RechargeOrderCreateRecord, max_unpaid_orders: u64) -> RechargeResult<RechargeOrder>;
    async fn find_payment_channel(&self, code: &str) -> RechargeResult<Option<types::recharge::PaymentChannel>>;
    async fn payment_channel_config(&self, code: &str) -> RechargeResult<PaymentChannelStoredConfig>;
    async fn list_payment_channels(&self) -> RechargeResult<Vec<types::recharge::PaymentChannel>>;
    async fn update_payment_channel(&self, code: &str, input: PaymentChannelUpdateRecord) -> RechargeResult<types::recharge::PaymentChannel>;
    async fn sync_payment_channels(&self, registrations: &[PaymentChannelRegistration]) -> RechargeResult<()>;
    async fn get_system_settings(&self) -> RechargeResult<SystemSettings>;
    async fn settle_paid_order(&self, input: RechargePaymentSettlementRecord) -> RechargeResult<RechargePaymentSettlementResult>;
    async fn expire_pending_orders(&self, now: time::OffsetDateTime) -> RechargeResult<u64>;
}

#[async_trait]
pub trait RechargeUseCase: Send + Sync + 'static {
    async fn list_packages(&self, page: PageRequest, filters: RechargePackageListFilters) -> RechargeResult<RechargePackageListResponse>;
    async fn list_user_packages(&self, page: PageRequest) -> RechargeResult<UserRechargePackageListResponse>;
    async fn create_package(&self, input: RechargePackageCreatePayload) -> RechargeResult<RechargePackage>;
    async fn update_package(&self, id: &str, input: RechargePackageUpdatePayload) -> RechargeResult<RechargePackage>;
    async fn list_orders(&self, page: PageRequest, filters: RechargeOrderListFilters) -> RechargeResult<RechargeOrderListResponse>;
    async fn list_user_orders(&self, user_id: &str, page: PageRequest) -> RechargeResult<RechargeOrderListResponse>;
    async fn list_payment_callbacks(&self, page: PageRequest, filters: PaymentCallbackListFilters) -> RechargeResult<PaymentCallbackRecordListResponse>;
    async fn create_user_order(&self, user_id: &str, input: RechargeOrderCreatePayload) -> RechargeResult<RechargeOrderCreateResponse>;
    async fn list_payment_channels(&self) -> RechargeResult<Vec<types::recharge::PaymentChannel>>;
    async fn list_user_payment_channels(&self) -> RechargeResult<Vec<PublicPaymentChannelResponse>>;
    async fn update_payment_channel(&self, code: &str, input: PaymentChannelUpdatePayload) -> RechargeResult<types::recharge::PaymentChannel>;
    async fn handle_payment_callback(&self, request: RechargePaymentCallbackRequest) -> RechargeResult<RechargePaymentCallbackResult>;
    async fn poll_pending_payment_orders(&self, limit: u64) -> RechargeResult<RechargePaymentPollResult>;
    async fn expire_pending_orders(&self) -> RechargeResult<u64>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RechargeOrderCreateRecord {
    pub order_no: String,
    pub user_id: String,
    pub package_id: Option<String>,
    pub package_name: String,
    pub recharge_amount: rust_decimal::Decimal,
    pub gift_amount: rust_decimal::Decimal,
    pub total_arrival_amount: rust_decimal::Decimal,
    pub payable_amount: rust_decimal::Decimal,
    pub payment_channel_code: String,
    pub payment_channel_name: String,
    pub payment_method: String,
    pub payment_request_json: serde_json::Value,
    pub expires_at: time::OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentChannelStoredConfig {
    pub config: serde_json::Value,
    pub encrypted_secret: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentChannelUpdateRecord {
    pub enabled: bool,
    pub config: Option<serde_json::Value>,
    pub encrypted_secret: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentCallbackCreateRecord {
    pub payment_channel_code: String,
    pub callback_kind: String,
    pub http_method: String,
    pub raw_params: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentCallbackUpdateRecord {
    pub order_no: Option<String>,
    pub provider_trade_no: Option<String>,
    pub payment_method: Option<String>,
    pub trade_status: Option<String>,
    pub status: String,
    pub settled: bool,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RechargePaymentSettlementRecord {
    pub order_no: String,
    pub payment_channel_code: String,
    pub provider_trade_no: Option<String>,
    pub payment_method: String,
    pub payable_amount: Option<rust_decimal::Decimal>,
    pub callback_payload: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RechargePaymentSettlementResult {
    pub order: RechargeOrder,
    pub settled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RechargePaymentCallbackRequest {
    pub channel_code: String,
    pub callback_kind: PaymentCallbackKind,
    pub http_method: String,
    pub payment: PaymentCallbackRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RechargePaymentCallbackResult {
    pub response_body: String,
    pub settled: bool,
    pub order_no: Option<String>,
    pub provider_trade_no: Option<String>,
    pub payment_method: Option<String>,
    pub trade_status: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RechargePaymentPollResult {
    pub checked: u64,
    pub paid: u64,
    pub unsupported: u64,
}

pub trait RechargeSecretCipher: Send + Sync + 'static {
    fn encrypt_secret(&self, plaintext: &str) -> RechargeResult<String>;
    fn decrypt_secret(&self, ciphertext: &str) -> RechargeResult<String>;
}

#[derive(Clone, Copy, Default)]
pub struct NoRechargeSecretCipher;

impl RechargeSecretCipher for NoRechargeSecretCipher {
    fn encrypt_secret(&self, _plaintext: &str) -> RechargeResult<String> {
        Err(super::RechargeError::Infrastructure("recharge secret cipher is not configured".into()))
    }

    fn decrypt_secret(&self, _ciphertext: &str) -> RechargeResult<String> {
        Err(super::RechargeError::Infrastructure("recharge secret cipher is not configured".into()))
    }
}
