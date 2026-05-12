use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::application::CaptchaError;

#[derive(Debug)]
pub struct CaptchaApiError(pub CaptchaError);

impl From<CaptchaError> for CaptchaApiError {
    fn from(value: CaptchaError) -> Self {
        Self(value)
    }
}

impl IntoResponse for CaptchaApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorResponse::new(self.0.to_string());
        (status_code(&self.0), Json(body)).into_response()
    }
}

fn status_code(error: &CaptchaError) -> StatusCode {
    match error {
        CaptchaError::InvalidInput(_) => StatusCode::OK,
        CaptchaError::Infrastructure(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
