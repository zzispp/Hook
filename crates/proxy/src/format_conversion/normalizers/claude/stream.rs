use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalContentBlock, InternalStreamEvent, StreamConversionState};

use super::common::{claude_stop_reason, claude_usage, empty_claude_usage, map_claude_stop_reason, required_object, usage_from_claude};

pub fn to_internal(chunks: &[Value]) -> Result<Vec<InternalStreamEvent>, FormatConversionError> {
    let mut state = StreamConversionState::default();
    let mut events = Vec::new();
    for chunk in chunks {
        events.extend(chunk_to_internal(chunk, &mut state)?);
    }
    Ok(events)
}

pub fn from_internal(events: &[InternalStreamEvent]) -> Result<Vec<Value>, FormatConversionError> {
    let mut state = StreamConversionState::default();
    let mut output = Vec::new();
    for event in events {
        output.extend(event_from_internal(event, &mut state)?);
    }
    Ok(output)
}

pub fn chunk_to_internal(chunk: &Value, _state: &mut StreamConversionState) -> Result<Vec<InternalStreamEvent>, FormatConversionError> {
    let mut events = Vec::new();
    parse_event(chunk, &mut events)?;
    Ok(events)
}

pub fn event_from_internal(event: &InternalStreamEvent, state: &mut StreamConversionState) -> Result<Vec<Value>, FormatConversionError> {
    if state.target_claude_id.is_empty() {
        state.target_claude_id = "msg_unknown".to_owned();
    }
    if state.target_claude_model.is_empty() {
        state.target_claude_model = "claude-unknown".to_owned();
    }
    let mut output = Vec::new();
    push_event(event, state, &mut output);
    Ok(output)
}

fn parse_event(chunk: &Value, events: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    match chunk.get("type").and_then(Value::as_str).unwrap_or_default() {
        "message_start" => parse_message_start(chunk, events),
        "content_block_start" => parse_content_start(chunk, events),
        "content_block_delta" => parse_content_delta(chunk, events),
        "content_block_stop" => parse_content_stop(chunk, events),
        "message_delta" => parse_message_delta(chunk, events),
        _ => Ok(()),
    }
}

fn parse_message_start(chunk: &Value, events: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let message = required_object(chunk.get("message"), "$.message")?;
    events.push(InternalStreamEvent::Start {
        id: message.get("id").and_then(Value::as_str).map(str::to_owned),
        model: message.get("model").and_then(Value::as_str).map(str::to_owned),
    });
    Ok(())
}

fn parse_content_start(chunk: &Value, events: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let Some(block) = chunk.get("content_block").and_then(Value::as_object) else {
        return Ok(());
    };
    let index = chunk
        .get("index")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(0);
    match block.get("type").and_then(Value::as_str).unwrap_or_default() {
        "tool_use" => events.push(InternalStreamEvent::ContentBlockStart {
            index,
            block: InternalContentBlock::ToolUse {
                id: block.get("id").and_then(Value::as_str).unwrap_or_default().to_owned(),
                name: block.get("name").and_then(Value::as_str).unwrap_or_default().to_owned(),
                input: block.get("input").cloned().unwrap_or_else(|| json!({})),
                kind: crate::format_conversion::InternalToolKind::Function,
            },
        }),
        "thinking" => events.push(InternalStreamEvent::ContentBlockStart {
            index,
            block: InternalContentBlock::Thinking {
                text: String::new(),
                signature: None,
            },
        }),
        _ => {}
    }
    Ok(())
}

fn parse_content_delta(chunk: &Value, events: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let delta = required_object(chunk.get("delta"), "$.delta")?;
    match delta.get("type").and_then(Value::as_str).unwrap_or_default() {
        "text_delta" => {
            let text = delta.get("text").and_then(Value::as_str).unwrap_or_default();
            if !text.is_empty() {
                events.push(InternalStreamEvent::TextDelta(text.to_owned()));
            }
        }
        "thinking_delta" => {
            let text = delta.get("thinking").and_then(Value::as_str).unwrap_or_default();
            if !text.is_empty() {
                events.push(InternalStreamEvent::ThinkingDelta {
                    text: text.to_owned(),
                    signature: None,
                });
            }
        }
        "signature_delta" => {
            if let Some(signature) = delta.get("signature").and_then(Value::as_str) {
                events.push(InternalStreamEvent::ThinkingDelta {
                    text: String::new(),
                    signature: Some(signature.to_owned()),
                });
            }
        }
        "input_json_delta" => {
            if let Some(partial_json) = delta.get("partial_json").and_then(Value::as_str) {
                events.push(InternalStreamEvent::ToolCallDelta {
                    index: chunk
                        .get("index")
                        .and_then(Value::as_u64)
                        .and_then(|value| u32::try_from(value).ok())
                        .unwrap_or(0),
                    id: None,
                    name: None,
                    arguments_delta: partial_json.to_owned(),
                });
            }
        }
        _ => {}
    }
    Ok(())
}

