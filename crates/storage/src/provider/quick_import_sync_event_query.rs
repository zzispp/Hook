use sea_orm::{ActiveModelTrait, Set};

use crate::StorageResult;

use super::{ProviderQuickImportSyncEventRecordInput, ProviderStore, record::provider_quick_import_sync_events};

pub async fn create_events(store: &ProviderStore, input: Vec<ProviderQuickImportSyncEventRecordInput>) -> StorageResult<()> {
    for event in input {
        event_active_model(store, event).insert(store.connection()).await?;
    }
    Ok(())
}

fn event_active_model(store: &ProviderStore, input: ProviderQuickImportSyncEventRecordInput) -> provider_quick_import_sync_events::ActiveModel {
    provider_quick_import_sync_events::ActiveModel {
        id: Set(store.next_id()),
        provider_id: Set(input.provider_id),
        source_id: Set(input.source_id),
        key_id: Set(input.key_id),
        status: Set(input.status.as_str().to_owned()),
        title: Set(input.title),
        detail: Set(input.detail),
        created_at: Set(time::OffsetDateTime::now_utc()),
    }
}
