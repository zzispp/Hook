use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet, hash_map::Entry},
};

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use types::provider::{ProviderListRequest, ProviderListResponse};

use crate::{StorageError, StorageResult};

use super::{
    ProviderStore,
    record::{ProviderRecord, provider_endpoints, provider_group_providers, provider_groups, provider_models, providers},
    repository_helpers::{ProviderFilterIds, filter_provider_records},
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ProviderGroupRank {
    sort_order: i64,
    name: String,
    id: String,
}

pub async fn list_providers(store: &ProviderStore, request: ProviderListRequest) -> StorageResult<ProviderListResponse> {
    let records = provider_records(store).await?;
    let ids = provider_filter_ids(store, &request).await?;
    let mut records = filter_provider_records(records, &request, ids);
    let total = records.len() as u64;
    let ranks = provider_sort_ranks(store).await?;
    sort_provider_records(&mut records, &ranks);
    let providers = records
        .into_iter()
        .skip(request.skip as usize)
        .take(request.limit as usize)
        .map(Into::into)
        .collect();
    Ok(ProviderListResponse { providers, total })
}

async fn provider_records(store: &ProviderStore) -> StorageResult<Vec<super::ProviderRecord>> {
    providers::Entity::find()
        .order_by_asc(providers::Column::Priority)
        .order_by_asc(providers::Column::Name)
        .all(store.connection())
        .await
        .map_err(StorageError::from)
}

async fn provider_sort_ranks(store: &ProviderStore) -> StorageResult<HashMap<String, ProviderGroupRank>> {
    let group_ranks = provider_group_ranks(store).await?;
    let memberships = provider_group_providers::Entity::find()
        .all(store.connection())
        .await
        .map_err(StorageError::from)?;
    let mut provider_ranks = HashMap::new();
    for membership in memberships {
        let Some(rank) = group_ranks.get(&membership.provider_group_id) else {
            continue;
        };
        set_provider_rank(&mut provider_ranks, membership.provider_id, rank.clone());
    }
    Ok(provider_ranks)
}

async fn provider_group_ranks(store: &ProviderStore) -> StorageResult<HashMap<String, ProviderGroupRank>> {
    let groups = provider_groups::Entity::find()
        .order_by_asc(provider_groups::Column::SortOrder)
        .order_by_asc(provider_groups::Column::Name)
        .order_by_asc(provider_groups::Column::Id)
        .all(store.connection())
        .await
        .map_err(StorageError::from)?;
    Ok(groups
        .into_iter()
        .map(|group| {
            let rank = ProviderGroupRank {
                sort_order: group.sort_order,
                name: group.name,
                id: group.id.clone(),
            };
            (group.id, rank)
        })
        .collect())
}

fn set_provider_rank(ranks: &mut HashMap<String, ProviderGroupRank>, provider_id: String, rank: ProviderGroupRank) {
    match ranks.entry(provider_id) {
        Entry::Vacant(entry) => {
            entry.insert(rank);
        }
        Entry::Occupied(mut entry) => {
            if rank < *entry.get() {
                entry.insert(rank);
            }
        }
    }
}

fn sort_provider_records(records: &mut [ProviderRecord], ranks: &HashMap<String, ProviderGroupRank>) {
    records.sort_by(|left, right| {
        compare_group_rank(ranks.get(&left.id), ranks.get(&right.id))
            .then_with(|| left.priority.cmp(&right.priority))
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn compare_group_rank(left: Option<&ProviderGroupRank>, right: Option<&ProviderGroupRank>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

async fn provider_filter_ids(store: &ProviderStore, request: &ProviderListRequest) -> StorageResult<ProviderFilterIds> {
    Ok(ProviderFilterIds {
        api_format: provider_ids_by_api_format(store, request.api_format.as_deref()).await?,
        model: provider_ids_by_model(store, request.model_id.as_deref()).await?,
    })
}

async fn provider_ids_by_api_format(store: &ProviderStore, api_format: Option<&str>) -> StorageResult<Option<HashSet<String>>> {
    let Some(api_format) = api_format else {
        return Ok(None);
    };
    let records = provider_endpoints::Entity::find()
        .filter(provider_endpoints::Column::ApiFormat.eq(api_format))
        .all(store.connection())
        .await?;
    Ok(Some(records.into_iter().map(|record| record.provider_id).collect()))
}

async fn provider_ids_by_model(store: &ProviderStore, model_id: Option<&str>) -> StorageResult<Option<HashSet<String>>> {
    let Some(model_id) = model_id else {
        return Ok(None);
    };
    let records = provider_models::Entity::find()
        .filter(provider_models::Column::GlobalModelId.eq(model_id))
        .all(store.connection())
        .await?;
    Ok(Some(records.into_iter().map(|record| record.provider_id).collect()))
}
