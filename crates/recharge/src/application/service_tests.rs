use rust_decimal::Decimal;
use std::sync::Arc;
use types::{
    pagination::PageRequest,
    recharge::{PaymentChannel, RECHARGE_PACKAGE_STATUS_DISABLED, RechargeOrderListFilters, RechargePackageListFilters, RechargePackageUpdatePayload},
};

use crate::application::{PaymentChannelRegistration, PaymentChannelRegistry, RechargeService, RechargeUseCase};

use self::settings_support::system_settings;
use self::support::MemoryRechargeRepository;
use self::support_fixtures::{create_payload, disabled_payment_channel, order, package, page_request, payment_channel};
use self::support_payment::{PlainCipher, TestPaymentProvider};

#[path = "service_tests/order_creation.rs"]
mod order_creation;
#[path = "service_tests/order_expiration.rs"]
mod order_expiration;
#[path = "service_tests/payment_callbacks.rs"]
mod payment_callbacks;
#[path = "service_tests/payment_polling.rs"]
mod payment_polling;
#[path = "service_tests/settings_support.rs"]
mod settings_support;
#[path = "service_tests/support.rs"]
mod support;
#[path = "service_tests/support_fixtures.rs"]
mod support_fixtures;
#[path = "service_tests/support_payment.rs"]
mod support_payment;

#[tokio::test]
async fn new_syncs_registered_payment_channels() {
    let repository = MemoryRechargeRepository::default();
    let registry = test_registry();

    RechargeService::new(repository.clone(), registry).await.unwrap();

    assert_eq!(
        repository.synced_channels(),
        vec![PaymentChannelRegistration {
            code: "testpay".into(),
            name: "Test Pay".into(),
        }]
    );
}

#[tokio::test]
async fn new_with_empty_registry_exposes_empty_channels() {
    let repository = MemoryRechargeRepository::default();
    let service = RechargeService::new(repository, PaymentChannelRegistry::empty()).await.unwrap();

    let channels = service.list_payment_channels().await.unwrap();

    assert_eq!(channels, Vec::<PaymentChannel>::new());
}

#[tokio::test]
async fn list_user_payment_channels_returns_enabled_public_methods_only() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_channel(payment_channel(true));
    repository.insert_channel(disabled_payment_channel("disabledpay"));
    let service = RechargeService::with_secret_cipher(repository, test_registry(), PlainCipher).await.unwrap();

    let channels = service.list_user_payment_channels().await.unwrap();

    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0].code, "testpay");
    assert_eq!(channels[0].methods[0].code, "test");
}

#[tokio::test]
async fn create_package_sanitizes_input_and_defaults_active_status() {
    let repository = MemoryRechargeRepository::default();
    let service = RechargeService::new(repository.clone(), PaymentChannelRegistry::empty()).await.unwrap();

    let created = service.create_package(create_payload("  Starter  ")).await.unwrap();

    assert_eq!(created.name, "Starter");
    assert_eq!(created.description.as_deref(), None);
    assert_eq!(created.status, "active");
    assert_eq!(repository.packages().len(), 1);
}

#[tokio::test]
async fn update_package_rejects_disabled_negative_gift_amount() {
    let repository = MemoryRechargeRepository::default();
    let service = RechargeService::new(repository, PaymentChannelRegistry::empty()).await.unwrap();

    let result = service
        .update_package(
            "package-1",
            RechargePackageUpdatePayload {
                name: "Starter".into(),
                description: None,
                recharge_amount: Decimal::new(10, 0),
                gift_amount: Decimal::new(-1, 0),
                status: "disabled".into(),
                sort_order: 0,
            },
        )
        .await;

    assert_eq!(result.unwrap_err().to_string(), "invalid input: gift_amount must be greater than or equal to 0");
}

#[tokio::test]
async fn package_response_calculates_total_arrival_amount() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_package(package("package-1", "Starter", Decimal::new(10, 0), Decimal::new(2, 0)));
    let service = RechargeService::new(repository, PaymentChannelRegistry::empty()).await.unwrap();

    let response = service.list_packages(page_request(), RechargePackageListFilters::default()).await.unwrap();

    assert_eq!(response.items[0].total_arrival_amount, Decimal::new(12, 0));
}

