use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::application::I18nError;

#[derive(Debug)]
pub struct I18nApiError(pub I18nError);

impl From<I18nError> for I18nApiError {
    fn from(value: I18nError) -> Self {
        Self(value)
    }
}

impl IntoResponse for I18nApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorResponse::new(self.0.to_string());
        (StatusCode::OK, Json(body)).into_response()
    }
}
