use async_trait::async_trait;
use types::system_setting::{SystemSettingsResponse, SystemSettingsUpdate};

use crate::application::{SettingRepository, SettingResult, SettingUseCase};

use super::validation::{sanitize_update, validate_update};

pub struct SettingService<R> {
    repository: R,
}

impl<R> SettingService<R>
where
    R: SettingRepository,
{
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> SettingUseCase for SettingService<R>
where
    R: SettingRepository,
{
    async fn get_system_settings(&self) -> SettingResult<SystemSettingsResponse> {
        self.repository.get_system_settings().await
    }

    async fn update_system_settings(&self, input: SystemSettingsUpdate) -> SettingResult<SystemSettingsResponse> {
        let input = sanitize_update(input);
        validate_update(&input)?;
        self.repository.update_system_settings(input).await
    }
}
