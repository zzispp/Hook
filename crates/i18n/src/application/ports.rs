use async_trait::async_trait;
use types::i18n::{
    I18nResourceResponse, TranslationBundleResponse, TranslationBundleUpsert, TranslationEntryCreate, TranslationEntryListRequest,
    TranslationEntryListResponse, TranslationEntryResponse, TranslationEntryUpdate, TranslationLanguage, TranslationLanguageCreate,
    TranslationLanguageListRequest, TranslationLanguageListResponse, TranslationLanguageResponse, TranslationLanguageUpdate,
};

use super::I18nResult;

#[async_trait]
pub trait I18nRepository: Send + Sync + 'static {
    async fn resource_bundle(&self, lang: &str, namespace: &str) -> I18nResult<I18nResourceResponse>;
    async fn list_languages(&self, request: TranslationLanguageListRequest) -> I18nResult<TranslationLanguageListResponse>;
    async fn create_language(&self, input: TranslationLanguageCreate) -> I18nResult<TranslationLanguageResponse>;
    async fn update_language(&self, code: &str, input: TranslationLanguageUpdate) -> I18nResult<TranslationLanguageResponse>;
    async fn delete_language(&self, code: &str) -> I18nResult<()>;
    async fn find_language(&self, code: &str) -> I18nResult<Option<TranslationLanguage>>;
    async fn list_entries(&self, request: TranslationEntryListRequest) -> I18nResult<TranslationEntryListResponse>;
    async fn create_entry(&self, input: TranslationEntryCreate) -> I18nResult<TranslationEntryResponse>;
    async fn update_entry(&self, id: &str, input: TranslationEntryUpdate) -> I18nResult<TranslationEntryResponse>;
    async fn delete_entry(&self, id: &str) -> I18nResult<()>;
    async fn upsert_bundle(&self, namespace: &str, group_key: &str, item_key: &str, input: TranslationBundleUpsert) -> I18nResult<TranslationBundleResponse>;
    async fn entry_exists(&self, input: &TranslationEntryCreate) -> I18nResult<bool>;
}

#[async_trait]
pub trait I18nUseCase: Send + Sync + 'static {
    async fn resource_bundle(&self, lang: &str, namespace: &str) -> I18nResult<I18nResourceResponse>;
    async fn list_languages(&self, request: TranslationLanguageListRequest) -> I18nResult<TranslationLanguageListResponse>;
    async fn create_language(&self, input: TranslationLanguageCreate) -> I18nResult<TranslationLanguageResponse>;
    async fn update_language(&self, code: &str, input: TranslationLanguageUpdate) -> I18nResult<TranslationLanguageResponse>;
    async fn delete_language(&self, code: &str) -> I18nResult<()>;
    async fn list_entries(&self, request: TranslationEntryListRequest) -> I18nResult<TranslationEntryListResponse>;
    async fn create_entry(&self, input: TranslationEntryCreate) -> I18nResult<TranslationEntryResponse>;
    async fn update_entry(&self, id: &str, input: TranslationEntryUpdate) -> I18nResult<TranslationEntryResponse>;
    async fn delete_entry(&self, id: &str) -> I18nResult<()>;
    async fn upsert_bundle(&self, namespace: &str, group_key: &str, item_key: &str, input: TranslationBundleUpsert) -> I18nResult<TranslationBundleResponse>;
}
