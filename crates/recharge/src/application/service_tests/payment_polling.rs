use rust_decimal::Decimal;
use std::sync::{Arc, Mutex};

use crate::application::{PaymentChannelRegistry, RechargeService, RechargeUseCase};

use super::{
    support::MemoryRechargeRepository,
    support_fixtures::{order, payment_channel},
    support_payment::{PlainCipher, QueryPaymentProvider},
    test_registry,
};

#[tokio::test]
async fn poll_pending_payment_orders_settles_paid_provider_orders() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_order(order("order-1", "R1001", "pending"));
    repository.insert_channel(payment_channel(true));
    let queried = Arc::new(Mutex::new(Vec::new()));
    let registry = PaymentChannelRegistry::with_providers(vec![Arc::new(QueryPaymentProvider {
        status: payment::PaymentOrderStatus::Paid,
        queried: queried.clone(),
        provider_trade_no: Some("query-trade-1".into()),
        amount: Some(Decimal::new(10, 0)),
    })]);
    let service = RechargeService::with_secret_cipher(repository.clone(), registry, PlainCipher).await.unwrap();

    let result = service.poll_pending_payment_orders(10).await.unwrap();

    assert_eq!(result.checked, 1);
    assert_eq!(result.paid, 1);
    assert_eq!(result.unsupported, 0);
    assert_eq!(queried.lock().unwrap().as_slice(), ["R1001"]);
    assert_eq!(repository.orders()[0].status, "paid");
}

#[tokio::test]
async fn poll_pending_payment_orders_rejects_mismatched_provider_amount() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_order(order("order-1", "R1001", "pending"));
    repository.insert_channel(payment_channel(true));
    let registry = PaymentChannelRegistry::with_providers(vec![Arc::new(QueryPaymentProvider {
        status: payment::PaymentOrderStatus::Paid,
        queried: Arc::new(Mutex::new(Vec::new())),
        provider_trade_no: Some("query-trade-1".into()),
        amount: Some(Decimal::new(11, 0)),
    })]);
    let service = RechargeService::with_secret_cipher(repository.clone(), registry, PlainCipher).await.unwrap();

    let result = service.poll_pending_payment_orders(10).await;

    assert_eq!(result.unwrap_err().to_string(), "recharge conflict: payment amount mismatch");
    assert_eq!(repository.orders()[0].status, "pending");
}

#[tokio::test]
async fn poll_pending_payment_orders_requires_provider_trade_no() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_order(order("order-1", "R1001", "pending"));
    repository.insert_channel(payment_channel(true));
    let registry = PaymentChannelRegistry::with_providers(vec![Arc::new(QueryPaymentProvider {
        status: payment::PaymentOrderStatus::Paid,
        queried: Arc::new(Mutex::new(Vec::new())),
        provider_trade_no: None,
        amount: Some(Decimal::new(10, 0)),
    })]);
    let service = RechargeService::with_secret_cipher(repository.clone(), registry, PlainCipher).await.unwrap();

    let result = service.poll_pending_payment_orders(10).await;

    assert_eq!(result.unwrap_err().to_string(), "recharge conflict: provider trade number is required");
    assert_eq!(repository.orders()[0].status, "pending");
}

#[tokio::test]
async fn poll_pending_payment_orders_counts_unsupported_channels() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_order(order("order-1", "R1001", "pending"));
    repository.insert_channel(payment_channel(true));
    let service = RechargeService::with_secret_cipher(repository.clone(), test_registry(), PlainCipher)
        .await
        .unwrap();

    let result = service.poll_pending_payment_orders(10).await.unwrap();

    assert_eq!(result.checked, 1);
    assert_eq!(result.paid, 0);
    assert_eq!(result.unsupported, 1);
    assert_eq!(repository.orders()[0].status, "pending");
}
