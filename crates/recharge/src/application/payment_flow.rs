use payment::{
    PaymentCallbackEndpointKind, PaymentCallbackRequest, PaymentChannelConfig, PaymentError, PaymentOrderRequest, PaymentOrderStatus, VerifiedPaymentCallback,
};
use types::{recharge::RechargeOrderCreateResponse, system_setting::SystemSettings};
use uuid::Uuid;

use crate::application::{
    PaymentCallbackCreateRecord, PaymentCallbackUpdateRecord, RechargeError, RechargeOrderCreateRecord, RechargePaymentCallbackRequest,
    RechargePaymentCallbackResult, RechargePaymentPollResult, RechargePaymentSettlementRecord, RechargeRepository, RechargeResult, RechargeSecretCipher,
};

use super::{RechargeService, service::RechargeOrderItem};

const API_PATH_PREFIX: &str = "/api";

pub(super) struct PaymentChannelContext {
    pub code: String,
    pub name: String,
    pub config: PaymentChannelConfig,
}

impl<R, C> RechargeService<R, C>
where
    R: RechargeRepository,
    C: RechargeSecretCipher,
{
    pub(super) async fn create_payment_order(
        &self,
        user_id: &str,
        channel_code: &str,
        method: &str,
        item: &RechargeOrderItem,
        settings: &SystemSettings,
        channel: PaymentChannelContext,
    ) -> RechargeResult<RechargeOrderCreateResponse> {
        let provider = self.registry.provider(channel_code).ok_or(RechargeError::NotFound)?;
        let order_no = new_order_no();
        let request = self.payment_request(&order_no, channel_code, item, method, settings, channel.config.clone())?;
        let payment = provider.create_payment_order(request).await.map_err(payment_error)?;
        let order = self
            .repository
            .create_order(
                order_record(user_id, &order_no, item, settings, &channel, method, &payment),
                max_unpaid_orders(settings)?,
            )
            .await?;
        Ok(RechargeOrderCreateResponse { order: order.into(), payment })
    }

    fn payment_request(
        &self,
        order_no: &str,
        channel_code: &str,
        item: &RechargeOrderItem,
        payment_method: &str,
        settings: &SystemSettings,
        config: PaymentChannelConfig,
    ) -> RechargeResult<PaymentOrderRequest> {
        Ok(PaymentOrderRequest {
            order_no: order_no.into(),
            subject: item.package_name.clone(),
            amount: item.recharge_amount * settings.recharge_arrival_ratio,
            payment_method: payment_method.into(),
            notify_url: self.callback_url(settings, channel_code, PaymentCallbackEndpointKind::Notify)?,
            return_url: self.callback_url(settings, channel_code, PaymentCallbackEndpointKind::Return)?,
            config,
        })
    }

    fn callback_url(&self, settings: &SystemSettings, channel_code: &str, kind: PaymentCallbackEndpointKind) -> RechargeResult<String> {
        let endpoint = self
            .registry
            .callback_endpoint(channel_code, kind)
            .ok_or_else(|| RechargeError::Payment(format!("payment callback endpoint is not registered: {channel_code} {kind:?}")))?;
        if !endpoint.path_pattern.starts_with('/') {
            return Err(RechargeError::Payment(format!(
                "payment callback endpoint path must start with /: {}",
                endpoint.path_pattern
            )));
        }
        Ok(format!(
            "{}{}{}",
            settings.public_base_url.trim().trim_end_matches('/'),
            API_PATH_PREFIX,
            endpoint.path_pattern
        ))
    }

    pub(super) async fn handle_logged_provider_callback(&self, request: RechargePaymentCallbackRequest) -> RechargeResult<RechargePaymentCallbackResult> {
        let callback = self.record_callback_received(&request).await?;
        let result = self.process_provider_callback(&request.channel_code, request.payment).await;
        self.record_callback_processed(&callback.id, &result).await?;
        result
    }

    async fn process_provider_callback(&self, code: &str, request: PaymentCallbackRequest) -> RechargeResult<RechargePaymentCallbackResult> {
        let provider = self.registry.provider(code).ok_or(RechargeError::NotFound)?;
        let config = self.payment_config(code).await?;
        let verified = provider.verify_callback(PaymentCallbackRequest { config, ..request }).map_err(payment_error)?;
        if verified.trade_status != PaymentOrderStatus::Paid {
            return Ok(callback_result(false, verified));
        }
        let settled = self.repository.settle_paid_order(settlement_record(code, verified.clone())).await?;
        Ok(callback_result(settled.settled, verified))
    }

    async fn record_callback_received(&self, request: &RechargePaymentCallbackRequest) -> RechargeResult<types::recharge::PaymentCallbackRecord> {
        self.repository
            .create_payment_callback(PaymentCallbackCreateRecord {
                payment_channel_code: request.channel_code.clone(),
                callback_kind: request.callback_kind.as_str().to_owned(),
                http_method: request.http_method.clone(),
                raw_params: serde_json::json!(request.payment.params),
            })
            .await
    }

    async fn record_callback_processed(&self, id: &str, result: &RechargeResult<RechargePaymentCallbackResult>) -> RechargeResult<()> {
        let patch = match result {
            Ok(callback) => callback_update_from_success(callback),
            Err(error) => callback_update_from_error(error),
        };
        self.repository.update_payment_callback(id, patch).await?;
        Ok(())
    }

    pub(super) async fn poll_pending_orders(&self, limit: u64) -> RechargeResult<RechargePaymentPollResult> {
        let mut result = RechargePaymentPollResult::default();
        for order in self.repository.list_pending_unexpired_orders(limit).await? {
            result.checked += 1;
            match self.query_order_status(&order).await? {
                PendingQueryResult::Paid(settlement) => {
                    let settled = self.repository.settle_paid_order(settlement).await?;
                    if settled.settled {
                        result.paid += 1;
                    }
                }
                PendingQueryResult::Pending => {}
                PendingQueryResult::Unsupported => result.unsupported += 1,
            }
        }
        Ok(result)
    }

    async fn query_order_status(&self, order: &types::recharge::RechargeOrder) -> RechargeResult<PendingQueryResult> {
        let channel_code = required_order_value(order.payment_channel_code.as_deref(), "payment_channel_code")?;
        let payment_method = required_order_value(order.payment_method.as_deref(), "payment_method")?;
        let provider = self.registry.provider(channel_code).ok_or(RechargeError::NotFound)?;
        let config = self.payment_config(channel_code).await?;
        match provider.query_payment_order(&order.order_no, config).await {
            Ok(query) => match query.status {
                PaymentOrderStatus::Paid => Ok(PendingQueryResult::Paid(RechargePaymentSettlementRecord {
                    order_no: order.order_no.clone(),
                    payment_channel_code: channel_code.to_owned(),
                    provider_trade_no: query.provider_trade_no,
                    payment_method: query.payment_method.unwrap_or_else(|| payment_method.to_owned()),
                    payable_amount: query.amount,
                    callback_payload: serde_json::json!({"source": "poll", "order_no": order.order_no, "provider": query.raw_payload}),
                })),
                PaymentOrderStatus::Pending | PaymentOrderStatus::Failed => Ok(PendingQueryResult::Pending),
            },
            Err(PaymentError::Unsupported(_)) => Ok(PendingQueryResult::Unsupported),
            Err(error) => Err(payment_error(error)),
        }
    }
}

