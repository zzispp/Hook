use async_trait::async_trait;
use rust_decimal::Decimal;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};
use types::{
    system_setting::{EmailSuffixMode, SmtpEncryption},
    user::RegistrationEmailCodeRequest,
};

use super::{NoInitialGrantLedger, NoSystemUserProvider, NoUserWalletCatalog};
use crate::{
    application::{
        AppError, AppResult, EmailSettings, RegistrationEmail, RegistrationEmailCodeStore, RegistrationEmailConfig, RegistrationEmailMailer,
        RegistrationEmailTemplate, RegistrationPolicy, RegistrationSettings, UserService,
    },
    test_support::{MemoryUserRepository, TestPasswordHasher},
};

type TestRegistrationEmailService = UserService<
    MemoryUserRepository,
    TestPasswordHasher,
    NoSystemUserProvider,
    TestRegistrationPolicy,
    NoInitialGrantLedger,
    NoUserWalletCatalog,
    crate::application::service::NoPasswordResetConfig,
    crate::application::service::NoPasswordResetMailer,
    TestRegistrationEmailConfig,
    TestRegistrationEmailMailer,
    TestRegistrationEmailCodeStore,
>;

#[derive(Clone)]
pub(super) struct TestRegistrationPolicy {
    settings: RegistrationSettings,
}

#[derive(Clone, Default)]
pub(super) struct TestRegistrationEmailCodeStore {
    state: Arc<Mutex<CodeStoreState>>,
}

#[derive(Clone, Default)]
pub(super) struct TestRegistrationEmailMailer {
    sent: Arc<Mutex<Vec<RegistrationEmail>>>,
}

#[derive(Clone)]
pub(super) struct TestRegistrationEmailConfig {
    settings: EmailSettings,
    template: RegistrationEmailTemplate,
}

