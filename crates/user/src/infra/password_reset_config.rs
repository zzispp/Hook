use async_trait::async_trait;
use storage::{Database, i18n::I18nStore, setting::SettingStore};

use crate::application::{AppError, AppResult, PasswordResetConfig, PasswordResetSettings, PasswordResetTemplate};

const AUTH_NAMESPACE: &str = "auth";
const TEMPLATE_GROUP: &str = "emailTemplates";
const PASSWORD_RESET_SUBJECT_KEY: &str = "passwordReset.subject";
const PASSWORD_RESET_HTML_KEY: &str = "passwordReset.html";

#[derive(Clone)]
pub struct StoragePasswordResetConfig {
    settings: SettingStore,
    i18n: I18nStore,
}

impl StoragePasswordResetConfig {
    pub fn new(database: Database) -> Self {
        Self {
            settings: SettingStore::new(database.clone()),
            i18n: I18nStore::new(database),
        }
    }
}

#[async_trait]
impl PasswordResetConfig for StoragePasswordResetConfig {
    async fn password_reset_settings(&self) -> AppResult<PasswordResetSettings> {
        let settings = self.settings.get_system_settings().await.map_err(storage_error)?;
        Ok(PasswordResetSettings {
            site_name: settings.site_name,
            password_reset_enabled: settings.password_reset_enabled,
            email_config_enabled: settings.email_config_enabled,
            smtp_host: settings.smtp_host,
            smtp_username: settings.smtp_username,
            smtp_password_set: settings.smtp_password_set,
            smtp_from_email: settings.smtp_from_email,
            smtp_from_name: settings.smtp_from_name,
            smtp_encryption: settings.smtp_encryption,
        })
    }

    async fn password_reset_template(&self, lang: &str) -> AppResult<PasswordResetTemplate> {
        let subject = self.template_value(lang, PASSWORD_RESET_SUBJECT_KEY).await?;
        let html = self.template_value(lang, PASSWORD_RESET_HTML_KEY).await?;
        Ok(PasswordResetTemplate { subject, html })
    }
}

impl StoragePasswordResetConfig {
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