enum PendingQueryResult {
    Paid(RechargePaymentSettlementRecord),
    Pending,
    Unsupported,
}

fn required_order_value<'a>(value: Option<&'a str>, field: &str) -> RechargeResult<&'a str> {
    value
        .filter(|item| !item.trim().is_empty())
        .ok_or_else(|| RechargeError::InvalidInput(format!("{field} is required")))
}

fn order_record(
    user_id: &str,
    order_no: &str,
    item: &RechargeOrderItem,
    settings: &SystemSettings,
    channel: &PaymentChannelContext,
    method: &str,
    payment: &payment::PaymentOrderAction,
) -> RechargeOrderCreateRecord {
    let now = time::OffsetDateTime::now_utc();
    RechargeOrderCreateRecord {
        order_no: order_no.into(),
        user_id: user_id.to_owned(),
        package_id: item.package_id.clone(),
        package_name: item.package_name.clone(),
        recharge_amount: item.recharge_amount,
        gift_amount: item.gift_amount,
        total_arrival_amount: item.recharge_amount + item.gift_amount,
        payable_amount: item.recharge_amount * settings.recharge_arrival_ratio,
        payment_channel_code: channel.code.clone(),
        payment_channel_name: channel.name.clone(),
        payment_method: method.into(),
        payment_request_json: serde_json::to_value(payment).unwrap_or_else(|_| serde_json::json!({})),
        expires_at: now + time::Duration::minutes(settings.recharge_order_expire_minutes),
    }
}

