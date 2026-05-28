use rust_decimal::Decimal;

use crate::application::{RechargeService, RechargeUseCase};

use super::{
    settings_support::system_settings,
    support::MemoryRechargeRepository,
    support_fixtures::{custom_amount_order_payload, order, order_payload, package, payment_channel},
    support_payment::PlainCipher,
    test_registry,
};

#[tokio::test]
async fn create_user_order_writes_pending_snapshot_without_wallet_effects() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_package(package("package-1", "Starter", Decimal::new(10, 0), Decimal::new(2, 0)));
    repository.insert_channel(payment_channel(true));
    let service = RechargeService::with_secret_cipher(repository.clone(), test_registry(), PlainCipher)
        .await
        .unwrap();

    let response = service.create_user_order("user-1", order_payload("package-1")).await.unwrap();

    assert_eq!(response.order.status, "pending");
    assert_eq!(response.order.user_id, "user-1");
    assert_eq!(response.order.package_name, "Starter");
    assert_eq!(response.order.recharge_amount, Decimal::new(10, 0));
    assert_eq!(response.order.gift_amount, Decimal::new(2, 0));
    assert_eq!(response.order.total_arrival_amount, Decimal::new(12, 0));
    assert_eq!(response.order.payable_amount, Decimal::new(70, 0));
    assert_eq!(response.order.payment_channel_code.as_deref(), Some("testpay"));
    assert_eq!(response.order.payment_method.as_deref(), Some("test"));
    assert_eq!(repository.orders().len(), 1);
}

#[tokio::test]
async fn create_user_order_accepts_custom_amount_without_package() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_channel(payment_channel(true));
    let service = RechargeService::with_secret_cipher(repository.clone(), test_registry(), PlainCipher)
        .await
        .unwrap();

    let response = service
        .create_user_order("user-1", custom_amount_order_payload(Decimal::new(12, 0)))
        .await
        .unwrap();

    assert_eq!(response.order.package_id, None);
    assert_eq!(response.order.package_name, "Custom recharge");
    assert_eq!(response.order.recharge_amount, Decimal::new(12, 0));
    assert_eq!(response.order.gift_amount, Decimal::ZERO);
    assert_eq!(response.order.total_arrival_amount, Decimal::new(12, 0));
    assert_eq!(response.order.payable_amount, Decimal::new(84, 0));
    assert_eq!(repository.orders().len(), 1);
}

#[tokio::test]
async fn create_user_order_rejects_package_and_custom_amount_together() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_package(package("package-1", "Starter", Decimal::new(10, 0), Decimal::ZERO));
    repository.insert_channel(payment_channel(true));
    let service = RechargeService::with_secret_cipher(repository, test_registry(), PlainCipher).await.unwrap();
    let mut payload = order_payload("package-1");
    payload.recharge_amount = Some(Decimal::new(10, 0));

    let result = service.create_user_order("user-1", payload).await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: package_id and recharge_amount cannot be used together"
    );
}

#[tokio::test]
async fn create_user_order_rejects_missing_package_and_custom_amount() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_channel(payment_channel(true));
    let service = RechargeService::with_secret_cipher(repository, test_registry(), PlainCipher).await.unwrap();
    let mut payload = custom_amount_order_payload(Decimal::new(10, 0));
    payload.recharge_amount = None;

    let result = service.create_user_order("user-1", payload).await;

    assert_eq!(result.unwrap_err().to_string(), "invalid input: package_id or recharge_amount is required");
}

#[tokio::test]
async fn create_user_order_rejects_disabled_recharge() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_package(package("package-1", "Starter", Decimal::new(10, 0), Decimal::ZERO));
    repository.insert_channel(payment_channel(true));
    let mut settings = system_settings();
    settings.recharge_enabled = false;
    repository.set_settings(settings);
    let service = RechargeService::with_secret_cipher(repository, test_registry(), PlainCipher).await.unwrap();

    let result = service.create_user_order("user-1", order_payload("package-1")).await;

    assert_eq!(result.unwrap_err().to_string(), "invalid input: recharge is disabled");
}

#[tokio::test]
async fn create_user_order_rejects_disabled_package() {
    let repository = MemoryRechargeRepository::default();
    let mut disabled = package("package-1", "Starter", Decimal::new(10, 0), Decimal::ZERO);
    disabled.status = types::recharge::RECHARGE_PACKAGE_STATUS_DISABLED.into();
    repository.insert_package(disabled);
    repository.insert_channel(payment_channel(true));
    let service = RechargeService::with_secret_cipher(repository, test_registry(), PlainCipher).await.unwrap();

    let result = service.create_user_order("user-1", order_payload("package-1")).await;

    assert_eq!(result.unwrap_err().to_string(), "invalid input: recharge package is disabled");
}

#[tokio::test]
async fn create_user_order_rejects_amount_outside_system_limits() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_package(package("package-1", "Starter", Decimal::new(10, 0), Decimal::ZERO));
    repository.insert_channel(payment_channel(true));
    let mut settings = system_settings();
    settings.recharge_max_amount = Decimal::new(5, 0);
    repository.set_settings(settings);
    let service = RechargeService::with_secret_cipher(repository, test_registry(), PlainCipher).await.unwrap();

    let result = service.create_user_order("user-1", order_payload("package-1")).await;

    assert_eq!(result.unwrap_err().to_string(), "invalid input: recharge amount exceeds maximum");
}

#[tokio::test]
async fn create_user_order_rejects_when_unpaid_order_limit_reached() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_package(package("package-1", "Starter", Decimal::new(10, 0), Decimal::ZERO));
    repository.insert_channel(payment_channel(true));
    repository.insert_order(order("order-1", "R1001", "pending"));
    let mut settings = system_settings();
    settings.recharge_max_unpaid_orders = 1;
    repository.set_settings(settings);
    let service = RechargeService::with_secret_cipher(repository, test_registry(), PlainCipher).await.unwrap();

    let result = service.create_user_order("user-1", order_payload("package-1")).await;

    assert_eq!(result.unwrap_err().to_string(), "recharge conflict: unpaid recharge order limit reached: 1");
}
