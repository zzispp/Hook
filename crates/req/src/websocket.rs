use std::time::Duration;

use axum::http::HeaderMap;
use futures_util::FutureExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream as TokioWebSocketStream, connect_async,
    tungstenite::{Message, client::IntoClientRequest, http::Request},
};

use crate::{ClientError, Url};

pub type WebSocketRequest = Request<()>;

pub type WebSocketStream = TokioWebSocketStream<MaybeTlsStream<TcpStream>>;

pub type WebSocketMessage = Message;

pub fn set_ws_scheme(url: &mut Url) -> Result<(), ClientError> {
    let scheme = match url.scheme() {
        "http" => "ws",
        "https" => "wss",
        "ws" | "wss" => return Ok(()),
        other => return Err(ClientError::Network(format!("unsupported websocket scheme: {other}"))),
    };
    url.set_scheme(scheme)
        .map_err(|_| ClientError::Network("failed to set websocket scheme".into()))
}

pub fn build_request(url: Url, headers: HeaderMap) -> Result<WebSocketRequest, ClientError> {
    let mut request = url.as_str().into_client_request().map_err(|error| ClientError::Network(error.to_string()))?;
    for (name, value) in headers.iter() {
        request.headers_mut().insert(name, value.clone());
    }
    Ok(request)
}

pub async fn connect_websocket(request: WebSocketRequest, timeout: Option<Duration>) -> Result<(WebSocketStream, HeaderMap), ClientError> {
    let connect = connect_async(request).map(|result| result.map_err(|error| ClientError::Network(error.to_string())));
    let result = match timeout {
        Some(timeout) => tokio::time::timeout(timeout, connect).await.map_err(|_| ClientError::Timeout)?,
        None => connect.await,
    };
    let (stream, response) = result?;
    Ok((stream, response.headers().clone()))
}
