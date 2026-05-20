use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Map, Value};

use super::LlmProxyError;

const IMAGE_FETCH_ACCEPT: &str = "image/avif,image/webp,image/apng,image/*,*/*;q=0.8";
const IMAGE_FETCH_ACCEPT_LANGUAGE: &str = "zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7";
const IMAGE_FETCH_ORIGIN: &str = "https://platform.openai.com";
const IMAGE_FETCH_REFERER: &str = "https://platform.openai.com/";
const IMAGE_FETCH_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.7339.249 Electron/38.7.0 Safari/537.36";

pub(super) async fn normalize_image_response_bytes(http: &req::ReqwestClient, bytes: &[u8]) -> Result<Vec<u8>, LlmProxyError> {
    let response = parse_image_response(bytes)?;
    let response = normalize_image_response_json(http, response).await?;
    serde_json::to_vec(&response).map_err(|error| LlmProxyError::Infrastructure(error.to_string()))
}

fn parse_image_response(bytes: &[u8]) -> Result<Value, LlmProxyError> {
    serde_json::from_slice(bytes).map_err(|error| LlmProxyError::Upstream(format!("upstream returned invalid image response payload: {error}")))
}

async fn normalize_image_response_json(http: &req::ReqwestClient, response_json: Value) -> Result<Value, LlmProxyError> {
    let Some(response_object) = response_json.as_object() else {
        return Err(LlmProxyError::Upstream("upstream returned invalid image response payload".into()));
    };
    let Some(raw_items) = response_object.get("data") else {
        return Ok(response_json);
    };
    let items = raw_items
        .as_array()
        .ok_or_else(|| LlmProxyError::Upstream("upstream returned invalid image data payload".into()))?;

    let mut changed = false;
    let mut client_items = Vec::with_capacity(items.len());
    for item in items {
        let item_object = item
            .as_object()
            .ok_or_else(|| LlmProxyError::Upstream("upstream returned invalid image item payload".into()))?;
        let (client_item, item_changed) = normalize_image_item(http, item_object).await?;
        changed |= item_changed;
        client_items.push(Value::Object(client_item));
    }
    if !changed {
        return Ok(response_json);
    }

    let mut client_response = response_object.clone();
    client_response.insert("data".into(), Value::Array(client_items));
    Ok(Value::Object(client_response))
}

async fn normalize_image_item(http: &req::ReqwestClient, item: &Map<String, Value>) -> Result<(Map<String, Value>, bool), LlmProxyError> {
    let mut client_item = item.clone();
    let mut changed = false;
    let mut b64_json = extract_text(&client_item, "b64_json");

    if b64_json.is_none()
        && let Some(url) = extract_text(&client_item, "url")
    {
        b64_json = Some(fetch_b64_json_from_url(http, &url).await?);
        changed = true;
    }

    if let Some(value) = b64_json {
        client_item.insert("b64_json".into(), Value::String(value));
        if extract_text(&client_item, "url").is_some() {
            changed = true;
        }
        client_item.insert("url".into(), Value::String(String::new()));
    }

    Ok((client_item, changed))
}

