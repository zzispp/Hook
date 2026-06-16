use serde::{Deserialize, Serialize};

const TINY_REQUEST_MAX_TOKENS: u64 = 512;
const SMALL_REQUEST_MAX_TOKENS: u64 = 2_000;
const MEDIUM_REQUEST_MAX_TOKENS: u64 = 8_000;
const LARGE_REQUEST_MAX_TOKENS: u64 = 32_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingRequestSizeBucket {
    #[default]
    Unknown,
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
}

impl RoutingRequestSizeBucket {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Tiny => "tiny",
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
            Self::Huge => "huge",
        }
    }

    pub const fn from_input_tokens(tokens: Option<u64>) -> Self {
        let Some(tokens) = tokens else {
            return Self::Unknown;
        };
        if tokens <= TINY_REQUEST_MAX_TOKENS {
            return Self::Tiny;
        }
        if tokens <= SMALL_REQUEST_MAX_TOKENS {
            return Self::Small;
        }
        if tokens <= MEDIUM_REQUEST_MAX_TOKENS {
            return Self::Medium;
        }
        if tokens <= LARGE_REQUEST_MAX_TOKENS {
            return Self::Large;
        }
        Self::Huge
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct RoutingRequestFeatures {
    pub client_api_format: String,
    pub is_stream: bool,
    pub input_token_estimate: Option<u64>,
    pub output_token_estimate: Option<u64>,
    pub request_size_bucket: RoutingRequestSizeBucket,
    pub required_capability: Option<String>,
}

impl RoutingRequestFeatures {
    pub fn unknown(client_api_format: impl Into<String>, is_stream: bool, required_capability: Option<&str>) -> Self {
        Self {
            client_api_format: client_api_format.into(),
            is_stream,
            input_token_estimate: None,
            output_token_estimate: None,
            request_size_bucket: RoutingRequestSizeBucket::Unknown,
            required_capability: required_capability.map(str::to_owned),
        }
    }

    pub fn new(
        client_api_format: impl Into<String>,
        is_stream: bool,
        input_token_estimate: Option<u64>,
        output_token_estimate: Option<u64>,
        required_capability: Option<&str>,
    ) -> Self {
        Self {
            client_api_format: client_api_format.into(),
            is_stream,
            input_token_estimate,
            output_token_estimate,
            request_size_bucket: RoutingRequestSizeBucket::from_input_tokens(input_token_estimate),
            required_capability: required_capability.map(str::to_owned),
        }
    }
}
