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
        login_captcha_enabled: input.login_captcha_enabled,
        registration_captcha_enabled: input.registration_captcha_enabled,
        auto_delete_expired_tokens: input.auto_delete_expired_tokens,
        request_record_retention_days: input.request_record_retention_days,
        request_record_payload_retention_days: input.request_record_payload_retention_days,
        request_record_level: input.request_record_level,
        max_request_body_size_kb: input.max_request_body_size_kb,
        max_response_body_size_kb: input.max_response_body_size_kb,
        sensitive_request_headers: input.sensitive_request_headers,
        record_request_headers: input.record_request_headers,
        record_request_body: input.record_request_body,
        record_response_body: input.record_response_body,
        default_user_grant: input.default_user_grant,
        default_rate_limit_rpm: input.default_rate_limit_rpm,
        scheduling_mode: input.scheduling_mode,
        currency: input.currency,
    }
}

fn storage_error(error: StorageError) -> SettingError {
    match error {
        StorageError::NotFound => SettingError::NotFound,
        StorageError::Conflict(message) | StorageError::Database(message) => SettingError::Infrastructure(message),
    }
}
