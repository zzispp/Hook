use std::sync::Arc;

use crate::application::I18nUseCase;

#[derive(Clone)]
pub struct I18nApiState {
    pub i18n: Arc<dyn I18nUseCase>,
}

impl I18nApiState {
    pub fn new(i18n: Arc<dyn I18nUseCase>) -> Self {
        Self { i18n }
    }
}
