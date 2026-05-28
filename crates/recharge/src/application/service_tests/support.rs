use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;
use types::{
    pagination::{Page, PageRequest},
    recharge::{
        PaymentCallbackListFilters, PaymentCallbackRecord, PaymentChannel, RECHARGE_ORDER_STATUS_PENDING, RECHARGE_PACKAGE_STATUS_ACTIVE, RechargeOrder,
        RechargeOrderListFilters, RechargePackage, RechargePackageCreatePayload, RechargePackageListFilters, RechargePackageUpdatePayload,
    },
    system_setting::SystemSettings,
};

use crate::application::{
    PaymentCallbackCreateRecord, PaymentCallbackUpdateRecord, PaymentChannelRegistration, PaymentChannelStoredConfig, PaymentChannelUpdateRecord,
    RechargeOrderCreateRecord, RechargePaymentSettlementRecord, RechargePaymentSettlementResult, RechargeRepository, RechargeResult,
};

use super::settings_support::system_settings;
use super::support_fixtures::{created_package, page_response, updated_package};

#[derive(Clone, Default)]
pub(super) struct MemoryRechargeRepository {
    state: Arc<Mutex<MemoryState>>,
}

struct MemoryState {
    packages: Vec<RechargePackage>,
    orders: Vec<RechargeOrder>,
    callbacks: Vec<PaymentCallbackRecord>,
    channels: Vec<PaymentChannel>,
    synced_channels: Vec<PaymentChannelRegistration>,
    last_order_query: Option<(PageRequest, RechargeOrderListFilters)>,
    settings: SystemSettings,
}

impl MemoryRechargeRepository {
    pub(super) fn insert_package(&self, package: RechargePackage) {
        self.state.lock().unwrap().packages.push(package);
    }

    pub(super) fn insert_order(&self, order: RechargeOrder) {
        self.state.lock().unwrap().orders.push(order);
    }

    pub(super) fn insert_channel(&self, channel: PaymentChannel) {
        self.state.lock().unwrap().channels.push(channel);
    }

    pub(super) fn packages(&self) -> Vec<RechargePackage> {
        self.state.lock().unwrap().packages.clone()
    }

    pub(super) fn synced_channels(&self) -> Vec<PaymentChannelRegistration> {
        self.state.lock().unwrap().synced_channels.clone()
    }

    pub(super) fn last_order_query(&self) -> Option<(PageRequest, RechargeOrderListFilters)> {
        self.state.lock().unwrap().last_order_query.clone()
    }

    pub(super) fn set_settings(&self, settings: SystemSettings) {
        self.state.lock().unwrap().settings = settings;
    }

    pub(super) fn orders(&self) -> Vec<RechargeOrder> {
        self.state.lock().unwrap().orders.clone()
    }

    pub(super) fn callbacks(&self) -> Vec<PaymentCallbackRecord> {
        self.state.lock().unwrap().callbacks.clone()
    }
}

#[async_trait]
impl RechargeRepository for MemoryRechargeRepository {
    async fn list_packages(&self, page: PageRequest, filters: RechargePackageListFilters) -> RechargeResult<Page<RechargePackage>> {
        let items = filtered_items(&self.state.lock().unwrap().packages, filters.status.as_ref());
        Ok(page_response(items, page))
    }

    async fn list_active_packages(&self, page: PageRequest) -> RechargeResult<Page<RechargePackage>> {
        let items = self
            .state
            .lock()
            .unwrap()
            .packages
            .iter()
            .filter(|package| package.status == RECHARGE_PACKAGE_STATUS_ACTIVE)
            .cloned()
            .collect();
        Ok(page_response(items, page))
    }

    async fn create_package(&self, input: RechargePackageCreatePayload) -> RechargeResult<RechargePackage> {
        let package = created_package(input);
        self.insert_package(package.clone());
        Ok(package)
    }

    async fn update_package(&self, id: &str, input: RechargePackageUpdatePayload) -> RechargeResult<RechargePackage> {
        let package = updated_package(id, input);
        self.insert_package(package.clone());
        Ok(package)
    }

    async fn find_package(&self, id: &str) -> RechargeResult<Option<RechargePackage>> {
        Ok(self.state.lock().unwrap().packages.iter().find(|package| package.id == id).cloned())
    }

