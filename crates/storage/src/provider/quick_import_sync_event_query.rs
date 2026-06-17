use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use types::provider::{
    ProviderQuickImportSyncEventDetailResponse, ProviderQuickImportSyncEventPayload, ProviderQuickImportSyncEventSnapshotStatus,
};

use crate::{StorageError, StorageResult};

use super::{ProviderQuickImportSyncEventRecordInput, ProviderStore, record::provider_quick_import_sync_events};

pub async fn create_events(store: &ProviderStore, input: Vec<ProviderQuickImportSyncEventRecordInput>) -> StorageResult<()> {
    for event in input {
        event_active_model(store, event)?.insert(store.connection()).await?;
    }
    Ok(())
}

pub async fn event_detail(store: &ProviderStore, id: &str) -> StorageResult<Option<ProviderQuickImportSyncEventDetailResponse>> {
    provider_quick_import_sync_events::Entity::find_by_id(id.to_owned())
        .one(store.connection())
        .await?
        .map(event_detail_response)
        .transpose()
}

fn event_active_model(
    store: &ProviderStore,
    input: ProviderQuickImportSyncEventRecordInput,
) -> StorageResult<provider_quick_import_sync_events::ActiveModel> {
    Ok(provider_quick_import_sync_events::ActiveModel {
        id: Set(store.next_id()),
        provider_id: Set(input.provider_id),
        source_id: Set(input.source_id),
        key_id: Set(input.key_id),
        status: Set(input.status.as_str().to_owned()),
        title: Set(input.title),
        detail: Set(input.detail),
        payload_json: Set(input.payload.map(serde_json::to_value).transpose()?),
        created_at: Set(time::OffsetDateTime::now_utc()),
    })
}

fn event_detail_response(record: provider_quick_import_sync_events::Model) -> StorageResult<ProviderQuickImportSyncEventDetailResponse> {
    let status = record
        .status
        .as_str()
        .try_into()
        .map_err(|message: String| StorageError::Database(message))?;
    let payload = record
        .payload_json
        .map(serde_json::from_value::<ProviderQuickImportSyncEventPayload>)
        .transpose()
        .map_err(StorageError::from)?;
    Ok(ProviderQuickImportSyncEventDetailResponse {
        id: record.id,
        status,
        title: record.title,
        detail: record.detail,
        created_at: super::record::format_timestamp(record.created_at),
        snapshot_status: if payload.is_some() {
            ProviderQuickImportSyncEventSnapshotStatus::Available
        } else {
            ProviderQuickImportSyncEventSnapshotStatus::Missing
        },
        payload,
    })
}
