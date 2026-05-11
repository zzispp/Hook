use serde_json::Value;

use super::{FormatConversionError, InternalRequest, InternalResponse, InternalStreamEvent};

pub trait FormatNormalizer {
    fn request_to_internal(&self, request: &Value) -> Result<InternalRequest, FormatConversionError>;
    fn request_from_internal(&self, internal: &InternalRequest) -> Result<Value, FormatConversionError>;

    fn response_to_internal(&self, response: &Value) -> Result<InternalResponse, FormatConversionError>;
    fn response_from_internal(&self, internal: &InternalResponse) -> Result<Value, FormatConversionError>;

    fn stream_to_internal(&self, chunks: &[Value]) -> Result<Vec<InternalStreamEvent>, FormatConversionError>;
    fn stream_from_internal(&self, events: &[InternalStreamEvent]) -> Result<Vec<Value>, FormatConversionError>;
}
