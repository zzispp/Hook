use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::application::ModelError;

#[derive(Debug)]
pub struct ModelApiError(pub ModelError);

impl From<ModelError> for ModelApiError {
    fn from(value: ModelError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ModelApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorResponse::new(self.0.to_string());
        (StatusCode::OK, Json(body)).into_response()
    }
}
