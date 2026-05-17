use serde_json::Value;

use crate::llm_proxy::audit::TokenUsage;

pub(super) fn finalize(mut usage: TokenUsage) -> Option<TokenUsage> {
    usage.total_tokens = usage.total_tokens.or_else(|| Some(usage.prompt_tokens? + usage.completion_tokens?));
    has_any_value(&usage).then_some(usage)
}

pub(super) fn number(value: Option<&Value>) -> Option<i64> {
    value?.as_i64().or_else(|| value?.as_u64().and_then(|number| i64::try_from(number).ok()))
}

pub(super) fn nested_number(value: Option<&Value>, key: &str) -> Option<i64> {
    number(value?.get(key))
}

pub(super) fn nested_number_any(value: Option<&Value>, keys: &[&str]) -> Option<i64> {
    keys.iter().find_map(|key| nested_number(value, key))
}

pub(super) fn number_any(value: &Value, keys: &[&str]) -> Option<i64> {
    keys.iter().find_map(|key| number(value.get(*key)))
}

pub(super) fn child<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|key| value.get(*key))
}

pub(super) fn sum_optional(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left + right),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn has_any_value(usage: &TokenUsage) -> bool {
    usage.prompt_tokens.is_some()
        || usage.completion_tokens.is_some()
        || usage.total_tokens.is_some()
        || usage.cache_creation_input_tokens.is_some()
        || usage.cache_read_input_tokens.is_some()
        || usage.input_text_tokens.is_some()
        || usage.input_audio_tokens.is_some()
        || usage.input_image_tokens.is_some()
        || usage.output_text_tokens.is_some()
        || usage.output_audio_tokens.is_some()
        || usage.output_image_tokens.is_some()
        || usage.reasoning_tokens.is_some()
        || usage.cache_creation_5m_input_tokens.is_some()
        || usage.cache_creation_1h_input_tokens.is_some()
}
