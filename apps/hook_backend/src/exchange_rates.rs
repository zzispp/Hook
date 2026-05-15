use redis::AsyncCommands;
use req::ReqwestClient;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{error::Error, time::Duration};
use time::OffsetDateTime;

use crate::BackendResult;
use setting::api::ExchangeRateReader;

const FRANKFURTER_URL: &str = "https://api.frankfurter.dev/v1/latest?base=USD&symbols=CNY";
const REFRESH_INTERVAL: Duration = Duration::from_secs(300);
const SOURCE_NAME: &str = "frankfurter";

#[derive(Clone)]
pub struct ExchangeRateCache {
    connection: redis::aio::ConnectionManager,
    key_prefix: String,
    http: ReqwestClient,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExchangeRateSnapshot {
    pub base: String,
    pub target: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub rate: Decimal,
    pub source: String,
    pub source_date: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
struct FrankfurterResponse {
    base: String,
    date: String,
    rates: FrankfurterRates,
}

#[derive(Debug, Deserialize)]
struct FrankfurterRates {
    #[serde(rename = "CNY", with = "rust_decimal::serde::float")]
    cny: Decimal,
}

impl ExchangeRateCache {
    pub async fn connect(url: &str, key_prefix: String) -> BackendResult<Self> {
        let client = redis::Client::open(url)?;
        let connection = client.get_connection_manager().await?;
        let http = ReqwestClient::default();
        Ok(Self { connection, key_prefix, http })
    }

    pub fn spawn_refresh_task(self) {
        tokio::spawn(async move {
            refresh_loop(self).await;
        });
    }

    pub async fn read_usd_cny(&self) -> BackendResult<ExchangeRateSnapshot> {
        let mut connection = self.connection.clone();
        let value: Option<String> = connection.get(self.usd_cny_key()).await?;
        let value = value.ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "USD/CNY exchange rate cache is missing"))?;
        Ok(serde_json::from_str(&value)?)
    }

    async fn refresh_usd_cny(&self) -> BackendResult<ExchangeRateSnapshot> {
        let response = self.http.get_json::<FrankfurterResponse>(FRANKFURTER_URL).await?;
        let snapshot = ExchangeRateSnapshot {
            base: response.base,
            target: "CNY".into(),
            rate: response.rates.cny,
            source: SOURCE_NAME.into(),
            source_date: response.date,
            updated_at: OffsetDateTime::now_utc().to_string(),
        };
        let mut connection = self.connection.clone();
        let value = serde_json::to_string(&snapshot)?;
        let _: () = connection.set(self.usd_cny_key(), value).await?;
        Ok(snapshot)
    }

    fn usd_cny_key(&self) -> String {
        format!("{}:exchange_rates:USD:CNY", self.key_prefix)
    }
}

#[async_trait::async_trait]
impl ExchangeRateReader for ExchangeRateCache {
    async fn usd_cny_rate(&self) -> Result<types::system_setting::ExchangeRateResponse, String> {
        self.read_usd_cny()
            .await
            .map(|snapshot| types::system_setting::ExchangeRateResponse {
                base: snapshot.base,
                target: snapshot.target,
                rate: snapshot.rate,
                source: snapshot.source,
                source_date: snapshot.source_date,
                updated_at: snapshot.updated_at,
            })
            .map_err(|error| error.to_string())
    }
}

async fn refresh_loop(cache: ExchangeRateCache) {
    if let Err(error) = cache.refresh_usd_cny().await {
        hook_tracing::error("failed to refresh USD/CNY exchange rate", error.as_ref() as &(dyn Error + Send + Sync));
    }
    let mut interval = tokio::time::interval(REFRESH_INTERVAL);
    loop {
        interval.tick().await;
        if let Err(error) = cache.refresh_usd_cny().await {
            hook_tracing::error("failed to refresh USD/CNY exchange rate", error.as_ref() as &(dyn Error + Send + Sync));
        }
    }
}
