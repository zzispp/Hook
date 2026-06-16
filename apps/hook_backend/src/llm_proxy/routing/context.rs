use types::provider::RoutingRequestFeatures;

pub(crate) fn routing_context_key(group_code: &str, global_model_id: &str, features: &RoutingRequestFeatures) -> String {
    let capability = features.required_capability.as_deref().unwrap_or("none");
    [
        segment("group", group_code),
        segment("model", global_model_id),
        segment("format", &features.client_api_format),
        segment("stream", stream_value(features.is_stream)),
        segment("size", features.request_size_bucket.as_str()),
        segment("cap", capability),
    ]
    .join("|")
}

fn segment(name: &str, value: &str) -> String {
    format!("{name}={}", escape(value))
}

fn stream_value(is_stream: bool) -> &'static str {
    if is_stream { "true" } else { "false" }
}

fn escape(value: &str) -> String {
    value.replace(['|', '='], "_")
}

#[cfg(test)]
mod tests {
    use types::provider::{RoutingRequestFeatures, RoutingRequestSizeBucket};

    use super::routing_context_key;

    #[test]
    fn context_key_contains_all_context_dimensions() {
        let features = RoutingRequestFeatures {
            client_api_format: "openai:chat".into(),
            is_stream: true,
            input_token_estimate: Some(42),
            output_token_estimate: Some(100),
            request_size_bucket: RoutingRequestSizeBucket::Tiny,
            required_capability: Some("tool_calling".into()),
        };

        let key = routing_context_key("vip", "gpt-4.1", &features);

        assert_eq!(key, "group=vip|model=gpt-4.1|format=openai:chat|stream=true|size=tiny|cap=tool_calling");
    }
}
