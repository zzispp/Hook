use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::application::CardCodeError;

#[derive(Debug)]
pub struct CardCodeApiError(pub CardCodeError);

impl From<CardCodeError> for CardCodeApiError {
    fn from(value: CardCodeError) -> Self {
        Self(value)
    }
}

impl IntoResponse for CardCodeApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorResponse::new(self.0.to_string());
        (StatusCode::OK, Json(body)).into_response()
    }
}
