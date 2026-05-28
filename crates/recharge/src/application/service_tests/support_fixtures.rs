use rust_decimal::Decimal;
use serde_json::json;
use types::{
    pagination::{Page, PageRequest},
    recharge::{PaymentChannel, RechargeOrder, RechargeOrderCreatePayload, RechargePackage, RechargePackageCreatePayload, RechargePackageUpdatePayload},
};

use super::support::timestamp;

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
        payment_channel_code: Some("testpay".into()),
        payment_channel_name: None,
        payment_method: Some("test".into()),
        provider_trade_no: None,
        payment_request_json: None,
        refund_status: None,
        refund_amount: None,
        paid_at: None,
        refunded_at: None,
        expires_at: timestamp(),
        created_at: timestamp(),
        updated_at: timestamp(),
    }
}

pub(super) fn payment_channel(enabled: bool) -> PaymentChannel {
    PaymentChannel {
        code: "testpay".into(),
        name: "Test Pay".into(),
        enabled,
        config: json!({}),
        secret_set: true,
        config_schema: None,
        registered_at: timestamp(),
        updated_at: timestamp(),
    }
}

pub(super) fn disabled_payment_channel(code: &str) -> PaymentChannel {
    PaymentChannel {
        code: code.into(),
        name: "Disabled Pay".into(),
        enabled: false,
        config: json!({}),
        secret_set: true,
        config_schema: None,
        registered_at: timestamp(),
        updated_at: timestamp(),
    }
}

pub(super) fn order_payload(package_id: &str) -> RechargeOrderCreatePayload {
    RechargeOrderCreatePayload {
        package_id: Some(package_id.into()),
        recharge_amount: None,
        payment_channel_code: "testpay".into(),
        payment_method: "test".into(),
        captcha_token: None,
    }
}

pub(super) fn custom_amount_order_payload(amount: Decimal) -> RechargeOrderCreatePayload {
    RechargeOrderCreatePayload {
        package_id: None,
        recharge_amount: Some(amount),
        payment_channel_code: "testpay".into(),
        payment_method: "test".into(),
        captcha_token: None,
    }
}

pub(super) fn page_request() -> PageRequest {
    PageRequest { page: 1, page_size: 10 }
}

pub(super) fn created_package(input: RechargePackageCreatePayload) -> RechargePackage {
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

pub(super) fn updated_package(id: &str, input: RechargePackageUpdatePayload) -> RechargePackage {
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

pub(super) fn page_response<T>(items: Vec<T>, page: PageRequest) -> Page<T> {
    Page {
        total: items.len() as u64,
        items,
        page: page.page,
        page_size: page.page_size,
    }
}
