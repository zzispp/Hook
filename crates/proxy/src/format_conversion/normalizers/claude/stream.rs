use serde_json::{Value, json};

use crate::format_conversion::{FormatConversionError, InternalStreamEvent};

use super::common::{claude_stop_reason, claude_usage, map_claude_stop_reason, required_object, usage_from_claude};

pub fn to_internal(chunks: &[Value]) -> Result<Vec<InternalStreamEvent>, FormatConversionError> {
    let mut events = Vec::new();
    for chunk in chunks {
        parse_event(chunk, &mut events)?;
    }
    Ok(events)
}

pub fn from_internal(events: &[InternalStreamEvent]) -> Result<Vec<Value>, FormatConversionError> {
    let mut output = Vec::new();
    let mut id = "msg_unknown".to_owned();
    let mut model = "claude-unknown".to_owned();
    for event in events {
        push_event(event, &mut id, &mut model, &mut output);
    }
    Ok(output)
}

fn parse_event(chunk: &Value, events: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    match chunk.get("type").and_then(Value::as_str).unwrap_or_default() {
        "message_start" => parse_message_start(chunk, events),
        "content_block_delta" => parse_content_delta(chunk, events),
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

fn parse_content_delta(chunk: &Value, events: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let delta = required_object(chunk.get("delta"), "$.delta")?;
    if delta.get("type").and_then(Value::as_str) == Some("text_delta") {
        let text = delta.get("text").and_then(Value::as_str).unwrap_or_default();
        if !text.is_empty() {
            events.push(InternalStreamEvent::TextDelta(text.to_owned()));
        }
    }
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

fn push_event(event: &InternalStreamEvent, id: &mut String, model: &mut String, output: &mut Vec<Value>) {
    match event {
        InternalStreamEvent::Start {
            id: event_id,
            model: event_model,
        } => push_start(event_id, event_model, id, model, output),
        InternalStreamEvent::TextDelta(text) => output.push(json!({
            "type": "content_block_delta",
            "index": 0,
            "delta": { "type": "text_delta", "text": text },
        })),
        InternalStreamEvent::Done { reason, usage } => push_done(reason.as_ref(), usage.as_ref(), output),
    }
}

fn push_start(event_id: &Option<String>, event_model: &Option<String>, id: &mut String, model: &mut String, output: &mut Vec<Value>) {
    *id = event_id.clone().unwrap_or_else(|| id.clone());
    *model = event_model.clone().unwrap_or_else(|| model.clone());
    output.push(json!({
        "type": "message_start",
        "message": {
            "id": id,
            "type": "message",
            "role": "assistant",
            "model": model,
            "content": [],
            "stop_reason": null,
            "stop_sequence": null,
        },
    }));
    output.push(json!({ "type": "content_block_start", "index": 0, "content_block": { "type": "text", "text": "" } }));
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
