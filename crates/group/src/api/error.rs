use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::application::GroupError;

#[derive(Debug)]
pub struct GroupApiError(pub GroupError);

impl From<GroupError> for GroupApiError {
    fn from(value: GroupError) -> Self {
        Self(value)
    }
}

impl IntoResponse for GroupApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorResponse::new(self.0.to_string());
        (StatusCode::OK, Json(body)).into_response()
    }
}