#[derive(Default)]
struct CodeStoreState {
    codes: BTreeMap<String, String>,
    cooldown_emails: BTreeSet<String>,
    saves: Vec<SavedCode>,
    cooldowns: Vec<SavedCooldown>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SavedCode {
    pub(super) email: String,
    pub(super) code: String,
    pub(super) ttl_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SavedCooldown {
    pub(super) email: String,
    pub(super) ttl_seconds: u64,
}

pub(super) fn service_with_registration_email(store: TestRegistrationEmailCodeStore, mailer: TestRegistrationEmailMailer) -> TestRegistrationEmailService {
    UserService::with_system_user_and_registration(
        MemoryUserRepository::default(),
        TestPasswordHasher,
        NoSystemUserProvider,
        test_registration_policy(),
        NoInitialGrantLedger,
        NoUserWalletCatalog,
    )
    .with_registration_email(test_email_config(), mailer, store)
}

pub(super) fn service_with_registration_email_repository(
    repository: MemoryUserRepository,
    store: TestRegistrationEmailCodeStore,
) -> TestRegistrationEmailService {
    UserService::with_system_user_and_registration(
        repository,
        TestPasswordHasher,
        NoSystemUserProvider,
        test_registration_policy(),
        NoInitialGrantLedger,
        NoUserWalletCatalog,
    )
    .with_registration_email(test_email_config(), TestRegistrationEmailMailer::default(), store)
}

pub(super) fn email_code_request(email: &str) -> RegistrationEmailCodeRequest {
    RegistrationEmailCodeRequest {
        email: email.into(),
        lang: "en".into(),
    }
}

pub(super) fn assert_invalid_input<T>(result: Result<T, AppError>, expected: &str) {
    match result {
        Err(AppError::InvalidInput(message)) => assert_eq!(message, expected),
        Err(error) => panic!("expected invalid input error, got {error:?}"),
        Ok(_) => panic!("expected invalid input error, got ok"),
    }
}

fn test_registration_policy() -> TestRegistrationPolicy {
    TestRegistrationPolicy {
        settings: RegistrationSettings {
            allow_registration: true,
            registration_email_verification_enabled: true,
            default_user_grant: Decimal::ZERO,
            default_user_group_code: constants::user_group::DEFAULT_USER_GROUP_CODE.into(),
            email_suffix_mode: EmailSuffixMode::None,
            email_suffixes: String::new(),
        },
    }
}

fn test_email_config() -> TestRegistrationEmailConfig {
    TestRegistrationEmailConfig {
        settings: EmailSettings {
            site_name: "Hook".into(),
            feature_enabled: true,
            email_config_enabled: true,
            smtp_host: "smtp.example.com".into(),
            smtp_username: "smtp-user".into(),
            smtp_password_set: true,
            smtp_from_email: "noreply@example.com".into(),
            smtp_from_name: "Hook".into(),
            smtp_encryption: SmtpEncryption::Tls,
        },
        template: RegistrationEmailTemplate {
            subject: "Your code".into(),
            html: "Code {{code}} expires in {{expire_minutes}} minutes".into(),
        },
    }
}

#[async_trait]
impl RegistrationPolicy for TestRegistrationPolicy {
    async fn registration_settings(&self) -> AppResult<RegistrationSettings> {
        Ok(self.settings.clone())
    }
}

#[async_trait]
impl RegistrationEmailCodeStore for TestRegistrationEmailCodeStore {
    async fn active_registration_email_code(&self, email: &str) -> AppResult<Option<String>> {
        Ok(self.state.lock().unwrap().codes.get(email).cloned())
    }

    async fn save_registration_email_code(&self, email: &str, code: &str, ttl_seconds: u64) -> AppResult<()> {
        let mut state = self.state.lock().unwrap();
        state.codes.insert(email.into(), code.into());
        state.saves.push(SavedCode {
            email: email.into(),
            code: code.into(),
            ttl_seconds,
        });
        Ok(())
    }

    async fn begin_registration_email_code_cooldown(&self, email: &str, ttl_seconds: u64) -> AppResult<bool> {
        let mut state = self.state.lock().unwrap();
        let inserted = state.cooldown_emails.insert(email.into());
        if inserted {
            state.cooldowns.push(SavedCooldown {
                email: email.into(),
                ttl_seconds,
            });
        }
        Ok(inserted)
    }

    async fn consume_registration_email_code(&self, email: &str, code: &str) -> AppResult<bool> {
        let mut state = self.state.lock().unwrap();
        if state.codes.get(email).is_some_and(|active_code| active_code == code) {
            state.codes.remove(email);
            return Ok(true);
        }
        Ok(false)
    }
}

impl TestRegistrationEmailCodeStore {
    pub(super) fn clear_cooldown(&self, email: &str) {
        self.state.lock().unwrap().cooldown_emails.remove(email);
    }

    pub(super) fn saved_codes(&self) -> Vec<SavedCode> {
        self.state.lock().unwrap().saves.clone()
    }

    pub(super) fn saved_cooldowns(&self) -> Vec<SavedCooldown> {
        self.state.lock().unwrap().cooldowns.clone()
    }

    pub(super) fn seed_code(&self, email: &str, code: &str) {
        self.state.lock().unwrap().codes.insert(email.into(), code.into());
    }
}

#[async_trait]
impl RegistrationEmailMailer for TestRegistrationEmailMailer {
    async fn send_registration_email(&self, email: RegistrationEmail) -> AppResult<()> {
        self.sent.lock().unwrap().push(email);
        Ok(())
    }
}

impl TestRegistrationEmailMailer {
    pub(super) fn sent(&self) -> Vec<RegistrationEmail> {
        self.sent.lock().unwrap().clone()
    }
}

#[async_trait]
impl RegistrationEmailConfig for TestRegistrationEmailConfig {
    async fn auth_config(&self) -> AppResult<types::user::AuthConfigResponse> {
        Ok(types::user::AuthConfigResponse {
            allow_registration: true,
            registration_email_verification_enabled: true,
            email_verification_available: true,
            providers: types::user::AuthProviderConfigResponse::default(),
        })
    }

    async fn registration_email_settings(&self) -> AppResult<EmailSettings> {
        Ok(self.settings.clone())
    }

    async fn registration_email_template(&self, _lang: &str) -> AppResult<RegistrationEmailTemplate> {
        Ok(self.template.clone())
    }
}
