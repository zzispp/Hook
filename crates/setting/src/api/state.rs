use std::sync::Arc;

use crate::application::SettingUseCase;

#[derive(Clone)]
pub struct SettingApiState {
    pub settings: Arc<dyn SettingUseCase>,
    pub exchange_rates: Arc<dyn ExchangeRateReader>,
}

impl SettingApiState {
    pub fn new(settings: Arc<dyn SettingUseCase>, exchange_rates: impl ExchangeRateReader) -> Self {
        Self {
            settings,
            exchange_rates: Arc::new(exchange_rates),
        }
    }
}

#[async_trait::async_trait]
pub trait ExchangeRateReader: Send + Sync + 'static {
    async fn usd_cny_rate(&self) -> Result<types::system_setting::ExchangeRateResponse, String>;
}