fn extract_text(item: &Map<String, Value>, key: &str) -> Option<String> {
    item.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

async fn fetch_b64_json_from_url(http: &req::ReqwestClient, url: &str) -> Result<String, LlmProxyError> {
    let url = req::Url::parse(url).map_err(|error| LlmProxyError::Upstream(format!("invalid upstream image url {url:?}: {error}")))?;
    let request = http.build_request(
        http.get(url)
            .header("user-agent", IMAGE_FETCH_USER_AGENT)
            .header("accept", IMAGE_FETCH_ACCEPT)
            .header("accept-language", IMAGE_FETCH_ACCEPT_LANGUAGE)
            .header("origin", IMAGE_FETCH_ORIGIN)
            .header("referer", IMAGE_FETCH_REFERER),
    )?;
    let response = http.execute(request).await?;
    let status = response.status();
    if !status.is_success() {
        return Err(LlmProxyError::Upstream(format!("upstream image url returned status {}", status.as_u16())));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|error| LlmProxyError::Upstream(format!("failed to read upstream image url: {error}")))?;
    if bytes.is_empty() {
        return Err(LlmProxyError::Upstream("upstream image url returned empty content".into()));
    }
    Ok(STANDARD.encode(bytes))
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use axum::{Router, routing::get};
    use axum::{extract::State, http::HeaderMap, http::StatusCode};
    use base64::Engine as _;
    use serde_json::{Value, json};

    use super::normalize_image_response_bytes;

    #[tokio::test]
    async fn normalize_image_response_bytes_converts_url_to_b64_json() {
        let base_url = spawn_image_server().await;
        let client = req::ReqwestClient::default();
        let body = json!({
            "created": 1779256600,
            "data": [{
                "revised_prompt": "A simple solid red image filling the entire frame, uniform bright red color, minimalistic.",
                "url": format!("{base_url}/image.png"),
            }]
        });

        let body_json = body.to_string();
        let response = normalize_image_response_bytes(&client, body_json.as_bytes()).await.unwrap();
        let value: Value = serde_json::from_slice(&response).unwrap();

        assert_eq!(value["data"][0]["b64_json"], super::STANDARD.encode(b"png-bytes"));
        assert_eq!(value["data"][0]["url"], "");
    }

    #[tokio::test]
    async fn normalize_image_response_bytes_keeps_existing_b64_json() {
        let client = req::ReqwestClient::default();
        let body = json!({
            "created": 1779256600,
            "data": [{
                "revised_prompt": "A simple solid red image filling the entire frame, uniform bright red color, minimalistic.",
                "url": "http://127.0.0.1:9/image.png",
                "b64_json": "ready-b64",
            }]
        });

        let body_json = body.to_string();
        let response = normalize_image_response_bytes(&client, body_json.as_bytes()).await.unwrap();
        let value: Value = serde_json::from_slice(&response).unwrap();

        assert_eq!(value["data"][0]["b64_json"], "ready-b64");
        assert_eq!(value["data"][0]["url"], "");
    }

    #[tokio::test]
    async fn normalize_image_response_bytes_sends_browser_like_headers() {
        let seen_headers = Arc::new(Mutex::new(None::<HeaderMap>));
        let base_url = spawn_header_guard_server(seen_headers.clone()).await;
        let client = req::ReqwestClient::default();
        let body = json!({
            "created": 1779256600,
            "data": [{
                "revised_prompt": "A simple solid red image filling the entire frame, uniform bright red color, minimalistic.",
                "url": format!("{base_url}/image.png"),
            }]
        });

        let body_json = body.to_string();
        let response = normalize_image_response_bytes(&client, body_json.as_bytes()).await.unwrap();
        let value: Value = serde_json::from_slice(&response).unwrap();

        assert_eq!(value["data"][0]["b64_json"], super::STANDARD.encode(b"png-bytes"));
        let headers = seen_headers.lock().unwrap().clone().unwrap();
        assert_eq!(
            headers.get("user-agent").and_then(|value| value.to_str().ok()),
            Some(super::IMAGE_FETCH_USER_AGENT)
        );
        assert_eq!(headers.get("accept").and_then(|value| value.to_str().ok()), Some(super::IMAGE_FETCH_ACCEPT));
        assert_eq!(
            headers.get("accept-language").and_then(|value| value.to_str().ok()),
            Some(super::IMAGE_FETCH_ACCEPT_LANGUAGE)
        );
        assert_eq!(headers.get("origin").and_then(|value| value.to_str().ok()), Some(super::IMAGE_FETCH_ORIGIN));
        assert_eq!(headers.get("referer").and_then(|value| value.to_str().ok()), Some(super::IMAGE_FETCH_REFERER));
    }

    async fn spawn_image_server() -> String {
        let app = Router::new().route("/image.png", get(|| async { axum::body::Bytes::from_static(b"png-bytes") }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("http://{addr}")
    }

    async fn spawn_header_guard_server(seen_headers: Arc<Mutex<Option<HeaderMap>>>) -> String {
        async fn image_handler(State(seen_headers): State<Arc<Mutex<Option<HeaderMap>>>>, headers: HeaderMap) -> (StatusCode, &'static str) {
            *seen_headers.lock().unwrap() = Some(headers.clone());
            if headers.get("user-agent").and_then(|value| value.to_str().ok()) == Some(super::IMAGE_FETCH_USER_AGENT) {
                return (StatusCode::OK, "png-bytes");
            }
            (StatusCode::FORBIDDEN, "forbidden")
        }

        let app = Router::new().route("/image.png", get(image_handler)).with_state(seen_headers);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("http://{addr}")
    }
}
