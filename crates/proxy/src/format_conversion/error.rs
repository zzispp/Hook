use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FormatConversionError {
    #[error("unsupported format: {0}")]
    InvalidFormat(String),
    #[error("invalid payload for {format}: {path}")]
    InvalidPayload { format: &'static str, path: String },
    #[error("unsupported content in {format}: {detail}")]
    UnsupportedContent { format: &'static str, detail: String },
    #[error("unsupported feature in {format}: {feature}")]
    UnsupportedFeature { format: &'static str, feature: String },
}

impl FormatConversionError {
    pub fn invalid_payload(format: &'static str, path: impl Into<String>) -> Self {
        Self::InvalidPayload { format, path: path.into() }
    }

    pub fn unsupported_content(format: &'static str, detail: impl Into<String>) -> Self {
        Self::UnsupportedContent { format, detail: detail.into() }
    }

    pub fn unsupported_feature(format: &'static str, feature: impl Into<String>) -> Self {
        Self::UnsupportedFeature {
            format,
            feature: feature.into(),
        }
    }
}

impl From<formats::FormatError> for FormatConversionError {
    fn from(error: formats::FormatError) -> Self {
        match error {
            formats::FormatError::UnsupportedFormat(format) => Self::InvalidFormat(format),
            formats::FormatError::RequestParseFailed { format } => Self::unsupported_content("request", format),
            formats::FormatError::RequestEmitFailed { format } => Self::unsupported_content("request", format),
            formats::FormatError::ResponseParseFailed { format } => Self::unsupported_content("response", format),
            formats::FormatError::ResponseEmitFailed { format } => Self::unsupported_content("response", format),
        }
    }
}
