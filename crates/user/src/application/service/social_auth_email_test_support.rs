use async_trait::async_trait;
use rust_decimal::Decimal;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};
use types::{
    system_setting::{EmailSuffixMode, SmtpEncryption},
    user::OAuthProviderPublicConfig,
};

use crate::application::{
    AppResult, EmailSettings, PurposeEmailCodeStore, RegistrationEmail, RegistrationEmailConfig, RegistrationEmailMailer, RegistrationEmailTemplate,
    RegistrationPolicy, RegistrationSettings,
};

#[derive(Clone, Default)]
pub(super) struct TestPurposeEmailCodeStore {
    state: Arc<Mutex<EmailCodeState>>,
}

#[derive(Default)]
struct EmailCodeState {
    codes: BTreeMap<(String, String), String>,
    cooldowns: BTreeSet<(String, String)>,
}

#[derive(Clone, Default)]
pub(super) struct TestMailer {
    sent: Arc<Mutex<Vec<RegistrationEmail>>>,
}

#[derive(Clone)]
pub(super) struct TestEmailConfig;

#[derive(Clone)]
pub(super) struct TestRegistrationPolicy;

#[async_trait]
impl PurposeEmailCodeStore for TestPurposeEmailCodeStore {
    async fn active_email_code(&self, purpose: &str, email: &str) -> AppResult<Option<String>> {
        Ok(self.state.lock().unwrap().codes.get(&(purpose.into(), email.into())).cloned())
    }

    async fn save_email_code(&self, purpose: &str, email: &str, code: &str, _ttl_seconds: u64) -> AppResult<()> {
        self.state.lock().unwrap().codes.insert((purpose.into(), email.into()), code.into());
        Ok(())
    }

    async fn begin_email_code_cooldown(&self, purpose: &str, email: &str, _ttl_seconds: u64) -> AppResult<bool> {
        Ok(self.state.lock().unwrap().cooldowns.insert((purpose.into(), email.into())))
    }

    async fn consume_email_code(&self, purpose: &str, email: &str, code: &str) -> AppResult<bool> {
        let key = (purpose.to_owned(), email.to_owned());
        let mut state = self.state.lock().unwrap();
        if state.codes.get(&key).is_some_and(|active| active == code) {
            state.codes.remove(&key);
            return Ok(true);
        }
        Ok(false)
    }
}

impl TestPurposeEmailCodeStore {
    pub(super) fn saved_code(&self, purpose: &str, email: &str) -> String {
        self.state.lock().unwrap().codes.get(&(purpose.into(), email.into())).cloned().unwrap()
    }

    pub(super) fn seed_code(&self, purpose: &str, email: &str, code: &str) {
        self.state.lock().unwrap().codes.insert((purpose.into(), email.into()), code.into());
    }
}

#[async_trait]
impl RegistrationEmailConfig for TestEmailConfig {
    async fn auth_config(&self) -> AppResult<types::user::AuthConfigResponse> {
        Ok(types::user::AuthConfigResponse {
            allow_registration: true,
            registration_email_verification_enabled: true,
            providers: types::user::AuthProviderConfigResponse {
                github: OAuthProviderPublicConfig { enabled: true },
                ..Default::default()
            },
        })
    }

    async fn registration_email_settings(&self) -> AppResult<EmailSettings> {
        Ok(EmailSettings {
            site_name: "Hook".into(),
            feature_enabled: true,
            email_config_enabled: true,
            smtp_host: "smtp.example.com".into(),
            smtp_username: "smtp-user".into(),
            smtp_password_set: true,
            smtp_from_email: "noreply@example.com".into(),
            smtp_from_name: "Hook".into(),
            smtp_encryption: SmtpEncryption::Tls,
        })
    }

    async fn registration_email_template(&self, _lang: &str) -> AppResult<RegistrationEmailTemplate> {
        Ok(RegistrationEmailTemplate {
            subject: "Your code".into(),
            html: "Code {{code}}".into(),
        })
    }
}

#[async_trait]
impl RegistrationEmailMailer for TestMailer {
    async fn send_registration_email(&self, email: RegistrationEmail) -> AppResult<()> {
        self.sent.lock().unwrap().push(email);
        Ok(())
    }
}

#[async_trait]
impl RegistrationPolicy for TestRegistrationPolicy {
    async fn registration_settings(&self) -> AppResult<RegistrationSettings> {
        Ok(RegistrationSettings {
            allow_registration: true,
            registration_email_verification_enabled: true,
            default_user_grant: Decimal::ZERO,
            default_user_group_code: constants::user_group::DEFAULT_USER_GROUP_CODE.into(),
            email_suffix_mode: EmailSuffixMode::None,
            email_suffixes: String::new(),
        })
    }
}
