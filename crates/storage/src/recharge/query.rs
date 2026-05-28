use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};
use types::recharge::{PaymentCallbackListFilters, RECHARGE_PACKAGE_STATUS_ACTIVE, RechargeOrderListFilters, RechargePackageListFilters};

use crate::user::UserColumn;

use super::record::{payment_callback_records, recharge_orders as recharge_order_records, recharge_packages as recharge_package_records};

pub(super) fn filtered_packages(filters: RechargePackageListFilters) -> sea_orm::Select<recharge_package_records::Entity> {
    let mut query = recharge_package_records::Entity::find();
    if let Some(status) = filters.status.filter(|value| !value.is_empty()) {
        query = query.filter(recharge_package_records::Column::Status.eq(status));
    }
    match filters.search {
        Some(search) if !search.is_empty() => query.filter(package_search_condition(&search)),
        _ => query,
    }
}

pub(super) fn filtered_orders(filters: RechargeOrderListFilters) -> sea_orm::SelectTwo<recharge_order_records::Entity, crate::user::UserEntity> {
    let mut query = recharge_order_records::Entity::find().find_also_related(crate::user::UserEntity);
    if let Some(status) = filters.status.filter(|value| !value.is_empty()) {
        query = query.filter(recharge_order_records::Column::Status.eq(status));
    }
    match filters.search {
        Some(search) if !search.is_empty() => query.filter(order_search_condition(&search)),
        _ => query,
    }
}

pub(super) fn active_packages() -> sea_orm::Select<recharge_package_records::Entity> {
    recharge_package_records::Entity::find().filter(recharge_package_records::Column::Status.eq(RECHARGE_PACKAGE_STATUS_ACTIVE))
}

pub(super) fn user_orders(user_id: &str) -> sea_orm::Select<recharge_order_records::Entity> {
    recharge_order_records::Entity::find().filter(recharge_order_records::Column::UserId.eq(user_id.to_owned()))
}

pub(super) fn filtered_payment_callbacks(filters: PaymentCallbackListFilters) -> sea_orm::Select<payment_callback_records::Entity> {
    let mut query = payment_callback_records::Entity::find();
    if let Some(status) = filters.status.filter(|value| !value.is_empty()) {
        query = query.filter(payment_callback_records::Column::Status.eq(status));
    }
    match filters.search {
        Some(search) if !search.is_empty() => query.filter(payment_callback_search_condition(&search)),
        _ => query,
    }
}

fn package_search_condition(search: &str) -> Condition {
    Condition::any()
        .add(recharge_package_records::Column::Name.contains(search))
        .add(recharge_package_records::Column::Description.contains(search))
}

fn order_search_condition(search: &str) -> Condition {
    Condition::any()
        .add(recharge_order_records::Column::OrderNo.contains(search))
        .add(recharge_order_records::Column::PackageName.contains(search))
        .add(UserColumn::Username.contains(search))
        .add(UserColumn::Email.contains(search))
}

fn payment_callback_search_condition(search: &str) -> Condition {
    Condition::any()
        .add(payment_callback_records::Column::OrderNo.contains(search))
        .add(payment_callback_records::Column::ProviderTradeNo.contains(search))
        .add(payment_callback_records::Column::PaymentChannelCode.contains(search))
}
