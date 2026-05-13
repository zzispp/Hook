use serde::{Deserialize, Serialize};

use super::SmtpEncryption;

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct SystemSettingsSmtpTestRequest {
    #[serde(default)]
    pub smtp_host: Option<String>,
    #[serde(default)]
    pub smtp_port: Option<i64>,
    #[serde(default)]
    pub smtp_username: Option<String>,
    #[serde(default)]
    pub smtp_password: Option<String>,
    #[serde(default)]
    pub smtp_from_email: Option<String>,
    #[serde(default)]
    pub smtp_from_name: Option<String>,
    #[serde(default)]
    pub smtp_encryption: Option<SmtpEncryption>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SystemSettingsSmtpTestResponse {
    pub success: bool,
    pub message: String,
}

impl SystemSettingsSmtpTestResponse {
    pub fn succeeded(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }

    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
        }
    }
}
