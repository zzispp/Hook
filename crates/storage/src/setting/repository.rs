use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use types::system_setting::SystemSettings;

use crate::{Database, StorageError, StorageResult};

use super::{
    SystemSettingsRecordPatch, SystemSettingsSmtpRecord,
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
        let record = self.get_system_settings_record().await?;
        record.try_into().map_err(StorageError::Database)
    }

    pub async fn get_smtp_settings(&self) -> StorageResult<SystemSettingsSmtpRecord> {
        let record = self.get_system_settings_record().await?;
        Ok(SystemSettingsSmtpRecord {
            smtp_host: record.smtp_host,
            smtp_port: record.smtp_port,
            smtp_username: record.smtp_username,
            encrypted_smtp_password: record.encrypted_smtp_password,
            smtp_from_email: record.smtp_from_email,
            smtp_from_name: record.smtp_from_name,
            smtp_encryption: record.smtp_encryption.as_str().try_into().map_err(StorageError::Database)?,
        })
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

    async fn get_system_settings_record(&self) -> StorageResult<system_settings::Model> {
        system_settings::Entity::find_by_id(SYSTEM_SETTINGS_ID.to_owned())
            .one(self.database.connection())
            .await?
            .ok_or(StorageError::NotFound)
    }
}

fn apply_patch(active: &mut SystemSettingsActiveModel, input: SystemSettingsRecordPatch) {
    apply_base_patch(active, &input);
    apply_request_record_patch(active, &input);
    apply_mail_patch(active, input);
}

fn apply_base_patch(active: &mut SystemSettingsActiveModel, input: &SystemSettingsRecordPatch) {
    if let Some(value) = &input.site_name {
        active.site_name = Set(value.clone());
    }
    if let Some(value) = &input.site_subtitle {
        active.site_subtitle = Set(value.clone());
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
    if let Some(value) = input.registration_email_verification_enabled {
        active.registration_email_verification_enabled = Set(value);
    }
    if let Some(value) = input.email_config_enabled {
        active.email_config_enabled = Set(value);
    }
    if let Some(value) = input.support_ticket_email_notifications_enabled {
        active.support_ticket_email_notifications_enabled = Set(value);
    }
    if let Some(value) = input.auto_delete_expired_tokens {
        active.auto_delete_expired_tokens = Set(value);
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
    if let Some(value) = &input.currency {
        active.currency = Set(value.as_str().to_owned());
    }
}

fn apply_request_record_patch(active: &mut SystemSettingsActiveModel, input: &SystemSettingsRecordPatch) {
    if let Some(value) = input.request_record_retention_days {
        active.request_record_retention_days = Set(value);
    }
    if let Some(value) = input.request_record_payload_retention_days {
        active.request_record_payload_retention_days = Set(value);
    }
    if let Some(value) = input.performance_monitoring_retention_days {
        active.performance_monitoring_retention_days = Set(value);
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
    if let Some(value) = &input.sensitive_request_headers {
        active.sensitive_request_headers = Set(value.clone());
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
}

fn apply_mail_patch(active: &mut SystemSettingsActiveModel, input: SystemSettingsRecordPatch) {
    if let Some(value) = input.smtp_host {
        active.smtp_host = Set(value);
    }
    if let Some(value) = input.smtp_port {
        active.smtp_port = Set(value);
    }
    if let Some(value) = input.smtp_username {
        active.smtp_username = Set(value);
    }
    if let Some(value) = input.encrypted_smtp_password {
        active.encrypted_smtp_password = Set(value);
    }
    if let Some(value) = input.smtp_from_email {
        active.smtp_from_email = Set(value);
    }
    if let Some(value) = input.smtp_from_name {
        active.smtp_from_name = Set(value);
    }
    if let Some(value) = input.smtp_encryption {
        active.smtp_encryption = Set(value.as_str().to_owned());
    }
    if let Some(value) = input.email_suffix_mode {
        active.email_suffix_mode = Set(value.as_str().to_owned());
    }
    if let Some(value) = input.email_suffixes {
        active.email_suffixes = Set(value);
    }
    if let Some(value) = input.email_template_registration_subject {
        active.email_template_registration_subject = Set(value);
    }
    if let Some(value) = input.email_template_registration_html {
        active.email_template_registration_html = Set(value);
    }
    if let Some(value) = input.email_template_password_reset_subject {
        active.email_template_password_reset_subject = Set(value);
    }
    if let Some(value) = input.email_template_password_reset_html {
        active.email_template_password_reset_html = Set(value);
    }
}