fn settlement_record(channel_code: &str, verified: payment::VerifiedPaymentCallback) -> RechargePaymentSettlementRecord {
    RechargePaymentSettlementRecord {
        order_no: verified.order_no,
        payment_channel_code: channel_code.to_owned(),
        provider_trade_no: verified.provider_trade_no,
        payment_method: verified.payment_method,
        payable_amount: verified.amount,
        callback_payload: verified.raw_payload,
    }
}

fn callback_result(settled: bool, verified: VerifiedPaymentCallback) -> RechargePaymentCallbackResult {
    RechargePaymentCallbackResult {
        response_body: "success".into(),
        settled,
        order_no: Some(verified.order_no),
        provider_trade_no: verified.provider_trade_no,
        payment_method: Some(verified.payment_method),
        trade_status: Some(payment_order_status_name(verified.trade_status).into()),
    }
}

fn callback_update_from_success(callback: &RechargePaymentCallbackResult) -> PaymentCallbackUpdateRecord {
    let status = if callback.trade_status.as_deref() == Some("paid") {
        types::recharge::PAYMENT_CALLBACK_STATUS_PROCESSED
    } else {
        types::recharge::PAYMENT_CALLBACK_STATUS_IGNORED
    };
    PaymentCallbackUpdateRecord {
        order_no: callback.order_no.clone(),
        provider_trade_no: callback.provider_trade_no.clone(),
        payment_method: callback.payment_method.clone(),
        trade_status: callback.trade_status.clone(),
        status: status.into(),
        settled: callback.settled,
        error_message: None,
    }
}

fn callback_update_from_error(error: &RechargeError) -> PaymentCallbackUpdateRecord {
    PaymentCallbackUpdateRecord {
        order_no: None,
        provider_trade_no: None,
        payment_method: None,
        trade_status: None,
        status: types::recharge::PAYMENT_CALLBACK_STATUS_FAILED.into(),
        settled: false,
        error_message: Some(error.to_string()),
    }
}

fn payment_order_status_name(status: PaymentOrderStatus) -> &'static str {
    match status {
        PaymentOrderStatus::Pending => "pending",
        PaymentOrderStatus::Paid => "paid",
        PaymentOrderStatus::Failed => "failed",
    }
}

fn new_order_no() -> String {
    format!("R{}", Uuid::now_v7().simple())
}

fn payment_error(error: payment::PaymentError) -> RechargeError {
    RechargeError::Payment(error.to_string())
}

fn max_unpaid_orders(settings: &SystemSettings) -> RechargeResult<u64> {
    u64::try_from(settings.recharge_max_unpaid_orders).map_err(|_| RechargeError::InvalidInput("recharge_max_unpaid_orders must be greater than 0".into()))
}
