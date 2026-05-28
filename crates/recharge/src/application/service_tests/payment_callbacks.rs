use rust_decimal::Decimal;
use serde_json::json;
use std::{collections::BTreeMap, sync::Arc};

use crate::application::{PaymentCallbackKind, PaymentChannelRegistry, RechargePaymentCallbackRequest, RechargeService, RechargeUseCase};

use super::{
    support::MemoryRechargeRepository,
    support_fixtures::{order, payment_channel},
    support_payment::{CallbackPaymentProvider, PlainCipher},
};

#[tokio::test]
async fn handle_payment_callback_records_processed_callback() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_order(order("order-1", "R1001", "pending"));
    repository.insert_channel(payment_channel(true));
    let service = service(repository.clone(), paid_callback(None)).await;

    let result = service.handle_payment_callback(callback_request("POST", callback_params())).await.unwrap();

    assert_eq!(result.response_body, "success");
    assert!(result.settled);
    assert_eq!(repository.orders()[0].status, "paid");
    let callback = only_callback(&repository);
    assert_eq!(callback.status, types::recharge::PAYMENT_CALLBACK_STATUS_PROCESSED);
    assert_eq!(callback.callback_kind, types::recharge::PAYMENT_CALLBACK_KIND_NOTIFY);
    assert_eq!(callback.http_method, "POST");
    assert_eq!(callback.order_no.as_deref(), Some("R1001"));
    assert_eq!(callback.provider_trade_no.as_deref(), Some("trade-1"));
    assert_eq!(callback.payment_method.as_deref(), Some("alipay"));
    assert_eq!(callback.trade_status.as_deref(), Some("paid"));
    assert_eq!(callback.raw_params["out_trade_no"], "R1001");
    assert!(callback.settled);
    assert_eq!(callback.error_message, None);
    assert!(callback.processed_at.is_some());
}

#[tokio::test]
async fn handle_payment_callback_records_failed_verification() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_channel(payment_channel(true));
    let service = service(repository.clone(), paid_callback(Some("signature mismatch"))).await;

    let error = service.handle_payment_callback(callback_request("GET", callback_params())).await.unwrap_err();

    assert_eq!(error.to_string(), "payment error: payment verification failed: signature mismatch");
    let callback = only_callback(&repository);
    assert_eq!(callback.status, types::recharge::PAYMENT_CALLBACK_STATUS_FAILED);
    assert_eq!(callback.raw_params["sign"], "ok");
    assert_eq!(callback.order_no, None);
    assert!(!callback.settled);
    assert_eq!(
        callback.error_message.as_deref(),
        Some("payment error: payment verification failed: signature mismatch")
    );
    assert!(callback.processed_at.is_some());
}

async fn service(repository: MemoryRechargeRepository, provider: CallbackPaymentProvider) -> RechargeService<MemoryRechargeRepository, PlainCipher> {
    let registry = PaymentChannelRegistry::with_providers(vec![Arc::new(provider)]);
    RechargeService::with_secret_cipher(repository, registry, PlainCipher).await.unwrap()
}

fn callback_request(http_method: &str, params: BTreeMap<String, String>) -> RechargePaymentCallbackRequest {
    RechargePaymentCallbackRequest {
        channel_code: "testpay".into(),
        callback_kind: PaymentCallbackKind::Notify,
        http_method: http_method.into(),
        payment: payment::PaymentCallbackRequest {
            params,
            config: payment::PaymentChannelConfig {
                config: json!({}),
                secret: None,
            },
        },
    }
}

fn callback_params() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("out_trade_no".into(), "R1001".into()),
        ("trade_no".into(), "trade-1".into()),
        ("type".into(), "alipay".into()),
        ("money".into(), "10.00".into()),
        ("sign".into(), "ok".into()),
    ])
}

fn paid_callback(verification_error: Option<&str>) -> CallbackPaymentProvider {
    CallbackPaymentProvider {
        verification_error: verification_error.map(str::to_owned),
        verified: payment::VerifiedPaymentCallback {
            order_no: "R1001".into(),
            provider_trade_no: Some("trade-1".into()),
            payment_method: "alipay".into(),
            amount: Some(Decimal::new(10, 0)),
            trade_status: payment::PaymentOrderStatus::Paid,
            raw_payload: json!({"out_trade_no": "R1001", "trade_no": "trade-1"}),
        },
    }
}

fn only_callback(repository: &MemoryRechargeRepository) -> types::recharge::PaymentCallbackRecord {
    let callbacks = repository.callbacks();
    assert_eq!(callbacks.len(), 1);
    callbacks[0].clone()
}
