use async_trait::async_trait;
use types::system_setting::{
    PublicSiteInfoResponse, SystemSettingsResponse, SystemSettingsSmtpTestRequest, SystemSettingsSmtpTestResponse, SystemSettingsUpdate,
};

use crate::application::{SettingRepository, SettingResult, SettingSecretCipher, SettingUseCase, SmtpConnectionTester};

use super::{
    email_config::validate_email_feature_prerequisites,
    smtp::{failure_response, sanitize_smtp_test_request, smtp_connection_config, success_response},
    validation::{sanitize_update, validate_update},
};

pub struct SettingService<R, C, T> {
    repository: R,
    cipher: C,
    smtp_tester: T,
}

impl<R, C, T> SettingService<R, C, T>
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
        }
    }
}

#[async_trait]
impl<R, C, T> SettingUseCase for SettingService<R, C, T>
where
    R: SettingRepository,
    C: SettingSecretCipher,
    T: SmtpConnectionTester,
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
        validate_email_feature_prerequisites(&input, &current)?;
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

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
