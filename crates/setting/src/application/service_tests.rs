use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use types::system_setting::{SmtpEncryption, SystemSettingsSmtpTestRequest, SystemSettingsUpdate};

use crate::application::{
    SettingRepository, SettingResult, SettingSecretCipher, SettingService, SettingUseCase, SmtpConnectionConfig, SmtpConnectionTester, StoredSmtpSettings,
};

struct FakeRepository {
    stored: StoredSmtpSettings,
}

#[async_trait]
impl SettingRepository for FakeRepository {
    async fn get_system_settings(&self) -> SettingResult<types::system_setting::SystemSettingsResponse> {
        unimplemented!("not needed for smtp tests")
    }

    async fn get_smtp_settings(&self) -> SettingResult<StoredSmtpSettings> {
        Ok(self.stored.clone())
    }

    async fn update_system_settings(
        &self,
        _input: SystemSettingsUpdate,
        _encrypted_smtp_password: Option<String>,
    ) -> SettingResult<types::system_setting::SystemSettingsResponse> {
        unimplemented!("not needed for smtp tests")
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

fn test_service(tester: RecordingTester, stored: StoredSmtpSettings) -> SettingService<FakeRepository, FakeCipher, RecordingTester> {
    SettingService::new(FakeRepository { stored }, FakeCipher, tester)
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
