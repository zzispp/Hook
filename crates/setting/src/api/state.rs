use std::sync::Arc;

use crate::application::SettingUseCase;

#[derive(Clone)]
pub struct SettingApiState {
    pub settings: Arc<dyn SettingUseCase>,
}

impl SettingApiState {
    pub const fn new(settings: Arc<dyn SettingUseCase>) -> Self {
        Self { settings }
    }
}
