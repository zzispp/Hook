use types::system_setting::SystemSettingsUpdate;

use crate::application::SettingUseCase;

use super::{system_settings_response, test_update_service};

#[tokio::test]
async fn update_rejects_enabled_oauth_without_public_base_url() {
    let mut settings = system_settings_response();
    settings.public_base_url = String::new();
    let (service, update) = test_update_service(settings);

    let result = service
        .update_system_settings(SystemSettingsUpdate {
            auth_google_enabled: Some(true),
            ..Default::default()
        })
        .await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: public_base_url is required before enabling OAuth provider"
    );
    assert!(update.lock().unwrap().is_none());
}

#[tokio::test]
async fn update_rejects_clearing_public_base_url_while_oauth_remains_enabled() {
    let mut settings = system_settings_response();
    settings.auth_github_enabled = true;
    let (service, update) = test_update_service(settings);

    let result = service
        .update_system_settings(SystemSettingsUpdate {
            public_base_url: Some(String::new()),
            ..Default::default()
        })
        .await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: public_base_url is required before enabling OAuth provider"
    );
    assert!(update.lock().unwrap().is_none());
}

#[tokio::test]
async fn update_allows_enabling_oauth_with_public_base_url() {
    let mut settings = system_settings_response();
    settings.public_base_url = String::new();
    let (service, update) = test_update_service(settings);
    let input = SystemSettingsUpdate {
        public_base_url: Some("https://hook.example.com".into()),
        auth_github_enabled: Some(true),
        ..Default::default()
    };

    service.update_system_settings(input.clone()).await.unwrap();

    assert_eq!(update.lock().unwrap().clone().unwrap().input, input);
}
