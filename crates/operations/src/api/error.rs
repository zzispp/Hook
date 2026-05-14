use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::application::OperationsError;

#[derive(Debug)]
pub struct OperationsApiError(pub OperationsError);

impl From<OperationsError> for OperationsApiError {
    fn from(value: OperationsError) -> Self {
        Self(value)
    }
}

impl IntoResponse for OperationsApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorResponse::new(self.0.to_string());
        (status_code(&self.0), Json(body)).into_response()
    }
}

fn status_code(error: &OperationsError) -> StatusCode {
    match error {
        OperationsError::Forbidden => StatusCode::FORBIDDEN,
        _ => StatusCode::OK,
    }
}
