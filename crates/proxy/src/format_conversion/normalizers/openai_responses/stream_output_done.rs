use serde_json::{Value, json};

use crate::format_conversion::{InternalUsage, StreamConversionState};

use super::{
    stream_output_common::{
        allocate_output_index, message_item, message_item_id, message_output_index, next_sequence, output_text_part, response_state_payload,
    },
    stream_output_reasoning::{final_reasoning_item, push_reasoning_done},
    stream_output_tool::final_tool_items,
};

pub(super) fn push_text_started(state: &mut StreamConversionState, output: &mut Vec<Value>) {
    if !state.target_openai_responses_message_started {
        let output_index = allocate_output_index(state);
        state.target_openai_responses_message_output_index = Some(output_index);
        state.target_openai_responses_message_started = true;
        output.push(json!({
            "type": "response.output_item.added",
            "sequence_number": next_sequence(state),
            "output_index": output_index,
            "item": message_item(state, "in_progress"),
        }));
    }
    if !state.target_openai_responses_text_started {
        state.target_openai_responses_text_started = true;
        output.push(json!({
            "type": "response.content_part.added",
            "sequence_number": next_sequence(state),
            "item_id": message_item_id(state),
            "output_index": message_output_index(state),
            "content_index": 0,
            "part": { "type": "output_text", "text": "", "annotations": [] },
        }));
    }
}

pub(super) fn push_completed_event(usage: Option<&InternalUsage>, state: &StreamConversionState, output: &mut Vec<Value>) {
    let mut state = state.clone();
    push_reasoning_done(&mut state, output);
    push_text_done(&mut state, output);
    let output_items = final_output_items(&state);
    let mut response = response_state_payload(&state, "completed", output_items);
    if let Some(usage_value) = usage_json(usage) {
        response["usage"] = usage_value;
    }
    output.push(json!({
        "type": "response.completed",
        "sequence_number": next_sequence(&mut state),
        "response": response,
    }));
}

fn push_text_done(state: &mut StreamConversionState, output: &mut Vec<Value>) {
    if !state.target_openai_responses_text_started {
        return;
    }
    let text = state.target_openai_responses_text.clone();
    output.push(json!({
        "type": "response.output_text.done",
        "sequence_number": next_sequence(state),
        "item_id": message_item_id(state),
        "output_index": message_output_index(state),
        "content_index": 0,
        "text": text,
    }));
    output.push(json!({
        "type": "response.content_part.done",
        "sequence_number": next_sequence(state),
        "item_id": message_item_id(state),
        "output_index": message_output_index(state),
        "content_index": 0,
        "part": output_text_part(&text),
    }));
    output.push(json!({
        "type": "response.output_item.done",
        "sequence_number": next_sequence(state),
        "output_index": message_output_index(state),
        "item": message_item(state, "completed"),
    }));
}

fn final_output_items(state: &StreamConversionState) -> Vec<Value> {
    let mut items = final_tool_items(state);
    if let Some(reasoning) = final_reasoning_item(state) {
        items.push(reasoning);
    }
    if state.target_openai_responses_message_started {
        items.push((message_output_index(state), message_item(state, "completed")));
    }
    items.sort_by_key(|item| item.0);
    items.into_iter().map(|(_, item)| item).collect()
}

fn usage_json(usage: Option<&InternalUsage>) -> Option<Value> {
    let complete = usage.cloned()?.with_total();
    let prompt_tokens = complete.prompt_tokens.unwrap_or_default();
    let completion_tokens = complete.completion_tokens.unwrap_or_default();
    let mut output = json!({
        "input_tokens": prompt_tokens,
        "output_tokens": completion_tokens,
        "total_tokens": complete.total_tokens.unwrap_or(prompt_tokens.saturating_add(completion_tokens)),
    });
    insert_usage_details(&mut output, &complete);
    Some(output)
}

fn insert_usage_details(output: &mut Value, usage: &InternalUsage) {
    if usage.cache_read_tokens.is_some() || usage.cache_creation_tokens.is_some() {
        output["input_tokens_details"] = json!({
            "cached_tokens": usage.cache_read_tokens.unwrap_or_default(),
            "cache_creation_tokens": usage.cache_creation_tokens.unwrap_or_default(),
        });
    }
    if let Some(reasoning_tokens) = usage.reasoning_tokens {
        output["output_tokens_details"] = json!({ "reasoning_tokens": reasoning_tokens });
    }
}
