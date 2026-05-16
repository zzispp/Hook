use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::StorageResult;

use super::{PerformanceMonitoringStore, record::snapshots};

pub(super) async fn delete_snapshots_before(store: &PerformanceMonitoringStore, cutoff: time::OffsetDateTime) -> StorageResult<u64> {
    let result = snapshots::Entity::delete_many()
        .filter(snapshots::Column::BucketStartedAt.lt(cutoff))
        .exec(store.connection())
        .await?;
    Ok(result.rows_affected)
}
