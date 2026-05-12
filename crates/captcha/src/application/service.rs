use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use types::captcha::{CaptchaChallengeResponse, CaptchaChallengeSpec, CaptchaConfigResponse, CaptchaRedeemPayload, CaptchaRedeemResponse};

use crate::application::{CaptchaError, CaptchaResult, CaptchaSettingsReader, CaptchaStore, CaptchaUseCase, ChallengeRecord};

use super::pow::solution_matches;

const CHALLENGE_COUNT: usize = 50;
const CHALLENGE_SIZE: usize = 32;
const CHALLENGE_DIFFICULTY: usize = 4;
const CHALLENGE_TTL_SECONDS: u64 = 10 * 60;
const REDEEMED_TOKEN_TTL_SECONDS: u64 = 20 * 60;
const CHALLENGE_TOKEN_BYTES: usize = 25;
const REDEEMED_ID_BYTES: usize = 8;
const REDEEMED_SECRET_BYTES: usize = 15;
const MILLIS_PER_SECOND: i64 = 1_000;

pub struct CaptchaService<S, R> {
    settings: S,
    store: R,
}

impl<S, R> CaptchaService<S, R>
where
    S: CaptchaSettingsReader,
    R: CaptchaStore,
{
    pub const fn new(settings: S, store: R) -> Self {
        Self { settings, store }
    }
}

#[async_trait]
impl<S, R> CaptchaUseCase for CaptchaService<S, R>
where
    S: CaptchaSettingsReader,
    R: CaptchaStore,
{
    async fn config(&self) -> CaptchaResult<CaptchaConfigResponse> {
        Ok(CaptchaConfigResponse {
            login_captcha_enabled: self.settings.login_captcha_enabled().await?,
            registration_captcha_enabled: self.settings.registration_captcha_enabled().await?,
        })
    }

    async fn challenge(&self) -> CaptchaResult<CaptchaChallengeResponse> {
        let challenge = default_challenge();
        let token = random_hex(CHALLENGE_TOKEN_BYTES);
        let expires = expires_at(CHALLENGE_TTL_SECONDS);
        let record = ChallengeRecord {
            challenge: challenge.clone(),
            expires,
        };
        self.store.save_challenge(&token, &record, CHALLENGE_TTL_SECONDS).await?;
        Ok(CaptchaChallengeResponse { challenge, token, expires })
    }

    async fn redeem(&self, payload: CaptchaRedeemPayload) -> CaptchaResult<CaptchaRedeemResponse> {
        let Some(record) = self.store.consume_challenge(&payload.token).await? else {
            return Ok(CaptchaRedeemResponse::failure("invalid_or_expired_challenge"));
        };
        if !solutions_match(&payload.token, &record.challenge, &payload.solutions) {
            return Ok(CaptchaRedeemResponse::failure("invalid_solution"));
        }
        let token = redeemed_token();
        let key = redeemed_token_key(&token)?;
        let expires = expires_at(REDEEMED_TOKEN_TTL_SECONDS);
        self.store.save_redeemed(&key, expires, REDEEMED_TOKEN_TTL_SECONDS).await?;
        Ok(CaptchaRedeemResponse::success(token, expires))
    }

    async fn verify_login(&self, token: Option<&str>) -> CaptchaResult<()> {
        self.verify_if_enabled(self.settings.login_captcha_enabled().await?, token).await
    }

    async fn verify_registration(&self, token: Option<&str>) -> CaptchaResult<()> {
        self.verify_if_enabled(self.settings.registration_captcha_enabled().await?, token).await
    }
}

impl<S, R> CaptchaService<S, R>
where
    S: CaptchaSettingsReader,
    R: CaptchaStore,
{
    async fn verify_if_enabled(&self, enabled: bool, token: Option<&str>) -> CaptchaResult<()> {
        if !enabled {
            return Ok(());
        }
        let token = token.filter(|value| !value.trim().is_empty()).ok_or_else(required_error)?;
        let key = redeemed_token_key(token)?;
        if self.store.consume_redeemed(&key).await? {
            return Ok(());
        }
        Err(CaptchaError::InvalidInput("captcha verification failed".into()))
    }
}

fn default_challenge() -> CaptchaChallengeSpec {
    CaptchaChallengeSpec {
        c: CHALLENGE_COUNT,
        s: CHALLENGE_SIZE,
        d: CHALLENGE_DIFFICULTY,
    }
}

fn solutions_match(token: &str, challenge: &CaptchaChallengeSpec, solutions: &[u64]) -> bool {
    solutions.len() == challenge.c
        && solutions
            .iter()
            .enumerate()
            .all(|(index, solution)| solution_matches(token, index + 1, challenge, *solution))
}

fn redeemed_token() -> String {
    format!("{}:{}", random_hex(REDEEMED_ID_BYTES), random_hex(REDEEMED_SECRET_BYTES))
}

fn redeemed_token_key(token: &str) -> CaptchaResult<String> {
    let Some((id, secret)) = token.split_once(':') else {
        return Err(CaptchaError::InvalidInput("captcha token is invalid".into()));
    };
    if id.is_empty() || secret.is_empty() {
        return Err(CaptchaError::InvalidInput("captcha token is invalid".into()));
    }
    let hash = Sha256::digest(secret.as_bytes());
    Ok(format!("{id}:{}", hex::encode(hash)))
}

fn random_hex(bytes: usize) -> String {
    let mut buffer = vec![0_u8; bytes];
    OsRng.fill_bytes(&mut buffer);
    hex::encode(buffer)
}

fn expires_at(ttl_seconds: u64) -> i64 {
    now_ms() + i64::try_from(ttl_seconds).expect("captcha TTL seconds must fit i64") * MILLIS_PER_SECOND
}

fn now_ms() -> i64 {
    let elapsed = SystemTime::now().duration_since(UNIX_EPOCH).expect("system clock must be after Unix epoch");
    i64::try_from(elapsed.as_millis()).expect("current timestamp must fit i64")
}

fn required_error() -> CaptchaError {
    CaptchaError::InvalidInput("captcha verification is required".into())
}
