use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use serde_json::{Value, json};

pub(super) const MODEL_SERVICE_UNAVAILABLE_MESSAGE: &str = "The model service is temporarily unavailable. Please retry later.";
pub(super) const SERVICE_UNAVAILABLE_MESSAGE: &str = "The service is temporarily unavailable. Please retry later.";
pub(super) const MODEL_REQUEST_INVALID_MESSAGE: &str = "The request could not be processed by the model service.";
pub(super) const MODEL_SERVICE_UNAVAILABLE_CODE: &str = "model_service_unavailable";
pub(super) const SERVICE_UNAVAILABLE_CODE: &str = "service_unavailable";
pub(super) const MODEL_REQUEST_INVALID_CODE: &str = "model_request_invalid";
pub(super) const SERVER_ERROR_TYPE: &str = "server_error";
pub(super) const INVALID_REQUEST_ERROR_TYPE: &str = "invalid_request_error";

pub(super) struct ClientErrorBody {
    pub(super) status: StatusCode,
    pub(super) value: Value,
}

pub(super) fn upstream_failure(status: StatusCode) -> ClientErrorBody {
    if is_client_request_failure(status) {
        return ClientErrorBody::new(status, MODEL_REQUEST_INVALID_MESSAGE, INVALID_REQUEST_ERROR_TYPE, MODEL_REQUEST_INVALID_CODE);
    }
    model_service_unavailable()
}

pub(super) fn model_service_unavailable() -> ClientErrorBody {
    ClientErrorBody::new(
        StatusCode::BAD_GATEWAY,
        MODEL_SERVICE_UNAVAILABLE_MESSAGE,
        SERVER_ERROR_TYPE,
        MODEL_SERVICE_UNAVAILABLE_CODE,
    )
}

pub(super) fn json_content_type() -> HeaderValue {
    HeaderValue::from_static("application/json")
}

pub(super) fn json_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, json_content_type());
    headers
}

fn is_client_request_failure(status: StatusCode) -> bool {
    status.is_client_error()
        && !matches!(
            status,
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN | StatusCode::REQUEST_TIMEOUT | StatusCode::TOO_MANY_REQUESTS
        )
}

impl ClientErrorBody {
    fn new(status: StatusCode, message: &'static str, error_type: &'static str, code: &'static str) -> Self {
        Self {
            status,
            value: json!({
                "error": {
                    "message": message,
                    "type": error_type,
                    "code": code
                }
            }),
        }
    }

    pub(super) fn bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self.value)
    }
}
