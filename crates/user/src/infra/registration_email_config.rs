use async_trait::async_trait;
use storage::{Database, i18n::I18nStore, setting::SettingStore};
use types::user::{AuthConfigResponse, AuthProviderConfigResponse, OAuthProviderPublicConfig, WalletProviderPublicConfig};

use crate::application::{AppError, AppResult, EmailSettings, RegistrationEmailConfig, RegistrationEmailTemplate};

const AUTH_NAMESPACE: &str = "auth";
const TEMPLATE_GROUP: &str = "emailTemplates";
const REGISTRATION_SUBJECT_KEY: &str = "registration.subject";
const REGISTRATION_HTML_KEY: &str = "registration.html";

#[derive(Clone)]
pub struct StorageRegistrationEmailConfig {
    settings: SettingStore,
    i18n: I18nStore,
}

impl StorageRegistrationEmailConfig {
    pub fn new(database: Database) -> Self {
        Self {
            settings: SettingStore::new(database.clone()),
            i18n: I18nStore::new(database),
        }
    }
}

#[async_trait]
impl RegistrationEmailConfig for StorageRegistrationEmailConfig {
    async fn auth_config(&self) -> AppResult<AuthConfigResponse> {
        let settings = self.settings.get_system_settings().await.map_err(storage_error)?;
        Ok(AuthConfigResponse {
            allow_registration: settings.allow_registration,
            registration_email_verification_enabled: settings.registration_email_verification_enabled,
            providers: AuthProviderConfigResponse {
                github: OAuthProviderPublicConfig {
                    enabled: settings.auth_github_enabled,
                },
                google: OAuthProviderPublicConfig {
                    enabled: settings.auth_google_enabled,
                },
                evm: WalletProviderPublicConfig {
                    enabled: settings.auth_evm_enabled,
                    domain: settings.auth_wallet_domain.clone(),
                    statement: settings.auth_wallet_statement.clone(),
                    evm_chain_ids: evm_chain_ids(&settings.auth_evm_chain_ids)?,
                    solana_network: String::new(),
                },
                solana: WalletProviderPublicConfig {
                    enabled: settings.auth_solana_enabled,
                    domain: settings.auth_wallet_domain,
                    statement: settings.auth_wallet_statement,
                    evm_chain_ids: Vec::new(),
                    solana_network: settings.auth_solana_network,
                },
            },
        })
    }

    async fn registration_email_settings(&self) -> AppResult<EmailSettings> {
        let settings = self.settings.get_system_settings().await.map_err(storage_error)?;
        Ok(EmailSettings {
            site_name: settings.site_name,
            feature_enabled: settings.registration_email_verification_enabled,
            email_config_enabled: settings.email_config_enabled,
            smtp_host: settings.smtp_host,
            smtp_username: settings.smtp_username,
            smtp_password_set: settings.smtp_password_set,
            smtp_from_email: settings.smtp_from_email,
            smtp_from_name: settings.smtp_from_name,
            smtp_encryption: settings.smtp_encryption,
        })
    }

    async fn registration_email_template(&self, lang: &str) -> AppResult<RegistrationEmailTemplate> {
        let subject = self.template_value(lang, REGISTRATION_SUBJECT_KEY).await?;
        let html = self.template_value(lang, REGISTRATION_HTML_KEY).await?;
        Ok(RegistrationEmailTemplate { subject, html })
    }
}

fn evm_chain_ids(value: &str) -> AppResult<Vec<u64>> {
    value.split(',').map(str::trim).filter(|item| !item.is_empty()).map(parse_chain_id).collect()
}

fn parse_chain_id(value: &str) -> AppResult<u64> {
    value.parse().map_err(|_| AppError::InvalidInput(format!("invalid EVM chain id: {value}")))
}

impl StorageRegistrationEmailConfig {
    async fn template_value(&self, lang: &str, item_key: &str) -> AppResult<String> {
        self.i18n
            .entry_value(lang, AUTH_NAMESPACE, TEMPLATE_GROUP, item_key)
            .await
            .map_err(storage_error)?
            .ok_or_else(|| AppError::InvalidInput(format!("missing auth email template: {lang}.{item_key}")))
    }
}

fn storage_error(error: storage::StorageError) -> AppError {
    match error {
        storage::StorageError::NotFound => AppError::NotFound,
        storage::StorageError::Conflict(message) | storage::StorageError::Database(message) => AppError::Infrastructure(message),
    }
}
