mod common;
mod request;
mod request_codec;
mod request_messages;
mod response;
mod stream;

use serde_json::Value;

use crate::format_conversion::{
    FormatConversionError, InternalError, InternalRequest, InternalResponse, InternalStreamEvent, StreamConversionState, normalizer::FormatNormalizer,
};

#[derive(Default)]
pub struct OpenAiNormalizer;

impl FormatNormalizer for OpenAiNormalizer {
    fn request_to_internal(&self, request: &Value) -> Result<InternalRequest, FormatConversionError> {
        request::to_internal(request)
    }

    fn request_from_internal(&self, internal: &InternalRequest) -> Result<Value, FormatConversionError> {
        request::from_internal(internal)
    }

    fn response_to_internal(&self, response: &Value) -> Result<InternalResponse, FormatConversionError> {
        response::to_internal(response)
    }

    fn response_from_internal(&self, internal: &InternalResponse) -> Result<Value, FormatConversionError> {
        response::from_internal(internal)
    }

    fn error_to_internal(&self, error: &Value, status: Option<u16>) -> Result<InternalError, FormatConversionError> {
        Ok(crate::format_conversion::error_codec::to_internal(error, status))
    }

    fn error_from_internal(&self, internal: &InternalError) -> Result<Value, FormatConversionError> {
        Ok(crate::format_conversion::error_codec::openai_error(internal))
    }

    fn stream_to_internal(&self, chunks: &[Value]) -> Result<Vec<InternalStreamEvent>, FormatConversionError> {
        stream::to_internal(chunks)
    }

    fn stream_from_internal(&self, events: &[InternalStreamEvent]) -> Result<Vec<Value>, FormatConversionError> {
        stream::from_internal(events)
    }

    fn stream_chunk_to_internal(&self, chunk: &Value, state: &mut StreamConversionState) -> Result<Vec<InternalStreamEvent>, FormatConversionError> {
        stream::chunk_to_internal(chunk, state)
    }

    fn stream_flush_to_internal(&self, state: &mut StreamConversionState) -> Result<Vec<InternalStreamEvent>, FormatConversionError> {
        Ok(stream::flush_to_internal(state))
    }

    fn stream_event_from_internal(&self, event: &InternalStreamEvent, state: &mut StreamConversionState) -> Result<Vec<Value>, FormatConversionError> {
        stream::event_from_internal(event, state)
    }
}
