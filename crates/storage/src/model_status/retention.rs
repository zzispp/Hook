use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    StorageResult,
    model_status::{
        ModelStatusRetentionReport, ModelStatusStore,
        entities::{hourly_stats, runs},
    },
};

pub(super) async fn delete_history_before(store: &ModelStatusStore, cutoff: time::OffsetDateTime) -> StorageResult<ModelStatusRetentionReport> {
    let runs = runs::Entity::delete_many()
        .filter(runs::Column::CheckedAt.lt(cutoff))
        .exec(store.connection())
        .await?;
    let hourly_stats = hourly_stats::Entity::delete_many()
        .filter(hourly_stats::Column::BucketStartedAt.lt(cutoff))
        .exec(store.connection())
        .await?;
    Ok(ModelStatusRetentionReport {
        deleted_runs: runs.rows_affected,
        deleted_hourly_stats: hourly_stats.rows_affected,
    })
}
