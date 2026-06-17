use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait};

use crate::StorageResult;

use super::{
    ProviderKeyModelMappingRecordInput, ProviderKeyModelMappingView, ProviderKeyModelMappingsForKeyRecord, ProviderKeyModelMappingsForProviderRecord,
    record::{provider_api_keys, provider_key_model_mappings, provider_models, provider_quick_import_keys},
    repository::ProviderStore,
};

pub async fn key_model_mappings_for_provider(store: &ProviderStore, provider_id: &str) -> StorageResult<Vec<ProviderKeyModelMappingsForProviderRecord>> {
    let keys = provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
        .order_by_asc(provider_api_keys::Column::InternalPriority)
        .all(store.connection())
        .await?;

    let mappings = provider_key_model_mappings::Entity::find()
        .filter(provider_key_model_mappings::Column::ProviderId.eq(provider_id))
        .order_by_asc(provider_key_model_mappings::Column::KeyId)
        .order_by_asc(provider_key_model_mappings::Column::ProviderModelId)
        .all(store.connection())
        .await?;

    let provider_models = provider_models::Entity::find()
        .filter(provider_models::Column::ProviderId.eq(provider_id))
        .all(store.connection())
        .await?;

    let quick_import_key_ids = provider_quick_import_keys::Entity::find()
        .filter(provider_quick_import_keys::Column::ProviderId.eq(provider_id))
        .all(store.connection())
        .await?
        .into_iter()
        .map(|record| record.key_id)
        .collect::<std::collections::BTreeSet<_>>();

    let global_by_provider_model = provider_models
        .into_iter()
        .map(|model| (model.id, model.global_model_id))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut by_key = std::collections::BTreeMap::<String, Vec<ProviderKeyModelMappingView>>::new();
    for record in mappings {
        let Some(global_model_id) = global_by_provider_model.get(&record.provider_model_id).cloned() else {
            continue;
        };
        by_key.entry(record.key_id.clone()).or_default().push(view(record, global_model_id));
    }

    Ok(keys
        .into_iter()
        .map(|key| ProviderKeyModelMappingsForProviderRecord {
            provider_id: key.provider_id,
            key_id: key.id.clone(),
            key_name: key.name,
            is_quick_import_key: quick_import_key_ids.contains(&key.id),
            mappings: by_key.remove(&key.id).unwrap_or_default(),
        })
        .collect())
}

pub async fn key_model_mappings_for_key(store: &ProviderStore, provider_id: &str, key_id: &str) -> StorageResult<Option<ProviderKeyModelMappingsForKeyRecord>> {
    let Some(key) = provider_api_keys::Entity::find_by_id(key_id.to_owned())
        .filter(provider_api_keys::Column::ProviderId.eq(provider_id))
        .one(store.connection())
        .await?
    else {
        return Ok(None);
    };

    let is_quick_import_key = provider_quick_import_keys::Entity::find()
        .filter(provider_quick_import_keys::Column::ProviderId.eq(provider_id))
        .filter(provider_quick_import_keys::Column::KeyId.eq(key_id))
        .one(store.connection())
        .await?
        .is_some();

    let provider_models = provider_models::Entity::find()
        .filter(provider_models::Column::ProviderId.eq(provider_id))
        .all(store.connection())
        .await?
        .into_iter()
        .map(|model| (model.id, model.global_model_id))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mappings = provider_key_model_mappings::Entity::find()
        .filter(provider_key_model_mappings::Column::ProviderId.eq(provider_id))
        .filter(provider_key_model_mappings::Column::KeyId.eq(key_id))
        .order_by_asc(provider_key_model_mappings::Column::ProviderModelId)
        .all(store.connection())
        .await?
        .into_iter()
        .filter_map(|record| {
            provider_models
                .get(&record.provider_model_id)
                .cloned()
                .map(|global_model_id| view(record, global_model_id))
        })
        .collect();

    Ok(Some(ProviderKeyModelMappingsForKeyRecord {
        provider_id: key.provider_id,
        key_id: key.id,
        key_name: key.name,
        is_quick_import_key,
        mappings,
    }))
}

pub async fn replace_key_model_mappings(
    store: &ProviderStore,
    provider_id: &str,
    key_id: &str,
    inputs: Vec<ProviderKeyModelMappingRecordInput>,
) -> StorageResult<Vec<ProviderKeyModelMappingView>> {
    let tx = store.connection().begin().await?;
    provider_key_model_mappings::Entity::delete_many()
        .filter(provider_key_model_mappings::Column::ProviderId.eq(provider_id))
        .filter(provider_key_model_mappings::Column::KeyId.eq(key_id))
        .exec(&tx)
        .await?;

    let now = time::OffsetDateTime::now_utc();
    for input in inputs {
        provider_key_model_mappings::ActiveModel {
            id: Set(store.next_id()),
            provider_id: Set(input.provider_id),
            key_id: Set(input.key_id),
            provider_model_id: Set(input.provider_model_id),
            upstream_model_name: Set(input.upstream_model_name),
            reasoning_effort: Set(input.reasoning_effort),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&tx)
        .await?;
    }
    tx.commit().await?;

    Ok(key_model_mappings_for_key(store, provider_id, key_id)
        .await?
        .map(|record| record.mappings)
        .unwrap_or_default())
}

pub async fn model_mappings_by_key_id(
    store: &ProviderStore,
    key_ids: &[String],
) -> StorageResult<std::collections::BTreeMap<String, Vec<ProviderKeyModelMappingView>>> {
    if key_ids.is_empty() {
        return Ok(std::collections::BTreeMap::new());
    }
    let records = provider_key_model_mappings::Entity::find()
        .filter(provider_key_model_mappings::Column::KeyId.is_in(key_ids.iter().cloned()))
        .all(store.connection())
        .await?;
    let provider_model_ids = records.iter().map(|record| record.provider_model_id.clone()).collect::<Vec<_>>();
    let models = provider_models::Entity::find()
        .filter(provider_models::Column::Id.is_in(provider_model_ids))
        .all(store.connection())
        .await?
        .into_iter()
        .map(|model| (model.id, model.global_model_id))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut output = std::collections::BTreeMap::<String, Vec<ProviderKeyModelMappingView>>::new();
    for record in records {
        let Some(global_model_id) = models.get(&record.provider_model_id).cloned() else {
            continue;
        };
        output.entry(record.key_id.clone()).or_default().push(view(record, global_model_id));
    }
    Ok(output)
}

fn view(record: provider_key_model_mappings::Model, global_model_id: String) -> ProviderKeyModelMappingView {
    ProviderKeyModelMappingView {
        id: record.id,
        provider_id: record.provider_id,
        key_id: record.key_id,
        provider_model_id: record.provider_model_id,
        global_model_id,
        upstream_model_name: record.upstream_model_name,
        reasoning_effort: record.reasoning_effort,
        created_at: record.created_at.to_string(),
        updated_at: record.updated_at.to_string(),
    }
}
