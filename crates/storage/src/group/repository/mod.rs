mod binding_maps;
mod bindings;
mod mapping;

use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait};
use types::group::{BillingGroup, BillingGroupListRequest};
use types::model::PatchField;

use crate::{Database, StorageError, StorageResult};

use super::{
    BillingGroupRecordInput, BillingGroupRecordPatch,
    record::{BillingGroupRecord, billing_group_user_groups, billing_groups},
};

use bindings::{
    delete_group_bindings, model_ids_by_group_codes, model_ids_for_group, provider_key_group_ids_by_group_codes, provider_key_group_ids_for_group,
    replace_group_models, replace_group_provider_key_groups, replace_group_user_groups, user_group_codes_by_group_codes, user_group_codes_for_group,
};
use mapping::{apply_group_patch, domain_group, group_active_model};

#[derive(Clone)]
pub struct GroupStore {
    pub(crate) database: Database,
}

impl GroupStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create_group(&self, input: BillingGroupRecordInput) -> StorageResult<BillingGroup> {
        let bindings = GroupBindingInput::from(&input);
        let tx = self.database.connection().begin().await?;
        let record = group_active_model(self.database.next_id(), input).insert(&tx).await?;
        replace_all_bindings(&record.code, bindings, self, &tx).await?;
        tx.commit().await?;
        self.find_group(&record.id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn update_group(&self, id: &str, input: BillingGroupRecordPatch) -> StorageResult<BillingGroup> {
        let record = self.find_group_record(id).await?.ok_or(StorageError::NotFound)?;
        let bindings = GroupBindingPatch::from(&input);
        let tx = self.database.connection().begin().await?;
        let mut active: billing_groups::ActiveModel = record.into();
        apply_group_patch(&mut active, input);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        let updated = active.update(&tx).await?;
        replace_patched_bindings(&updated.code, bindings, self, &tx).await?;
        tx.commit().await?;
        self.find_group(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_group(&self, id: &str) -> StorageResult<()> {
        let record = self.find_group_record(id).await?.ok_or(StorageError::NotFound)?;
        let tx = self.database.connection().begin().await?;
        delete_group_bindings(&record.code, &tx).await?;
        let active: billing_groups::ActiveModel = record.into();
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

    pub async fn active_groups_for_user_groups(&self, user_group_codes: &[String]) -> StorageResult<Vec<BillingGroup>> {
        let user_group_codes = user_group_codes.iter().collect::<std::collections::BTreeSet<_>>();
        let groups = self.active_groups().await?;
        Ok(groups
            .into_iter()
            .filter(|group| group.visible_user_group_codes.iter().any(|code| user_group_codes.contains(code)))
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
        let key_group_ids = provider_key_group_ids_for_group(&record.code, self.database.connection()).await?;
        let user_group_codes = user_group_codes_for_group(&record.code, self.database.connection()).await?;
        Ok(domain_group(record, model_ids, key_group_ids, user_group_codes))
    }

    async fn groups_from_records(&self, records: Vec<BillingGroupRecord>) -> StorageResult<Vec<BillingGroup>> {
        let codes = records.iter().map(|record| record.code.clone()).collect::<Vec<_>>();
        let model_bindings = model_ids_by_group_codes(codes.clone(), self.database.connection()).await?;
        let key_group_bindings = provider_key_group_ids_by_group_codes(codes.clone(), self.database.connection()).await?;
        let user_group_bindings = user_group_codes_by_group_codes(codes, self.database.connection()).await?;
        Ok(records
            .into_iter()
            .map(|record| {
                let code = record.code.clone();
                domain_group(
                    record,
                    model_bindings.get(&code).cloned().unwrap_or_default(),
                    key_group_bindings.get(&code).cloned().unwrap_or_default(),
                    user_group_bindings.get(&code).cloned().unwrap_or_default(),
                )
            })
            .collect())
    }
}

struct GroupBindingInput {
    model_ids: Vec<String>,
    provider_key_group_ids: Vec<String>,
    user_group_codes: Vec<String>,
}

struct GroupBindingPatch {
    model_ids: PatchField<Vec<String>>,
    provider_key_group_ids: PatchField<Vec<String>>,
    user_group_codes: PatchField<Vec<String>>,
}

impl From<&BillingGroupRecordInput> for GroupBindingInput {
    fn from(input: &BillingGroupRecordInput) -> Self {
        Self {
            model_ids: input.allowed_model_ids.clone(),
            provider_key_group_ids: input.allowed_provider_key_group_ids.clone(),
            user_group_codes: input.visible_user_group_codes.clone(),
        }
    }
}

impl From<&BillingGroupRecordPatch> for GroupBindingPatch {
    fn from(input: &BillingGroupRecordPatch) -> Self {
        Self {
            model_ids: input.allowed_model_ids.clone(),
            provider_key_group_ids: input.allowed_provider_key_group_ids.clone(),
            user_group_codes: input.visible_user_group_codes.clone(),
        }
    }
}

async fn replace_all_bindings(group_code: &str, input: GroupBindingInput, store: &GroupStore, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    replace_group_models(group_code, input.model_ids, store, tx).await?;
    replace_group_provider_key_groups(group_code, input.provider_key_group_ids, store, tx).await?;
    replace_group_user_groups(group_code, input.user_group_codes, store, tx).await
}

async fn replace_patched_bindings(group_code: &str, patch: GroupBindingPatch, store: &GroupStore, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    if let PatchField::Value(model_ids) = patch.model_ids {
        replace_group_models(group_code, model_ids, store, tx).await?;
    }
    if let PatchField::Value(key_group_ids) = patch.provider_key_group_ids {
        replace_group_provider_key_groups(group_code, key_group_ids, store, tx).await?;
    }
    if let PatchField::Value(user_group_codes) = patch.user_group_codes {
        replace_group_user_groups(group_code, user_group_codes, store, tx).await?;
    }
    Ok(())
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
