use card_code::application::{CardCodeCurrencyProvider, CardCodeError, CardCodeResult};

use crate::exchange_rates::ExchangeRateCache;

#[derive(Clone)]
pub(crate) struct BackendCardCodeCurrencyProvider {
    exchange_rates: ExchangeRateCache,
}

impl BackendCardCodeCurrencyProvider {
    pub(crate) fn new(exchange_rates: ExchangeRateCache) -> Self {
        Self { exchange_rates }
    }
}

#[async_trait::async_trait]
impl CardCodeCurrencyProvider for BackendCardCodeCurrencyProvider {
    async fn usd_cny_rate(&self) -> CardCodeResult<rust_decimal::Decimal> {
        self.exchange_rates
            .read_usd_cny()
            .await
            .map(|snapshot| snapshot.rate)
            .map_err(|error| CardCodeError::Infrastructure(error.to_string()))
    }
}
