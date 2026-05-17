use axum::{body::to_bytes, http::StatusCode};
use serde_json::Value;

use super::{UpstreamFailure, failure_response};

#[tokio::test]
async fn failure_response_hides_upstream_body() {
    let failure = UpstreamFailure {
        status: StatusCode::TOO_MANY_REQUESTS,
        cooldown_triggered: false,
    };

    let response = failure_response(failure).unwrap();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(response.headers().get("content-type").unwrap(), "application/json");

    let body = response_text(response).await;
    assert!(!body.contains("api.86gamestore.com"));
    assert!(!body.contains("sk-secret"));

    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["message"], "The model service is temporarily unavailable. Please retry later.");
    assert_eq!(json["error"]["type"], "server_error");
    assert_eq!(json["error"]["code"], "model_service_unavailable");
}

#[tokio::test]
async fn failure_response_keeps_sanitized_client_error_status() {
    let failure = UpstreamFailure {
        status: StatusCode::BAD_REQUEST,
        cooldown_triggered: false,
    };

    let response = failure_response(failure).unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response.headers().get("content-type").unwrap(), "application/json");

    let body = response_text(response).await;
    assert!(!body.contains("provider-specific"));

    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["message"], "The request could not be processed by the model service.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "model_request_invalid");
}

async fn response_text(response: axum::response::Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}
