use crate::llm_proxy::cache::snapshot::CachedEndpoint;
use crate::llm_proxy::formats;

pub fn upstream_url_checked(endpoint: &CachedEndpoint, provider_model_name: &str, is_stream: bool) -> Result<String, crate::llm_proxy::LlmProxyError> {
    let path = endpoint_path_checked(endpoint, provider_model_name, is_stream)?;
    Ok(join_url(&endpoint.base_url, &path))
}

fn endpoint_path_checked(endpoint: &CachedEndpoint, provider_model_name: &str, is_stream: bool) -> Result<String, crate::llm_proxy::LlmProxyError> {
    let path = match endpoint.custom_path.as_deref().map(str::trim) {
        Some(path) if !path.is_empty() => path,
        _ => formats::endpoint_metadata(&endpoint.api_format, is_stream)?.default_path,
    };
    Ok(formats::render_path(&endpoint.api_format, path, provider_model_name, is_stream))
}

fn join_url(base_url: &str, path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
}
