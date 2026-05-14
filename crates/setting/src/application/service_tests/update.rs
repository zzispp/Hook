use types::system_setting::SystemSettingsUpdate;

use crate::application::SettingUseCase;

use super::{complete_email_settings, complete_email_verification_update, complete_ticket_email_update, system_settings_response, test_update_service};

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

#[tokio::test]
async fn update_rejects_ticket_email_notifications_without_enabled_email_config() {
    let mut settings = complete_email_settings();
    settings.email_config_enabled = false;
    let (service, update) = test_update_service(settings);

    let result = service
        .update_system_settings(SystemSettingsUpdate {
            support_ticket_email_notifications_enabled: Some(true),
            ..Default::default()
        })
        .await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: support_ticket_email_notifications_enabled requires email_config_enabled and complete SMTP configuration"
    );
    assert!(update.lock().unwrap().is_none());
}

#[tokio::test]
async fn update_rejects_ticket_email_notifications_without_complete_smtp_config() {
    let mut settings = system_settings_response();
    settings.email_config_enabled = true;
    let (service, update) = test_update_service(settings);

    let result = service
        .update_system_settings(SystemSettingsUpdate {
            support_ticket_email_notifications_enabled: Some(true),
            ..Default::default()
        })
        .await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: support_ticket_email_notifications_enabled requires email_config_enabled and complete SMTP configuration"
    );
    assert!(update.lock().unwrap().is_none());
}

#[tokio::test]
async fn update_allows_ticket_email_notifications_when_payload_completes_email_config() {
    let (service, update) = test_update_service(system_settings_response());
    let input = complete_ticket_email_update();

    service.update_system_settings(input.clone()).await.unwrap();

    let record = update.lock().unwrap().clone().unwrap();
    assert_eq!(record.input, input);
    assert_eq!(record.encrypted_smtp_password.as_deref(), Some("encrypted:smtp-password"));
}

#[tokio::test]
async fn update_rejects_disabling_email_config_while_ticket_email_notifications_remain_enabled() {
    let mut settings = complete_email_settings();
    settings.support_ticket_email_notifications_enabled = true;
    let (service, update) = test_update_service(settings);

    let result = service
        .update_system_settings(SystemSettingsUpdate {
            email_config_enabled: Some(false),
            ..Default::default()
        })
        .await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: support_ticket_email_notifications_enabled requires email_config_enabled and complete SMTP configuration"
    );
    assert!(update.lock().unwrap().is_none());
}
