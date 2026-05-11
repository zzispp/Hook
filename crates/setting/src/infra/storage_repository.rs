use async_trait::async_trait;
use storage::{Database, StorageError, setting::SettingStore};
use types::system_setting::{SystemSettingsResponse, SystemSettingsUpdate};

use crate::application::{SettingError, SettingRepository, SettingResult};

#[derive(Clone)]
pub struct StorageSettingRepository {
    store: SettingStore,
}

impl StorageSettingRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: SettingStore::new(database),
        }
    }
}

#[async_trait]
impl SettingRepository for StorageSettingRepository {
    async fn get_system_settings(&self) -> SettingResult<SystemSettingsResponse> {
        self.store.get_system_settings().await.map(Into::into).map_err(storage_error)
    }

    async fn update_system_settings(&self, input: SystemSettingsUpdate) -> SettingResult<SystemSettingsResponse> {
        self.store
            .update_system_settings(record_patch(input))
            .await
            .map(Into::into)
            .map_err(storage_error)
    }
}

fn record_patch(input: SystemSettingsUpdate) -> storage::setting::SystemSettingsRecordPatch {
    storage::setting::SystemSettingsRecordPatch {
        site_name: input.site_name,
        site_subtitle: input.site_subtitle,
        allow_registration: input.allow_registration,
        auto_delete_expired_tokens: input.auto_delete_expired_tokens,
        default_user_grant: input.default_user_grant,
        default_rate_limit_rpm: input.default_rate_limit_rpm,
        scheduling_mode: input.scheduling_mode,
    }
}

fn storage_error(error: StorageError) -> SettingError {
    match error {
        StorageError::NotFound => SettingError::NotFound,
        StorageError::Conflict(message) | StorageError::Database(message) => SettingError::Infrastructure(message),
    }
}