fn parse_content_stop(chunk: &Value, events: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let index = chunk
        .get("index")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(0);
    events.push(InternalStreamEvent::ContentBlockStop { index });
    Ok(())
}

fn parse_message_delta(chunk: &Value, events: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let delta = required_object(chunk.get("delta"), "$.delta")?;
    let reason = delta.get("stop_reason").and_then(Value::as_str).map(map_claude_stop_reason);
    events.push(InternalStreamEvent::Done {
        reason,
        usage: usage_from_claude(chunk.get("usage")),
    });
    Ok(())
}

fn push_event(event: &InternalStreamEvent, state: &mut StreamConversionState, output: &mut Vec<Value>) {
    match event {
        InternalStreamEvent::Start {
            id: event_id,
            model: event_model,
        } => push_start(event_id, event_model, state, output),
        InternalStreamEvent::TextDelta(text) => output.push(json!({
            "type": "content_block_delta",
            "index": 0,
            "delta": { "type": "text_delta", "text": text },
        })),
        InternalStreamEvent::ThinkingDelta { text, signature } => {
            if !text.is_empty() {
                output.push(json!({
                    "type": "content_block_delta",
                    "index": 0,
                    "delta": { "type": "thinking_delta", "thinking": text },
                }));
            }
            if let Some(signature) = signature {
                output.push(json!({
                    "type": "content_block_delta",
                    "index": 0,
                    "delta": { "type": "signature_delta", "signature": signature },
                }));
            }
        }
        InternalStreamEvent::ToolCallDelta { index, arguments_delta, .. } => output.push(json!({
            "type": "content_block_delta",
            "index": index,
            "delta": { "type": "input_json_delta", "partial_json": arguments_delta },
        })),
        InternalStreamEvent::Usage(usage) => push_done(None, Some(usage), output),
        InternalStreamEvent::ContentBlockStart { index, block } => push_content_block_start(*index, block, output),
        InternalStreamEvent::ContentBlockStop { index } => output.push(json!({ "type": "content_block_stop", "index": index })),
        InternalStreamEvent::Error(error) => output.push(json!({
            "type": "error",
            "error": { "message": error.message, "type": error.error_type },
        })),
        InternalStreamEvent::Done { reason, usage } => push_done(reason.as_ref(), usage.as_ref(), output),
    }
}

fn push_content_block_start(index: u32, block: &InternalContentBlock, output: &mut Vec<Value>) {
    match block {
        InternalContentBlock::Thinking { .. } => output.push(json!({
            "type": "content_block_start",
            "index": index,
            "content_block": { "type": "thinking", "thinking": "" },
        })),
        InternalContentBlock::Text { .. } => output.push(json!({
            "type": "content_block_start",
            "index": index,
            "content_block": { "type": "text", "text": "" },
        })),
        InternalContentBlock::ToolUse { id, name, input, .. } => output.push(json!({
            "type": "content_block_start",
            "index": index,
            "content_block": { "type": "tool_use", "id": id, "name": name, "input": input },
        })),
        _ => {}
    }
}

fn push_start(event_id: &Option<String>, event_model: &Option<String>, state: &mut StreamConversionState, output: &mut Vec<Value>) {
    state.target_claude_id = event_id.clone().unwrap_or_else(|| state.target_claude_id.clone());
    state.target_claude_model = event_model.clone().unwrap_or_else(|| state.target_claude_model.clone());
    output.push(json!({
        "type": "message_start",
        "message": {
            "id": state.target_claude_id,
            "type": "message",
            "role": "assistant",
            "model": state.target_claude_model,
            "content": [],
            "stop_reason": null,
            "stop_sequence": null,
            "usage": empty_claude_usage(),
        },
    }));
}

fn push_done(reason: Option<&crate::format_conversion::StopReason>, usage: Option<&crate::format_conversion::InternalUsage>, output: &mut Vec<Value>) {
    output.push(json!({ "type": "content_block_stop", "index": 0 }));
    let mut delta = json!({
        "type": "message_delta",
        "delta": { "stop_reason": reason.map(claude_stop_reason) },
    });
    if let Some(usage_value) = usage {
        delta["usage"] = claude_usage(usage_value);
    }
    output.push(delta);
    output.push(json!({ "type": "message_stop" }));
}
