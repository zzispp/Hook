use std::fmt;

#[derive(Debug)]
pub enum ClientError {
    Network(String),
    Timeout,
    Http { status: u16, body: String },
    Serialization(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(message) => write!(f, "Network error: {message}"),
            Self::Timeout => f.write_str("Timeout error"),
            Self::Http { status, body } => write!(f, "HTTP error: status {status}, body: {body}"),
            Self::Serialization(message) => write!(f, "Serialization error: {message}"),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<serde_json::Error> for ClientError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(format!("JSON error: {error}"))
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            Self::Timeout
        } else if error.is_connect() {
            Self::Network(format!("Connection error: {error}"))
        } else {
            Self::Network(error.to_string())
        }
    }
}
