use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::application::DashboardError;

#[derive(Debug)]
pub struct DashboardApiError(pub DashboardError);

impl From<DashboardError> for DashboardApiError {
    fn from(value: DashboardError) -> Self {
        Self(value)
    }
}

impl IntoResponse for DashboardApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            DashboardError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            DashboardError::Forbidden(_) => StatusCode::FORBIDDEN,
            DashboardError::Infrastructure(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(ApiErrorResponse::new(self.0.to_string()))).into_response()
    }
}
