use serde_json::{Value, json};

use crate::format_conversion::{InternalStreamEvent, StreamConversionState};

use super::{
    stream_output_common::{message_item_id, message_output_index, next_sequence, response_state_payload},
    stream_output_done::{push_completed_event, push_text_started},
    stream_output_reasoning::push_reasoning_delta,
    stream_output_tool::{push_block_start, push_block_stop, push_tool_delta},
};

pub fn event_from_internal(event: &InternalStreamEvent, state: &mut StreamConversionState) -> Vec<Value> {
    ensure_target(state);
    let mut output = Vec::new();
    push_event(event, state, &mut output);
    output
}

fn ensure_target(state: &mut StreamConversionState) {
    if state.target_openai_responses_id.is_empty() {
        state.target_openai_responses_id = "resp_unknown".to_owned();
    }
    if state.target_openai_responses_model.is_empty() {
        state.target_openai_responses_model = "openai-responses-unknown".to_owned();
    }
}

fn push_event(event: &InternalStreamEvent, state: &mut StreamConversionState, output: &mut Vec<Value>) {
    match event {
        InternalStreamEvent::Start {
            id: event_id,
            model: event_model,
        } => push_start(event_id, event_model, state, output),
        InternalStreamEvent::TextDelta(text) => push_text_delta(text, state, output),
        InternalStreamEvent::ThinkingDelta { text, signature } => push_reasoning_delta(text, signature.as_deref(), state, output),
        InternalStreamEvent::ToolCallDelta {
            index,
            id,
            name,
            arguments_delta,
        } => push_tool_delta(*index, id.as_deref(), name.as_deref(), arguments_delta, state, output),
        InternalStreamEvent::Usage(usage) => push_completed_event(Some(usage), state, output),
        InternalStreamEvent::ContentBlockStart { index, block } => push_block_start(*index, block, state, output),
        InternalStreamEvent::ContentBlockStop { index } => push_block_stop(*index, state, output),
        InternalStreamEvent::Error(error) => output.push(json!({"type": "error", "error": {
            "message": error.message,
            "type": error.error_type,
            "code": error.code,
            "param": error.param,
        }})),
        InternalStreamEvent::Done { usage, .. } => push_completed_event(usage.as_ref(), state, output),
    }
}

fn push_start(event_id: &Option<String>, event_model: &Option<String>, state: &mut StreamConversionState, output: &mut Vec<Value>) {
    state.target_openai_responses_id = event_id.clone().unwrap_or_else(|| state.target_openai_responses_id.clone());
    state.target_openai_responses_model = event_model.clone().unwrap_or_else(|| state.target_openai_responses_model.clone());
    let response = response_state_payload(state, "in_progress", Vec::new());
    output.push(json!({
        "type": "response.created",
        "sequence_number": next_sequence(state),
        "response": response,
    }));
    output.push(json!({
        "type": "response.in_progress",
        "sequence_number": next_sequence(state),
        "response": response_state_payload(state, "in_progress", Vec::new()),
    }));
}

fn push_text_delta(text: &str, state: &mut StreamConversionState, output: &mut Vec<Value>) {
    push_text_started(state, output);
    state.target_openai_responses_text.push_str(text);
    output.push(json!({
        "type": "response.output_text.delta",
        "sequence_number": next_sequence(state),
        "item_id": message_item_id(state),
        "output_index": message_output_index(state),
        "content_index": 0,
        "delta": text,
    }));
}
