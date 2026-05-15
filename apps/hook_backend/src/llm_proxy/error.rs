use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub enum LlmProxyError {
    Unauthorized,
    Forbidden(String),
    CodedForbidden {
        message: String,
        error_type: &'static str,
        code: &'static str,
    },
    RateLimited(String),
    InvalidRequest(String),
    NotFound(String),
    Upstream(String),
    Infrastructure(String),
}

impl IntoResponse for LlmProxyError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = json!({
            "error": {
                "message": self.message(),
                "type": self.error_type(),
                "code": self.error_code(status)
            }
        });
        (status, Json(body)).into_response()
    }
}

impl Display for LlmProxyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.message())
    }
}

impl std::error::Error for LlmProxyError {}

impl LlmProxyError {
    fn status(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) | Self::CodedForbidden { .. } => StatusCode::FORBIDDEN,
            Self::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Upstream(_) | Self::Infrastructure(_) => StatusCode::BAD_GATEWAY,
        }
    }

    fn error_type(&self) -> &'static str {
        match self {
            Self::Unauthorized => "unauthorized",
            Self::Forbidden(_) => "forbidden",
            Self::CodedForbidden { error_type, .. } => error_type,
            Self::RateLimited(_) => "rate_limit_error",
            Self::InvalidRequest(_) => "invalid_request_error",
            Self::NotFound(_) => "not_found_error",
            Self::Upstream(_) => "upstream_error",
            Self::Infrastructure(_) => "infrastructure_error",
        }
    }

    fn message(&self) -> String {
        match self {
            Self::Unauthorized => "missing or invalid bearer token".into(),
            Self::CodedForbidden { message, .. } => message.clone(),
            Self::Forbidden(message)
            | Self::RateLimited(message)
            | Self::InvalidRequest(message)
            | Self::NotFound(message)
            | Self::Upstream(message)
            | Self::Infrastructure(message) => message.clone(),
        }
    }

    fn error_code(&self, status: StatusCode) -> Value {
        match self {
            Self::CodedForbidden { code, .. } => Value::String((*code).into()),
            _ => Value::Number(status.as_u16().into()),
        }
    }

    pub fn new_api_forbidden(message: impl Into<String>, code: &'static str) -> Self {
        Self::CodedForbidden {
            message: message.into(),
            error_type: "new_api_error",
            code,
        }
    }
}

impl From<storage::StorageError> for LlmProxyError {
    fn from(value: storage::StorageError) -> Self {
        Self::Infrastructure(value.to_string())
    }
}

impl From<api_token::application::ApiTokenError> for LlmProxyError {
    fn from(value: api_token::application::ApiTokenError) -> Self {
        Self::Infrastructure(value.to_string())
    }
}

impl From<provider::application::ProviderError> for LlmProxyError {
    fn from(value: provider::application::ProviderError) -> Self {
        Self::Infrastructure(value.to_string())
    }
}

impl From<req::ClientError> for LlmProxyError {
    fn from(value: req::ClientError) -> Self {
        Self::Upstream(value.to_string())
    }
}

impl From<sea_orm::DbErr> for LlmProxyError {
    fn from(value: sea_orm::DbErr) -> Self {
        Self::Infrastructure(value.to_string())
    }
}
