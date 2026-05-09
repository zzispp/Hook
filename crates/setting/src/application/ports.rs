use async_trait::async_trait;
use types::system_setting::{SystemSettingsResponse, SystemSettingsUpdate};

use super::SettingResult;

#[async_trait]
pub trait SettingRepository: Send + Sync + 'static {
    async fn get_system_settings(&self) -> SettingResult<SystemSettingsResponse>;
    async fn update_system_settings(&self, input: SystemSettingsUpdate) -> SettingResult<SystemSettingsResponse>;
}

#[async_trait]
pub trait SettingUseCase: Send + Sync + 'static {
    async fn get_system_settings(&self) -> SettingResult<SystemSettingsResponse>;
    async fn update_system_settings(&self, input: SystemSettingsUpdate) -> SettingResult<SystemSettingsResponse>;
}
