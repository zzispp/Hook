use async_trait::async_trait;
use storage::{
    Database, StorageError,
    dashboard::{
        DashboardBucketFilter, DashboardScopeFilter, DashboardStore, DashboardStoreActivityQuery, DashboardStoreFilterOptionsQuery, DashboardStoreOverviewQuery,
    },
};
use types::dashboard::{DashboardActivityResponse, DashboardFilterOptionsResponse, DashboardOverviewResponse};

use crate::application::{DashboardActivityQuery, DashboardError, DashboardFilterOptionsQuery, DashboardOverviewQuery, DashboardRepository, DashboardResult};

#[derive(Clone)]
pub struct StorageDashboardRepository {
    store: DashboardStore,
}

impl StorageDashboardRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: DashboardStore::new(database),
        }
    }
}

#[async_trait]
impl DashboardRepository for StorageDashboardRepository {
    async fn overview(&self, query: DashboardOverviewQuery) -> DashboardResult<DashboardOverviewResponse> {
        self.store.overview(store_overview_query(query)).await.map_err(storage_error)
    }

    async fn activity(&self, query: DashboardActivityQuery) -> DashboardResult<DashboardActivityResponse> {
        self.store.activity(store_activity_query(query)).await.map_err(storage_error)
    }

    async fn filter_options(&self, query: DashboardFilterOptionsQuery) -> DashboardResult<DashboardFilterOptionsResponse> {
        self.store.filter_options(store_filter_query(query)).await.map_err(storage_error)
    }
}

fn store_overview_query(query: DashboardOverviewQuery) -> DashboardStoreOverviewQuery {
    DashboardStoreOverviewQuery {
        preset: query.preset,
        scope: store_scope(query.scope),
        started_at: query.window.started_at,
        ended_at: query.window.ended_at,
        bucket: store_bucket(query.bucket),
        include_admin_breakdowns: query.admin,
        tz_offset_minutes: query.tz_offset_minutes,
    }
}

fn store_activity_query(query: DashboardActivityQuery) -> DashboardStoreActivityQuery {
    DashboardStoreActivityQuery {
        scope: store_scope(query.scope),
        start_date: query.start_date,
        end_date: query.end_date,
        started_at: query.started_at,
        ended_at: query.ended_at,
        tz_offset_minutes: query.tz_offset_minutes,
    }
}

fn store_filter_query(query: DashboardFilterOptionsQuery) -> DashboardStoreFilterOptionsQuery {
    DashboardStoreFilterOptionsQuery {
        scope: store_scope(query.scope),
    }
}

fn store_scope(scope: crate::application::DashboardScope) -> DashboardScopeFilter {
    match scope {
        crate::application::DashboardScope::Me { user_id } => DashboardScopeFilter::Me { user_id },
        crate::application::DashboardScope::Global => DashboardScopeFilter::Global,
        crate::application::DashboardScope::User { user_id } => DashboardScopeFilter::User { user_id },
        crate::application::DashboardScope::Token { token_id } => DashboardScopeFilter::Token { token_id },
    }
}

fn store_bucket(bucket: crate::application::DashboardBucket) -> DashboardBucketFilter {
    match bucket {
        crate::application::DashboardBucket::Hour => DashboardBucketFilter::Hour,
        crate::application::DashboardBucket::Day => DashboardBucketFilter::Day,
    }
}

fn storage_error(error: StorageError) -> DashboardError {
    DashboardError::Infrastructure(error.to_string())
}
