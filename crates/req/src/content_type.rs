use std::str::FromStr;

pub const CONTENT_TYPE: &str = "Content-Type";
const APPLICATION_JSON: &str = "application/json";
const TEXT_PLAIN: &str = "text/plain";
const APPLICATION_FORM_URL_ENCODED: &str = "application/x-www-form-urlencoded";
const APPLICATION_X_BINARY: &str = "application/x-binary";

#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    ApplicationJson,
    TextPlain,
    ApplicationFormUrlEncoded,
    ApplicationXBinary,
}

impl ContentType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ApplicationJson => APPLICATION_JSON,
            Self::TextPlain => TEXT_PLAIN,
            Self::ApplicationFormUrlEncoded => APPLICATION_FORM_URL_ENCODED,
            Self::ApplicationXBinary => APPLICATION_X_BINARY,
        }
    }
}

impl FromStr for ContentType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            APPLICATION_JSON => Ok(Self::ApplicationJson),
            TEXT_PLAIN => Ok(Self::TextPlain),
            APPLICATION_FORM_URL_ENCODED => Ok(Self::ApplicationFormUrlEncoded),
            APPLICATION_X_BINARY => Ok(Self::ApplicationXBinary),
            _ => Err("Unknown content type"),
        }
    }
}
