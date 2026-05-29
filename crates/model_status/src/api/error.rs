use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::application::ModelStatusError;

#[derive(Debug)]
pub struct ModelStatusApiError(pub ModelStatusError);

impl From<ModelStatusError> for ModelStatusApiError {
    fn from(value: ModelStatusError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ModelStatusApiError {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(ApiErrorResponse::new(self.0.to_string()))).into_response()
    }
}
