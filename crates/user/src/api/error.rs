use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::application::AppError;

#[derive(Debug)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorResponse::new(self.0.to_string());
        (status_code(&self.0), Json(body)).into_response()
    }
}

fn status_code(error: &AppError) -> StatusCode {
    match error {
        AppError::InvalidCredentials | AppError::PasswordNotSet | AppError::Unauthorized => StatusCode::UNAUTHORIZED,
        AppError::InvalidInput(_) | AppError::Conflict(_) | AppError::NotFound | AppError::Infrastructure(_) => StatusCode::OK,
    }
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    use super::{ApiError, StatusCode};
    use crate::application::AppError;

    #[test]
    fn api_error_uses_unauthorized_status() {
        let response = ApiError(AppError::Unauthorized).into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
