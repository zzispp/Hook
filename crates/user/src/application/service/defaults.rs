use std::collections::BTreeMap;

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{system_setting::EmailSuffixMode, user::UserWalletSummaryResponse};

use crate::application::{
    AppResult, InitialGrantLedger, PasswordResetConfig, PasswordResetEmail, PasswordResetMailer, PasswordResetTemplate, RegistrationEmail,
    RegistrationEmailCodeStore, RegistrationEmailConfig, RegistrationEmailMailer, RegistrationEmailTemplate, RegistrationPolicy, RegistrationSettings,
    SystemUserProvider, SystemUserRecord, UserWalletCatalog,
};

#[derive(Clone, Copy)]
pub struct NoSystemUserProvider;

#[derive(Clone, Copy)]
pub struct AllowRegistrationPolicy;

#[derive(Clone, Copy)]
pub struct NoInitialGrantLedger;

#[derive(Clone, Copy)]
pub struct NoUserWalletCatalog;

#[derive(Clone, Copy)]
pub struct NoPasswordResetConfig;

#[derive(Clone, Copy)]
pub struct NoPasswordResetMailer;

#[derive(Clone, Copy)]
pub struct NoRegistrationEmailConfig;

#[derive(Clone, Copy)]
pub struct NoRegistrationEmailMailer;

#[derive(Clone, Copy)]
pub struct NoRegistrationEmailCodeStore;

impl SystemUserProvider for NoSystemUserProvider {
    fn system_user(&self) -> Option<SystemUserRecord> {
        None
    }
}

#[async_trait]
impl RegistrationPolicy for AllowRegistrationPolicy {
    async fn registration_settings(&self) -> AppResult<RegistrationSettings> {
        Ok(RegistrationSettings {
            allow_registration: true,
            registration_email_verification_enabled: false,
            default_user_grant: Decimal::ZERO,
            default_user_group_code: constants::user_group::DEFAULT_USER_GROUP_CODE.into(),
            email_suffix_mode: EmailSuffixMode::None,
            email_suffixes: String::new(),
        })
    }
}

#[async_trait]
impl InitialGrantLedger for NoInitialGrantLedger {
    async fn grant_initial_balance(&self, _user_id: &str, _amount: Decimal) -> AppResult<()> {
        Ok(())
    }
}

#[async_trait]
impl UserWalletCatalog for NoUserWalletCatalog {
    async fn wallet_summaries(&self, _user_ids: &[String]) -> AppResult<BTreeMap<String, UserWalletSummaryResponse>> {
        Ok(BTreeMap::new())
    }
}

#[async_trait]
impl PasswordResetConfig for NoPasswordResetConfig {
    async fn password_reset_settings(&self) -> AppResult<crate::application::EmailSettings> {
        Err(crate::application::AppError::Infrastructure(
            "password reset configuration is not available".into(),
        ))
    }

    async fn password_reset_template(&self, _lang: &str) -> AppResult<PasswordResetTemplate> {
        Err(crate::application::AppError::Infrastructure("password reset template is not available".into()))
    }
}

#[async_trait]
impl PasswordResetMailer for NoPasswordResetMailer {
    async fn send_password_reset(&self, _email: PasswordResetEmail) -> AppResult<()> {
        Err(crate::application::AppError::Infrastructure("password reset mailer is not available".into()))
    }
}

#[async_trait]
impl RegistrationEmailConfig for NoRegistrationEmailConfig {
    async fn auth_config(&self) -> AppResult<types::user::AuthConfigResponse> {
        Ok(types::user::AuthConfigResponse {
            allow_registration: true,
            registration_email_verification_enabled: false,
        })
    }

    async fn registration_email_settings(&self) -> AppResult<crate::application::EmailSettings> {
        Err(crate::application::AppError::Infrastructure(
            "registration email configuration is not available".into(),
        ))
    }

    async fn registration_email_template(&self, _lang: &str) -> AppResult<RegistrationEmailTemplate> {
        Err(crate::application::AppError::Infrastructure(
            "registration email template is not available".into(),
        ))
    }
}

#[async_trait]
impl RegistrationEmailMailer for NoRegistrationEmailMailer {
    async fn send_registration_email(&self, _email: RegistrationEmail) -> AppResult<()> {
        Err(crate::application::AppError::Infrastructure(
            "registration email mailer is not available".into(),
        ))
    }
}

#[async_trait]
impl RegistrationEmailCodeStore for NoRegistrationEmailCodeStore {
    async fn active_registration_email_code(&self, _email: &str) -> AppResult<Option<String>> {
        Err(crate::application::AppError::Infrastructure(
            "registration email code store is not available".into(),
        ))
    }

    async fn save_registration_email_code(&self, _email: &str, _code: &str, _ttl_seconds: u64) -> AppResult<()> {
        Err(crate::application::AppError::Infrastructure(
            "registration email code store is not available".into(),
        ))
    }

    async fn begin_registration_email_code_cooldown(&self, _email: &str, _ttl_seconds: u64) -> AppResult<bool> {
        Err(crate::application::AppError::Infrastructure(
            "registration email code store is not available".into(),
        ))
    }

    async fn consume_registration_email_code(&self, _email: &str, _code: &str) -> AppResult<bool> {
        Err(crate::application::AppError::Infrastructure(
            "registration email code store is not available".into(),
        ))
    }
}
