use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use captcha::application::{CaptchaResult, CaptchaUseCase};
use tower::ServiceExt;
use types::{
    captcha::{CaptchaChallengeResponse, CaptchaConfigResponse, CaptchaRedeemPayload, CaptchaRedeemResponse},
    pagination::PageRequest,
    recharge::{
        PaymentCallbackListFilters, PaymentCallbackRecordListResponse, PaymentChannelUpdatePayload, PublicPaymentChannelResponse, RechargeOrderCreatePayload,
        RechargeOrderCreateResponse, RechargeOrderDatePreset, RechargeOrderListFilters, RechargeOrderListResponse, RechargeOrderSummaryResponse,
        RechargeOrderSummaryResponseTotals, RechargePackage, RechargePackageCreatePayload, RechargePackageListFilters, RechargePackageListResponse,
        RechargePackageUpdatePayload, UserRechargePackageListResponse,
    },
};

use crate::{
    api::{RechargeApiState, create_router},
    application::{RechargeError, RechargePaymentCallbackRequest, RechargePaymentCallbackResult, RechargePaymentPollResult, RechargeResult, RechargeUseCase},
};

#[tokio::test]
async fn trailing_slash_payment_return_route_redirects_to_wallet() {
    let recharge = Arc::new(RecordingRecharge::default());
    let app = create_router(RechargeApiState::new(recharge.clone(), Arc::new(NoopCaptcha)));

    let response = app
        .oneshot(request("/payment/epay/return/?out_trade_no=R1001&trade_status=TRADE_SUCCESS"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/dashboard/wallet");
    let callback = recharge.last_callback();
    assert_eq!(callback.code, "epay");
    assert_eq!(callback.kind, types::recharge::PAYMENT_CALLBACK_KIND_RETURN);
    assert_eq!(callback.method, "GET");
    assert_eq!(callback.params.get("out_trade_no").map(String::as_str), Some("R1001"));
}

#[tokio::test]
async fn order_summary_route_passes_date_filters() {
    let recharge = Arc::new(RecordingRecharge::default());
    let app = create_router(RechargeApiState::new(recharge.clone(), Arc::new(NoopCaptcha)));

    let response = app
        .oneshot(request(
            "/admin/recharge-orders/summary?page=2&page_size=25&status=paid&date_preset=custom&start_date=2026-06-01&end_date=2026-06-03&tz_offset_minutes=480",
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let (page, filters) = recharge.last_summary_query();
    assert_eq!(page, PageRequest { page: 2, page_size: 25 });
    assert_eq!(filters.status.as_deref(), Some("paid"));
    assert_eq!(filters.date_preset, RechargeOrderDatePreset::Custom);
    assert_eq!(filters.start_date.as_deref(), Some("2026-06-01"));
    assert_eq!(filters.end_date.as_deref(), Some("2026-06-03"));
    assert_eq!(filters.tz_offset_minutes, 480);
}

fn request(uri: &str) -> Request<Body> {
    Request::builder().method(Method::GET).uri(uri).body(Body::empty()).unwrap()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RecordedCallback {
    code: String,
    kind: &'static str,
    method: String,
    params: BTreeMap<String, String>,
}

#[derive(Default)]
struct RecordingRecharge {
    callback: Mutex<Option<RecordedCallback>>,
    summary_query: Mutex<Option<(PageRequest, RechargeOrderListFilters)>>,
}

impl RecordingRecharge {
    fn last_callback(&self) -> RecordedCallback {
        self.callback.lock().unwrap().clone().unwrap()
    }

    fn last_summary_query(&self) -> (PageRequest, RechargeOrderListFilters) {
        self.summary_query.lock().unwrap().clone().unwrap()
    }
}

#[async_trait]
impl RechargeUseCase for RecordingRecharge {
    async fn list_packages(&self, _page: PageRequest, _filters: RechargePackageListFilters) -> RechargeResult<RechargePackageListResponse> {
        Err(unused())
    }

    async fn list_user_packages(&self, _page: PageRequest) -> RechargeResult<UserRechargePackageListResponse> {
        Err(unused())
    }

    async fn create_package(&self, _input: RechargePackageCreatePayload) -> RechargeResult<RechargePackage> {
        Err(unused())
    }

    async fn update_package(&self, _id: &str, _input: RechargePackageUpdatePayload) -> RechargeResult<RechargePackage> {
        Err(unused())
    }

    async fn list_orders(&self, _page: PageRequest, _filters: RechargeOrderListFilters) -> RechargeResult<RechargeOrderListResponse> {
        Err(unused())
    }

    async fn list_order_summary(&self, page: PageRequest, filters: RechargeOrderListFilters) -> RechargeResult<RechargeOrderSummaryResponse> {
        *self.summary_query.lock().unwrap() = Some((page, filters));
        Ok(RechargeOrderSummaryResponse {
            summary: RechargeOrderSummaryResponseTotals {
                total_payable_amount: rust_decimal::Decimal::ZERO,
                order_count: 0,
                user_count: 0,
            },
            items: Vec::new(),
            total: 0,
            page: page.page,
            page_size: page.page_size,
        })
    }

    async fn list_user_orders(&self, _user_id: &str, _page: PageRequest) -> RechargeResult<RechargeOrderListResponse> {
        Err(unused())
    }

    async fn list_payment_callbacks(&self, _page: PageRequest, _filters: PaymentCallbackListFilters) -> RechargeResult<PaymentCallbackRecordListResponse> {
        Err(unused())
    }

    async fn create_user_order(&self, _user_id: &str, _input: RechargeOrderCreatePayload) -> RechargeResult<RechargeOrderCreateResponse> {
        Err(unused())
    }

    async fn list_payment_channels(&self) -> RechargeResult<Vec<types::recharge::PaymentChannel>> {
        Err(unused())
    }

    async fn list_user_payment_channels(&self) -> RechargeResult<Vec<PublicPaymentChannelResponse>> {
        Err(unused())
    }

    async fn update_payment_channel(&self, _code: &str, _input: PaymentChannelUpdatePayload) -> RechargeResult<types::recharge::PaymentChannel> {
        Err(unused())
    }

    async fn handle_payment_callback(&self, request: RechargePaymentCallbackRequest) -> RechargeResult<RechargePaymentCallbackResult> {
        *self.callback.lock().unwrap() = Some(RecordedCallback {
            code: request.channel_code,
            kind: request.callback_kind.as_str(),
            method: request.http_method,
            params: request.payment.params,
        });
        Ok(RechargePaymentCallbackResult {
            response_body: "success".into(),
            settled: true,
            order_no: Some("R1001".into()),
            provider_trade_no: Some("T1001".into()),
            payment_method: Some("alipay".into()),
            trade_status: Some("paid".into()),
        })
    }

    async fn poll_pending_payment_orders(&self, _limit: u64) -> RechargeResult<RechargePaymentPollResult> {
        Err(unused())
    }

    async fn expire_pending_orders(&self) -> RechargeResult<u64> {
        Err(unused())
    }
}

#[derive(Clone, Copy)]
struct NoopCaptcha;

#[async_trait]
impl CaptchaUseCase for NoopCaptcha {
    async fn config(&self) -> CaptchaResult<CaptchaConfigResponse> {
        Err(captcha_unused())
    }

    async fn challenge(&self) -> CaptchaResult<CaptchaChallengeResponse> {
        Err(captcha_unused())
    }

    async fn redeem(&self, _payload: CaptchaRedeemPayload) -> CaptchaResult<CaptchaRedeemResponse> {
        Err(captcha_unused())
    }

    async fn verify_login(&self, _token: Option<&str>) -> CaptchaResult<()> {
        Err(captcha_unused())
    }

    async fn verify_registration(&self, _token: Option<&str>) -> CaptchaResult<()> {
        Err(captcha_unused())
    }

    async fn verify_support_ticket(&self, _token: Option<&str>) -> CaptchaResult<()> {
        Err(captcha_unused())
    }

    async fn verify_recharge(&self, _token: Option<&str>) -> CaptchaResult<()> {
        Ok(())
    }
}

fn unused() -> RechargeError {
    RechargeError::Infrastructure("unused test method".into())
}

fn captcha_unused() -> captcha::application::CaptchaError {
    captcha::application::CaptchaError::Infrastructure("unused test method".into())
}
