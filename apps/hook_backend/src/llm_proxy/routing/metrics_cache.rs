use std::{collections::HashMap, sync::Arc, time::Duration as StdDuration};

use storage::{
    Database,
    provider::{ProviderStore, RoutingContextRouteStateRecord, RoutingMetricRecord, RoutingRouteStateRecord},
};
use tokio::sync::RwLock;
use types::provider::RoutingMetricWindow;

use crate::llm_proxy::LlmProxyError;

const REFRESH_INTERVAL_SECONDS: u64 = 5;
const LONG_WINDOW_REFRESH_INTERVAL_SECONDS: i64 = 60;
const SHORT_WINDOWS: [RoutingMetricWindow; 3] = [
    RoutingMetricWindow::OneMinute,
    RoutingMetricWindow::FiveMinutes,
    RoutingMetricWindow::FifteenMinutes,
];
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
    pub(crate) long_windows_refreshed_at: Option<time::OffsetDateTime>,
}

impl RoutingMetricsCache {
    pub(crate) async fn load(database: Database) -> Result<Self, LlmProxyError> {
        let snapshot = refresh_snapshot(&database, None).await?;
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
        let current = self.snapshot().await;
        let snapshot = refresh_snapshot(&self.database, Some(&current)).await?;
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

async fn refresh_snapshot(database: &Database, current: Option<&RoutingMetricsSnapshot>) -> Result<RoutingMetricsSnapshot, LlmProxyError> {
    let refreshed_at = time::OffsetDateTime::now_utc();
    let due_windows: &[RoutingMetricWindow] = match current {
        Some(snapshot) => windows_due_for_refresh(snapshot, refreshed_at),
        None => &WINDOWS,
    };
    let store = ProviderStore::new(database.clone());
    let mut windows = current.map_or_else(HashMap::new, |snapshot| snapshot.windows.clone());
    for window in due_windows {
        windows.insert(*window, store.list_routing_metrics(*window).await?);
    }
    let route_states = store.list_routing_route_states().await?;
    let context_route_states = store.list_routing_context_route_states().await?;
    Ok(RoutingMetricsSnapshot {
        windows,
        route_states,
        context_route_states,
        refreshed_at: Some(refreshed_at),
        long_windows_refreshed_at: long_windows_refreshed_at(current, due_windows, refreshed_at),
    })
}

fn windows_due_for_refresh(snapshot: &RoutingMetricsSnapshot, now: time::OffsetDateTime) -> &'static [RoutingMetricWindow] {
    match snapshot.long_windows_refreshed_at {
        Some(refreshed_at) if now - refreshed_at < time::Duration::seconds(LONG_WINDOW_REFRESH_INTERVAL_SECONDS) => &SHORT_WINDOWS,
        _ => &WINDOWS,
    }
}

fn long_windows_refreshed_at(
    current: Option<&RoutingMetricsSnapshot>,
    due_windows: &[RoutingMetricWindow],
    refreshed_at: time::OffsetDateTime,
) -> Option<time::OffsetDateTime> {
    (due_windows.len() == WINDOWS.len())
        .then_some(refreshed_at)
        .or_else(|| current.and_then(|snapshot| snapshot.long_windows_refreshed_at))
}

#[cfg(test)]
mod tests {
    use super::WINDOWS;

    #[test]
    fn routing_metrics_cache_loads_every_supported_window() {
        assert_eq!(WINDOWS.len(), 6);
    }

    #[test]
    fn cache_refreshes_long_windows_once_per_minute() {
        let snapshot = super::RoutingMetricsSnapshot {
            long_windows_refreshed_at: Some(time::OffsetDateTime::from_unix_timestamp(120).unwrap()),
            ..Default::default()
        };

        assert_eq!(
            super::windows_due_for_refresh(&snapshot, time::OffsetDateTime::from_unix_timestamp(125).unwrap()),
            super::SHORT_WINDOWS
        );
        assert_eq!(
            super::windows_due_for_refresh(&snapshot, time::OffsetDateTime::from_unix_timestamp(180).unwrap()),
            super::WINDOWS
        );
    }

    #[test]
    fn cache_refreshes_every_window_without_a_complete_snapshot() {
        let now = time::OffsetDateTime::from_unix_timestamp(120).unwrap();

        assert_eq!(super::windows_due_for_refresh(&super::RoutingMetricsSnapshot::default(), now), super::WINDOWS);
    }
}
