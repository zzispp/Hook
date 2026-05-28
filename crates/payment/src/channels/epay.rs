use async_trait::async_trait;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

use crate::{
    PaymentCallbackEndpoint, PaymentCallbackEndpointKind, PaymentCallbackRequest, PaymentChannelConfig, PaymentChannelConfigField, PaymentChannelConfigSchema,
    PaymentChannelProvider, PaymentChannelRegistration, PaymentError, PaymentMethodOption, PaymentOrderAction, PaymentOrderQueryResult, PaymentOrderRequest,
    PaymentOrderStatus, PaymentRefundRequest, PaymentRefundResult, PaymentResult, VerifiedPaymentCallback,
};

const EPAY_CODE: &str = "epay";
const EPAY_NAME: &str = "易支付";
const CALLBACK_NOTIFY_PATH: &str = "/payment/{code}/notify";
const CALLBACK_RETURN_PATH: &str = "/payment/{code}/return";
const SUBMIT_PATH: &str = "/submit.php";
const QUERY_PATH: &str = "/api.php";
const SIGN_TYPE_MD5: &str = "MD5";
const DEVICE_PC: &str = "pc";
const TRADE_SUCCESS: &str = "TRADE_SUCCESS";

#[derive(Clone, Copy, Debug, Default)]
pub struct EpayChannel;

#[async_trait]
impl PaymentChannelProvider for EpayChannel {
    fn registration(&self) -> PaymentChannelRegistration {
        PaymentChannelRegistration {
            code: EPAY_CODE.into(),
            name: EPAY_NAME.into(),
        }
    }

    fn config_schema(&self) -> PaymentChannelConfigSchema {
        PaymentChannelConfigSchema {
            fields: vec![
                config_field("merchant_id", "商户 ID", false),
                config_field("api_base_url", "接口地址", false),
                config_field("api_key", "API 密钥", true),
            ],
            methods: vec![method("alipay", "支付宝"), method("wxpay", "微信")],
        }
    }

    fn callback_endpoints(&self) -> Vec<PaymentCallbackEndpoint> {
        vec![
            callback_endpoint(PaymentCallbackEndpointKind::Notify, CALLBACK_NOTIFY_PATH),
            callback_endpoint(PaymentCallbackEndpointKind::Return, CALLBACK_RETURN_PATH),
        ]
    }

    async fn create_payment_order(&self, request: PaymentOrderRequest) -> PaymentResult<PaymentOrderAction> {
        let config = epay_config(request.config.clone())?;
        ensure_supported_method(&request.payment_method)?;
        Ok(PaymentOrderAction::FormPost {
            action: submit_url(&config.api_base_url),
            method: "POST".into(),
            fields: signed_purchase_fields(&config, &request),
        })
    }

    async fn query_payment_order(&self, order_no: &str, config: PaymentChannelConfig) -> PaymentResult<PaymentOrderQueryResult> {
        let config = epay_config(config)?;
        let response = reqwest::get(query_url(&config, order_no)?)
            .await
            .map_err(|error| PaymentError::Provider(format!("epay query request failed: {error}")))?
            .error_for_status()
            .map_err(|error| PaymentError::Provider(format!("epay query returned error status: {error}")))?
            .json::<EpayQueryResponse>()
            .await
            .map_err(|error| PaymentError::Provider(format!("epay query response is invalid: {error}")))?;
        query_result(response)
    }

    async fn refund_payment_order(&self, _request: PaymentRefundRequest) -> PaymentResult<PaymentRefundResult> {
        Err(PaymentError::Unsupported(
            "epay refund is not available in the configured generic protocol".into(),
        ))
    }

    fn verify_callback(&self, request: PaymentCallbackRequest) -> PaymentResult<VerifiedPaymentCallback> {
        let config = epay_config(request.config)?;
        verify_signature(&request.params, &config.api_key)?;
        let order_no = required_param(&request.params, "out_trade_no")?;
        let payment_method = required_param(&request.params, "type")?;
        ensure_supported_method(&payment_method)?;
        let trade_status = verified_trade_status(&request.params)?;
        Ok(VerifiedPaymentCallback {
            order_no,
            provider_trade_no: request.params.get("trade_no").cloned().filter(|value| !value.is_empty()),
            payment_method,
            amount: optional_money(&request.params)?,
            trade_status,
            raw_payload: json!(request.params),
        })
    }
}

fn signed_purchase_fields(config: &EpayConfig, request: &PaymentOrderRequest) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::from([
        ("pid".into(), config.merchant_id.clone()),
        ("type".into(), request.payment_method.clone()),
        ("out_trade_no".into(), request.order_no.clone()),
        ("notify_url".into(), request.notify_url.clone()),
        ("return_url".into(), request.return_url.clone()),
        ("name".into(), request.subject.clone()),
        ("money".into(), format_money(request.amount)),
        ("device".into(), DEVICE_PC.into()),
        ("sign_type".into(), SIGN_TYPE_MD5.into()),
    ]);
    fields.insert("sign".into(), sign_params(&fields, &config.api_key));
    fields
}

fn verify_signature(params: &BTreeMap<String, String>, api_key: &str) -> PaymentResult<()> {
    let Some(sign) = params.get("sign") else {
        return Err(PaymentError::VerificationFailed("sign is required".into()));
    };
    let expected = sign_params(params, api_key);
    if sign == &expected {
        return Ok(());
    }
    Err(PaymentError::VerificationFailed("signature mismatch".into()))
}

fn sign_params(params: &BTreeMap<String, String>, api_key: &str) -> String {
    let unsigned = params
        .iter()
        .filter(|(key, value)| key.as_str() != "sign" && key.as_str() != "sign_type" && !value.is_empty())
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&");
    let digest = Md5::digest(format!("{unsigned}{api_key}").as_bytes());
    format!("{digest:x}")
}

