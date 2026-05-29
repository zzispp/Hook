use async_trait::async_trait;
use setting::application::SettingSecretCipher;
use storage::{Database, setting::SettingStore};
use types::user::IdentityProvider;

use crate::application::{AppError, AppResult, AuthProviderConfig, OAuthProviderSettings, WalletProviderSettings};

#[derive(Clone)]
pub struct StorageAuthProviderConfig<C> {
    store: SettingStore,
    cipher: C,
}

impl<C> StorageAuthProviderConfig<C> {
    pub fn new(database: Database, cipher: C) -> Self {
        Self {
            store: SettingStore::new(database),
            cipher,
        }
    }
}

#[async_trait]
impl<C> AuthProviderConfig for StorageAuthProviderConfig<C>
where
    C: SettingSecretCipher,
{
    async fn oauth_provider_settings(&self, provider: IdentityProvider) -> AppResult<OAuthProviderSettings> {
        let settings = self.store.get_auth_provider_settings().await.map_err(storage_error)?;
        match provider {
            IdentityProvider::Github => Ok(OAuthProviderSettings {
                enabled: settings.auth_github_enabled,
                client_id: settings.auth_github_client_id,
                client_secret: decrypt_secret(&self.cipher, &settings.encrypted_auth_github_client_secret)?,
            }),
            IdentityProvider::Google => Ok(OAuthProviderSettings {
                enabled: settings.auth_google_enabled,
                client_id: settings.auth_google_client_id,
                client_secret: decrypt_secret(&self.cipher, &settings.encrypted_auth_google_client_secret)?,
            }),
            _ => Err(AppError::InvalidInput("OAuth provider is invalid".into())),
        }
    }

    async fn wallet_provider_settings(&self) -> AppResult<WalletProviderSettings> {
        let settings = self.store.get_auth_provider_settings().await.map_err(storage_error)?;
        Ok(WalletProviderSettings {
            evm_enabled: settings.auth_evm_enabled,
            evm_chain_ids: evm_chain_ids(&settings.auth_evm_chain_ids)?,
            solana_enabled: settings.auth_solana_enabled,
            solana_network: settings.auth_solana_network,
            domain: settings.auth_wallet_domain,
            statement: settings.auth_wallet_statement,
        })
    }
}

fn decrypt_secret<C>(cipher: &C, encrypted: &str) -> AppResult<String>
where
    C: SettingSecretCipher,
{
    if encrypted.is_empty() {
        return Ok(String::new());
    }
    cipher.decrypt_secret(encrypted).map_err(setting_error)
}

fn evm_chain_ids(value: &str) -> AppResult<Vec<u64>> {
    value.split(',').map(str::trim).filter(|item| !item.is_empty()).map(parse_chain_id).collect()
}

fn parse_chain_id(value: &str) -> AppResult<u64> {
    value.parse().map_err(|_| AppError::InvalidInput(format!("invalid EVM chain id: {value}")))
}

fn storage_error(error: storage::StorageError) -> AppError {
    match error {
        storage::StorageError::NotFound => AppError::NotFound,
        storage::StorageError::Conflict(message) | storage::StorageError::Database(message) => AppError::Infrastructure(message),
    }
}

fn setting_error(error: setting::application::SettingError) -> AppError {
    AppError::Infrastructure(error.to_string())
}
