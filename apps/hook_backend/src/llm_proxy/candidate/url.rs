use crate::llm_proxy::cache::snapshot::CachedEndpoint;
use crate::llm_proxy::formats;

pub fn upstream_url(endpoint: &CachedEndpoint, provider_model_name: &str, is_stream: bool) -> String {
    let path = endpoint_path(endpoint, provider_model_name, is_stream);
    join_url(&endpoint.base_url, &path)
}

fn endpoint_path(endpoint: &CachedEndpoint, provider_model_name: &str, is_stream: bool) -> String {
    let path = match endpoint.custom_path.as_deref().map(str::trim) {
        Some(path) if !path.is_empty() => path,
        _ => formats::default_path(&endpoint.api_format, is_stream),
    };
    formats::render_path(&endpoint.api_format, path, provider_model_name, is_stream)
}

fn join_url(base_url: &str, path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
}
