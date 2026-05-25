use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::application::RechargeError;

#[derive(Debug)]
pub struct RechargeApiError(pub RechargeError);

impl From<RechargeError> for RechargeApiError {
    fn from(value: RechargeError) -> Self {
        Self(value)
    }
}

impl IntoResponse for RechargeApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorResponse::new(self.0.to_string());
        (status_code(&self.0), Json(body)).into_response()
    }
}

fn status_code(error: &RechargeError) -> StatusCode {
    match error {
        RechargeError::NotFound => StatusCode::NOT_FOUND,
        RechargeError::Forbidden => StatusCode::FORBIDDEN,
        _ => StatusCode::OK,
    }
}
