use std::sync::Arc;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, Response, StatusCode, header},
};
use captcha::application::{CaptchaResult, CaptchaUseCase};
use serde_json::{Value, json};
use tower::ServiceExt;
use types::captcha::{CaptchaChallengeResponse, CaptchaConfigResponse, CaptchaRedeemPayload, CaptchaRedeemResponse};

use super::create_router;
use crate::{
    api::{ApiState, TokenService, TokenSettings},
    application::UserService,
    test_support::{MemoryUserRepository, TestPasswordHasher, VALID_PASSWORD, stored_user},
};

const TEST_SECRET: &str = "test-secret-with-enough-entropy";
const ACCESS_TTL_SECONDS: u64 = 900;
const REFRESH_TTL_SECONDS: u64 = 604800;

#[tokio::test]
async fn sign_in_accepts_email_identifier_and_returns_token_pair() {
    let app = test_router();

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-in",
            json!({
                "identifier": "alice@example.com",
                "password": VALID_PASSWORD
            }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_success(&body);
    assert_eq!(body["data"]["user"]["username"], "alice");
    assert_non_empty_string(&body["data"]["access_token"]);
    assert_non_empty_string(&body["data"]["refresh_token"]);
}

#[tokio::test]
async fn sign_up_accepts_public_payload_and_sets_backend_fields() {
    let app = test_router();

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-up",
            json!({
                "username": "bob",
                "email": "bob@example.com",
                "password": VALID_PASSWORD
            }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_success(&body);
    assert_eq!(body["data"]["user"]["role"], "user");
    assert_eq!(body["data"]["user"]["is_active"], true);
    assert_eq!(body["data"]["user"]["auth_source"], "local");
    assert_eq!(body["data"]["user"]["email_verified"], false);
    assert_non_empty_string(&body["data"]["access_token"]);
}

#[tokio::test]
async fn me_returns_user_for_bearer_access_token() {
    let app = test_router();
    let tokens = sign_in(app.clone()).await;

    let response = app
        .oneshot(authenticated_request(Method::GET, "/api/auth/me", &tokens.access_token))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_success(&body);
    assert_eq!(body["data"]["user"]["email"], "alice@example.com");
}

#[tokio::test]
async fn refresh_returns_new_token_pair_and_me_accepts_new_access_token() {
    let app = test_router();
    let tokens = sign_in(app.clone()).await;

    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/auth/refresh",
            json!({ "refresh_token": tokens.refresh_token }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_success(&body);
    let access_token = body["data"]["access_token"].as_str().unwrap();
    assert_non_empty_string(&body["data"]["refresh_token"]);

    let response = app.oneshot(authenticated_request(Method::GET, "/api/auth/me", access_token)).await.unwrap();
    let body = response_json(response).await;

    assert_eq!(body["data"]["user"]["username"], "alice");
}

#[tokio::test]
async fn refresh_rejects_access_token() {
    let app = test_router();
    let tokens = sign_in(app.clone()).await;

    let response = app
        .oneshot(json_request(Method::POST, "/api/auth/refresh", json!({ "refresh_token": tokens.access_token })))
        .await
        .unwrap();
    let body = error_response_json(response, StatusCode::UNAUTHORIZED).await;

    assert_eq!(body["success"], false);
    assert_eq!(body["message"], "unauthorized");
}

#[tokio::test]
async fn sign_in_rejects_invalid_password_with_credentials_message() {
    let app = test_router();

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-in",
            json!({
                "identifier": "alice",
                "password": "bad-password"
            }),
        ))
        .await
        .unwrap();
    let body = error_response_json(response, StatusCode::UNAUTHORIZED).await;

    assert_eq!(body["success"], false);
    assert_eq!(body["message"], "username or password is incorrect");
}

#[tokio::test]
async fn admin_affiliate_overview_route_returns_summary() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/affiliates/overview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_success(&body);
    assert_eq!(body["data"]["total_referred_users"], 0);
    assert_eq!(body["data"]["affiliate_commission_percent"], 0.0);
}

#[tokio::test]
async fn admin_affiliate_relation_changes_route_returns_page() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/affiliates/relation-changes?page=1&page_size=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_success(&body);
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 0);
    assert_eq!(body["data"]["total"], 0);
    assert_eq!(body["data"]["page"], 1);
    assert_eq!(body["data"]["page_size"], 10);
}

#[tokio::test]
async fn admin_affiliate_export_route_returns_csv() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/affiliates/reports/export")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let csv = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(content_type.as_deref(), Some("text/csv; charset=utf-8"));
    assert_eq!(
        csv,
        "\"id\",\"referrer_id\",\"referrer_username\",\"referred_id\",\"referred_username\",\"recharge_order_id\",\"payable_amount\",\"commission_percent\",\"commission_amount\",\"wallet_transaction_id\",\"status\",\"failure_reason\",\"created_at\""
    );
}

struct SessionTokens {
    access_token: String,
    refresh_token: String,
}

fn test_router() -> Router {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let users = Arc::new(UserService::new(repository, TestPasswordHasher));
    Router::new().nest(
        "/api",
        create_router(ApiState::new(
            users.clone(),
            Arc::new(TestAffiliates),
            users,
            Arc::new(TestUserGroups),
            token_service(),
            Arc::new(TestCaptcha),
        )),
    )
}

struct TestAffiliates;

