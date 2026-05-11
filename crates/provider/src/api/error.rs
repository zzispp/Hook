use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::application::ProviderError;

#[derive(Debug)]
pub struct ProviderApiError(pub ProviderError);

impl From<ProviderError> for ProviderApiError {
    fn from(value: ProviderError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ProviderApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorResponse::new(self.0.to_string());
        (StatusCode::OK, Json(body)).into_response()
    }
}
