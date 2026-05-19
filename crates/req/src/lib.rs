pub mod client_config;
pub mod query;
pub mod response;
pub mod retry;
pub mod websocket;

mod content_type;
mod reqwest_client;
mod types;

pub use axum::body::Bytes;
pub use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};
pub use reqwest::{Request, RequestBuilder, Response, StatusCode, Url};

pub use client_config::{builder, default_timeout, long_stream_builder};
pub use content_type::{CONTENT_TYPE, ContentType};
pub use query::build_path_with_query;
pub use reqwest_client::ReqwestClient;
pub use response::{content_type_header, response_builder, response_bytes, response_bytes_stream, response_content_type, response_status_code, response_text};
pub use retry::{default_should_retry, retry};
pub use types::ClientError;
pub use websocket::{WebSocketMessage, WebSocketRequest, WebSocketStream, build_request as build_websocket_request, connect_websocket, set_ws_scheme};

pub type Data = Vec<u8>;
pub const X_CACHE_TTL: &str = "x-cache-ttl";
