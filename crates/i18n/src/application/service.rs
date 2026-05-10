use async_trait::async_trait;
use types::i18n::{
    I18nResourceResponse, TranslationBundleResponse, TranslationBundleUpsert, TranslationEntryCreate, TranslationEntryListRequest,
    TranslationEntryListResponse, TranslationEntryResponse, TranslationEntryUpdate, TranslationLanguageCreate, TranslationLanguageListRequest,
    TranslationLanguageListResponse, TranslationLanguageResponse, TranslationLanguageUpdate,
};

use crate::application::{I18nError, I18nRepository, I18nResult, I18nUseCase};

use super::validation::{
    sanitize_bundle, sanitize_entry_create, sanitize_entry_update, sanitize_language_code, sanitize_language_create, sanitize_language_update,
    sanitize_namespace, validate_bundle, validate_entry_create, validate_entry_list, validate_entry_update, validate_language_create, validate_language_list,
    validate_language_update, validate_namespace,
};

pub struct I18nService<R> {
    repository: R,
}

impl<R> I18nService<R>
where
    R: I18nRepository,
{
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> I18nUseCase for I18nService<R>
where
    R: I18nRepository,
{
    async fn resource_bundle(&self, lang: &str, namespace: &str) -> I18nResult<I18nResourceResponse> {
        let lang = sanitize_language_code(lang);
        let namespace = sanitize_namespace(namespace);
        validate_namespace(&namespace)?;
        ensure_language_exists(&self.repository, &lang).await?;
        self.repository.resource_bundle(&lang, &namespace).await
    }

    async fn list_languages(&self, request: TranslationLanguageListRequest) -> I18nResult<TranslationLanguageListResponse> {
        validate_language_list(&request)?;
        self.repository.list_languages(request).await
    }

    async fn create_language(&self, input: TranslationLanguageCreate) -> I18nResult<TranslationLanguageResponse> {
        let input = sanitize_language_create(input);
        validate_language_create(&input)?;
        reject_duplicate_language(&self.repository, &input.code).await?;
        self.repository.create_language(input).await
    }

    async fn update_language(&self, code: &str, input: TranslationLanguageUpdate) -> I18nResult<TranslationLanguageResponse> {
        let code = sanitize_language_code(code);
        let input = sanitize_language_update(input);
        validate_language_update(&input)?;
        ensure_language_exists(&self.repository, &code).await?;
        self.repository.update_language(&code, input).await
    }

    async fn delete_language(&self, code: &str) -> I18nResult<()> {
        let code = sanitize_language_code(code);
        let language = self.repository.find_language(&code).await?.ok_or(I18nError::NotFound)?;
        if language.system {
            return Err(I18nError::Conflict("system language cannot be deleted".into()));
        }
        self.repository.delete_language(&code).await
    }

    async fn list_entries(&self, request: TranslationEntryListRequest) -> I18nResult<TranslationEntryListResponse> {
        validate_entry_list(&request)?;
        self.repository.list_entries(request).await
    }

    async fn create_entry(&self, input: TranslationEntryCreate) -> I18nResult<TranslationEntryResponse> {
        let input = sanitize_entry_create(input);
        validate_entry_create(&input)?;
        ensure_language_exists(&self.repository, &input.lang_code).await?;
        reject_duplicate_entry(&self.repository, &input).await?;
        self.repository.create_entry(input).await
    }

    async fn update_entry(&self, id: &str, input: TranslationEntryUpdate) -> I18nResult<TranslationEntryResponse> {
        let input = sanitize_entry_update(input);
        validate_entry_update(&input)?;
        self.repository.update_entry(id, input).await
    }

    async fn delete_entry(&self, id: &str) -> I18nResult<()> {
        self.repository.delete_entry(id).await
    }

    async fn upsert_bundle(&self, namespace: &str, group_key: &str, item_key: &str, input: TranslationBundleUpsert) -> I18nResult<TranslationBundleResponse> {
        let namespace = sanitize_namespace(namespace);
        let group_key = group_key.trim().to_owned();
        let item_key = item_key.trim().to_owned();
        let input = sanitize_bundle(input);
        validate_namespace(&namespace)?;
        validate_bundle(&input)?;
        for lang in input.values.keys() {
            ensure_language_exists(&self.repository, lang).await?;
        }
        self.repository.upsert_bundle(&namespace, &group_key, &item_key, input).await
    }
}

async fn ensure_language_exists<R>(repository: &R, code: &str) -> I18nResult<()>
where
    R: I18nRepository,
{
    if repository.find_language(code).await?.is_some() {
        return Ok(());
    }
    Err(I18nError::InvalidInput(format!("language does not exist: {code}")))
}

async fn reject_duplicate_language<R>(repository: &R, code: &str) -> I18nResult<()>
where
    R: I18nRepository,
{
    if repository.find_language(code).await?.is_some() {
        return Err(I18nError::Conflict(format!("language already exists: {code}")));
    }
    Ok(())
}

async fn reject_duplicate_entry<R>(repository: &R, input: &TranslationEntryCreate) -> I18nResult<()>
where
    R: I18nRepository,
{
    if repository.entry_exists(input).await? {
        return Err(I18nError::Conflict("translation key already exists for this language".into()));
    }
    Ok(())
}
