use async_trait::async_trait;
use storage::{Database, StorageError, setting::SettingStore};
use types::system_setting::{SystemSettingsResponse, SystemSettingsUpdate};

use crate::application::{SettingError, SettingRepository, SettingResult, StoredSmtpSettings};

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

    async fn get_smtp_settings(&self) -> SettingResult<StoredSmtpSettings> {
        self.store.get_smtp_settings().await.map(stored_smtp_settings).map_err(storage_error)
    }

    async fn update_system_settings(&self, input: SystemSettingsUpdate, encrypted_smtp_password: Option<String>) -> SettingResult<SystemSettingsResponse> {
        self.store
            .update_system_settings(record_patch(input, encrypted_smtp_password))
            .await
            .map(Into::into)
            .map_err(storage_error)
    }
}

fn stored_smtp_settings(value: storage::setting::SystemSettingsSmtpRecord) -> StoredSmtpSettings {
    StoredSmtpSettings {
        smtp_host: value.smtp_host,
        smtp_port: value.smtp_port,
        smtp_username: value.smtp_username,
        encrypted_smtp_password: value.encrypted_smtp_password,
        smtp_from_email: value.smtp_from_email,
        smtp_from_name: value.smtp_from_name,
        smtp_encryption: value.smtp_encryption,
    }
}

fn record_patch(input: SystemSettingsUpdate, encrypted_smtp_password: Option<String>) -> storage::setting::SystemSettingsRecordPatch {
    storage::setting::SystemSettingsRecordPatch {
        site_name: input.site_name,
        site_subtitle: input.site_subtitle,
        site_logo_base64: input.site_logo_base64,
        allow_registration: input.allow_registration,
        login_captcha_enabled: input.login_captcha_enabled,
        registration_captcha_enabled: input.registration_captcha_enabled,
        registration_email_verification_enabled: input.registration_email_verification_enabled,
        password_reset_enabled: input.password_reset_enabled,
        email_config_enabled: input.email_config_enabled,
        support_ticket_email_notifications_enabled: input.support_ticket_email_notifications_enabled,
        auto_delete_expired_tokens: input.auto_delete_expired_tokens,
        request_record_cleanup_enabled: input.request_record_cleanup_enabled,
        request_record_cleanup_interval_hours: input.request_record_cleanup_interval_hours,
        performance_monitoring_cleanup_enabled: input.performance_monitoring_cleanup_enabled,
        performance_monitoring_cleanup_interval_hours: input.performance_monitoring_cleanup_interval_hours,
        request_record_retention_days: input.request_record_retention_days,
        request_record_payload_retention_days: input.request_record_payload_retention_days,
        performance_monitoring_retention_days: input.performance_monitoring_retention_days,
        client_request_record_level: input.client_request_record_level,
        client_max_request_body_size_kb: input.client_max_request_body_size_kb,
        client_max_response_body_size_kb: input.client_max_response_body_size_kb,
        client_sensitive_request_headers: input.client_sensitive_request_headers,
        provider_request_record_level: input.provider_request_record_level,
        provider_max_request_body_size_kb: input.provider_max_request_body_size_kb,
        provider_max_response_body_size_kb: input.provider_max_response_body_size_kb,
        provider_sensitive_request_headers: input.provider_sensitive_request_headers,
        default_user_grant: input.default_user_grant,
        default_rate_limit_rpm: input.default_rate_limit_rpm,
        scheduling_mode: input.scheduling_mode,
        cache_affinity_ttl_minutes: input.cache_affinity_ttl_minutes,
        provider_cooldown_policy: input.provider_cooldown_policy,
        smtp_host: input.smtp_host,
        smtp_port: input.smtp_port,
        smtp_username: input.smtp_username,
        encrypted_smtp_password,
        smtp_from_email: input.smtp_from_email,
        smtp_from_name: input.smtp_from_name,
        smtp_encryption: input.smtp_encryption,
        email_suffix_mode: input.email_suffix_mode,
        email_suffixes: input.email_suffixes,
        email_template_registration_subject: input.email_template_registration_subject,
        email_template_registration_html: input.email_template_registration_html,
        email_template_password_reset_subject: input.email_template_password_reset_subject,
        email_template_password_reset_html: input.email_template_password_reset_html,
    }
}

fn storage_error(error: StorageError) -> SettingError {
    match error {
        StorageError::NotFound => SettingError::NotFound,
        StorageError::Conflict(message) | StorageError::Database(message) => SettingError::Infrastructure(message),
    }
}
