use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    pagination::{Page, PageRequest},
    recharge::{
        PaymentChannel, PaymentChannelUpdatePayload, RECHARGE_ORDER_STATUS_PENDING, RECHARGE_PACKAGE_STATUS_ACTIVE, RechargeOrder, RechargeOrderListFilters,
        RechargePackage, RechargePackageCreatePayload, RechargePackageListFilters, RechargePackageUpdatePayload,
    },
    system_setting::SystemSettings,
};

use crate::application::{PaymentChannelRegistration, RechargeOrderCreateRecord, RechargeRepository, RechargeResult};

use super::settings_support::system_settings;

#[derive(Clone, Default)]
pub(super) struct MemoryRechargeRepository {
    state: Arc<Mutex<MemoryState>>,
}

struct MemoryState {
    packages: Vec<RechargePackage>,
    orders: Vec<RechargeOrder>,
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

    async fn create_order(&self, input: RechargeOrderCreateRecord) -> RechargeResult<RechargeOrder> {
        let order = order_from_record(input);
        self.insert_order(order.clone());
        Ok(order)
    }

    async fn list_payment_channels(&self) -> RechargeResult<Vec<PaymentChannel>> {
        Ok(self.state.lock().unwrap().channels.clone())
    }

    async fn update_payment_channel(&self, code: &str, input: PaymentChannelUpdatePayload) -> RechargeResult<PaymentChannel> {
        Ok(PaymentChannel {
            code: code.into(),
            name: code.into(),
            enabled: input.enabled,
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
}

impl Default for MemoryState {
    fn default() -> Self {
        Self {
            packages: Vec::new(),
            orders: Vec::new(),
            channels: Vec::new(),
            synced_channels: Vec::new(),
            last_order_query: None,
            settings: system_settings(),
        }
    }
}

pub(super) fn create_payload(name: &str) -> RechargePackageCreatePayload {
    RechargePackageCreatePayload {
        name: name.into(),
        description: Some("   ".into()),
        recharge_amount: Decimal::new(10, 0),
        gift_amount: Decimal::new(2, 0),
        status: None,
        sort_order: 0,
    }
}

pub(super) fn package(id: &str, name: &str, recharge_amount: Decimal, gift_amount: Decimal) -> RechargePackage {
    RechargePackage {
        id: id.into(),
        name: name.into(),
        description: None,
        recharge_amount,
        gift_amount,
        status: "active".into(),
        sort_order: 0,
        created_at: timestamp(),
        updated_at: timestamp(),
    }
}

pub(super) fn order(id: &str, order_no: &str, status: &str) -> RechargeOrder {
    RechargeOrder {
        id: id.into(),
        order_no: order_no.into(),
        user_id: "user-1".into(),
        username: "alice".into(),
        user_email: "alice@example.com".into(),
        package_id: Some("package-1".into()),
        package_name: "Starter".into(),
        recharge_amount: Decimal::new(10, 0),
        gift_amount: Decimal::new(2, 0),
        total_arrival_amount: Decimal::new(12, 0),
        payable_amount: Decimal::new(10, 0),
        status: status.into(),
        payment_channel_code: None,
        payment_channel_name: None,
        expires_at: timestamp(),
        created_at: timestamp(),
        updated_at: timestamp(),
    }
}

pub(super) fn page_request() -> PageRequest {
    PageRequest { page: 1, page_size: 10 }
}

fn created_package(input: RechargePackageCreatePayload) -> RechargePackage {
    RechargePackage {
        id: "package-created".into(),
        name: input.name,
        description: input.description,
        recharge_amount: input.recharge_amount,
        gift_amount: input.gift_amount,
        status: input.status.unwrap_or_else(|| "active".into()),
        sort_order: input.sort_order,
        created_at: timestamp(),
        updated_at: timestamp(),
    }
}

fn updated_package(id: &str, input: RechargePackageUpdatePayload) -> RechargePackage {
    RechargePackage {
        id: id.into(),
        name: input.name,
        description: input.description,
        recharge_amount: input.recharge_amount,
        gift_amount: input.gift_amount,
        status: input.status,
        sort_order: input.sort_order,
        created_at: timestamp(),
        updated_at: timestamp(),
    }
}

fn order_from_record(input: RechargeOrderCreateRecord) -> RechargeOrder {
    RechargeOrder {
        id: "order-created".into(),
        order_no: "Rordercreated".into(),
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
        payment_channel_code: None,
        payment_channel_name: None,
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

fn page_response<T>(items: Vec<T>, page: PageRequest) -> Page<T> {
    Page {
        total: items.len() as u64,
        items,
        page: page.page,
        page_size: page.page_size,
    }
}

fn timestamp() -> String {
    "2026-05-25T00:00:00Z".into()
}
