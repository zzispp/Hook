use serde_json::Value;

use crate::format_conversion::{FormatConversionError, InternalStreamEvent, InternalToolKind, StopReason, StreamConversionState};

use super::{
    response::usage_from_response,
    stream_input_common::nested_string,
    stream_input_reasoning::{append_reasoning_done_item, push_reasoning_delta, push_reasoning_done, push_reasoning_start},
    stream_input_text::{push_delta, push_text_done},
    stream_input_tool::{finish_tool_item, push_tool_delta, push_tool_start, stop_open_tools, tool_kind_from_item_type},
    stream_output,
};

const FORMAT: &str = "openai_responses";

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

pub fn chunk_to_internal(chunk: &Value, state: &mut StreamConversionState) -> Result<Vec<InternalStreamEvent>, FormatConversionError> {
    let mut events = Vec::new();
    parse_chunk(chunk, state, &mut events)?;
    Ok(events)
}

pub fn event_from_internal(event: &InternalStreamEvent, state: &mut StreamConversionState) -> Result<Vec<Value>, FormatConversionError> {
    Ok(stream_output::event_from_internal(event, state))
}

fn parse_chunk(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    if chunk.get("error").is_some() {
        output.push(InternalStreamEvent::Error(crate::format_conversion::error_codec::to_internal(chunk, None)));
        return Ok(());
    }
    match chunk.get("type").and_then(Value::as_str).unwrap_or_default() {
        "response.created" => push_start(chunk, state, output),
        "response.output_item.added" => push_output_item_start(chunk, state, output)?,
        "response.output_item.done" => push_output_item_done(chunk, state, output)?,
        "response.output_text.delta" => push_delta(chunk, state, output),
        "response.output_text.done" => push_text_done(state, output),
        "response.reasoning_summary_text.delta" | "response.reasoning_text.delta" => push_reasoning_delta(chunk, state, output),
        "response.function_call_arguments.delta" => push_tool_delta(chunk, InternalToolKind::Function, state, output)?,
        "response.custom_tool_call_input.delta" => push_tool_delta(chunk, InternalToolKind::Custom, state, output)?,
        "response.function_call_arguments.done" => sync_done_arguments(chunk, state, output)?,
        "response.failed" => output.push(InternalStreamEvent::Error(crate::format_conversion::error_codec::to_internal(chunk, None))),
        "response.completed" => push_completed(chunk, state, output),
        "response.in_progress" | "response.content_part.added" | "response.content_part.done" | "response.reasoning_summary_part.added" => {}
        "" => return Err(FormatConversionError::invalid_payload(FORMAT, "$.type")),
        other => return Err(unsupported_event(other)),
    }
    Ok(())
}

fn push_start(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    let response = chunk.get("response");
    output.push(InternalStreamEvent::Start {
        id: nested_string(response, "id"),
        model: nested_string(response, "model"),
    });
    state.openai_responses_started = true;
}

fn push_output_item_start(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let Some(item) = chunk.get("item") else {
        return Ok(());
    };
    let item_type = item.get("type").and_then(Value::as_str).unwrap_or_default();
    if let Some(kind) = tool_kind_from_item_type(item_type) {
        push_tool_start(item, kind, state, output);
        return Ok(());
    }
    match item_type {
        "reasoning" => push_reasoning_start(chunk, item, state, output),
        "message" => {}
        other => return Err(unsupported_item(other)),
    }
    Ok(())
}

fn push_output_item_done(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let Some(item) = chunk.get("item") else {
        return Ok(());
    };
    let item_type = item.get("type").and_then(Value::as_str).unwrap_or_default();
    if tool_kind_from_item_type(item_type).is_some() {
        return finish_tool_item(item, state, output);
    }
    match item_type {
        "reasoning" => append_reasoning_done_item(item, state, output),
        "message" => {}
        other => return Err(unsupported_item(other)),
    }
    Ok(())
}

fn sync_done_arguments(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) -> Result<(), FormatConversionError> {
    let value = chunk.get("arguments");
    let item = serde_json::json!({
        "type": "function_call",
        "id": chunk.get("item_id").cloned().unwrap_or_default(),
        "call_id": chunk.get("call_id").cloned().or_else(|| chunk.get("item_id").cloned()).unwrap_or_default(),
        "arguments": value.cloned().unwrap_or_default(),
    });
    finish_tool_item(&item, state, output)
}

fn push_completed(chunk: &Value, state: &mut StreamConversionState, output: &mut Vec<InternalStreamEvent>) {
    push_text_done(state, output);
    push_reasoning_done(state, output);
    stop_open_tools(state, output);
    let usage = chunk
        .get("response")
        .and_then(|response| usage_from_response(response.get("usage")))
        .or_else(|| usage_from_response(chunk.get("usage")));
    let reason = if state.openai_responses_source_tools.is_empty() {
        Some(StopReason::EndTurn)
    } else {
        Some(StopReason::ToolUse)
    };
    output.push(InternalStreamEvent::Done { reason, usage });
}

fn unsupported_event(event_type: &str) -> FormatConversionError {
    FormatConversionError::unsupported_content(FORMAT, format!("unsupported stream event type {event_type}"))
}

fn unsupported_item(item_type: &str) -> FormatConversionError {
    FormatConversionError::unsupported_content(FORMAT, format!("unsupported output item type {item_type}"))
}
