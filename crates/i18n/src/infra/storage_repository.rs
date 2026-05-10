use async_trait::async_trait;
use storage::{Database, StorageError, i18n::I18nStore};
use types::i18n::{
    I18nResourceResponse, TranslationBundleResponse, TranslationBundleUpsert, TranslationEntryCreate, TranslationEntryListRequest,
    TranslationEntryListResponse, TranslationEntryResponse, TranslationEntryUpdate, TranslationLanguage, TranslationLanguageCreate,
    TranslationLanguageListRequest, TranslationLanguageListResponse, TranslationLanguageResponse, TranslationLanguageUpdate,
};

use crate::application::{I18nError, I18nRepository, I18nResult};

#[derive(Clone)]
pub struct StorageI18nRepository {
    store: I18nStore,
}

impl StorageI18nRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: I18nStore::new(database),
        }
    }
}

#[async_trait]
impl I18nRepository for StorageI18nRepository {
    async fn resource_bundle(&self, lang: &str, namespace: &str) -> I18nResult<I18nResourceResponse> {
        self.store.resource_bundle(lang, namespace).await.map_err(storage_error)
    }

    async fn list_languages(&self, request: TranslationLanguageListRequest) -> I18nResult<TranslationLanguageListResponse> {
        self.store.list_languages(request).await.map_err(storage_error)
    }

    async fn create_language(&self, input: TranslationLanguageCreate) -> I18nResult<TranslationLanguageResponse> {
        self.store.create_language(language_record_input(input, false)).await.map_err(storage_error)
    }

    async fn update_language(&self, code: &str, input: TranslationLanguageUpdate) -> I18nResult<TranslationLanguageResponse> {
        self.store.update_language(code, language_record_patch(input)).await.map_err(storage_error)
    }

    async fn delete_language(&self, code: &str) -> I18nResult<()> {
        self.store.delete_language(code).await.map_err(storage_error)
    }

    async fn find_language(&self, code: &str) -> I18nResult<Option<TranslationLanguage>> {
        self.store.find_language(code).await.map_err(storage_error)
    }

    async fn list_entries(&self, request: TranslationEntryListRequest) -> I18nResult<TranslationEntryListResponse> {
        self.store.list_entries(request).await.map_err(storage_error)
    }

    async fn create_entry(&self, input: TranslationEntryCreate) -> I18nResult<TranslationEntryResponse> {
        self.store.create_entry(entry_record_input(input)).await.map_err(storage_error)
    }

    async fn update_entry(&self, id: &str, input: TranslationEntryUpdate) -> I18nResult<TranslationEntryResponse> {
        self.store.update_entry(id, entry_record_patch(input)).await.map_err(storage_error)
    }

    async fn delete_entry(&self, id: &str) -> I18nResult<()> {
        self.store.delete_entry(id).await.map_err(storage_error)
    }

    async fn upsert_bundle(&self, namespace: &str, group_key: &str, item_key: &str, input: TranslationBundleUpsert) -> I18nResult<TranslationBundleResponse> {
        let mut entries = Vec::with_capacity(input.values.len());
        for (lang_code, value) in input.values {
            let entry = TranslationEntryCreate {
                namespace: namespace.to_owned(),
                group_key: group_key.to_owned(),
                item_key: item_key.to_owned(),
                lang_code,
                value,
                description: input.description.clone(),
                enabled: input.enabled,
            };
            entries.push(self.store.upsert_entry(entry_record_input(entry)).await.map_err(storage_error)?);
        }
        Ok(TranslationBundleResponse {
            namespace: namespace.to_owned(),
            group_key: group_key.to_owned(),
            item_key: item_key.to_owned(),
            entries,
        })
    }

    async fn entry_exists(&self, input: &TranslationEntryCreate) -> I18nResult<bool> {
        self.store
            .entry_exists(&input.namespace, &input.group_key, &input.item_key, &input.lang_code)
            .await
            .map_err(storage_error)
    }
}

fn language_record_input(input: TranslationLanguageCreate, system: bool) -> storage::i18n::TranslationLanguageRecordInput {
    storage::i18n::TranslationLanguageRecordInput {
        code: input.code,
        name: input.name,
        native_name: input.native_name,
        enabled: input.enabled.unwrap_or(true),
        system,
        sort_order: input.sort_order.unwrap_or(0),
    }
}

fn language_record_patch(input: TranslationLanguageUpdate) -> storage::i18n::TranslationLanguageRecordPatch {
    storage::i18n::TranslationLanguageRecordPatch {
        name: input.name,
        native_name: input.native_name,
        enabled: input.enabled,
        sort_order: input.sort_order,
    }
}

fn entry_record_input(input: TranslationEntryCreate) -> storage::i18n::TranslationEntryRecordInput {
    storage::i18n::TranslationEntryRecordInput {
        namespace: input.namespace,
        group_key: input.group_key,
        item_key: input.item_key,
        lang_code: input.lang_code,
        value: input.value,
        description: input.description,
        enabled: input.enabled.unwrap_or(true),
    }
}

fn entry_record_patch(input: TranslationEntryUpdate) -> storage::i18n::TranslationEntryRecordPatch {
    storage::i18n::TranslationEntryRecordPatch {
        value: input.value,
        description: input.description,
        enabled: input.enabled,
    }
}

fn storage_error(error: StorageError) -> I18nError {
    match error {
        StorageError::NotFound => I18nError::NotFound,
        StorageError::Conflict(message) => I18nError::Conflict(message),
        StorageError::Database(message) => I18nError::Infrastructure(message),
    }
}
