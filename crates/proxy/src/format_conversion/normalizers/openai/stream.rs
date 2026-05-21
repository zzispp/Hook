use serde_json::Value;

use crate::format_conversion::{FormatConversionError, InternalStreamEvent, StreamConversionState};

pub fn to_internal(chunks: &[Value]) -> Result<Vec<InternalStreamEvent>, FormatConversionError> {
    let mut state = StreamConversionState::default();
    let mut events = Vec::new();
    for chunk in chunks {
        events.extend(chunk_to_internal(chunk, &mut state)?);
    }
    events.extend(flush_to_internal(&mut state));
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
    super::stream_input::chunk_to_internal(chunk, state)
}

pub fn event_from_internal(event: &InternalStreamEvent, state: &mut StreamConversionState) -> Result<Vec<Value>, FormatConversionError> {
    super::stream_output::event_from_internal(event, state)
}

pub fn flush_to_internal(state: &mut StreamConversionState) -> Vec<InternalStreamEvent> {
    super::stream_input::flush_to_internal(state)
}
