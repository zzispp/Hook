use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use sha2::{Digest, Sha256};

use super::{CaptchaResult, CaptchaService, CaptchaSettingsReader, CaptchaStore, CaptchaUseCase, ChallengeRecord};

#[tokio::test]
async fn config_includes_recharge_captcha_flag() {
    let service = service(settings(true), store());

    let config = service.config().await.expect("config must load");

    assert_eq!(config.recharge_captcha_enabled, true);
}

#[tokio::test]
async fn verify_recharge_allows_missing_token_when_disabled() {
    let service = service(settings(false), store());

    let result = service.verify_recharge(None).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn verify_recharge_requires_token_when_enabled() {
    let service = service(settings(true), store());

    let error = service.verify_recharge(None).await.expect_err("missing token must fail");

    assert_eq!(error.to_string(), "captcha verification is required");
}

#[tokio::test]
async fn verify_recharge_consumes_redeemed_token_once() {
    let token = "redeemed-id:redeemed-secret";
    let store = store_with_redeemed(token);
    let service = service(settings(true), store);

    service.verify_recharge(Some(token)).await.expect("first token use must pass");
    let error = service.verify_recharge(Some(token)).await.expect_err("second token use must fail");

    assert_eq!(error.to_string(), "captcha verification failed");
}

fn service(settings: TestSettings, store: TestStore) -> CaptchaService<TestSettings, TestStore> {
    CaptchaService::new(settings, store)
}

fn settings(recharge_enabled: bool) -> TestSettings {
    TestSettings { recharge_enabled }
}

fn store() -> TestStore {
    TestStore::default()
}

fn store_with_redeemed(token: &str) -> TestStore {
    let store = store();
    store.insert_redeemed(token);
    store
}

#[derive(Clone)]
struct TestSettings {
    recharge_enabled: bool,
}

#[async_trait]
impl CaptchaSettingsReader for TestSettings {
    async fn login_captcha_enabled(&self) -> CaptchaResult<bool> {
        Ok(false)
    }

    async fn registration_captcha_enabled(&self) -> CaptchaResult<bool> {
        Ok(false)
    }

    async fn support_ticket_captcha_enabled(&self) -> CaptchaResult<bool> {
        Ok(false)
    }

    async fn recharge_captcha_enabled(&self) -> CaptchaResult<bool> {
        Ok(self.recharge_enabled)
    }
}

#[derive(Clone, Default)]
struct TestStore {
    redeemed: Arc<Mutex<HashSet<String>>>,
}

impl TestStore {
    fn insert_redeemed(&self, token: &str) {
        let mut redeemed = self.redeemed.lock().expect("test store lock must not be poisoned");
        redeemed.insert(redeemed_key(token));
    }
}

#[async_trait]
impl CaptchaStore for TestStore {
    async fn save_challenge(&self, _token: &str, _record: &ChallengeRecord, _ttl_seconds: u64) -> CaptchaResult<()> {
        unimplemented!("recharge verification tests do not save challenges")
    }

    async fn consume_challenge(&self, _token: &str) -> CaptchaResult<Option<ChallengeRecord>> {
        unimplemented!("recharge verification tests do not consume challenges")
    }

    async fn save_redeemed(&self, token_key: &str, _expires: i64, _ttl_seconds: u64) -> CaptchaResult<()> {
        let mut redeemed = self.redeemed.lock().expect("test store lock must not be poisoned");
        redeemed.insert(token_key.into());
        Ok(())
    }

    async fn consume_redeemed(&self, token_key: &str) -> CaptchaResult<bool> {
        let mut redeemed = self.redeemed.lock().expect("test store lock must not be poisoned");
        Ok(redeemed.remove(token_key))
    }
}

fn redeemed_key(token: &str) -> String {
    let (id, secret) = token.split_once(':').expect("test token must include id and secret");
    let hash = Sha256::digest(secret.as_bytes());
    format!("{id}:{}", hex::encode(hash))
}
