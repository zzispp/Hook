use async_trait::async_trait;
use storage::{Database, StorageError, setting::SettingStore};

use crate::application::{CaptchaError, CaptchaResult, CaptchaSettingsReader};

#[derive(Clone)]
pub struct StorageCaptchaSettingsReader {
    store: SettingStore,
}

impl StorageCaptchaSettingsReader {
    pub fn new(database: Database) -> Self {
        Self {
            store: SettingStore::new(database),
        }
    }
}

#[async_trait]
impl CaptchaSettingsReader for StorageCaptchaSettingsReader {
    async fn login_captcha_enabled(&self) -> CaptchaResult<bool> {
        self.store
            .get_system_settings()
            .await
            .map(|settings| settings.login_captcha_enabled)
            .map_err(storage_error)
    }

    async fn registration_captcha_enabled(&self) -> CaptchaResult<bool> {
        self.store
            .get_system_settings()
            .await
            .map(|settings| settings.registration_captcha_enabled)
            .map_err(storage_error)
    }
}

fn storage_error(error: StorageError) -> CaptchaError {
    match error {
        StorageError::NotFound => CaptchaError::Infrastructure("system settings are missing".into()),
        StorageError::Conflict(message) | StorageError::Database(message) => CaptchaError::Infrastructure(message),
    }
}
