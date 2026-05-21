use serde_json::Value;

use super::{FormatConversionError, InternalError, InternalRequest, InternalResponse, InternalStreamEvent, StreamConversionState};

pub trait FormatNormalizer {
    fn request_to_internal(&self, request: &Value) -> Result<InternalRequest, FormatConversionError>;
    fn request_from_internal(&self, internal: &InternalRequest) -> Result<Value, FormatConversionError>;

    fn response_to_internal(&self, response: &Value) -> Result<InternalResponse, FormatConversionError>;
    fn response_from_internal(&self, internal: &InternalResponse) -> Result<Value, FormatConversionError>;

    fn error_to_internal(&self, error: &Value, status: Option<u16>) -> Result<InternalError, FormatConversionError>;
    fn error_from_internal(&self, internal: &InternalError) -> Result<Value, FormatConversionError>;

    fn stream_to_internal(&self, chunks: &[Value]) -> Result<Vec<InternalStreamEvent>, FormatConversionError>;
    fn stream_from_internal(&self, events: &[InternalStreamEvent]) -> Result<Vec<Value>, FormatConversionError>;
    fn stream_chunk_to_internal(&self, chunk: &Value, state: &mut StreamConversionState) -> Result<Vec<InternalStreamEvent>, FormatConversionError>;
    fn stream_flush_to_internal(&self, _state: &mut StreamConversionState) -> Result<Vec<InternalStreamEvent>, FormatConversionError> {
        Ok(Vec::new())
    }
    fn stream_event_from_internal(&self, event: &InternalStreamEvent, state: &mut StreamConversionState) -> Result<Vec<Value>, FormatConversionError>;
}