#[tokio::test]
async fn list_orders_passes_pagination_and_filters_to_repository() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_order(order("order-1", "PENDING-1", "pending"));
    let service = RechargeService::new(repository.clone(), PaymentChannelRegistry::empty()).await.unwrap();

    let response = service
        .list_orders(
            PageRequest { page: 2, page_size: 25 },
            RechargeOrderListFilters {
                search: Some("PENDING".into()),
                status: Some("pending".into()),
            },
        )
        .await
        .unwrap();

    assert_eq!(response.total, 1);
    assert_eq!(
        repository.last_order_query().unwrap(),
        (
            PageRequest { page: 2, page_size: 25 },
            RechargeOrderListFilters {
                search: Some("PENDING".into()),
                status: Some("pending".into()),
            },
        )
    );
}

#[tokio::test]
async fn list_orders_rejects_unsupported_status() {
    let repository = MemoryRechargeRepository::default();
    let service = RechargeService::new(repository, PaymentChannelRegistry::empty()).await.unwrap();

    let result = service
        .list_orders(
            page_request(),
            RechargeOrderListFilters {
                search: None,
                status: Some("settled".into()),
            },
        )
        .await;

    assert_eq!(result.unwrap_err().to_string(), "invalid input: order status is unsupported");
}

#[tokio::test]
async fn list_user_packages_returns_active_packages_with_payable_preview() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_package(package("package-1", "Starter", Decimal::new(10, 0), Decimal::new(2, 0)));
    let mut disabled = package("package-2", "Disabled", Decimal::new(20, 0), Decimal::ZERO);
    disabled.status = RECHARGE_PACKAGE_STATUS_DISABLED.into();
    repository.insert_package(disabled);
    let service = RechargeService::new(repository, PaymentChannelRegistry::empty()).await.unwrap();

    let response = service.list_user_packages(page_request()).await.unwrap();

    assert!(response.recharge_enabled);
    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].id, "package-1");
    assert_eq!(response.items[0].estimated_payable_amount, Decimal::new(70, 0));
}

#[tokio::test]
async fn update_payment_channel_rejects_enabled_channel_without_public_base_url() {
    let repository = MemoryRechargeRepository::default();
    let mut settings = system_settings();
    settings.public_base_url = String::new();
    repository.set_settings(settings);
    let service = RechargeService::with_secret_cipher(repository, test_registry(), PlainCipher).await.unwrap();

    let result = service.update_payment_channel("testpay", payment_channel_update(true)).await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: public_base_url is required before enabling payment channel"
    );
}

#[tokio::test]
async fn update_payment_channel_rejects_enabled_channel_with_invalid_public_base_url() {
    let repository = MemoryRechargeRepository::default();
    let mut settings = system_settings();
    settings.public_base_url = "https://".into();
    repository.set_settings(settings);
    let service = RechargeService::with_secret_cipher(repository, test_registry(), PlainCipher).await.unwrap();

    let result = service.update_payment_channel("testpay", payment_channel_update(true)).await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: public_base_url must be a valid HTTP or HTTPS URL"
    );
}

#[tokio::test]
async fn update_payment_channel_allows_disabled_channel_without_public_base_url() {
    let repository = MemoryRechargeRepository::default();
    let mut settings = system_settings();
    settings.public_base_url = String::new();
    repository.set_settings(settings);
    let service = RechargeService::with_secret_cipher(repository, test_registry(), PlainCipher).await.unwrap();

    let channel = service.update_payment_channel("testpay", payment_channel_update(false)).await.unwrap();

    assert!(!channel.enabled);
}

fn test_registry() -> PaymentChannelRegistry {
    PaymentChannelRegistry::with_providers(vec![Arc::new(TestPaymentProvider)])
}

fn payment_channel_update(enabled: bool) -> types::recharge::PaymentChannelUpdatePayload {
    types::recharge::PaymentChannelUpdatePayload {
        enabled,
        config: Some(serde_json::json!({})),
        api_key: Some("secret".into()),
    }
}
