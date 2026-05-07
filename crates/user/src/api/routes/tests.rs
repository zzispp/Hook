use std::sync::Arc;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, Response, StatusCode, header},
};
use serde_json::{Value, json};
use tower::ServiceExt;

use super::create_router;
use crate::{
    api::{ApiState, TokenService, TokenSettings},
    application::UserService,
    test_support::{MemoryUserRepository, TestPasswordHasher, stored_user},
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
                "password": "secret"
            }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_success(&body);
    assert_eq!(body["data"]["user"]["username"], "alice");
    assert_non_empty_string(&body["data"]["accessToken"]);
    assert_non_empty_string(&body["data"]["refreshToken"]);
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
        .oneshot(json_request(Method::POST, "/api/auth/refresh", json!({ "refreshToken": tokens.refresh_token })))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_success(&body);
    let access_token = body["data"]["accessToken"].as_str().unwrap();
    assert_non_empty_string(&body["data"]["refreshToken"]);

    let response = app.oneshot(authenticated_request(Method::GET, "/api/auth/me", access_token)).await.unwrap();
    let body = response_json(response).await;

    assert_eq!(body["data"]["user"]["username"], "alice");
}

#[tokio::test]
async fn refresh_rejects_access_token() {
    let app = test_router();
    let tokens = sign_in(app.clone()).await;

    let response = app
        .oneshot(json_request(Method::POST, "/api/auth/refresh", json!({ "refreshToken": tokens.access_token })))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["success"], false);
}

struct SessionTokens {
    access_token: String,
    refresh_token: String,
}

fn test_router() -> Router {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret"));
    let users = UserService::new(repository, TestPasswordHasher);
    create_router(ApiState::new(Arc::new(users), token_service()))
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
                "password": "secret"
            }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    SessionTokens {
        access_token: body["data"]["accessToken"].as_str().unwrap().into(),
        refresh_token: body["data"]["refreshToken"].as_str().unwrap().into(),
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

fn assert_success(body: &Value) {
    assert_eq!(body["success"], true);
}

fn assert_non_empty_string(value: &Value) {
    assert!(!value.as_str().unwrap().is_empty());
}
