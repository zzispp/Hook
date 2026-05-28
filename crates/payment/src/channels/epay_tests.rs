use super::*;
use rust_decimal::Decimal;

#[tokio::test]
async fn create_payment_order_returns_signed_form_post() {
    let action = EpayChannel
        .create_payment_order(PaymentOrderRequest {
            order_no: "R1001".into(),
            subject: "Starter".into(),
            amount: Decimal::new(1234, 2),
            payment_method: "alipay".into(),
            notify_url: "https://hook.test/api/payment/epay/notify".into(),
            return_url: "https://hook.test/api/payment/epay/return".into(),
            config: config(),
        })
        .await
        .unwrap();

    let PaymentOrderAction::FormPost { action, method, fields } = action;
    assert_eq!(action, "https://pay.example.com/submit.php");
    assert_eq!(method, "POST");
    assert_eq!(fields.get("money").unwrap(), "12.34");
    assert_eq!(fields.get("sign_type").unwrap(), "MD5");
    assert_eq!(fields.get("sign").unwrap(), "22f3cf4d97eca6d7b7e0d0cc718f5649");
}

#[test]
fn callback_endpoints_include_notify_and_return() {
    let endpoints = EpayChannel.callback_endpoints();

    assert_eq!(endpoints.len(), 2);
    assert_eq!(endpoints[0].kind, PaymentCallbackEndpointKind::Notify);
    assert_eq!(endpoints[0].path_pattern, "/payment/{code}/notify");
    assert_eq!(endpoints[1].kind, PaymentCallbackEndpointKind::Return);
    assert_eq!(endpoints[1].path_pattern, "/payment/{code}/return");
}

#[test]
fn verify_callback_rejects_bad_signature() {
    let mut params = success_callback_params();
    params.insert("sign".into(), "bad".into());

    let result = EpayChannel.verify_callback(PaymentCallbackRequest { params, config: config() });

    assert_eq!(result.unwrap_err().to_string(), "payment verification failed: signature mismatch");
}

#[test]
fn verify_callback_accepts_trade_success() {
    let params = success_callback_params();

    let verified = EpayChannel.verify_callback(PaymentCallbackRequest { params, config: config() }).unwrap();

    assert_eq!(verified.order_no, "R1001");
    assert_eq!(verified.provider_trade_no.as_deref(), Some("202605271001"));
    assert_eq!(verified.payment_method, "alipay");
    assert_eq!(verified.amount, Some(Decimal::new(1234, 2)));
    assert_eq!(verified.trade_status, PaymentOrderStatus::Paid);
}

#[test]
fn query_status_maps_success_and_pending() {
    let paid = query_result(EpayQueryResponse {
        code: Some(1),
        trade_status: Some(TRADE_SUCCESS.into()),
        status: None,
        trade_no: Some("202605271001".into()),
        r#type: Some("alipay".into()),
        money: Some("12.34".into()),
        msg: None,
    })
    .unwrap();
    let pending = query_result(EpayQueryResponse {
        code: Some(1),
        trade_status: Some("WAIT_BUYER_PAY".into()),
        status: Some(0),
        trade_no: None,
        r#type: None,
        money: None,
        msg: None,
    })
    .unwrap();

    assert_eq!(paid.status, PaymentOrderStatus::Paid);
    assert_eq!(paid.provider_trade_no.as_deref(), Some("202605271001"));
    assert_eq!(paid.payment_method.as_deref(), Some("alipay"));
    assert_eq!(paid.amount, Some(Decimal::new(1234, 2)));
    assert_eq!(pending.status, PaymentOrderStatus::Pending);
}

#[tokio::test]
async fn refund_returns_explicit_unsupported_error() {
    let refund = EpayChannel
        .refund_payment_order(PaymentRefundRequest {
            order_no: "R1001".into(),
            amount: Decimal::new(1000, 2),
            config: config(),
        })
        .await;

    assert_eq!(
        refund.unwrap_err().to_string(),
        "unsupported payment capability: epay refund is not available in the configured generic protocol"
    );
}

fn success_callback_params() -> BTreeMap<String, String> {
    let mut params = BTreeMap::from([
        ("pid".into(), "1000".into()),
        ("type".into(), "alipay".into()),
        ("out_trade_no".into(), "R1001".into()),
        ("trade_no".into(), "202605271001".into()),
        ("name".into(), "Starter".into()),
        ("money".into(), "12.34".into()),
        ("trade_status".into(), TRADE_SUCCESS.into()),
        ("sign_type".into(), SIGN_TYPE_MD5.into()),
    ]);
    params.insert("sign".into(), sign_params(&params, "secret"));
    params
}

fn config() -> PaymentChannelConfig {
    PaymentChannelConfig {
        config: json!({
            "merchant_id": "1000",
            "api_base_url": "https://pay.example.com/"
        }),
        secret: Some("secret".into()),
    }
}
