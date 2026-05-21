use serde_json::Value;

use crate::format_conversion::{InternalContentBlock, InternalStreamEvent, StreamConversionState};

pub(super) fn push_reasoning_start(chunk: &Value, item: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    let index = chunk
        .get("output_index")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or_else(|| {
            super::stream_input_common::reserve_block_index(
                &mut state.openai_responses_reasoning_block_index,
                &mut state.openai_responses_next_source_block_index,
            )
        });
    state.openai_responses_reasoning_started = true;
    state.openai_responses_reasoning_block_index = Some(index);
    state.openai_responses_reasoning_signature = item.get("encrypted_content").and_then(Value::as_str).map(str::to_owned);
    output.push(InternalStreamEvent::ContentBlockStart {
        index,
        block: InternalContentBlock::Thinking {
            text: String::new(),
            signature: state.openai_responses_reasoning_signature.clone(),
        },
    });
}

pub(super) fn push_reasoning_delta(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    let Some(delta) = chunk.get("delta").and_then(Value::as_str).filter(|value| !value.is_empty()) else {
        return;
    };
    start_reasoning_if_needed(state, output);
    state.openai_responses_reasoning_text.push_str(delta);
    output.push(InternalStreamEvent::ThinkingDelta {
        text: delta.to_owned(),
        signature: None,
    });
}

pub(super) fn push_reasoning_done(state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    if state.openai_responses_reasoning_started && !state.openai_responses_reasoning_stopped {
        state.openai_responses_reasoning_stopped = true;
        if let Some(index) = state.openai_responses_reasoning_block_index {
            output.push(InternalStreamEvent::ContentBlockStop { index });
        }
    }
}

pub(super) fn append_reasoning_done_item(item: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    start_reasoning_if_needed(state, output);
    if let Some(signature) = item.get("encrypted_content").and_then(Value::as_str) {
        state.openai_responses_reasoning_signature = Some(signature.to_owned());
        output.push(InternalStreamEvent::ThinkingDelta {
            text: String::new(),
            signature: Some(signature.to_owned()),
        });
    }
    append_text_snapshot(reasoning_item_text(item), state, output);
    push_reasoning_done(state, output);
}

fn append_text_snapshot(snapshot: String, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    if snapshot.is_empty() || snapshot == state.openai_responses_reasoning_text {
        return;
    }
    let delta = snapshot
        .strip_prefix(state.openai_responses_reasoning_text.as_str())
        .unwrap_or(snapshot.as_str())
        .to_owned();
    state.openai_responses_reasoning_text = snapshot;
    output.push(InternalStreamEvent::ThinkingDelta { text: delta, signature: None });
}

fn start_reasoning_if_needed(state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    if state.openai_responses_reasoning_started {
        return;
    }
    let index = super::stream_input_common::reserve_block_index(
        &mut state.openai_responses_reasoning_block_index,
        &mut state.openai_responses_next_source_block_index,
    );
    state.openai_responses_reasoning_started = true;
    output.push(InternalStreamEvent::ContentBlockStart {
        index,
        block: InternalContentBlock::Thinking {
            text: String::new(),
            signature: state.openai_responses_reasoning_signature.clone(),
        },
    });
}

fn reasoning_item_text(item: &Value) -> String {
    item.get("summary")
        .and_then(Value::as_array)
        .map(|items| reasoning_text_parts(items))
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| {
            item.get("content")
                .and_then(Value::as_array)
                .map(|items| reasoning_text_parts(items))
                .unwrap_or_default()
        })
}

fn reasoning_text_parts(items: &[Value]) -> String {
    items
        .iter()
        .filter_map(|item| item.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("")
}
