use std::collections::BTreeMap;

use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait};
use types::{
    group::{BillingGroup, BillingGroupListRequest},
    model::PatchField,
};

use crate::{Database, StorageError, StorageResult};

use super::{
    BillingGroupRecordInput, BillingGroupRecordPatch,
    record::billing_group_models::ActiveModel as BillingGroupModelActiveModel,
    record::billing_group_providers::ActiveModel as BillingGroupProviderActiveModel,
    record::billing_group_user_groups::ActiveModel as BillingGroupUserGroupActiveModel,
    record::billing_groups::ActiveModel as BillingGroupActiveModel,
    record::{
        BillingGroupModelRecord, BillingGroupProviderRecord, BillingGroupRecord, BillingGroupUserGroupRecord, billing_group_models, billing_group_providers,
        billing_group_user_groups, billing_groups,
    },
};

#[derive(Clone)]
pub struct GroupStore {
    database: Database,
}

impl GroupStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create_group(&self, input: BillingGroupRecordInput) -> StorageResult<BillingGroup> {
        let model_ids = input.allowed_model_ids.clone();
        let provider_ids = input.allowed_provider_ids.clone();
        let user_group_codes = input.visible_user_group_codes.clone();
        let tx = self.database.connection().begin().await?;
        let record = group_active_model(self.database.next_id(), input).insert(&tx).await?;
        replace_group_models(&record.code, model_ids, self, &tx).await?;
        replace_group_providers(&record.code, provider_ids, self, &tx).await?;
        replace_group_user_groups(&record.code, user_group_codes, self, &tx).await?;
        tx.commit().await?;
        self.find_group(&record.id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn update_group(&self, id: &str, input: BillingGroupRecordPatch) -> StorageResult<BillingGroup> {
        let record = self.find_group_record(id).await?.ok_or(StorageError::NotFound)?;
        let model_patch = input.allowed_model_ids.clone();
        let provider_patch = input.allowed_provider_ids.clone();
        let user_group_patch = input.visible_user_group_codes.clone();
        let tx = self.database.connection().begin().await?;
        let mut active: BillingGroupActiveModel = record.into();
        apply_group_patch(&mut active, input);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        let updated = active.update(&tx).await?;
        if let PatchField::Value(model_ids) = model_patch {
            replace_group_models(&updated.code, model_ids, self, &tx).await?;
        }
        if let PatchField::Value(provider_ids) = provider_patch {
            replace_group_providers(&updated.code, provider_ids, self, &tx).await?;
        }
        if let PatchField::Value(user_group_codes) = user_group_patch {
            replace_group_user_groups(&updated.code, user_group_codes, self, &tx).await?;
        }
        tx.commit().await?;
        self.find_group(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_group(&self, id: &str) -> StorageResult<()> {
        let record = self.find_group_record(id).await?.ok_or(StorageError::NotFound)?;
        let tx = self.database.connection().begin().await?;
        billing_group_models::Entity::delete_many()
            .filter(billing_group_models::Column::GroupCode.eq(record.code.as_str()))
            .exec(&tx)
            .await?;
        billing_group_providers::Entity::delete_many()
            .filter(billing_group_providers::Column::GroupCode.eq(record.code.as_str()))
            .exec(&tx)
            .await?;
        billing_group_user_groups::Entity::delete_many()
            .filter(billing_group_user_groups::Column::BillingGroupCode.eq(record.code.as_str()))
            .exec(&tx)
            .await?;
        let active: BillingGroupActiveModel = record.into();
        active.delete(&tx).await?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn find_group(&self, id_or_code: &str) -> StorageResult<Option<BillingGroup>> {
        match self.find_group_record(id_or_code).await? {
            Some(record) => self.group_from_record(record).await.map(Some),
            None => Ok(None),
        }
    }

    pub async fn list_groups(&self, request: BillingGroupListRequest) -> StorageResult<types::group::BillingGroupListResponse> {
        let records = filtered_groups(request.clone())
            .order_by_asc(billing_groups::Column::SortOrder)
            .order_by_asc(billing_groups::Column::Code)
            .all(self.database.connection())
            .await?;
        let total = records.len() as u64;
        let page = records.into_iter().skip(request.skip as usize).take(request.limit as usize).collect();
        let groups = self.groups_from_records(page).await?.into_iter().map(Into::into).collect();
        Ok(types::group::BillingGroupListResponse { groups, total })
    }

    pub async fn active_groups(&self) -> StorageResult<Vec<BillingGroup>> {
        let records = billing_groups::Entity::find()
            .filter(billing_groups::Column::IsActive.eq(true))
            .order_by_asc(billing_groups::Column::SortOrder)
            .order_by_asc(billing_groups::Column::Code)
            .all(self.database.connection())
            .await?;
        self.groups_from_records(records).await
    }

    pub async fn active_groups_for_user_group(&self, user_group_code: &str) -> StorageResult<Vec<BillingGroup>> {
        let groups = self.active_groups().await?;
        Ok(groups
            .into_iter()
            .filter(|group| group.visible_user_group_codes.iter().any(|code| code == user_group_code))
            .collect())
    }

    pub async fn user_group_has_billing_groups(&self, user_group_code: &str) -> StorageResult<bool> {
        billing_group_user_groups::Entity::find()
            .filter(billing_group_user_groups::Column::UserGroupCode.eq(user_group_code))
            .one(self.database.connection())
            .await
            .map(|record| record.is_some())
            .map_err(StorageError::from)
    }

    pub async fn group_has_tokens(&self, code: &str) -> StorageResult<bool> {
        crate::api_token::api_token_records::Entity::find()
            .filter(crate::api_token::api_token_records::Column::GroupCode.eq(code))
            .one(self.database.connection())
            .await
            .map(|record| record.is_some())
            .map_err(StorageError::from)
    }

    async fn find_group_record(&self, id_or_code: &str) -> StorageResult<Option<BillingGroupRecord>> {
        let by_id = billing_groups::Entity::find_by_id(id_or_code.to_owned())
            .one(self.database.connection())
            .await?;
        match by_id {
            Some(record) => Ok(Some(record)),
            None => self.find_group_record_by_code(id_or_code).await,
        }
    }

    async fn find_group_record_by_code(&self, code: &str) -> StorageResult<Option<BillingGroupRecord>> {
        billing_groups::Entity::find()
            .filter(billing_groups::Column::Code.eq(code))
            .one(self.database.connection())
            .await
            .map_err(StorageError::from)
    }

    async fn group_from_record(&self, record: BillingGroupRecord) -> StorageResult<BillingGroup> {
        let model_ids = model_ids_for_group(&record.code, self.database.connection()).await?;
        let provider_ids = provider_ids_for_group(&record.code, self.database.connection()).await?;
        let user_group_codes = user_group_codes_for_group(&record.code, self.database.connection()).await?;
        Ok(domain_group(record, model_ids, provider_ids, user_group_codes))
    }

    async fn groups_from_records(&self, records: Vec<BillingGroupRecord>) -> StorageResult<Vec<BillingGroup>> {
        let codes = records.iter().map(|record| record.code.clone()).collect::<Vec<_>>();
        let model_bindings = model_ids_by_group_codes(codes.clone(), self.database.connection()).await?;
        let provider_bindings = provider_ids_by_group_codes(codes.clone(), self.database.connection()).await?;
        let user_group_bindings = user_group_codes_by_group_codes(codes, self.database.connection()).await?;
        Ok(records
            .into_iter()
            .map(|record| {
                let model_ids = model_bindings.get(&record.code).cloned().unwrap_or_default();
                let provider_ids = provider_bindings.get(&record.code).cloned().unwrap_or_default();
                let user_group_codes = user_group_bindings.get(&record.code).cloned().unwrap_or_default();
                domain_group(record, model_ids, provider_ids, user_group_codes)
            })
            .collect())
    }
}

fn filtered_groups(request: BillingGroupListRequest) -> sea_orm::Select<billing_groups::Entity> {
    let mut query = billing_groups::Entity::find();
    if let Some(is_active) = request.is_active {
        query = query.filter(billing_groups::Column::IsActive.eq(is_active));
    }
    match request.search {
        Some(search) if !search.is_empty() => query.filter(group_search_condition(&search)),
        _ => query,
    }
}

fn group_search_condition(search: &str) -> Condition {
    Condition::any()
        .add(billing_groups::Column::Code.contains(search))
        .add(billing_groups::Column::Name.contains(search))
        .add(billing_groups::Column::Description.contains(search))
}

fn group_active_model(id: String, input: BillingGroupRecordInput) -> BillingGroupActiveModel {
    let now = time::OffsetDateTime::now_utc();
    BillingGroupActiveModel {
        id: Set(id),
        code: Set(input.code),
        name: Set(input.name),
        description: Set(input.description),
        billing_multiplier: Set(input.billing_multiplier),
        is_active: Set(input.is_active),
        is_system: Set(input.is_system),
        sort_order: Set(input.sort_order),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

async fn replace_group_user_groups(
    group_code: &str,
    user_group_codes: Vec<String>,
    store: &GroupStore,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    billing_group_user_groups::Entity::delete_many()
        .filter(billing_group_user_groups::Column::BillingGroupCode.eq(group_code))
        .exec(tx)
        .await?;
    insert_group_user_groups(group_code, user_group_codes, store, tx).await
}

async fn insert_group_user_groups(group_code: &str, user_group_codes: Vec<String>, store: &GroupStore, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    if user_group_codes.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = user_group_codes
        .into_iter()
        .map(|user_group_code| group_user_group_active_model(store.database.next_id(), group_code, user_group_code, now));
    billing_group_user_groups::Entity::insert_many(records).exec(tx).await?;
    Ok(())
}

async fn replace_group_models(group_code: &str, model_ids: Vec<String>, store: &GroupStore, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    billing_group_models::Entity::delete_many()
        .filter(billing_group_models::Column::GroupCode.eq(group_code))
        .exec(tx)
        .await?;
    insert_group_models(group_code, model_ids, store, tx).await
}

async fn insert_group_models(group_code: &str, model_ids: Vec<String>, store: &GroupStore, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    if model_ids.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = model_ids
        .into_iter()
        .map(|model_id| group_model_active_model(store.database.next_id(), group_code, model_id, now));
    billing_group_models::Entity::insert_many(records).exec(tx).await?;
    Ok(())
}

async fn replace_group_providers(group_code: &str, provider_ids: Vec<String>, store: &GroupStore, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    billing_group_providers::Entity::delete_many()
        .filter(billing_group_providers::Column::GroupCode.eq(group_code))
        .exec(tx)
        .await?;
    insert_group_providers(group_code, provider_ids, store, tx).await
}

async fn insert_group_providers(group_code: &str, provider_ids: Vec<String>, store: &GroupStore, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    if provider_ids.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = provider_ids
        .into_iter()
        .map(|provider_id| group_provider_active_model(store.database.next_id(), group_code, provider_id, now));
    billing_group_providers::Entity::insert_many(records).exec(tx).await?;
    Ok(())
}

fn group_model_active_model(id: String, group_code: &str, model_id: String, now: time::OffsetDateTime) -> BillingGroupModelActiveModel {
    BillingGroupModelActiveModel {
        id: Set(id),
        group_code: Set(group_code.to_owned()),
        global_model_id: Set(model_id),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

fn group_provider_active_model(id: String, group_code: &str, provider_id: String, now: time::OffsetDateTime) -> BillingGroupProviderActiveModel {
    BillingGroupProviderActiveModel {
        id: Set(id),
        group_code: Set(group_code.to_owned()),
        provider_id: Set(provider_id),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

fn group_user_group_active_model(id: String, group_code: &str, user_group_code: String, now: time::OffsetDateTime) -> BillingGroupUserGroupActiveModel {
    BillingGroupUserGroupActiveModel {
        id: Set(id),
        billing_group_code: Set(group_code.to_owned()),
        user_group_code: Set(user_group_code),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

async fn model_ids_for_group(group_code: &str, db: &sea_orm::DatabaseConnection) -> StorageResult<Vec<String>> {
    billing_group_models::Entity::find()
        .filter(billing_group_models::Column::GroupCode.eq(group_code))
        .order_by_asc(billing_group_models::Column::GlobalModelId)
        .all(db)
        .await
        .map(binding_model_ids)
        .map_err(StorageError::from)
}

async fn provider_ids_for_group(group_code: &str, db: &sea_orm::DatabaseConnection) -> StorageResult<Vec<String>> {
    billing_group_providers::Entity::find()
        .filter(billing_group_providers::Column::GroupCode.eq(group_code))
        .order_by_asc(billing_group_providers::Column::ProviderId)
        .all(db)
        .await
        .map(binding_provider_ids)
        .map_err(StorageError::from)
}

async fn user_group_codes_for_group(group_code: &str, db: &sea_orm::DatabaseConnection) -> StorageResult<Vec<String>> {
    billing_group_user_groups::Entity::find()
        .filter(billing_group_user_groups::Column::BillingGroupCode.eq(group_code))
        .order_by_asc(billing_group_user_groups::Column::UserGroupCode)
        .all(db)
        .await
        .map(binding_user_group_codes)
        .map_err(StorageError::from)
}

async fn model_ids_by_group_codes(codes: Vec<String>, db: &sea_orm::DatabaseConnection) -> StorageResult<BTreeMap<String, Vec<String>>> {
    if codes.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = billing_group_models::Entity::find()
        .filter(billing_group_models::Column::GroupCode.is_in(codes))
        .order_by_asc(billing_group_models::Column::GlobalModelId)
        .all(db)
        .await?;
    Ok(bindings_by_group(records))
}

async fn provider_ids_by_group_codes(codes: Vec<String>, db: &sea_orm::DatabaseConnection) -> StorageResult<BTreeMap<String, Vec<String>>> {
    if codes.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = billing_group_providers::Entity::find()
        .filter(billing_group_providers::Column::GroupCode.is_in(codes))
        .order_by_asc(billing_group_providers::Column::ProviderId)
        .all(db)
        .await?;
    Ok(provider_bindings_by_group(records))
}

async fn user_group_codes_by_group_codes(codes: Vec<String>, db: &sea_orm::DatabaseConnection) -> StorageResult<BTreeMap<String, Vec<String>>> {
    if codes.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = billing_group_user_groups::Entity::find()
        .filter(billing_group_user_groups::Column::BillingGroupCode.is_in(codes))
        .order_by_asc(billing_group_user_groups::Column::UserGroupCode)
        .all(db)
        .await?;
    Ok(user_group_bindings_by_group(records))
}

fn bindings_by_group(records: Vec<BillingGroupModelRecord>) -> BTreeMap<String, Vec<String>> {
    let mut bindings = BTreeMap::<String, Vec<String>>::new();
    for record in records {
        bindings.entry(record.group_code).or_default().push(record.global_model_id);
    }
    bindings
}

fn provider_bindings_by_group(records: Vec<BillingGroupProviderRecord>) -> BTreeMap<String, Vec<String>> {
    let mut bindings = BTreeMap::<String, Vec<String>>::new();
    for record in records {
        bindings.entry(record.group_code).or_default().push(record.provider_id);
    }
    bindings
}

fn user_group_bindings_by_group(records: Vec<BillingGroupUserGroupRecord>) -> BTreeMap<String, Vec<String>> {
    let mut bindings = BTreeMap::<String, Vec<String>>::new();
    for record in records {
        bindings.entry(record.billing_group_code).or_default().push(record.user_group_code);
    }
    bindings
}

fn binding_model_ids(records: Vec<BillingGroupModelRecord>) -> Vec<String> {
    records.into_iter().map(|record| record.global_model_id).collect()
}

fn binding_provider_ids(records: Vec<BillingGroupProviderRecord>) -> Vec<String> {
    records.into_iter().map(|record| record.provider_id).collect()
}

fn binding_user_group_codes(records: Vec<BillingGroupUserGroupRecord>) -> Vec<String> {
    records.into_iter().map(|record| record.user_group_code).collect()
}

fn domain_group(
    record: BillingGroupRecord,
    allowed_model_ids: Vec<String>,
    allowed_provider_ids: Vec<String>,
    visible_user_group_codes: Vec<String>,
) -> BillingGroup {
    BillingGroup {
        allowed_model_ids,
        allowed_provider_ids,
        visible_user_group_codes,
        ..BillingGroup::from(record)
    }
}

fn apply_group_patch(active: &mut BillingGroupActiveModel, input: BillingGroupRecordPatch) {
    if let Some(name) = input.name {
        active.name = Set(name);
    }
    match input.description {
        PatchField::Value(value) => active.description = Set(Some(value)),
        PatchField::Null => active.description = Set(None),
        PatchField::Missing => {}
    }
    if let Some(multiplier) = input.billing_multiplier {
        active.billing_multiplier = Set(multiplier);
    }
    if let Some(is_active) = input.is_active {
        active.is_active = Set(is_active);
    }
    if let Some(sort_order) = input.sort_order {
        active.sort_order = Set(sort_order);
    }
}
