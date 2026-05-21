use serde_json::{Value, json};

use crate::format_conversion::StreamConversionState;

use super::stream_output_common::{allocate_output_index, next_sequence, reasoning_item, reasoning_item_id, reasoning_output_index, reasoning_summary_part};

pub(super) fn push_reasoning_started(state: &mut StreamConversionState, output: &mut Vec<Value>) {
    if state.target_openai_responses_reasoning_started {
        return;
    }
    let output_index = allocate_output_index(state);
    state.target_openai_responses_reasoning_output_index = Some(output_index);
    state.target_openai_responses_reasoning_started = true;
    output.push(json!({
        "type": "response.output_item.added",
        "sequence_number": next_sequence(state),
        "output_index": output_index,
        "item": reasoning_item(state, "in_progress"),
    }));
}

pub(super) fn push_reasoning_delta(text: &str, signature: Option<&str>, state: &mut StreamConversionState, output: &mut Vec<Value>) {
    push_reasoning_started(state, output);
    if let Some(signature) = signature.filter(|value| !value.is_empty()) {
        state.target_openai_responses_thinking_signature = Some(signature.to_owned());
    }
    if text.is_empty() {
        return;
    }
    state.target_openai_responses_thinking_text.push_str(text);
    output.push(json!({
        "type": "response.reasoning_summary_text.delta",
        "sequence_number": next_sequence(state),
        "item_id": reasoning_item_id(state),
        "output_index": reasoning_output_index(state),
        "summary_index": 0,
        "delta": text,
    }));
}

pub(super) fn push_reasoning_done(state: &mut StreamConversionState, output: &mut Vec<Value>) {
    if !state.target_openai_responses_reasoning_started {
        return;
    }
    let text = state.target_openai_responses_thinking_text.clone();
    output.push(json!({
        "type": "response.reasoning_summary_text.done",
        "sequence_number": next_sequence(state),
        "item_id": reasoning_item_id(state),
        "output_index": reasoning_output_index(state),
        "summary_index": 0,
        "text": text,
    }));
    output.push(json!({
        "type": "response.reasoning_summary_part.done",
        "sequence_number": next_sequence(state),
        "item_id": reasoning_item_id(state),
        "output_index": reasoning_output_index(state),
        "summary_index": 0,
        "part": reasoning_summary_part(&text),
    }));
    output.push(json!({
        "type": "response.output_item.done",
        "sequence_number": next_sequence(state),
        "output_index": reasoning_output_index(state),
        "item": reasoning_item(state, "completed"),
    }));
}

pub(super) fn final_reasoning_item(state: &StreamConversionState) -> Option<(u32, Value)> {
    state
        .target_openai_responses_reasoning_started
        .then(|| (reasoning_output_index(state), reasoning_item(state, "completed")))
}