    async fn list_orders(&self, page: PageRequest, filters: RechargeOrderListFilters) -> RechargeResult<Page<RechargeOrder>> {
        self.state.lock().unwrap().last_order_query = Some((page, filters.clone()));
        let items = filtered_items(&self.state.lock().unwrap().orders, filters.status.as_ref());
        Ok(page_response(items, page))
    }

    async fn list_user_orders(&self, user_id: &str, page: PageRequest) -> RechargeResult<Page<RechargeOrder>> {
        let items = self
            .state
            .lock()
            .unwrap()
            .orders
            .iter()
            .filter(|order| order.user_id == user_id)
            .cloned()
            .collect();
        Ok(page_response(items, page))
    }

    async fn list_pending_unexpired_orders(&self, limit: u64) -> RechargeResult<Vec<RechargeOrder>> {
        let items = self
            .state
            .lock()
            .unwrap()
            .orders
            .iter()
            .filter(|order| order.status == RECHARGE_ORDER_STATUS_PENDING)
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(items)
    }

    async fn list_payment_callbacks(&self, page: PageRequest, filters: PaymentCallbackListFilters) -> RechargeResult<Page<PaymentCallbackRecord>> {
        let items = filtered_items(&self.state.lock().unwrap().callbacks, filters.status.as_ref());
        Ok(page_response(items, page))
    }

    async fn create_payment_callback(&self, input: PaymentCallbackCreateRecord) -> RechargeResult<PaymentCallbackRecord> {
        let callback = callback_from_create(input);
        self.state.lock().unwrap().callbacks.push(callback.clone());
        Ok(callback)
    }

    async fn update_payment_callback(&self, id: &str, input: PaymentCallbackUpdateRecord) -> RechargeResult<PaymentCallbackRecord> {
        let mut guard = self.state.lock().unwrap();
        let callback = guard
            .callbacks
            .iter_mut()
            .find(|callback| callback.id == id)
            .expect("test callback should exist");
        apply_callback_update(callback, input);
        Ok(callback.clone())
    }

    async fn create_order(&self, input: RechargeOrderCreateRecord, max_unpaid_orders: u64) -> RechargeResult<RechargeOrder> {
        let order = order_from_record(input);
        let mut guard = self.state.lock().unwrap();
        let pending_count = guard
            .orders
            .iter()
            .filter(|item| item.user_id == order.user_id && item.status == RECHARGE_ORDER_STATUS_PENDING)
            .count() as u64;
        if pending_count >= max_unpaid_orders {
            return Err(crate::application::RechargeError::Conflict(format!(
                "unpaid recharge order limit reached: {max_unpaid_orders}"
            )));
        }
        guard.orders.push(order.clone());
        Ok(order)
    }

    async fn list_payment_channels(&self) -> RechargeResult<Vec<PaymentChannel>> {
        Ok(self.state.lock().unwrap().channels.clone())
    }

    async fn find_payment_channel(&self, code: &str) -> RechargeResult<Option<PaymentChannel>> {
        Ok(self.state.lock().unwrap().channels.iter().find(|channel| channel.code == code).cloned())
    }

    async fn payment_channel_config(&self, code: &str) -> RechargeResult<PaymentChannelStoredConfig> {
        let channel = self.find_payment_channel(code).await?.expect("test channel should exist");
        Ok(PaymentChannelStoredConfig {
            config: channel.config,
            encrypted_secret: Some("secret".into()),
        })
    }

    async fn update_payment_channel(&self, code: &str, input: PaymentChannelUpdateRecord) -> RechargeResult<PaymentChannel> {
        Ok(PaymentChannel {
            code: code.into(),
            name: code.into(),
            enabled: input.enabled,
            config: input.config.unwrap_or_else(|| json!({})),
            secret_set: input.encrypted_secret.is_some(),
            config_schema: None,
            registered_at: timestamp(),
            updated_at: timestamp(),
        })
    }

    async fn sync_payment_channels(&self, registrations: &[PaymentChannelRegistration]) -> RechargeResult<()> {
        self.state.lock().unwrap().synced_channels = registrations.to_vec();
        Ok(())
    }

