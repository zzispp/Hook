use serde_json::Value;

use crate::format_conversion::{InternalContentBlock, InternalStreamEvent, StreamConversionState};

pub(super) fn push_delta(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    if !state.openai_responses_started {
        output.push(InternalStreamEvent::Start { id: None, model: None });
        state.openai_responses_started = true;
    }
    if let Some(delta) = chunk.get("delta").and_then(Value::as_str).filter(|value| !value.is_empty()) {
        start_text_block(state, output);
        state.openai_responses_text.push_str(delta);
        output.push(InternalStreamEvent::TextDelta(delta.to_owned()));
    }
}

pub(super) fn push_text_done(state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    if state.openai_responses_text_started && !state.openai_responses_text_stopped {
        state.openai_responses_text_stopped = true;
        if let Some(index) = state.openai_responses_text_block_index {
            output.push(InternalStreamEvent::ContentBlockStop { index });
        }
    }
}

fn start_text_block(state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    if state.openai_responses_text_started {
        return;
    }
    let index = super::stream_input_common::reserve_block_index(
        &mut state.openai_responses_text_block_index,
        &mut state.openai_responses_next_source_block_index,
    );
    state.openai_responses_text_started = true;
    output.push(InternalStreamEvent::ContentBlockStart {
        index,
        block: InternalContentBlock::text(String::new()),
    });
}
