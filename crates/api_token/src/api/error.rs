use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::application::ApiTokenError;

#[derive(Debug)]
pub struct ApiTokenApiError(pub ApiTokenError);

impl From<ApiTokenError> for ApiTokenApiError {
    fn from(value: ApiTokenError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiTokenApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorResponse::new(self.0.to_string());
        (StatusCode::OK, Json(body)).into_response()
    }
}
