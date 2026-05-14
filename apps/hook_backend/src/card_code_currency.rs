use std::sync::Arc;

use card_code::application::{CardCodeCurrencyProvider, CardCodeError, CardCodeResult};
use types::system_setting::DisplayCurrency;

use crate::exchange_rates::ExchangeRateCache;

#[derive(Clone)]
pub(crate) struct BackendCardCodeCurrencyProvider {
    settings: Arc<dyn setting::application::SettingUseCase>,
    exchange_rates: ExchangeRateCache,
}

impl BackendCardCodeCurrencyProvider {
    pub(crate) fn new(settings: Arc<dyn setting::application::SettingUseCase>, exchange_rates: ExchangeRateCache) -> Self {
        Self { settings, exchange_rates }
    }
}

#[async_trait::async_trait]
impl CardCodeCurrencyProvider for BackendCardCodeCurrencyProvider {
    async fn current_currency(&self) -> CardCodeResult<DisplayCurrency> {
        self.settings
            .get_system_settings()
            .await
            .map(|settings| settings.currency)
            .map_err(|error| CardCodeError::Infrastructure(error.to_string()))
    }

    async fn usd_cny_rate(&self) -> CardCodeResult<rust_decimal::Decimal> {
        self.exchange_rates
            .read_usd_cny()
            .await
            .map(|snapshot| snapshot.rate)
            .map_err(|error| CardCodeError::Infrastructure(error.to_string()))
    }
}
