use serde_json::{Value, json};

use crate::format_conversion::StreamConversionState;

pub(super) fn response_state_payload(state: &StreamConversionState, status: &str, output: Vec<Value>) -> Value {
    json!({
        "id": state.target_openai_responses_id,
        "object": "response",
        "created_at": 0,
        "status": status,
        "background": false,
        "error": null,
        "model": state.target_openai_responses_model,
        "output": output,
    })
}

pub(super) fn message_item(state: &StreamConversionState, status: &str) -> Value {
    let content = if state.target_openai_responses_text.is_empty() {
        Vec::new()
    } else {
        vec![output_text_part(&state.target_openai_responses_text)]
    };
    json!({
        "type": "message",
        "id": message_item_id(state),
        "role": "assistant",
        "status": status,
        "content": content,
    })
}

pub(super) fn output_text_part(text: &str) -> Value {
    json!({ "type": "output_text", "text": text, "annotations": [] })
}

pub(super) fn reasoning_item(state: &StreamConversionState, status: &str) -> Value {
    let summary = if state.target_openai_responses_thinking_text.is_empty() {
        Vec::new()
    } else {
        vec![reasoning_summary_part(&state.target_openai_responses_thinking_text)]
    };
    let mut item = json!({
        "type": "reasoning",
        "id": reasoning_item_id(state),
        "summary": summary,
        "status": status,
    });
    if let Some(signature) = &state.target_openai_responses_thinking_signature {
        item["encrypted_content"] = Value::String(signature.clone());
    }
    item
}

pub(super) fn reasoning_summary_part(text: &str) -> Value {
    json!({ "type": "summary_text", "text": text })
}

pub(super) fn reasoning_item_id(state: &StreamConversionState) -> String {
    format!("rs_{}", state.target_openai_responses_id)
}

pub(super) fn reasoning_output_index(state: &StreamConversionState) -> u32 {
    state.target_openai_responses_reasoning_output_index.unwrap_or(0)
}

pub(super) fn message_item_id(state: &StreamConversionState) -> String {
    format!("msg_{}", state.target_openai_responses_id)
}

pub(super) fn message_output_index(state: &StreamConversionState) -> u32 {
    state.target_openai_responses_message_output_index.unwrap_or(0)
}

pub(super) fn function_call_item(call_id: &str, item_id: &str, name: &str, arguments: &str, status: &str) -> Value {
    json!({
        "type": "function_call",
        "call_id": call_id,
        "id": item_id,
        "name": name,
        "arguments": arguments,
        "status": status,
    })
}

pub(super) fn allocate_output_index(state: &mut StreamConversionState) -> u32 {
    let index = state.target_openai_responses_next_output_index;
    state.target_openai_responses_next_output_index = state.target_openai_responses_next_output_index.saturating_add(1);
    index
}

pub(super) fn next_sequence(state: &mut StreamConversionState) -> u32 {
    state.target_openai_responses_sequence = state.target_openai_responses_sequence.saturating_add(1);
    state.target_openai_responses_sequence
}
