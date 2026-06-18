mod enrich;
mod json;
mod redis_store;
mod store;
mod types;

pub(crate) use store::CodexChatHistoryStore;
pub(crate) use types::CodexChatHistoryError;

fn response_item_call_id(item: &serde_json::Value) -> Option<String> {
    item.get("call_id").or_else(|| item.get("id")).and_then(non_empty_string).map(ToOwned::to_owned)
}

fn is_call_item_type(item_type: &str) -> bool {
    item_type == "function_call"
}

fn is_call_output_item_type(item_type: &str) -> bool {
    item_type == "function_call_output"
}

fn non_empty_string(value: &serde_json::Value) -> Option<&str> {
    value.as_str().map(str::trim).filter(|value| !value.is_empty())
}

fn sorted_call_ids(call_ids: &std::collections::HashSet<String>) -> Vec<String> {
    let mut values = call_ids.iter().cloned().collect::<Vec<_>>();
    values.sort();
    values
}

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod tests;
