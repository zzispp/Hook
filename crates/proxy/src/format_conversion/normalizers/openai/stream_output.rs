use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalStreamEvent, InternalToolKind, InternalUsage, StreamConversionState};

use super::common::openai_finish_reason;

pub(super) fn event_from_internal(event: &InternalStreamEvent, state: &mut StreamConversionState) -> Result<Vec<Value>, FormatConversionError> {
    ensure_target(state);
    reject_custom_tool_event(event)?;
    let mut output = Vec::new();
    push_stream_event(event, state, &mut output);
    Ok(output)
}

fn ensure_target(state: &mut StreamConversionState) {
    if state.target_openai_id.is_empty() {
        state.target_openai_id = "chatcmpl_unknown".to_owned();
    }
    if state.target_openai_model.is_empty() {
        state.target_openai_model = "openai-unknown".to_owned();
    }
}

fn push_stream_event(event: &InternalStreamEvent, state: &mut StreamConversionState, output: &mut Vec<Value>) {
    match event {
        InternalStreamEvent::Start {
            id: event_id,
            model: event_model,
        } => push_start(event_id, event_model, state, output),
        InternalStreamEvent::TextDelta(text) => push_delta(json!({"content": text}), state, output),
        InternalStreamEvent::ThinkingDelta { text, .. } => push_delta(json!({"reasoning_content": text}), state, output),
        InternalStreamEvent::ToolCallDelta {
            index,
            id,
            name,
            arguments_delta,
        } => push_tool_delta(*index, id.as_ref(), name.as_ref(), arguments_delta, state, output),
        InternalStreamEvent::Usage(usage) => output.push(openai_stream_chunk(
            &state.target_openai_id,
            &state.target_openai_model,
            json!({}),
            None,
            usage_json(Some(usage)),
        )),
        InternalStreamEvent::ContentBlockStart { .. } | InternalStreamEvent::ContentBlockStop { .. } => {}
        InternalStreamEvent::Error(error) => output.push(json!({ "error": {
            "message": error.message,
            "type": error.error_type,
            "code": error.code,
            "param": error.param,
        }})),
        InternalStreamEvent::Done { reason, usage } => {
            let finish_reason = reason.as_ref().map(openai_finish_reason);
            output.push(openai_stream_chunk(
                &state.target_openai_id,
                &state.target_openai_model,
                json!({}),
                finish_reason,
                usage_json(usage.as_ref()),
            ));
        }
    }
}

fn push_start(event_id: &Option<String>, event_model: &Option<String>, state: &mut StreamConversionState, output: &mut Vec<Value>) {
    state.target_openai_id = event_id.clone().unwrap_or_else(|| state.target_openai_id.clone());
    state.target_openai_model = event_model.clone().unwrap_or_else(|| state.target_openai_model.clone());
    push_delta(json!({"role": "assistant"}), state, output);
}

fn push_delta(delta: Value, state: &StreamConversionState, output: &mut Vec<Value>) {
    output.push(openai_stream_chunk(&state.target_openai_id, &state.target_openai_model, delta, None, None));
}

fn push_tool_delta(index: u32, id: Option<&String>, name: Option<&String>, arguments_delta: &str, state: &StreamConversionState, output: &mut Vec<Value>) {
    output.push(openai_stream_chunk(
        &state.target_openai_id,
        &state.target_openai_model,
        json!({"tool_calls": [{
            "index": index,
            "id": id,
            "type": "function",
            "function": { "name": name, "arguments": arguments_delta },
        }]}),
        None,
        None,
    ));
}

fn usage_json(usage: Option<&InternalUsage>) -> Option<Value> {
    let complete = usage.cloned()?.with_total();
    Some(json!({
        "prompt_tokens": complete.prompt_tokens,
        "completion_tokens": complete.completion_tokens,
        "total_tokens": complete.total_tokens,
        "prompt_tokens_details": {
            "cached_tokens": complete.cache_read_tokens,
            "cache_creation_tokens": complete.cache_creation_tokens,
        },
        "completion_tokens_details": {
            "reasoning_tokens": complete.reasoning_tokens,
        },
    }))
}

fn openai_stream_chunk(id: &str, model: &str, delta: Value, finish_reason: Option<&str>, usage: Option<Value>) -> Value {
    let mut chunk = json!({
        "id": id,
        "model": model,
        "object": "chat.completion.chunk",
        "choices": [{
            "index": 0,
            "delta": delta,
            "finish_reason": finish_reason,
        }]
    });
    if let Some(usage_payload) = usage {
        chunk["usage"] = usage_payload;
    }
    chunk
}

fn reject_custom_tool_event(event: &InternalStreamEvent) -> Result<(), FormatConversionError> {
    let InternalStreamEvent::ContentBlockStart { block, .. } = event else {
        return Ok(());
    };
    if matches!(
        block,
        InternalContentBlock::ToolUse {
            kind: InternalToolKind::Custom,
            ..
        }
    ) {
        return Err(FormatConversionError::unsupported_content(
            super::common::FORMAT,
            "OpenAI Chat stream cannot represent custom tool calls",
        ));
    }
    Ok(())
}
