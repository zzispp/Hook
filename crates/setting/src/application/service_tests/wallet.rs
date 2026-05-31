use types::system_setting::SystemSettingsUpdate;

use crate::application::SettingUseCase;

use super::{system_settings_response, test_update_service};

#[tokio::test]
async fn update_rejects_enabled_wallet_without_public_base_url() {
    let mut settings = system_settings_response();
    settings.public_base_url = String::new();
    let (service, update) = test_update_service(settings);

    let result = service
        .update_system_settings(SystemSettingsUpdate {
            auth_evm_enabled: Some(true),
            ..Default::default()
        })
        .await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: public_base_url is required before enabling wallet provider"
    );
    assert!(update.lock().unwrap().is_none());
}

#[tokio::test]
async fn update_rejects_clearing_public_base_url_while_wallet_remains_enabled() {
    let mut settings = system_settings_response();
    settings.auth_evm_enabled = true;
    let (service, update) = test_update_service(settings);

    let result = service
        .update_system_settings(SystemSettingsUpdate {
            public_base_url: Some(String::new()),
            ..Default::default()
        })
        .await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: public_base_url is required before enabling wallet provider"
    );
    assert!(update.lock().unwrap().is_none());
}

#[tokio::test]
async fn update_allows_enabling_wallet_with_public_base_url() {
    let mut settings = system_settings_response();
    settings.public_base_url = String::new();
    let (service, update) = test_update_service(settings);
    let input = SystemSettingsUpdate {
        public_base_url: Some("https://hook.example.com".into()),
        auth_evm_enabled: Some(true),
        ..Default::default()
    };

    service.update_system_settings(input.clone()).await.unwrap();

    assert_eq!(update.lock().unwrap().clone().unwrap().input, input);
}

#[tokio::test]
async fn update_rejects_unsupported_evm_chain_id() {
    let (service, update) = test_update_service(system_settings_response());

    let result = service
        .update_system_settings(SystemSettingsUpdate {
            auth_evm_enabled: Some(true),
            auth_evm_chain_ids: Some("137".into()),
            ..Default::default()
        })
        .await;

    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid input: auth_evm_chain_ids contains unsupported EVM network"
    );
    assert!(update.lock().unwrap().is_none());
}
