use serde::Serialize;

const SUCCESS_MESSAGE: &str = "";

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: T,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct ApiErrorResponse {
    pub success: bool,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            success: true,
            message: SUCCESS_MESSAGE.into(),
            data,
        }
    }
}

impl<T> From<T> for ApiResponse<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl ApiErrorResponse {
    pub fn new(message: String) -> Self {
        Self { success: false, message }
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiErrorResponse, ApiResponse};
    use serde_json::json;

    #[test]
    fn api_response_matches_success_shape() {
        let response = ApiResponse::new(json!({ "id": 1 }));
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(
            value,
            json!({
                "success": true,
                "message": "",
                "data": { "id": 1 }
            })
        );
    }

    #[test]
    fn api_error_response_matches_error_shape() {
        let response = ApiErrorResponse::new("bad request".into());
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(
            value,
            json!({
                "success": false,
                "message": "bad request"
            })
        );
    }
}
