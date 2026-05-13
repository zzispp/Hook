use std::collections::{HashMap, HashSet};

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    StorageResult,
    api_token::api_token_records,
    model::global_models,
    provider::record::{provider_api_keys, provider_endpoints, providers},
    user::{UserColumn, UserEntity},
};

use super::record::{ProviderApiKeyRecord, ProviderEndpointRecord, ProviderRecord, RequestCandidateRecord, RequestRecordSummaryRecord};

#[derive(Default)]
pub(super) struct RecordRefs {
    pub(super) providers: HashMap<String, ProviderRecord>,
    pub(super) endpoints: HashMap<String, ProviderEndpointRecord>,
    pub(super) keys: HashMap<String, ProviderApiKeyRecord>,
    pub(super) tokens: HashMap<String, api_token_records::Model>,
    pub(super) users: HashMap<String, crate::user::UserRecord>,
    pub(super) models: HashMap<String, global_models::Model>,
}

pub(super) async fn load_refs(store: &super::ProviderStore, candidates: &[RequestCandidateRecord]) -> StorageResult<RecordRefs> {
    let provider_ids = ids(candidates.iter().filter_map(|item| item.provider_id.as_deref()));
    let endpoint_ids = ids(candidates.iter().filter_map(|item| item.endpoint_id.as_deref()));
    let key_ids = ids(candidates.iter().filter_map(|item| item.key_id.as_deref()));
    let token_ids = ids(candidates.iter().filter_map(|item| item.token_id.as_deref()));
    let model_ids = ids(candidates.iter().filter_map(|item| item.global_model_id.as_deref()));
    load_ref_records(store, provider_ids, endpoint_ids, key_ids, token_ids, model_ids).await
}

pub(super) async fn load_record_refs(store: &super::ProviderStore, records: &[RequestRecordSummaryRecord]) -> StorageResult<RecordRefs> {
    let provider_ids = ids(records.iter().filter_map(|item| item.provider_id.as_deref()));
    let endpoint_ids = ids(records.iter().filter_map(|item| item.endpoint_id.as_deref()));
    let key_ids = ids(records.iter().filter_map(|item| item.key_id.as_deref()));
    let token_ids = ids(records.iter().filter_map(|item| item.token_id.as_deref()));
    let model_ids = ids(records.iter().filter_map(|item| item.global_model_id.as_deref()));
    load_ref_records(store, provider_ids, endpoint_ids, key_ids, token_ids, model_ids).await
}

async fn load_ref_records(
    store: &super::ProviderStore,
    provider_ids: HashSet<String>,
    endpoint_ids: HashSet<String>,
    key_ids: HashSet<String>,
    token_ids: HashSet<String>,
    model_ids: HashSet<String>,
) -> StorageResult<RecordRefs> {
    let providers = records_by_id(
        providers::Entity::find()
            .filter(providers::Column::Id.is_in(provider_ids))
            .all(store.connection())
            .await?,
    );
    let endpoints = records_by_id(
        provider_endpoints::Entity::find()
            .filter(provider_endpoints::Column::Id.is_in(endpoint_ids))
            .all(store.connection())
            .await?,
    );
    let keys = records_by_id(
        provider_api_keys::Entity::find()
            .filter(provider_api_keys::Column::Id.is_in(key_ids))
            .all(store.connection())
            .await?,
    );
    let tokens = records_by_id(
        api_token_records::Entity::find()
            .filter(api_token_records::Column::Id.is_in(token_ids))
            .all(store.connection())
            .await?,
    );
    let user_ids = ids(tokens.values().filter_map(|item| item.user_id.as_deref()));
    let users = records_by_id(UserEntity::find().filter(UserColumn::Id.is_in(user_ids)).all(store.connection()).await?);
    let models = records_by_id(
        global_models::Entity::find()
            .filter(global_models::Column::Id.is_in(model_ids))
            .all(store.connection())
            .await?,
    );
    Ok(RecordRefs {
        providers,
        endpoints,
        keys,
        tokens,
        users,
        models,
    })
}

fn ids<'a>(values: impl Iterator<Item = &'a str>) -> HashSet<String> {
    values.map(str::to_owned).collect()
}

fn records_by_id<T>(items: Vec<T>) -> HashMap<String, T>
where
    T: HasId,
{
    items.into_iter().map(|item| (item.id().to_owned(), item)).collect()
}

trait HasId {
    fn id(&self) -> &str;
}

impl HasId for ProviderRecord {
    fn id(&self) -> &str {
        &self.id
    }
}

impl HasId for ProviderEndpointRecord {
    fn id(&self) -> &str {
        &self.id
    }
}

impl HasId for ProviderApiKeyRecord {
    fn id(&self) -> &str {
        &self.id
    }
}

impl HasId for api_token_records::Model {
    fn id(&self) -> &str {
        &self.id
    }
}

impl HasId for crate::user::UserRecord {
    fn id(&self) -> &str {
        &self.id
    }
}

impl HasId for global_models::Model {
    fn id(&self) -> &str {
        &self.id
    }
}
