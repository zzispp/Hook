mod bucket;
mod candidate;
mod common;
mod constants;
mod contribution;
mod recent_errors;
mod request;
mod state;
mod types;
mod writer;

use sea_orm::TransactionTrait;

use crate::{
    StorageResult,
    provider::record::{request_candidates, request_records},
};

use self::{
    bucket::BucketGranularity,
    constants::{SOURCE_CANDIDATE, SOURCE_REQUEST},
    contribution::{CandidateContribution, RequestContribution},
};

pub async fn sync_request_metric_buckets(
    connection: &sea_orm::DatabaseConnection,
    old_record: Option<&request_records::Model>,
    new_record: &request_records::Model,
) -> StorageResult<()> {
    let old_contribution = old_record.map(RequestContribution::from_record);
    let new_contribution = RequestContribution::from_record(new_record);
    let tx = connection.begin().await?;
    let applied = sync_contribution_state(&tx, SOURCE_REQUEST, &new_record.request_id, old_contribution.as_ref(), Some(&new_contribution)).await?;
    if applied {
        recent_errors::sync_recent_error_snapshot(&tx, new_record).await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn sync_candidate_metric_buckets(
    connection: &sea_orm::DatabaseConnection,
    old_record: Option<&request_candidates::Model>,
    new_record: &request_candidates::Model,
) -> StorageResult<()> {
    let old_contribution = old_record.and_then(CandidateContribution::from_record);
    let new_contribution = CandidateContribution::from_record(new_record);
    if old_contribution.is_none() && new_contribution.is_none() {
        return Ok(());
    }
    let tx = connection.begin().await?;
    sync_contribution_state(&tx, SOURCE_CANDIDATE, &new_record.id, old_contribution.as_ref(), new_contribution.as_ref()).await?;
    tx.commit().await?;
    Ok(())
}

async fn sync_contribution_state<C>(
    connection: &C,
    owner_type: &str,
    owner_id: &str,
    old_contribution: Option<&impl MetricSnapshotContribution>,
    new_contribution: Option<&impl MetricSnapshotContribution>,
) -> StorageResult<bool>
where
    C: sea_orm::ConnectionTrait,
{
    let state_exists = state::lock_exists(connection, owner_type, owner_id).await?;
    if !state_exists && old_contribution.is_some() {
        return Ok(false);
    }
    if state_exists {
        apply_delta(connection, old_contribution, -1).await?;
    }
    apply_delta(connection, new_contribution, 1).await?;
    match new_contribution {
        Some(_) => state::upsert(connection, owner_type, owner_id).await?,
        None if state_exists => state::delete(connection, owner_type, owner_id).await?,
        None => {}
    }
    Ok(true)
}

async fn apply_delta<C>(connection: &C, contribution: Option<&impl MetricSnapshotContribution>, multiplier: i64) -> StorageResult<()>
where
    C: sea_orm::ConnectionTrait,
{
    let Some(contribution) = contribution else {
        return Ok(());
    };
    for granularity in BucketGranularity::all() {
        writer::upsert_metric_delta(connection, contribution.metric(), granularity, multiplier).await?;
        writer::upsert_histogram_delta(connection, contribution.histogram(), granularity, multiplier).await?;
    }
    Ok(())
}

trait MetricSnapshotContribution {
    fn metric(&self) -> &types::MetricContribution;
    fn histogram(&self) -> &types::HistogramContribution;
}

impl MetricSnapshotContribution for RequestContribution {
    fn metric(&self) -> &types::MetricContribution {
        &self.metric
    }

    fn histogram(&self) -> &types::HistogramContribution {
        &self.histogram
    }
}

impl MetricSnapshotContribution for CandidateContribution {
    fn metric(&self) -> &types::MetricContribution {
        &self.metric
    }

    fn histogram(&self) -> &types::HistogramContribution {
        &self.histogram
    }
}
