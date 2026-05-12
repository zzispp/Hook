use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use types::captcha::{CaptchaChallengeSpec, CaptchaConfigResponse, CaptchaRedeemPayload, CaptchaRedeemResponse};

use super::CaptchaResult;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChallengeRecord {
    pub challenge: CaptchaChallengeSpec,
    pub expires: i64,
}

#[async_trait]
pub trait CaptchaStore: Send + Sync + 'static {
    async fn save_challenge(&self, token: &str, record: &ChallengeRecord, ttl_seconds: u64) -> CaptchaResult<()>;
    async fn consume_challenge(&self, token: &str) -> CaptchaResult<Option<ChallengeRecord>>;
    async fn save_redeemed(&self, token_key: &str, expires: i64, ttl_seconds: u64) -> CaptchaResult<()>;
    async fn consume_redeemed(&self, token_key: &str) -> CaptchaResult<bool>;
}

#[async_trait]
pub trait CaptchaSettingsReader: Send + Sync + 'static {
    async fn login_captcha_enabled(&self) -> CaptchaResult<bool>;
    async fn registration_captcha_enabled(&self) -> CaptchaResult<bool>;
}

#[async_trait]
pub trait CaptchaUseCase: Send + Sync + 'static {
    async fn config(&self) -> CaptchaResult<CaptchaConfigResponse>;
    async fn challenge(&self) -> CaptchaResult<types::captcha::CaptchaChallengeResponse>;
    async fn redeem(&self, payload: CaptchaRedeemPayload) -> CaptchaResult<CaptchaRedeemResponse>;
    async fn verify_login(&self, token: Option<&str>) -> CaptchaResult<()>;
    async fn verify_registration(&self, token: Option<&str>) -> CaptchaResult<()>;
}
