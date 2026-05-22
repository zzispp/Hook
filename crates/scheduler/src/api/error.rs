use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::runtime::SchedulerError;

#[derive(Debug)]
pub struct SchedulerApiError(pub SchedulerError);

impl From<SchedulerError> for SchedulerApiError {
    fn from(value: SchedulerError) -> Self {
        Self(value)
    }
}

impl IntoResponse for SchedulerApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            SchedulerError::NotFound(_) => StatusCode::NOT_FOUND,
            SchedulerError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            SchedulerError::Infrastructure(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(ApiErrorResponse::new(self.0.to_string()))).into_response()
    }
}
