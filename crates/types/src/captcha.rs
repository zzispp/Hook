use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CaptchaConfigResponse {
    pub login_captcha_enabled: bool,
    pub registration_captcha_enabled: bool,
    pub support_ticket_captcha_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptchaChallengeSpec {
    pub c: usize,
    pub s: usize,
    pub d: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CaptchaChallengeResponse {
    pub challenge: CaptchaChallengeSpec,
    pub token: String,
    pub expires: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CaptchaRedeemPayload {
    pub token: String,
    #[serde(default)]
    pub solutions: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CaptchaRedeemResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl CaptchaRedeemResponse {
    pub fn success(token: String, expires: i64) -> Self {
        Self {
            success: true,
            token: Some(token),
            expires: Some(expires),
            reason: None,
            error: None,
        }
    }

    pub fn failure(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self {
            success: false,
            token: None,
            expires: None,
            reason: Some(reason.clone()),
            error: Some(reason),
        }
    }
}
