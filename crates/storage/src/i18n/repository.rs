use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Set};
use types::i18n::{
    I18nResourceResponse, TranslationEntry, TranslationEntryListRequest, TranslationEntryListResponse, TranslationEntryResponse, TranslationLanguage,
    TranslationLanguageListRequest, TranslationLanguageListResponse, TranslationLanguageResponse,
};

use crate::{Database, StorageError, StorageResult};

use super::{
    TranslationEntryRecordInput, TranslationEntryRecordPatch, TranslationLanguageRecordInput, TranslationLanguageRecordPatch,
    record::{
        TranslationEntryRecord, TranslationLanguageRecord,
        translation_entries::{self, ActiveModel as TranslationEntryActiveModel},
        translation_languages::{self, ActiveModel as TranslationLanguageActiveModel},
    },
    resource::resource_json,
};

#[derive(Clone)]
pub struct I18nStore {
    database: Database,
}

impl I18nStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn resource_bundle(&self, lang: &str, namespace: &str) -> StorageResult<I18nResourceResponse> {
        let records = translation_entries::Entity::find()
            .filter(translation_entries::Column::LangCode.eq(lang))
            .filter(translation_entries::Column::Namespace.eq(namespace))
            .filter(translation_entries::Column::Enabled.eq(true))
            .order_by_asc(translation_entries::Column::GroupKey)
            .order_by_asc(translation_entries::Column::ItemKey)
            .all(self.database.connection())
            .await?;

        Ok(I18nResourceResponse {
            lang: lang.to_owned(),
            namespace: namespace.to_owned(),
            resources: resource_json(records),
        })
    }

    pub async fn list_languages(&self, request: TranslationLanguageListRequest) -> StorageResult<TranslationLanguageListResponse> {
        let records = filtered_languages(request.clone())
            .order_by_asc(translation_languages::Column::SortOrder)
            .order_by_asc(translation_languages::Column::Code)
            .all(self.database.connection())
            .await?;
        let total = records.len() as u64;
        let languages = records
            .into_iter()
            .skip(request.skip as usize)
            .take(request.limit as usize)
            .map(TranslationLanguage::from)
            .map(TranslationLanguageResponse::from)
            .collect();
        Ok(TranslationLanguageListResponse { languages, total })
    }

    pub async fn create_language(&self, input: TranslationLanguageRecordInput) -> StorageResult<TranslationLanguageResponse> {
        let code = input.code.clone();
        language_active_model(input).insert(self.database.connection()).await?;
        self.find_language_response(&code).await?.ok_or(StorageError::NotFound)
    }

    pub async fn update_language(&self, code: &str, input: TranslationLanguageRecordPatch) -> StorageResult<TranslationLanguageResponse> {
        let record = self.find_language_record(code).await?.ok_or(StorageError::NotFound)?;
        let mut active: TranslationLanguageActiveModel = record.into();
        apply_language_patch(&mut active, input);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?;
        self.find_language_response(code).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_language(&self, code: &str) -> StorageResult<()> {
        let record = self.find_language_record(code).await?.ok_or(StorageError::NotFound)?;
        let active: TranslationLanguageActiveModel = record.into();
        active.delete(self.database.connection()).await?;
        Ok(())
    }

    pub async fn find_language(&self, code: &str) -> StorageResult<Option<TranslationLanguage>> {
        self.find_language_record(code).await.map(|record| record.map(Into::into))
    }

    pub async fn list_entries(&self, request: TranslationEntryListRequest) -> StorageResult<TranslationEntryListResponse> {
        let records = filtered_entries(request.clone())
            .order_by_asc(translation_entries::Column::Namespace)
            .order_by_asc(translation_entries::Column::GroupKey)
            .order_by_asc(translation_entries::Column::ItemKey)
            .order_by_asc(translation_entries::Column::LangCode)
            .all(self.database.connection())
            .await?;
        let total = records.len() as u64;
        let translations = records
            .into_iter()
            .skip(request.skip as usize)
            .take(request.limit as usize)
            .map(TranslationEntry::from)
            .map(TranslationEntryResponse::from)
            .collect();
        Ok(TranslationEntryListResponse { translations, total })
    }

    pub async fn create_entry(&self, input: TranslationEntryRecordInput) -> StorageResult<TranslationEntryResponse> {
        let record = entry_active_model(self.database.next_id(), input).insert(self.database.connection()).await?;
        Ok(TranslationEntryResponse::from(TranslationEntry::from(record)))
    }

    pub async fn update_entry(&self, id: &str, input: TranslationEntryRecordPatch) -> StorageResult<TranslationEntryResponse> {
        let record = self.find_entry_record(id).await?.ok_or(StorageError::NotFound)?;
        let mut active: TranslationEntryActiveModel = record.into();
        apply_entry_patch(&mut active, input);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        let updated = active.update(self.database.connection()).await?;
        Ok(TranslationEntryResponse::from(TranslationEntry::from(updated)))
    }

    pub async fn delete_entry(&self, id: &str) -> StorageResult<()> {
        let record = self.find_entry_record(id).await?.ok_or(StorageError::NotFound)?;
        let active: TranslationEntryActiveModel = record.into();
        active.delete(self.database.connection()).await?;
        Ok(())
    }

    pub async fn upsert_entry(&self, input: TranslationEntryRecordInput) -> StorageResult<TranslationEntryResponse> {
        match self
            .find_entry_by_key(&input.namespace, &input.group_key, &input.item_key, &input.lang_code)
            .await?
        {
            Some(record) => {
                let id = record.id.clone();
                self.update_entry(
                    &id,
                    TranslationEntryRecordPatch {
                        value: Some(input.value),
                        description: Some(input.description),
                        enabled: Some(input.enabled),
                    },
                )
                .await
            }
            None => self.create_entry(input).await,
        }
    }

    pub async fn entry_exists(&self, namespace: &str, group_key: &str, item_key: &str, lang_code: &str) -> StorageResult<bool> {
        self.find_entry_by_key(namespace, group_key, item_key, lang_code)
            .await
            .map(|record| record.is_some())
    }

    async fn find_language_response(&self, code: &str) -> StorageResult<Option<TranslationLanguageResponse>> {
        self.find_language(code).await.map(|record| record.map(TranslationLanguageResponse::from))
    }

    async fn find_language_record(&self, code: &str) -> StorageResult<Option<TranslationLanguageRecord>> {
        translation_languages::Entity::find_by_id(code.to_owned())
            .one(self.database.connection())
            .await
            .map_err(StorageError::from)
    }

    async fn find_entry_record(&self, id: &str) -> StorageResult<Option<TranslationEntryRecord>> {
        translation_entries::Entity::find_by_id(id.to_owned())
            .one(self.database.connection())
            .await
            .map_err(StorageError::from)
    }

    async fn find_entry_by_key(&self, namespace: &str, group_key: &str, item_key: &str, lang_code: &str) -> StorageResult<Option<TranslationEntryRecord>> {
        translation_entries::Entity::find()
            .filter(translation_entries::Column::Namespace.eq(namespace))
            .filter(translation_entries::Column::GroupKey.eq(group_key))
            .filter(translation_entries::Column::ItemKey.eq(item_key))
            .filter(translation_entries::Column::LangCode.eq(lang_code))
            .one(self.database.connection())
            .await
            .map_err(StorageError::from)
    }
}