    async fn get_system_settings(&self) -> RechargeResult<SystemSettings> {
        Ok(self.state.lock().unwrap().settings.clone())
    }

    async fn settle_paid_order(&self, input: RechargePaymentSettlementRecord) -> RechargeResult<RechargePaymentSettlementResult> {
        let mut guard = self.state.lock().unwrap();
        let order = guard
            .orders
            .iter_mut()
            .find(|order| order.order_no == input.order_no)
            .expect("test order should exist");
        if order.payment_channel_code.as_deref() != Some(input.payment_channel_code.as_str()) {
            return Err(crate::application::RechargeError::Conflict("payment channel mismatch".into()));
        }
        if input.provider_trade_no.as_deref().is_none_or(|value| value.trim().is_empty()) {
            return Err(crate::application::RechargeError::Conflict("provider trade number is required".into()));
        }
        if input.payable_amount.is_none() {
            return Err(crate::application::RechargeError::Conflict("payment amount is required".into()));
        }
        if input.payable_amount.is_some_and(|amount| amount != order.payable_amount) {
            return Err(crate::application::RechargeError::Conflict("payment amount mismatch".into()));
        }
        order.status = types::recharge::RECHARGE_ORDER_STATUS_PAID.into();
        order.payment_method = Some(input.payment_method);
        order.provider_trade_no = input.provider_trade_no;
        let order = order.clone();
        Ok(RechargePaymentSettlementResult { order, settled: true })
    }
}

impl Default for MemoryState {
    fn default() -> Self {
        Self {
            packages: Vec::new(),
            orders: Vec::new(),
            callbacks: Vec::new(),
            channels: Vec::new(),
            synced_channels: Vec::new(),
            last_order_query: None,
            settings: system_settings(),
        }
    }
}

fn order_from_record(input: RechargeOrderCreateRecord) -> RechargeOrder {
    RechargeOrder {
        id: "order-created".into(),
        order_no: input.order_no,
        user_id: input.user_id,
        username: String::new(),
        user_email: String::new(),
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
        provider_trade_no: None,
        payment_request_json: Some(input.payment_request_json),
        refund_status: None,
        refund_amount: None,
        paid_at: None,
        refunded_at: None,
        expires_at: timestamp(),
        created_at: timestamp(),
        updated_at: timestamp(),
    }
}

fn filtered_items<T: Clone + HasStatus>(items: &[T], status: Option<&String>) -> Vec<T> {
    items.iter().filter(|item| status.is_none_or(|value| item.status() == value)).cloned().collect()
}

trait HasStatus {
    fn status(&self) -> &str;
}

impl HasStatus for RechargePackage {
    fn status(&self) -> &str {
        &self.status
    }
}

impl HasStatus for RechargeOrder {
    fn status(&self) -> &str {
        &self.status
    }
}

impl HasStatus for PaymentCallbackRecord {
    fn status(&self) -> &str {
        &self.status
    }
}

fn callback_from_create(input: PaymentCallbackCreateRecord) -> PaymentCallbackRecord {
    PaymentCallbackRecord {
        id: format!("callback-{}", input.payment_channel_code),
        payment_channel_code: input.payment_channel_code,
        callback_kind: input.callback_kind,
        http_method: input.http_method,
        order_no: None,
        provider_trade_no: None,
        payment_method: None,
        trade_status: None,
        status: types::recharge::PAYMENT_CALLBACK_STATUS_RECEIVED.into(),
        settled: false,
        error_message: None,
        raw_params: input.raw_params,
        received_at: timestamp(),
        processed_at: None,
    }
}

fn apply_callback_update(callback: &mut PaymentCallbackRecord, input: PaymentCallbackUpdateRecord) {
    callback.order_no = input.order_no;
    callback.provider_trade_no = input.provider_trade_no;
    callback.payment_method = input.payment_method;
    callback.trade_status = input.trade_status;
    callback.status = input.status;
    callback.settled = input.settled;
    callback.error_message = input.error_message;
    callback.processed_at = Some(timestamp());
}

pub(super) fn timestamp() -> String {
    "2026-05-25T00:00:00Z".into()
}
