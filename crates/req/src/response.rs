use axum::{
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::Response,
};
use futures_util::StreamExt;
use std::pin::Pin;

use crate::ClientError;

pub fn response_status_code(status: reqwest::StatusCode) -> StatusCode {
    StatusCode::from_u16(status.as_u16()).expect("reqwest status codes are valid HTTP status codes")
}

pub fn response_content_type(response: &reqwest::Response) -> Option<HeaderValue> {
    response.headers().get(header::CONTENT_TYPE).cloned()
}

pub fn content_type_header(content_type: Option<&HeaderValue>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Some(value) = content_type.cloned() {
        headers.insert(header::CONTENT_TYPE, value);
    }
    headers
}

pub async fn response_bytes(response: reqwest::Response) -> Result<Vec<u8>, ClientError> {
    response.bytes().await.map(|bytes| bytes.to_vec()).map_err(ClientError::from)
}

pub async fn response_text(response: reqwest::Response) -> Result<String, ClientError> {
    response.text().await.map_err(ClientError::from)
}

pub fn response_bytes_stream(response: reqwest::Response) -> Pin<Box<dyn futures_util::Stream<Item = Result<axum::body::Bytes, ClientError>> + Send>> {
    Box::pin(response.bytes_stream().map(|chunk| chunk.map_err(ClientError::from)))
}

pub fn response_builder(status: StatusCode, content_type: Option<HeaderValue>) -> axum::http::response::Builder {
    let mut builder = Response::builder().status(status);
    if let Some(value) = content_type {
        builder = builder.header(header::CONTENT_TYPE, value);
    }
    builder
}
