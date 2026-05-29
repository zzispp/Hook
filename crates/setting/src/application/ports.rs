use async_trait::async_trait;
use types::system_setting::{
    PublicSiteInfoResponse, SystemSettingsResponse, SystemSettingsSmtpTestRequest, SystemSettingsSmtpTestResponse, SystemSettingsUpdate,
};

use super::{SettingResult, SmtpConnectionConfig, StoredSmtpSettings};

#[async_trait]
pub trait SettingRepository: Send + Sync + 'static {
    async fn get_system_settings(&self) -> SettingResult<SystemSettingsResponse>;
    async fn get_smtp_settings(&self) -> SettingResult<StoredSmtpSettings>;
    async fn update_system_settings(
        &self,
        input: SystemSettingsUpdate,
        encrypted_smtp_password: Option<String>,
        encrypted_github_client_secret: Option<String>,
        encrypted_google_client_secret: Option<String>,
    ) -> SettingResult<SystemSettingsResponse>;
}

pub trait SettingSecretCipher: Send + Sync + 'static {
    fn encrypt_secret(&self, plaintext: &str) -> SettingResult<String>;
    fn decrypt_secret(&self, ciphertext: &str) -> SettingResult<String>;
}

#[async_trait]
pub trait SettingUserGroupCatalog: Send + Sync + 'static {
    async fn active_user_group_exists(&self, code: &str) -> SettingResult<bool>;
}

#[async_trait]
pub trait SettingPaymentChannelCatalog: Send + Sync + 'static {
    async fn has_ready_payment_channel(&self) -> SettingResult<bool>;
}

#[async_trait]
pub trait SmtpConnectionTester: Send + Sync + 'static {
    async fn test_connection(&self, config: &SmtpConnectionConfig) -> Result<(), String>;
}

#[async_trait]
pub trait SettingUseCase: Send + Sync + 'static {
    async fn get_system_settings(&self) -> SettingResult<SystemSettingsResponse>;
    async fn get_public_site_info(&self) -> SettingResult<PublicSiteInfoResponse>;
    async fn update_system_settings(&self, input: SystemSettingsUpdate) -> SettingResult<SystemSettingsResponse>;
    async fn test_smtp_connection(&self, input: SystemSettingsSmtpTestRequest) -> SettingResult<SystemSettingsSmtpTestResponse>;
}
