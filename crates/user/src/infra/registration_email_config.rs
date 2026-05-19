use async_trait::async_trait;
use storage::{Database, i18n::I18nStore, setting::SettingStore};
use types::user::AuthConfigResponse;

use crate::application::{AppError, AppResult, EmailSettings, RegistrationEmailConfig, RegistrationEmailTemplate};

const AUTH_NAMESPACE: &str = "auth";
const TEMPLATE_GROUP: &str = "emailTemplates";
const REGISTRATION_SUBJECT_KEY: &str = "registration.subject";
const REGISTRATION_HTML_KEY: &str = "registration.html";

#[derive(Clone)]
pub struct StorageRegistrationEmailConfig {
    settings: SettingStore,
    i18n: I18nStore,
}

impl StorageRegistrationEmailConfig {
    pub fn new(database: Database) -> Self {
        Self {
            settings: SettingStore::new(database.clone()),
            i18n: I18nStore::new(database),
        }
    }
}

#[async_trait]
impl RegistrationEmailConfig for StorageRegistrationEmailConfig {
    async fn auth_config(&self) -> AppResult<AuthConfigResponse> {
        let settings = self.settings.get_system_settings().await.map_err(storage_error)?;
        Ok(AuthConfigResponse {
            allow_registration: settings.allow_registration,
            registration_email_verification_enabled: settings.registration_email_verification_enabled,
        })
    }

    async fn registration_email_settings(&self) -> AppResult<EmailSettings> {
        let settings = self.settings.get_system_settings().await.map_err(storage_error)?;
        Ok(EmailSettings {
            site_name: settings.site_name,
            feature_enabled: settings.registration_email_verification_enabled,
            email_config_enabled: settings.email_config_enabled,
            smtp_host: settings.smtp_host,
            smtp_username: settings.smtp_username,
            smtp_password_set: settings.smtp_password_set,
            smtp_from_email: settings.smtp_from_email,
            smtp_from_name: settings.smtp_from_name,
            smtp_encryption: settings.smtp_encryption,
        })
    }

    async fn registration_email_template(&self, lang: &str) -> AppResult<RegistrationEmailTemplate> {
        let subject = self.template_value(lang, REGISTRATION_SUBJECT_KEY).await?;
        let html = self.template_value(lang, REGISTRATION_HTML_KEY).await?;
        Ok(RegistrationEmailTemplate { subject, html })
    }
}

impl StorageRegistrationEmailConfig {
    async fn template_value(&self, lang: &str, item_key: &str) -> AppResult<String> {
        self.i18n
            .entry_value(lang, AUTH_NAMESPACE, TEMPLATE_GROUP, item_key)
            .await
            .map_err(storage_error)?
            .ok_or_else(|| AppError::InvalidInput(format!("missing auth email template: {lang}.{item_key}")))
    }
}

fn storage_error(error: storage::StorageError) -> AppError {
    match error {
        storage::StorageError::NotFound => AppError::NotFound,
        storage::StorageError::Conflict(message) | storage::StorageError::Database(message) => AppError::Infrastructure(message),
    }
}
