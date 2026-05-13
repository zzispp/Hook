use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    provider::ProviderSchedulingMode,
    system_setting::{
        DisplayCurrency, EmailSuffixMode, RequestRecordLevel, SmtpEncryption, SystemSettingsResponse, SystemSettingsSmtpTestRequest, SystemSettingsUpdate,
    },
};

use crate::application::{
    SettingRepository, SettingResult, SettingSecretCipher, SettingService, SettingUseCase, SmtpConnectionConfig, SmtpConnectionTester, StoredSmtpSettings,
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

#[tokio::test]
async fn smtp_test_uses_saved_password_when_request_omits_password() {
    let tester = RecordingTester::default();
    let service = test_service(tester.clone(), stored_smtp_settings("encrypted:saved-password"));

    let response = service
        .test_smtp_connection(SystemSettingsSmtpTestRequest {
            smtp_host: Some("smtp.form.test".into()),
            smtp_port: Some(465),
            smtp_username: Some("form-user".into()),
            smtp_from_email: Some("noreply@example.com".into()),
            smtp_encryption: Some(SmtpEncryption::Ssl),
            ..Default::default()
        })
        .await
        .unwrap();

    let config = tester.config.lock().unwrap().clone().unwrap();
    assert!(response.success);
    assert_eq!(config.host, "smtp.form.test");
    assert_eq!(config.port, 465);
    assert_eq!(config.username, "form-user");
    assert_eq!(config.password, "saved-password");
    assert_eq!(config.encryption, SmtpEncryption::Ssl);
}

#[tokio::test]
async fn smtp_test_reports_missing_password_before_connection_attempt() {
    let tester = RecordingTester::default();
    let service = test_service(tester.clone(), stored_smtp_settings(""));

    let response = service
        .test_smtp_connection(SystemSettingsSmtpTestRequest {
            smtp_host: Some("smtp.example.com".into()),
            smtp_port: Some(587),
            smtp_username: Some("smtp-user".into()),
            smtp_from_email: Some("noreply@example.com".into()),
            smtp_encryption: Some(SmtpEncryption::Tls),
            ..Default::default()
        })
        .await
        .unwrap();

    assert!(!response.success);
    assert_eq!(response.message, "SMTP 配置不完整，请检查 SMTP 密码");
    assert!(tester.config.lock().unwrap().is_none());
}

#[tokio::test]
async fn update_rejects_email_verification_without_enabled_email_config() {
    let mut settings = complete_email_settings();
    settings.email_config_enabled = false;
    let (service, update) = test_update_service(settings);

    let result = service
        .update_system_settings(SystemSettingsUpdate {
            registration_email_verification_enabled: Some(true),
            ..Default::default()
        })
        .await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: registration_email_verification_enabled requires email_config_enabled and complete SMTP configuration"
    );
    assert!(update.lock().unwrap().is_none());
}

#[tokio::test]
async fn update_rejects_email_verification_without_complete_smtp_config() {
    let mut settings = system_settings_response();
    settings.email_config_enabled = true;
    let (service, update) = test_update_service(settings);

    let result = service
        .update_system_settings(SystemSettingsUpdate {
            registration_email_verification_enabled: Some(true),
            ..Default::default()
        })
        .await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: registration_email_verification_enabled requires email_config_enabled and complete SMTP configuration"
    );
    assert!(update.lock().unwrap().is_none());
}

#[tokio::test]
async fn update_allows_email_verification_when_payload_completes_email_config() {
    let (service, update) = test_update_service(system_settings_response());
    let input = complete_email_verification_update();

    service.update_system_settings(input.clone()).await.unwrap();

    let record = update.lock().unwrap().clone().unwrap();
    assert_eq!(record.input, input);
    assert_eq!(record.encrypted_smtp_password.as_deref(), Some("encrypted:smtp-password"));
}

#[tokio::test]
async fn update_rejects_disabling_email_config_while_email_verification_remains_enabled() {
    let mut settings = complete_email_settings();
    settings.registration_email_verification_enabled = true;
    let (service, update) = test_update_service(settings);

    let result = service
        .update_system_settings(SystemSettingsUpdate {
            email_config_enabled: Some(false),
            ..Default::default()
        })
        .await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: registration_email_verification_enabled requires email_config_enabled and complete SMTP configuration"
    );
    assert!(update.lock().unwrap().is_none());
}

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
        auto_delete_expired_tokens: false,
        request_record_retention_days: 365,
        request_record_payload_retention_days: 30,
        request_record_level: RequestRecordLevel::Basic,
        max_request_body_size_kb: 5120,
        max_response_body_size_kb: 5120,
        sensitive_request_headers: "authorization, x-api-key, api-key, cookie, set-cookie".into(),
        record_request_headers: false,
        record_request_body: false,
        record_response_body: false,
        default_user_grant: Decimal::ZERO,
        default_rate_limit_rpm: 0,
        scheduling_mode: ProviderSchedulingMode::CacheAffinity,
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
