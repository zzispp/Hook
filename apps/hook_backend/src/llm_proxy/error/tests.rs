use axum::{body::to_bytes, http::StatusCode, response::IntoResponse};
use serde_json::Value;

use super::LlmProxyError;

#[tokio::test]
async fn upstream_http_response_hides_internal_message() {
    let error = LlmProxyError::Upstream("connect https://api.86gamestore.com failed with sk-secret".into());
    assert!(error.to_string().contains("api.86gamestore.com"));

    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let body = response_text(response).await;
    assert!(!body.contains("api.86gamestore.com"));
    assert!(!body.contains("sk-secret"));

    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["message"], "The model service is temporarily unavailable. Please retry later.");
    assert_eq!(json["error"]["type"], "server_error");
    assert_eq!(json["error"]["code"], "model_service_unavailable");
}

#[tokio::test]
async fn infrastructure_http_response_hides_internal_message() {
    let error = LlmProxyError::Infrastructure("database password authentication failed for hook_internal".into());
    assert!(error.to_string().contains("hook_internal"));

    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let body = response_text(response).await;
    assert!(!body.contains("hook_internal"));

    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["message"], "The service is temporarily unavailable. Please retry later.");
    assert_eq!(json["error"]["type"], "server_error");
    assert_eq!(json["error"]["code"], "service_unavailable");
}

async fn response_text(response: axum::response::Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}
