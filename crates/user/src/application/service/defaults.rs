use std::collections::BTreeMap;

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{system_setting::EmailSuffixMode, user::UserWalletSummaryResponse};

use crate::application::{
    AppResult, InitialGrantLedger, PasswordResetConfig, PasswordResetEmail, PasswordResetMailer, PasswordResetTemplate, RegistrationPolicy,
    RegistrationSettings, SystemUserProvider, SystemUserRecord, UserWalletCatalog,
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
            default_user_grant: Decimal::ZERO,
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
    async fn password_reset_settings(&self) -> AppResult<crate::application::PasswordResetSettings> {
        Err(crate::application::AppError::Infrastructure(
            "password reset configuration is not available".into(),
        ))
    }

    async fn password_reset_template(&self, _lang: &str) -> AppResult<PasswordResetTemplate> {
        Err(crate::application::AppError::Infrastructure(
            "password reset template is not available".into(),
        ))
    }
}

#[async_trait]
impl PasswordResetMailer for NoPasswordResetMailer {
    async fn send_password_reset(&self, _email: PasswordResetEmail) -> AppResult<()> {
        Err(crate::application::AppError::Infrastructure(
            "password reset mailer is not available".into(),
        ))
    }
}
