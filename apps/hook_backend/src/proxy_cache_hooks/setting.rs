use std::sync::Arc;

use async_trait::async_trait;
use setting::application::{SettingError, SettingResult, SettingUseCase};
use types::system_setting::{SystemSettingsResponse, SystemSettingsSmtpTestRequest, SystemSettingsSmtpTestResponse, SystemSettingsUpdate};

use crate::llm_proxy::LlmProxyCache;

pub struct ProxyCachedSettingUseCase {
    inner: Arc<dyn SettingUseCase>,
    cache: LlmProxyCache,
}

impl ProxyCachedSettingUseCase {
    pub fn new(inner: Arc<dyn SettingUseCase>, cache: LlmProxyCache) -> Self {
        Self { inner, cache }
    }

    async fn refresh_scheduling(&self) -> SettingResult<()> {
        self.cache.refresh_scheduling_snapshot().await.map(|_| ()).map_err(cache_error)
    }
}

#[async_trait]
impl SettingUseCase for ProxyCachedSettingUseCase {
    async fn get_system_settings(&self) -> SettingResult<SystemSettingsResponse> {
        self.inner.get_system_settings().await
    }

    async fn update_system_settings(&self, input: SystemSettingsUpdate) -> SettingResult<SystemSettingsResponse> {
        let value = self.inner.update_system_settings(input).await?;
        self.refresh_scheduling().await?;
        Ok(value)
    }

    async fn test_smtp_connection(&self, input: SystemSettingsSmtpTestRequest) -> SettingResult<SystemSettingsSmtpTestResponse> {
        self.inner.test_smtp_connection(input).await
    }
}

fn cache_error(error: crate::llm_proxy::LlmProxyError) -> SettingError {
    SettingError::Infrastructure(error.to_string())
}
