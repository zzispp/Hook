use std::collections::BTreeSet;

use types::provider::ProviderEndpointCreate;

use super::quick_import_commit::SelectedToken;
use crate::application::ProviderResult;
use crate::application::validation::{sanitize_endpoint, validate_endpoint};

const IMAGE_STREAM_MODE_KEY: &str = "upstream_image_stream_mode";
const IMAGE_STREAM_MODE_NATIVE: &str = "native_stream";
const IMAGE_STREAM_MODE_SYNC_WRAPPED: &str = "sync_wrapped_stream";

pub(super) fn endpoint_creates(
    base_url: String,
    selected: &[SelectedToken<'_>],
    upstream_image_native_stream: bool,
) -> ProviderResult<Vec<ProviderEndpointCreate>> {
    let formats = selected.iter().flat_map(|token| token.endpoint_formats.iter()).collect::<BTreeSet<_>>();
    formats
        .into_iter()
        .map(|format| endpoint_create(format, &base_url, upstream_image_native_stream))
        .collect()
}

fn endpoint_create(format: &str, base_url: &str, upstream_image_native_stream: bool) -> ProviderResult<ProviderEndpointCreate> {
    let endpoint = sanitize_endpoint(ProviderEndpointCreate {
        api_format: format.to_owned(),
        base_url: base_url.to_owned(),
        custom_path: None,
        max_retries: None,
        is_active: Some(true),
        format_acceptance_config: image_format_acceptance_config(format, upstream_image_native_stream),
        header_rules: None,
        body_rules: None,
    });
    validate_endpoint(&endpoint)?;
    Ok(endpoint)
}

fn image_format_acceptance_config(format: &str, upstream_image_native_stream: bool) -> Option<serde_json::Value> {
    if !matches!(
        format,
        "openai_image" | "openai:image" | "openai:image_generation" | "openai_image_edit" | "openai:image_edit"
    ) {
        return None;
    }
    let mode = if upstream_image_native_stream {
        IMAGE_STREAM_MODE_NATIVE
    } else {
        IMAGE_STREAM_MODE_SYNC_WRAPPED
    };
    Some(serde_json::json!({ IMAGE_STREAM_MODE_KEY: mode }))
}
