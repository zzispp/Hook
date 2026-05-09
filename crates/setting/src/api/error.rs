use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::application::SettingError;

#[derive(Debug)]
pub struct SettingApiError(pub SettingError);

impl From<SettingError> for SettingApiError {
    fn from(value: SettingError) -> Self {
        Self(value)
    }
}

impl IntoResponse for SettingApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorResponse::new(self.0.to_string());
        (StatusCode::OK, Json(body)).into_response()
    }
}