fn filtered_languages(request: TranslationLanguageListRequest) -> sea_orm::Select<translation_languages::Entity> {
    let mut query = translation_languages::Entity::find();
    if let Some(enabled) = request.enabled {
        query = query.filter(translation_languages::Column::Enabled.eq(enabled));
    }
    match request.search {
        Some(search) if !search.is_empty() => query.filter(language_search_condition(&search)),
        _ => query,
    }
}

fn filtered_entries(request: TranslationEntryListRequest) -> sea_orm::Select<translation_entries::Entity> {
    let mut query = translation_entries::Entity::find();
    if let Some(namespace) = request.namespace {
        query = query.filter(translation_entries::Column::Namespace.eq(namespace));
    }
    if let Some(group_key) = request.group_key {
        query = query.filter(translation_entries::Column::GroupKey.eq(group_key));
    }
    if let Some(lang_code) = request.lang_code {
        query = query.filter(translation_entries::Column::LangCode.eq(lang_code));
    }
    if let Some(enabled) = request.enabled {
        query = query.filter(translation_entries::Column::Enabled.eq(enabled));
    }
    match request.search {
        Some(search) if !search.is_empty() => query.filter(entry_search_condition(&search)),
        _ => query,
    }
}

fn language_search_condition(search: &str) -> Condition {
    Condition::any()
        .add(translation_languages::Column::Code.contains(search))
        .add(translation_languages::Column::Name.contains(search))
        .add(translation_languages::Column::NativeName.contains(search))
}

fn entry_search_condition(search: &str) -> Condition {
    Condition::any()
        .add(translation_entries::Column::Namespace.contains(search))
        .add(translation_entries::Column::GroupKey.contains(search))
        .add(translation_entries::Column::ItemKey.contains(search))
        .add(translation_entries::Column::LangCode.contains(search))
        .add(translation_entries::Column::Value.contains(search))
}

fn language_active_model(input: TranslationLanguageRecordInput) -> TranslationLanguageActiveModel {
    let now = time::OffsetDateTime::now_utc();
    TranslationLanguageActiveModel {
        code: Set(input.code),
        name: Set(input.name),
        native_name: Set(input.native_name),
        enabled: Set(input.enabled),
        system: Set(input.system),
        sort_order: Set(input.sort_order),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

fn entry_active_model(id: String, input: TranslationEntryRecordInput) -> TranslationEntryActiveModel {
    let now = time::OffsetDateTime::now_utc();
    TranslationEntryActiveModel {
        id: Set(id),
        namespace: Set(input.namespace),
        group_key: Set(input.group_key),
        item_key: Set(input.item_key),
        lang_code: Set(input.lang_code),
        value: Set(input.value),
        description: Set(input.description),
        enabled: Set(input.enabled),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

fn apply_language_patch(active: &mut TranslationLanguageActiveModel, input: TranslationLanguageRecordPatch) {
    if let Some(value) = input.name {
        active.name = Set(value);
    }
    if let Some(value) = input.native_name {
        active.native_name = Set(value);
    }
    if let Some(value) = input.enabled {
        active.enabled = Set(value);
    }
    if let Some(value) = input.sort_order {
        active.sort_order = Set(value);
    }
}

fn apply_entry_patch(active: &mut TranslationEntryActiveModel, input: TranslationEntryRecordPatch) {
    if let Some(value) = input.value {
        active.value = Set(value);
    }
    if let Some(value) = input.description {
        active.description = Set(value);
    }
    if let Some(value) = input.enabled {
        active.enabled = Set(value);
    }
}
