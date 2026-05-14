use types::system_setting::{SmtpEncryption, SystemSettingsSmtpTestRequest};

use crate::application::SettingUseCase;

use super::{RecordingTester, stored_smtp_settings, test_service};

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