fn epay_config(config: PaymentChannelConfig) -> PaymentResult<EpayConfig> {
    let public: EpayPublicConfig = serde_json::from_value(config.config).map_err(|error| PaymentError::InvalidConfig(error.to_string()))?;
    let api_key = config.secret.filter(|value| !value.trim().is_empty());
    validate_epay_config(EpayConfig {
        merchant_id: public.merchant_id.trim().to_owned(),
        api_base_url: public.api_base_url.trim().trim_end_matches('/').to_owned(),
        api_key: api_key.ok_or_else(|| PaymentError::MissingConfig("api_key".into()))?,
    })
}

fn validate_epay_config(config: EpayConfig) -> PaymentResult<EpayConfig> {
    if config.merchant_id.is_empty() {
        return Err(PaymentError::MissingConfig("merchant_id".into()));
    }
    if config.api_base_url.is_empty() {
        return Err(PaymentError::MissingConfig("api_base_url".into()));
    }
    Ok(config)
}

fn verified_trade_status(params: &BTreeMap<String, String>) -> PaymentResult<PaymentOrderStatus> {
    match params.get("trade_status").map(String::as_str) {
        Some(TRADE_SUCCESS) => Ok(PaymentOrderStatus::Paid),
        Some(_) => Ok(PaymentOrderStatus::Pending),
        None => Err(PaymentError::VerificationFailed("trade_status is required".into())),
    }
}

fn submit_url(api_base_url: &str) -> String {
    format!("{api_base_url}{SUBMIT_PATH}")
}

fn query_url(config: &EpayConfig, order_no: &str) -> PaymentResult<String> {
    let mut url = reqwest::Url::parse(&format!("{}{}", config.api_base_url, QUERY_PATH))
        .map_err(|error| PaymentError::InvalidConfig(format!("api_base_url is invalid: {error}")))?;
    url.query_pairs_mut()
        .append_pair("act", "order")
        .append_pair("pid", &config.merchant_id)
        .append_pair("key", &config.api_key)
        .append_pair("out_trade_no", order_no);
    Ok(url.to_string())
}

fn required_param(params: &BTreeMap<String, String>, key: &str) -> PaymentResult<String> {
    params
        .get(key)
        .filter(|value| !value.is_empty())
        .cloned()
        .ok_or_else(|| PaymentError::VerificationFailed(format!("{key} is required")))
}

fn ensure_supported_method(value: &str) -> PaymentResult<()> {
    if matches!(value, "alipay" | "wxpay") {
        return Ok(());
    }
    Err(PaymentError::InvalidRequest(format!("unsupported epay payment method: {value}")))
}

fn format_money(amount: rust_decimal::Decimal) -> String {
    format!("{:.2}", amount.round_dp(2))
}

fn optional_money(params: &BTreeMap<String, String>) -> PaymentResult<Option<rust_decimal::Decimal>> {
    optional_callback_money(params.get("money").map(String::as_str))
}

fn optional_callback_money(value: Option<&str>) -> PaymentResult<Option<rust_decimal::Decimal>> {
    let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };
    value
        .trim()
        .parse::<rust_decimal::Decimal>()
        .map(Some)
        .map_err(|error| PaymentError::VerificationFailed(format!("money is invalid: {error}")))
}

fn optional_query_money(value: Option<&str>) -> PaymentResult<Option<rust_decimal::Decimal>> {
    let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };
    value
        .trim()
        .parse::<rust_decimal::Decimal>()
        .map(Some)
        .map_err(|error| PaymentError::Provider(format!("epay query money is invalid: {error}")))
}

fn config_field(key: &str, label: &str, secret: bool) -> PaymentChannelConfigField {
    PaymentChannelConfigField {
        key: key.into(),
        label: label.into(),
        secret,
        required: true,
    }
}

fn method(code: &str, name: &str) -> PaymentMethodOption {
    PaymentMethodOption {
        code: code.into(),
        name: name.into(),
    }
}

fn callback_endpoint(kind: PaymentCallbackEndpointKind, path_pattern: &str) -> PaymentCallbackEndpoint {
    PaymentCallbackEndpoint {
        kind,
        methods: vec!["GET".into(), "POST".into()],
        path_pattern: path_pattern.into(),
    }
}

#[derive(Debug, Deserialize)]
struct EpayPublicConfig {
    merchant_id: String,
    api_base_url: String,
}

struct EpayConfig {
    merchant_id: String,
    api_base_url: String,
    api_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct EpayQueryResponse {
    code: Option<i64>,
    trade_status: Option<String>,
    status: Option<i64>,
    trade_no: Option<String>,
    r#type: Option<String>,
    money: Option<String>,
    msg: Option<String>,
}

fn query_result(response: EpayQueryResponse) -> PaymentResult<PaymentOrderQueryResult> {
    if response.code.is_some_and(|code| code != 1) {
        return Err(PaymentError::Provider(response.msg.unwrap_or_else(|| "epay query failed".into())));
    }
    let status = if response.trade_status.as_deref() == Some(TRADE_SUCCESS) || response.status == Some(1) {
        PaymentOrderStatus::Paid
    } else {
        PaymentOrderStatus::Pending
    };
    let amount = optional_query_money(response.money.as_deref())?;
    Ok(PaymentOrderQueryResult {
        status,
        provider_trade_no: response.trade_no.clone().filter(|value| !value.is_empty()),
        payment_method: response.r#type.clone().filter(|value| !value.is_empty()),
        amount,
        raw_payload: json!(response),
    })
}

#[cfg(test)]
#[path = "epay_tests.rs"]
mod tests;
