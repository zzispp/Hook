use super::selection::CandidateParts;
use crate::llm_proxy::formats;

pub fn upstream_url(parts: &CandidateParts, is_stream: bool) -> String {
    let path = endpoint_path(parts, is_stream);
    join_url(&parts.endpoint.base_url, &path)
}

fn endpoint_path(parts: &CandidateParts, is_stream: bool) -> String {
    let path = match parts.endpoint.custom_path.as_deref().map(str::trim) {
        Some(path) if !path.is_empty() => path,
        _ => formats::default_path(&parts.endpoint.api_format, is_stream),
    };
    formats::render_path(&parts.endpoint.api_format, path, &parts.model.provider_model_name, is_stream)
}

fn join_url(base_url: &str, path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
}
