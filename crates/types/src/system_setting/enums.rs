use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DisplayCurrency {
    Usd,
    Cny,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestRecordLevel {
    #[default]
    Basic,
    Headers,
    Full,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SmtpEncryption {
    None,
    #[default]
    Tls,
    Ssl,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailSuffixMode {
    #[default]
    None,
    Whitelist,
    Blacklist,
}

impl RequestRecordLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Headers => "headers",
            Self::Full => "full",
        }
    }
}

impl TryFrom<&str> for RequestRecordLevel {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "basic" => Ok(Self::Basic),
            "headers" => Ok(Self::Headers),
            "full" => Ok(Self::Full),
            _ => Err(format!("unsupported request_record_level: {value}")),
        }
    }
}

impl SmtpEncryption {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Tls => "tls",
            Self::Ssl => "ssl",
        }
    }
}

impl TryFrom<&str> for SmtpEncryption {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "none" => Ok(Self::None),
            "tls" => Ok(Self::Tls),
            "ssl" => Ok(Self::Ssl),
            _ => Err(format!("unsupported smtp_encryption: {value}")),
        }
    }
}

impl EmailSuffixMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Whitelist => "whitelist",
            Self::Blacklist => "blacklist",
        }
    }
}

impl TryFrom<&str> for EmailSuffixMode {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "none" => Ok(Self::None),
            "whitelist" => Ok(Self::Whitelist),
            "blacklist" => Ok(Self::Blacklist),
            _ => Err(format!("unsupported email_suffix_mode: {value}")),
        }
    }
}

impl DisplayCurrency {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Usd => "USD",
            Self::Cny => "CNY",
        }
    }
}

impl From<&str> for DisplayCurrency {
    fn from(value: &str) -> Self {
        match value {
            "CNY" => Self::Cny,
            _ => Self::Usd,
        }
    }
}
