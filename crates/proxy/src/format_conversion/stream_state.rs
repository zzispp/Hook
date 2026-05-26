use formats::api::StreamingStandardFormatMatrix;

#[derive(Default)]
pub struct StreamConversionState {
    pub(super) matrix: StreamingStandardFormatMatrix,
}
