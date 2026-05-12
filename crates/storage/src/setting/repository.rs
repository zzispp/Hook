use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use types::system_setting::SystemSettings;

use crate::{Database, StorageError, StorageResult};

use super::{
    SystemSettingsRecordPatch,
    record::{system_settings, system_settings::ActiveModel as SystemSettingsActiveModel},
};

pub const SYSTEM_SETTINGS_ID: &str = "global";

#[derive(Clone)]
pub struct SettingStore {
    database: Database,
}

impl SettingStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn get_system_settings(&self) -> StorageResult<SystemSettings> {
        let record = system_settings::Entity::find_by_id(SYSTEM_SETTINGS_ID.to_owned())
            .one(self.database.connection())
            .await?
            .ok_or(StorageError::NotFound)?;
        record.try_into().map_err(StorageError::Database)
    }

    pub async fn update_system_settings(&self, input: SystemSettingsRecordPatch) -> StorageResult<SystemSettings> {
        let record = system_settings::Entity::find_by_id(SYSTEM_SETTINGS_ID.to_owned())
            .one(self.database.connection())
            .await?
            .ok_or(StorageError::NotFound)?;
        let mut active: SystemSettingsActiveModel = record.into();
        apply_patch(&mut active, input);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?;
        self.get_system_settings().await
    }
}

fn apply_patch(active: &mut SystemSettingsActiveModel, input: SystemSettingsRecordPatch) {
    if let Some(value) = input.site_name {
        active.site_name = Set(value);
    }
    if let Some(value) = input.site_subtitle {
        active.site_subtitle = Set(value);
    }
    if let Some(value) = input.allow_registration {
        active.allow_registration = Set(value);
    }
    if let Some(value) = input.login_captcha_enabled {
        active.login_captcha_enabled = Set(value);
    }
    if let Some(value) = input.registration_captcha_enabled {
        active.registration_captcha_enabled = Set(value);
    }
    if let Some(value) = input.auto_delete_expired_tokens {
        active.auto_delete_expired_tokens = Set(value);
    }
    if let Some(value) = input.request_record_retention_days {
        active.request_record_retention_days = Set(value);
    }
    if let Some(value) = input.request_record_payload_retention_days {
        active.request_record_payload_retention_days = Set(value);
    }
    if let Some(value) = input.request_record_level {
        active.request_record_level = Set(value.as_str().to_owned());
    }
    if let Some(value) = input.max_request_body_size_kb {
        active.max_request_body_size_kb = Set(value);
    }
    if let Some(value) = input.max_response_body_size_kb {
        active.max_response_body_size_kb = Set(value);
    }
    if let Some(value) = input.sensitive_request_headers {
        active.sensitive_request_headers = Set(value);
    }
    if let Some(value) = input.record_request_headers {
        active.record_request_headers = Set(value);
    }
    if let Some(value) = input.record_request_body {
        active.record_request_body = Set(value);
    }
    if let Some(value) = input.record_response_body {
        active.record_response_body = Set(value);
    }
    if let Some(value) = input.default_user_grant {
        active.default_user_grant = Set(value);
    }
    if let Some(value) = input.default_rate_limit_rpm {
        active.default_rate_limit_rpm = Set(value);
    }
    if let Some(value) = input.scheduling_mode {
        active.scheduling_mode = Set(value.as_str().to_owned());
    }
    if let Some(value) = input.currency {
        active.currency = Set(value.as_str().to_owned());
    }
}
