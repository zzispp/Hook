use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::response::ApiErrorResponse;

use crate::application::RbacError;

#[derive(Debug)]
pub struct RbacApiError(pub RbacError);

impl From<RbacError> for RbacApiError {
    fn from(value: RbacError) -> Self {
        Self(value)
    }
}

impl IntoResponse for RbacApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorResponse::new(self.0.to_string());
        (status_code(&self.0), Json(body)).into_response()
    }
}

fn status_code(error: &RbacError) -> StatusCode {
    match error {
        RbacError::Unauthorized => StatusCode::UNAUTHORIZED,
        RbacError::Forbidden => StatusCode::FORBIDDEN,
        RbacError::NotFound | RbacError::Conflict(_) | RbacError::InvalidInput(_) | RbacError::Infrastructure(_) => StatusCode::OK,
    }
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    use super::{RbacApiError, StatusCode};
    use crate::application::RbacError;

    #[test]
    fn unauthorized_uses_http_401() {
        let response = RbacApiError(RbacError::Unauthorized).into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn forbidden_uses_http_403() {
        let response = RbacApiError(RbacError::Forbidden).into_response();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
