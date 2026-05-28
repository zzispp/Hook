use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use payment::{
    PaymentCallbackEndpoint, PaymentCallbackEndpointKind, PaymentChannelConfig, PaymentChannelRegistration, PaymentOrderAction, VerifiedPaymentCallback,
};

use crate::application::{RechargeResult, RechargeSecretCipher};

#[derive(Clone, Copy)]
pub(super) struct PlainCipher;

impl RechargeSecretCipher for PlainCipher {
    fn encrypt_secret(&self, plaintext: &str) -> RechargeResult<String> {
        Ok(plaintext.into())
    }

    fn decrypt_secret(&self, ciphertext: &str) -> RechargeResult<String> {
        Ok(ciphertext.into())
    }
}

#[derive(Clone)]
pub(super) struct TestPaymentProvider;

#[derive(Clone)]
pub(super) struct QueryPaymentProvider {
    pub status: payment::PaymentOrderStatus,
    pub queried: Arc<Mutex<Vec<String>>>,
    pub provider_trade_no: Option<String>,
    pub amount: Option<rust_decimal::Decimal>,
}

#[derive(Clone)]
pub(super) struct CallbackPaymentProvider {
    pub verified: VerifiedPaymentCallback,
    pub verification_error: Option<String>,
}

#[async_trait]
impl payment::PaymentChannelProvider for TestPaymentProvider {
    fn registration(&self) -> PaymentChannelRegistration {
        PaymentChannelRegistration {
            code: "testpay".into(),
            name: "Test Pay".into(),
        }
    }

    fn config_schema(&self) -> payment::PaymentChannelConfigSchema {
        payment::PaymentChannelConfigSchema {
            fields: Vec::new(),
            methods: vec![payment::PaymentMethodOption {
                code: "test".into(),
                name: "Test".into(),
            }],
        }
    }

    fn callback_endpoints(&self) -> Vec<PaymentCallbackEndpoint> {
        vec![
            callback_endpoint(PaymentCallbackEndpointKind::Notify, "/payment/{code}/notify"),
            callback_endpoint(PaymentCallbackEndpointKind::Return, "/payment/{code}/return"),
        ]
    }

    async fn create_payment_order(&self, request: payment::PaymentOrderRequest) -> payment::PaymentResult<PaymentOrderAction> {
        Ok(PaymentOrderAction::FormPost {
            action: "https://pay.test/submit".into(),
            method: "POST".into(),
            fields: std::collections::BTreeMap::from([("out_trade_no".into(), request.order_no)]),
        })
    }

    async fn query_payment_order(&self, _order_no: &str, _config: PaymentChannelConfig) -> payment::PaymentResult<payment::PaymentOrderQueryResult> {
        Err(payment::PaymentError::Unsupported("test query unsupported".into()))
    }

    async fn refund_payment_order(&self, _request: payment::PaymentRefundRequest) -> payment::PaymentResult<payment::PaymentRefundResult> {
        Err(payment::PaymentError::Unsupported("test refund unsupported".into()))
    }

    fn verify_callback(&self, _request: payment::PaymentCallbackRequest) -> payment::PaymentResult<payment::VerifiedPaymentCallback> {
        Err(payment::PaymentError::Unsupported("test callback unsupported".into()))
    }
}

#[async_trait]
impl payment::PaymentChannelProvider for QueryPaymentProvider {
    fn registration(&self) -> PaymentChannelRegistration {
        TestPaymentProvider.registration()
    }

    fn config_schema(&self) -> payment::PaymentChannelConfigSchema {
        TestPaymentProvider.config_schema()
    }

    fn callback_endpoints(&self) -> Vec<PaymentCallbackEndpoint> {
        TestPaymentProvider.callback_endpoints()
    }

    async fn create_payment_order(&self, request: payment::PaymentOrderRequest) -> payment::PaymentResult<PaymentOrderAction> {
        TestPaymentProvider.create_payment_order(request).await
    }

    async fn query_payment_order(&self, order_no: &str, _config: PaymentChannelConfig) -> payment::PaymentResult<payment::PaymentOrderQueryResult> {
        self.queried.lock().unwrap().push(order_no.to_owned());
        Ok(payment::PaymentOrderQueryResult {
            status: self.status,
            provider_trade_no: self.provider_trade_no.clone(),
            payment_method: Some("test".into()),
            amount: self.amount,
            raw_payload: serde_json::json!({"order_no": order_no}),
        })
    }

    async fn refund_payment_order(&self, request: payment::PaymentRefundRequest) -> payment::PaymentResult<payment::PaymentRefundResult> {
        TestPaymentProvider.refund_payment_order(request).await
    }

    fn verify_callback(&self, request: payment::PaymentCallbackRequest) -> payment::PaymentResult<payment::VerifiedPaymentCallback> {
        TestPaymentProvider.verify_callback(request)
    }
}

#[async_trait]
impl payment::PaymentChannelProvider for CallbackPaymentProvider {
    fn registration(&self) -> PaymentChannelRegistration {
        TestPaymentProvider.registration()
    }

    fn config_schema(&self) -> payment::PaymentChannelConfigSchema {
        TestPaymentProvider.config_schema()
    }

    fn callback_endpoints(&self) -> Vec<PaymentCallbackEndpoint> {
        TestPaymentProvider.callback_endpoints()
    }

    async fn create_payment_order(&self, request: payment::PaymentOrderRequest) -> payment::PaymentResult<PaymentOrderAction> {
        TestPaymentProvider.create_payment_order(request).await
    }

    async fn query_payment_order(&self, order_no: &str, config: PaymentChannelConfig) -> payment::PaymentResult<payment::PaymentOrderQueryResult> {
        TestPaymentProvider.query_payment_order(order_no, config).await
    }

    async fn refund_payment_order(&self, request: payment::PaymentRefundRequest) -> payment::PaymentResult<payment::PaymentRefundResult> {
        TestPaymentProvider.refund_payment_order(request).await
    }

    fn verify_callback(&self, _request: payment::PaymentCallbackRequest) -> payment::PaymentResult<payment::VerifiedPaymentCallback> {
        match &self.verification_error {
            Some(message) => Err(payment::PaymentError::VerificationFailed(message.clone())),
            None => Ok(self.verified.clone()),
        }
    }
}

fn callback_endpoint(kind: PaymentCallbackEndpointKind, path_pattern: &str) -> PaymentCallbackEndpoint {
    PaymentCallbackEndpoint {
        kind,
        methods: vec!["GET".into(), "POST".into()],
        path_pattern: path_pattern.into(),
    }
}
