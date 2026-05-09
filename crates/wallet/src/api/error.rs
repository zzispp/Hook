use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::application::WalletError;

#[derive(Debug)]
pub struct WalletApiError(pub WalletError);

impl From<WalletError> for WalletApiError {
    fn from(value: WalletError) -> Self {
        Self(value)
    }
}

impl IntoResponse for WalletApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorResponse::new(self.0.to_string());
        (status_code(&self.0), Json(body)).into_response()
    }
}

fn status_code(error: &WalletError) -> StatusCode {
    match error {
        WalletError::Forbidden => StatusCode::FORBIDDEN,
        _ => StatusCode::OK,
    }
}
