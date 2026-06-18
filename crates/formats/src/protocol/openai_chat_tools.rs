use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

pub(crate) fn normalize_openai_responses_chat_tool_messages(messages: Vec<Value>) -> Vec<Value> {
    let tool_replies = openai_chat_tool_replies_by_id(&messages);
    if tool_replies.is_empty() && !messages.iter().any(openai_chat_message_has_tool_calls) {
        return messages;
    }

    let mut normalized = Vec::with_capacity(messages.len());
    let mut used_tool_reply_ids = BTreeSet::new();
    for message in messages {
        if openai_chat_tool_message_id(&message).is_some() {
            continue;
        }
        if openai_chat_message_has_tool_calls(&message) {
            append_normalized_tool_call_message(&mut normalized, message, &tool_replies, &mut used_tool_reply_ids);
        } else {
            normalized.push(message);
        }
    }
    normalized
}

fn append_normalized_tool_call_message(
    normalized: &mut Vec<Value>,
    message: Value,
    tool_replies: &BTreeMap<String, Value>,
    used_tool_reply_ids: &mut BTreeSet<String>,
) {
    let (kept_tool_calls, kept_tool_call_ids) = answered_openai_chat_tool_calls(&message, tool_replies, used_tool_reply_ids);
    let Some(message) = openai_chat_message_with_kept_tool_calls(message, kept_tool_calls) else {
        return;
    };
    normalized.push(message);
    for tool_call_id in kept_tool_call_ids {
        used_tool_reply_ids.insert(tool_call_id.clone());
        if let Some(tool_reply) = tool_replies.get(&tool_call_id) {
            normalized.push(tool_reply.clone());
        }
    }
}

fn answered_openai_chat_tool_calls(
    message: &Value,
    tool_replies: &BTreeMap<String, Value>,
    used_tool_reply_ids: &BTreeSet<String>,
) -> (Vec<Value>, Vec<String>) {
    let mut kept_tool_calls = Vec::new();
    let mut kept_tool_call_ids = Vec::new();
    let mut local_tool_call_ids = BTreeSet::new();
    for tool_call in openai_chat_tool_calls(message) {
        let Some(tool_call_id) = openai_chat_tool_call_id(tool_call) else {
            continue;
        };
        if tool_replies.contains_key(&tool_call_id) && !used_tool_reply_ids.contains(&tool_call_id) && local_tool_call_ids.insert(tool_call_id.clone()) {
            kept_tool_calls.push(tool_call.clone());
            kept_tool_call_ids.push(tool_call_id);
        }
    }
    (kept_tool_calls, kept_tool_call_ids)
}

fn openai_chat_message_with_kept_tool_calls(mut message: Value, tool_calls: Vec<Value>) -> Option<Value> {
    let object = message.as_object_mut()?;
    if tool_calls.is_empty() {
        object.remove("tool_calls");
        return openai_chat_message_has_non_tool_payload(&message).then_some(message);
    }
    object.insert("tool_calls".to_string(), Value::Array(tool_calls));
    Some(message)
}

fn openai_chat_tool_replies_by_id(messages: &[Value]) -> BTreeMap<String, Value> {
    let mut replies = BTreeMap::new();
    for message in messages {
        if let Some(tool_call_id) = openai_chat_tool_message_id(message) {
            replies.entry(tool_call_id).or_insert_with(|| message.clone());
        }
    }
    replies
}

fn openai_chat_tool_message_id(message: &Value) -> Option<String> {
    let object = message.as_object()?;
    if object.get("role").and_then(Value::as_str) != Some("tool") {
        return None;
    }
    object.get("tool_call_id").and_then(trimmed_string_value)
}

fn openai_chat_message_has_tool_calls(message: &Value) -> bool {
    !openai_chat_tool_calls(message).is_empty()
}

fn openai_chat_tool_calls(message: &Value) -> &[Value] {
    message.get("tool_calls").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[])
}

fn openai_chat_tool_call_id(tool_call: &Value) -> Option<String> {
    tool_call.as_object()?.get("id").and_then(trimmed_string_value)
}

fn trimmed_string_value(value: &Value) -> Option<String> {
    value.as_str().map(str::trim).filter(|value| !value.is_empty()).map(ToOwned::to_owned)
}

fn openai_chat_message_has_non_tool_payload(message: &Value) -> bool {
    let Some(object) = message.as_object() else {
        return false;
    };
    ["content", "reasoning_content", "reasoning_parts"]
        .iter()
        .filter_map(|key| object.get(*key))
        .any(openai_chat_payload_is_non_empty)
}

fn openai_chat_payload_is_non_empty(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(text) => !text.trim().is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(object) => !object.is_empty(),
        Value::Bool(_) | Value::Number(_) => true,
    }
}
