use async_trait::async_trait;
use types::system_setting::{
    PublicSiteInfoResponse, SystemSettingsResponse, SystemSettingsSmtpTestRequest, SystemSettingsSmtpTestResponse, SystemSettingsUpdate,
};

use crate::application::{SettingError, SettingRepository, SettingResult, SettingSecretCipher, SettingUseCase, SettingUserGroupCatalog, SmtpConnectionTester};

use super::{
    email_config::validate_email_feature_prerequisites,
    smtp::{failure_response, sanitize_smtp_test_request, smtp_connection_config, success_response},
    validation::{sanitize_update, validate_recharge_bounds, validate_update},
};

pub struct SettingService<R, C, T, U = NoSettingUserGroupCatalog> {
    repository: R,
    cipher: C,
    smtp_tester: T,
    user_groups: U,
}

impl<R, C, T> SettingService<R, C, T, NoSettingUserGroupCatalog>
where
    R: SettingRepository,
    C: SettingSecretCipher,
    T: SmtpConnectionTester,
{
    pub const fn new(repository: R, cipher: C, smtp_tester: T) -> Self {
        Self {
            repository,
            cipher,
            smtp_tester,
            user_groups: NoSettingUserGroupCatalog,
        }
    }
}

#[async_trait]
impl<R, C, T, U> SettingUseCase for SettingService<R, C, T, U>
where
    R: SettingRepository,
    C: SettingSecretCipher,
    T: SmtpConnectionTester,
    U: SettingUserGroupCatalog,
{
    async fn get_system_settings(&self) -> SettingResult<SystemSettingsResponse> {
        self.repository.get_system_settings().await
    }

    async fn get_public_site_info(&self) -> SettingResult<PublicSiteInfoResponse> {
        self.repository.get_system_settings().await.map(Into::into)
    }

    async fn update_system_settings(&self, input: SystemSettingsUpdate) -> SettingResult<SystemSettingsResponse> {
        let input = sanitize_update(input);
        validate_update(&input)?;
        let current = self.repository.get_system_settings().await?;
        validate_recharge_bounds(&input, &current)?;
        validate_email_feature_prerequisites(&input, &current)?;
        validate_default_user_group(&self.user_groups, input.default_user_group_code.as_deref()).await?;
        let encrypted_smtp_password = input
            .smtp_password
            .as_deref()
            .map(|password| self.cipher.encrypt_secret(password))
            .transpose()?;
        self.repository.update_system_settings(input, encrypted_smtp_password).await
    }

    async fn test_smtp_connection(&self, input: SystemSettingsSmtpTestRequest) -> SettingResult<SystemSettingsSmtpTestResponse> {
        let input = sanitize_smtp_test_request(input);
        let stored = self.repository.get_smtp_settings().await?;
        let config = match smtp_connection_config(input, stored, &self.cipher)? {
            Ok(config) => config,
            Err(message) => return Ok(failure_response(message)),
        };
        match self.smtp_tester.test_connection(&config).await {
            Ok(()) => Ok(success_response()),
            Err(message) => Ok(failure_response(message)),
        }
    }
}

impl<R, C, T, U> SettingService<R, C, T, U> {
    pub fn with_user_group_catalog<NU>(self, user_groups: NU) -> SettingService<R, C, T, NU> {
        SettingService {
            repository: self.repository,
            cipher: self.cipher,
            smtp_tester: self.smtp_tester,
            user_groups,
        }
    }
}

#[derive(Clone, Copy)]
pub struct NoSettingUserGroupCatalog;

#[async_trait]
impl SettingUserGroupCatalog for NoSettingUserGroupCatalog {
    async fn active_user_group_exists(&self, _code: &str) -> SettingResult<bool> {
        Err(SettingError::Infrastructure("setting user group catalog is not available".into()))
    }
}

async fn validate_default_user_group<U>(user_groups: &U, code: Option<&str>) -> SettingResult<()>
where
    U: SettingUserGroupCatalog,
{
    let Some(code) = code else {
        return Ok(());
    };
    if user_groups.active_user_group_exists(code).await? {
        return Ok(());
    }
    Err(SettingError::InvalidInput(format!("active user group does not exist: {code}")))
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
