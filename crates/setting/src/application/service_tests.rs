use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    provider::{ProviderCooldownPolicy, ProviderSchedulingMode},
    system_setting::{DisplayCurrency, EmailSuffixMode, RequestRecordLevel, SmtpEncryption, SystemSettingsResponse, SystemSettingsUpdate},
};

use crate::application::{
    SettingRepository, SettingResult, SettingSecretCipher, SettingService, SmtpConnectionConfig, SmtpConnectionTester, StoredSmtpSettings,
};

struct FakeRepository {
    stored: StoredSmtpSettings,
    settings: SystemSettingsResponse,
    update: Arc<Mutex<Option<UpdateRecord>>>,
}

#[derive(Clone, Debug, PartialEq)]
struct UpdateRecord {
    input: SystemSettingsUpdate,
    encrypted_smtp_password: Option<String>,
}

#[async_trait]
impl SettingRepository for FakeRepository {
    async fn get_system_settings(&self) -> SettingResult<SystemSettingsResponse> {
        Ok(self.settings.clone())
    }

    async fn get_smtp_settings(&self) -> SettingResult<StoredSmtpSettings> {
        Ok(self.stored.clone())
    }

    async fn update_system_settings(&self, input: SystemSettingsUpdate, encrypted_smtp_password: Option<String>) -> SettingResult<SystemSettingsResponse> {
        *self.update.lock().unwrap() = Some(UpdateRecord {
            input,
            encrypted_smtp_password,
        });
        Ok(self.settings.clone())
    }
}

struct FakeCipher;

impl SettingSecretCipher for FakeCipher {
    fn encrypt_secret(&self, plaintext: &str) -> SettingResult<String> {
        Ok(format!("encrypted:{plaintext}"))
    }

    fn decrypt_secret(&self, ciphertext: &str) -> SettingResult<String> {
        Ok(ciphertext.trim_start_matches("encrypted:").to_owned())
    }
}

#[derive(Clone, Default)]
struct RecordingTester {
    config: Arc<Mutex<Option<SmtpConnectionConfig>>>,
}

#[async_trait]
impl SmtpConnectionTester for RecordingTester {
    async fn test_connection(&self, config: &SmtpConnectionConfig) -> Result<(), String> {
        *self.config.lock().unwrap() = Some(config.clone());
        Ok(())
    }
}

#[path = "service_tests/smtp.rs"]
mod smtp;
#[path = "service_tests/update.rs"]
mod update;

fn test_service(tester: RecordingTester, stored: StoredSmtpSettings) -> SettingService<FakeRepository, FakeCipher, RecordingTester> {
    SettingService::new(
        fake_repository(stored, system_settings_response(), Arc::new(Mutex::new(None))),
        FakeCipher,
        tester,
    )
}

fn test_update_service(settings: SystemSettingsResponse) -> (SettingService<FakeRepository, FakeCipher, RecordingTester>, Arc<Mutex<Option<UpdateRecord>>>) {
    let update = Arc::new(Mutex::new(None));
    let repository = fake_repository(stored_smtp_settings("encrypted:saved-password"), settings, update.clone());
    (SettingService::new(repository, FakeCipher, RecordingTester::default()), update)
}

fn fake_repository(stored: StoredSmtpSettings, settings: SystemSettingsResponse, update: Arc<Mutex<Option<UpdateRecord>>>) -> FakeRepository {
    FakeRepository { stored, settings, update }
}

fn stored_smtp_settings(encrypted_smtp_password: &str) -> StoredSmtpSettings {
    StoredSmtpSettings {
        smtp_host: "smtp.saved.test".into(),
        smtp_port: 587,
        smtp_username: "saved-user".into(),
        encrypted_smtp_password: encrypted_smtp_password.into(),
        smtp_from_email: "saved@example.com".into(),
        smtp_from_name: "Hook".into(),
        smtp_encryption: SmtpEncryption::Tls,
    }
}

fn system_settings_response() -> SystemSettingsResponse {
    SystemSettingsResponse {
        site_name: "Hook".into(),
        site_subtitle: "AI API platform".into(),
        allow_registration: true,
        login_captcha_enabled: false,
        registration_captcha_enabled: false,
        registration_email_verification_enabled: false,
        email_config_enabled: false,
        support_ticket_email_notifications_enabled: false,
        auto_delete_expired_tokens: false,
        request_record_cleanup_enabled: true,
        request_record_cleanup_interval_hours: 24,
        performance_monitoring_cleanup_enabled: true,
        performance_monitoring_cleanup_interval_hours: 24,
        request_record_retention_days: 365,
        request_record_payload_retention_days: 30,
        performance_monitoring_retention_days: 30,
        client_request_record_level: RequestRecordLevel::Basic,
        client_max_request_body_size_kb: 5120,
        client_max_response_body_size_kb: 5120,
        client_sensitive_request_headers: "authorization, x-api-key, api-key, cookie, set-cookie".into(),
        provider_request_record_level: RequestRecordLevel::Basic,
        provider_max_request_body_size_kb: 5120,
        provider_max_response_body_size_kb: 5120,
        provider_sensitive_request_headers: "authorization, x-api-key, api-key, cookie, set-cookie".into(),
        default_user_grant: Decimal::ZERO,
        default_rate_limit_rpm: 0,
        scheduling_mode: ProviderSchedulingMode::CacheAffinity,
        provider_cooldown_policy: ProviderCooldownPolicy::default(),
        currency: DisplayCurrency::Usd,
        smtp_host: String::new(),
        smtp_port: 587,
        smtp_username: String::new(),
        smtp_password_set: false,
        smtp_from_email: String::new(),
        smtp_from_name: "Hook".into(),
        smtp_encryption: SmtpEncryption::Tls,
        email_suffix_mode: EmailSuffixMode::None,
        email_suffixes: String::new(),
        email_template_registration_subject: "注册验证码".into(),
        email_template_registration_html: "<p>{{code}}</p>".into(),
        email_template_password_reset_subject: "找回密码".into(),
        email_template_password_reset_html: "<p>{{reset_link}}</p>".into(),
        created_at: "2026-05-13T00:00:00Z".into(),
        updated_at: "2026-05-13T00:00:00Z".into(),
    }
}

fn complete_email_settings() -> SystemSettingsResponse {
    let mut settings = system_settings_response();
    settings.email_config_enabled = true;
    settings.smtp_host = "smtp.example.com".into();
    settings.smtp_username = "smtp-user".into();
    settings.smtp_password_set = true;
    settings.smtp_from_email = "noreply@example.com".into();
    settings
}

fn complete_email_verification_update() -> SystemSettingsUpdate {
    SystemSettingsUpdate {
        email_config_enabled: Some(true),
        registration_email_verification_enabled: Some(true),
        smtp_host: Some("smtp.example.com".into()),
        smtp_port: Some(587),
        smtp_username: Some("smtp-user".into()),
        smtp_password: Some("smtp-password".into()),
        smtp_from_email: Some("noreply@example.com".into()),
        ..Default::default()
    }
}

fn complete_ticket_email_update() -> SystemSettingsUpdate {
    SystemSettingsUpdate {
        email_config_enabled: Some(true),
        support_ticket_email_notifications_enabled: Some(true),
        smtp_host: Some("smtp.example.com".into()),
        smtp_port: Some(587),
        smtp_username: Some("smtp-user".into()),
        smtp_password: Some("smtp-password".into()),
        smtp_from_email: Some("noreply@example.com".into()),
        ..Default::default()
    }
}
