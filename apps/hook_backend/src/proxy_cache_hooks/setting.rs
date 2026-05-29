use async_trait::async_trait;
use setting::application::{SettingError, SettingRepository, SettingResult, StoredSmtpSettings};
use types::system_setting::{SystemSettingsResponse, SystemSettingsUpdate};

use super::cache::ProxyCacheInvalidator;

#[derive(Clone)]
pub struct CachedSettingRepository<R, C> {
    inner: R,
    cache: C,
}

impl<R, C> CachedSettingRepository<R, C> {
    pub const fn new(inner: R, cache: C) -> Self {
        Self { inner, cache }
    }
}

#[async_trait]
impl<R, C> SettingRepository for CachedSettingRepository<R, C>
where
    R: SettingRepository,
    C: ProxyCacheInvalidator,
{
    async fn get_system_settings(&self) -> SettingResult<SystemSettingsResponse> {
        self.inner.get_system_settings().await
    }

    async fn get_smtp_settings(&self) -> SettingResult<StoredSmtpSettings> {
        self.inner.get_smtp_settings().await
    }

    async fn update_system_settings(
        &self,
        input: SystemSettingsUpdate,
        encrypted_smtp_password: Option<String>,
        encrypted_github_client_secret: Option<String>,
        encrypted_google_client_secret: Option<String>,
    ) -> SettingResult<SystemSettingsResponse> {
        let settings = self
            .inner
            .update_system_settings(input, encrypted_smtp_password, encrypted_github_client_secret, encrypted_google_client_secret)
            .await?;
        self.refresh_scheduling().await?;
        Ok(settings)
    }
}

impl<R, C> CachedSettingRepository<R, C>
where
    C: ProxyCacheInvalidator,
{
    async fn refresh_scheduling(&self) -> SettingResult<()> {
        self.cache.refresh_scheduling().await.map_err(cache_error)
    }
}

fn cache_error(error: crate::llm_proxy::LlmProxyError) -> SettingError {
    SettingError::Infrastructure(error.to_string())
}