#[async_trait::async_trait]
impl crate::application::AffiliateUseCase for TestAffiliates {
    async fn affiliate_summary(&self, _id: types::user::UserId) -> crate::application::AppResult<types::user::AffiliateSummaryResponse> {
        Ok(types::user::AffiliateSummaryResponse {
            affiliate_code: "AFF-1".into(),
            affiliate_link: "/auth/sign-up?aff=AFF-1".into(),
            affiliate_enabled: false,
            referred_user_count: 0,
            total_referred_recharge_amount: rust_decimal::Decimal::ZERO,
            total_commission_amount: rust_decimal::Decimal::ZERO,
            today_commission_amount: rust_decimal::Decimal::ZERO,
            month_commission_amount: rust_decimal::Decimal::ZERO,
            affiliate_commission_percent: rust_decimal::Decimal::ZERO,
            last_commission_at: None,
        })
    }

    async fn list_affiliate_referrals(
        &self,
        _id: types::user::UserId,
        request: types::pagination::PageSliceRequest,
        _query: types::user::AffiliateReferralQuery,
    ) -> crate::application::AppResult<types::pagination::Page<types::user::AffiliateReferralItem>> {
        Ok(empty_page(request))
    }

    async fn list_affiliate_commissions(
        &self,
        _id: types::user::UserId,
        request: types::pagination::PageSliceRequest,
        _query: types::user::AffiliateCommissionQuery,
    ) -> crate::application::AppResult<types::pagination::Page<types::user::AffiliateCommissionItem>> {
        Ok(empty_page(request))
    }

    async fn export_affiliate_commissions(
        &self,
        _id: types::user::UserId,
        _query: types::user::AffiliateCommissionQuery,
    ) -> crate::application::AppResult<Vec<types::user::AffiliateCommissionItem>> {
        Ok(Vec::new())
    }
}

fn empty_page<T>(request: types::pagination::PageSliceRequest) -> types::pagination::Page<T> {
    types::pagination::Page {
        items: Vec::new(),
        total: 0,
        page: request.page,
        page_size: request.page_size,
    }
}

struct TestUserGroups;

#[async_trait::async_trait]
impl crate::application::UserGroupUseCase for TestUserGroups {
    async fn create_user_group(&self, _input: types::user_group::UserGroupCreate) -> crate::application::AppResult<types::user_group::UserGroupResponse> {
        unimplemented!("auth route tests do not call user group routes")
    }

    async fn update_user_group(
        &self,
        _code: &str,
        _input: types::user_group::UserGroupUpdate,
    ) -> crate::application::AppResult<types::user_group::UserGroupResponse> {
        unimplemented!("auth route tests do not call user group routes")
    }

    async fn delete_user_group(&self, _code: &str) -> crate::application::AppResult<()> {
        unimplemented!("auth route tests do not call user group routes")
    }

    async fn get_user_group(&self, _code: &str) -> crate::application::AppResult<types::user_group::UserGroupResponse> {
        unimplemented!("auth route tests do not call user group routes")
    }

    async fn list_user_groups(
        &self,
        _request: types::user_group::UserGroupListRequest,
    ) -> crate::application::AppResult<types::user_group::UserGroupPageResponse> {
        unimplemented!("auth route tests do not call user group routes")
    }

    async fn list_user_group_members(
        &self,
        _code: &str,
        _request: types::pagination::PageRequest,
        _filters: types::user::UserListFilters,
    ) -> crate::application::AppResult<types::pagination::Page<types::user::User>> {
        unimplemented!("auth route tests do not call user group routes")
    }
}

struct TestCaptcha;

#[async_trait::async_trait]
impl CaptchaUseCase for TestCaptcha {
    async fn config(&self) -> CaptchaResult<CaptchaConfigResponse> {
        Ok(CaptchaConfigResponse {
            login_captcha_enabled: false,
            registration_captcha_enabled: false,
            support_ticket_captcha_enabled: false,
            recharge_captcha_enabled: false,
        })
    }

    async fn challenge(&self) -> CaptchaResult<CaptchaChallengeResponse> {
        unimplemented!("auth route tests do not call captcha challenge")
    }

    async fn redeem(&self, _payload: CaptchaRedeemPayload) -> CaptchaResult<CaptchaRedeemResponse> {
        unimplemented!("auth route tests do not call captcha redeem")
    }

    async fn verify_login(&self, _token: Option<&str>) -> CaptchaResult<()> {
        Ok(())
    }

    async fn verify_registration(&self, _token: Option<&str>) -> CaptchaResult<()> {
        Ok(())
    }

    async fn verify_support_ticket(&self, _token: Option<&str>) -> CaptchaResult<()> {
        Ok(())
    }

    async fn verify_recharge(&self, _token: Option<&str>) -> CaptchaResult<()> {
        Ok(())
    }
}

fn token_service() -> TokenService {
    TokenService::new(TokenSettings {
        secret: TEST_SECRET.into(),
        access_token_ttl_seconds: ACCESS_TTL_SECONDS,
        refresh_token_ttl_seconds: REFRESH_TTL_SECONDS,
    })
}

async fn sign_in(app: Router) -> SessionTokens {
    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-in",
            json!({
                "identifier": "alice",
                "password": VALID_PASSWORD
            }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    SessionTokens {
        access_token: body["data"]["access_token"].as_str().unwrap().into(),
        refresh_token: body["data"]["refresh_token"].as_str().unwrap().into(),
    }
}

fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn authenticated_request(method: Method, uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

async fn response_json(response: Response<Body>) -> Value {
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

async fn error_response_json(response: Response<Body>, status: StatusCode) -> Value {
    assert_eq!(response.status(), status);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

fn assert_success(body: &Value) {
    assert_eq!(body["success"], true);
}

fn assert_non_empty_string(value: &Value) {
    assert!(!value.as_str().unwrap().is_empty());
}
