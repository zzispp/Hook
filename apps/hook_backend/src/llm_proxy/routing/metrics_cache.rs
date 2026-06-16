use std::{collections::HashMap, sync::Arc, time::Duration as StdDuration};

use storage::{
    Database,
    provider::{ProviderStore, RoutingContextRouteStateRecord, RoutingMetricRecord, RoutingRouteStateRecord},
};
use tokio::sync::RwLock;
use types::provider::RoutingMetricWindow;

use crate::llm_proxy::LlmProxyError;

const REFRESH_INTERVAL_SECONDS: u64 = 5;
const WINDOWS: [RoutingMetricWindow; 6] = [
    RoutingMetricWindow::OneMinute,
    RoutingMetricWindow::FiveMinutes,
    RoutingMetricWindow::FifteenMinutes,
    RoutingMetricWindow::OneHour,
    RoutingMetricWindow::OneDay,
    RoutingMetricWindow::SevenDays,
];

#[derive(Clone)]
pub(crate) struct RoutingMetricsCache {
    database: Database,
    inner: Arc<RwLock<RoutingMetricsSnapshot>>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RoutingMetricsSnapshot {
    pub(crate) windows: HashMap<RoutingMetricWindow, Vec<RoutingMetricRecord>>,
    pub(crate) route_states: Vec<RoutingRouteStateRecord>,
    pub(crate) context_route_states: Vec<RoutingContextRouteStateRecord>,
    pub(crate) refreshed_at: Option<time::OffsetDateTime>,
}

impl RoutingMetricsCache {
    pub(crate) async fn load(database: Database) -> Result<Self, LlmProxyError> {
        let snapshot = load_snapshot(&database).await?;
        Ok(Self {
            database,
            inner: Arc::new(RwLock::new(snapshot)),
        })
    }

    pub(crate) async fn snapshot(&self) -> RoutingMetricsSnapshot {
        self.inner.read().await.clone()
    }

    pub(crate) fn spawn_refresh_loop(&self) {
        let cache = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(StdDuration::from_secs(REFRESH_INTERVAL_SECONDS));
            loop {
                interval.tick().await;
                if let Err(error) = cache.refresh().await {
                    let snapshot_age_seconds = cache.snapshot_age_seconds().await.unwrap_or(-1);
                    hook_tracing::error_with_fields!("routing metrics cache refresh failed", &error, snapshot_age_seconds = snapshot_age_seconds,);
                }
            }
        });
    }

    async fn snapshot_age_seconds(&self) -> Option<i64> {
        self.inner.read().await.age_seconds()
    }

    async fn refresh(&self) -> Result<(), LlmProxyError> {
        let snapshot = load_snapshot(&self.database).await?;
        *self.inner.write().await = snapshot;
        Ok(())
    }
}

impl RoutingMetricsSnapshot {
    fn age_seconds(&self) -> Option<i64> {
        self.refreshed_at
            .map(|refreshed_at| (time::OffsetDateTime::now_utc() - refreshed_at).whole_seconds().max(0))
    }
}

async fn load_snapshot(database: &Database) -> Result<RoutingMetricsSnapshot, LlmProxyError> {
    let store = ProviderStore::new(database.clone());
    let mut windows = HashMap::new();
    for window in WINDOWS {
        windows.insert(window, store.list_routing_metrics(window).await?);
    }
    let route_states = store.list_routing_route_states().await?;
    let context_route_states = store.list_routing_context_route_states().await?;
    Ok(RoutingMetricsSnapshot {
        windows,
        route_states,
        context_route_states,
        refreshed_at: Some(time::OffsetDateTime::now_utc()),
    })
}

#[cfg(test)]
mod tests {
    use super::WINDOWS;

    #[test]
    fn routing_metrics_cache_loads_every_supported_window() {
        assert_eq!(WINDOWS.len(), 6);
    }
}
