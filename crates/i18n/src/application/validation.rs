use types::i18n::{
    TranslationBundleUpsert, TranslationEntryCreate, TranslationEntryListRequest, TranslationEntryUpdate, TranslationLanguageCreate,
    TranslationLanguageListRequest, TranslationLanguageUpdate,
};

use super::{I18nError, I18nResult};

const ADMIN_NAMESPACE: &str = "admin";
const MIN_KEY_LEN: usize = 1;
const MAX_CODE_LEN: usize = 32;
const MAX_KEY_LEN: usize = 120;
const MAX_TEXT_LEN: usize = 5000;
const MAX_LIMIT: u64 = 500;

pub fn sanitize_namespace(value: &str) -> String {
    value.trim().to_lowercase()
}

pub fn sanitize_language_code(value: &str) -> String {
    value.trim().to_lowercase()
}

pub fn sanitize_language_create(input: TranslationLanguageCreate) -> TranslationLanguageCreate {
    TranslationLanguageCreate {
        code: sanitize_language_code(&input.code),
        name: input.name.trim().to_owned(),
        native_name: input.native_name.trim().to_owned(),
        enabled: input.enabled,
        sort_order: input.sort_order,
    }
}

pub fn sanitize_language_update(input: TranslationLanguageUpdate) -> TranslationLanguageUpdate {
    TranslationLanguageUpdate {
        name: input.name.map(|value| value.trim().to_owned()),
        native_name: input.native_name.map(|value| value.trim().to_owned()),
        enabled: input.enabled,
        sort_order: input.sort_order,
    }
}

pub fn sanitize_entry_create(input: TranslationEntryCreate) -> TranslationEntryCreate {
    TranslationEntryCreate {
        namespace: sanitize_namespace(&input.namespace),
        group_key: sanitize_key(&input.group_key),
        item_key: sanitize_key(&input.item_key),
        lang_code: sanitize_language_code(&input.lang_code),
        value: input.value.trim().to_owned(),
        description: input.description.map(|value| value.trim().to_owned()).filter(|value| !value.is_empty()),
        enabled: input.enabled,
    }
}

pub fn sanitize_entry_update(input: TranslationEntryUpdate) -> TranslationEntryUpdate {
    TranslationEntryUpdate {
        value: input.value.map(|value| value.trim().to_owned()),
        description: input
            .description
            .map(|value| value.map(|inner| inner.trim().to_owned()).filter(|inner| !inner.is_empty())),
        enabled: input.enabled,
    }
}

pub fn sanitize_bundle(input: TranslationBundleUpsert) -> TranslationBundleUpsert {
    TranslationBundleUpsert {
        values: input
            .values
            .into_iter()
            .map(|(lang, value)| (sanitize_language_code(&lang), value.trim().to_owned()))
            .collect(),
        description: input.description.map(|value| value.trim().to_owned()).filter(|value| !value.is_empty()),
        enabled: input.enabled,
    }
}

pub fn validate_namespace(namespace: &str) -> I18nResult<()> {
    if namespace != ADMIN_NAMESPACE {
        return Err(I18nError::InvalidInput("only admin namespace is supported".into()));
    }
    Ok(())
}

pub fn validate_language_create(input: &TranslationLanguageCreate) -> I18nResult<()> {
    validate_code(&input.code, "language code")?;
    validate_text(&input.name, "language name", MAX_KEY_LEN)?;
    validate_text(&input.native_name, "native language name", MAX_KEY_LEN)
}

pub fn validate_language_update(input: &TranslationLanguageUpdate) -> I18nResult<()> {
    if input.is_empty() {
        return Err(I18nError::InvalidInput("language update cannot be empty".into()));
    }
    validate_optional_text(input.name.as_deref(), "language name", MAX_KEY_LEN)?;
    validate_optional_text(input.native_name.as_deref(), "native language name", MAX_KEY_LEN)
}

pub fn validate_entry_create(input: &TranslationEntryCreate) -> I18nResult<()> {
    validate_namespace(&input.namespace)?;
    validate_key(&input.group_key, "group key")?;
    validate_key(&input.item_key, "item key")?;
    validate_code(&input.lang_code, "language code")?;
    validate_text(&input.value, "translation value", MAX_TEXT_LEN)
}

pub fn validate_entry_update(input: &TranslationEntryUpdate) -> I18nResult<()> {
    if input.is_empty() {
        return Err(I18nError::InvalidInput("translation update cannot be empty".into()));
    }
    validate_optional_text(input.value.as_deref(), "translation value", MAX_TEXT_LEN)
}

pub fn validate_bundle(input: &TranslationBundleUpsert) -> I18nResult<()> {
    if input.values.is_empty() {
        return Err(I18nError::InvalidInput("translation values cannot be empty".into()));
    }
    for (lang, value) in &input.values {
        validate_code(lang, "language code")?;
        validate_text(value, "translation value", MAX_TEXT_LEN)?;
    }
    Ok(())
}

pub fn validate_language_list(request: &TranslationLanguageListRequest) -> I18nResult<()> {
    validate_limit(request.limit)
}

pub fn validate_entry_list(request: &TranslationEntryListRequest) -> I18nResult<()> {
    validate_limit(request.limit)?;
    if let Some(namespace) = &request.namespace {
        validate_namespace(&sanitize_namespace(namespace))?;
    }
    Ok(())
}

fn sanitize_key(value: &str) -> String {
    value.trim().to_owned()
}

fn validate_code(value: &str, label: &str) -> I18nResult<()> {
    validate_text(value, label, MAX_CODE_LEN)?;
    if !value.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_') {
        return Err(I18nError::InvalidInput(format!("{label} contains invalid characters")));
    }
    Ok(())
}

fn validate_key(value: &str, label: &str) -> I18nResult<()> {
    validate_text(value, label, MAX_KEY_LEN)?;
    if !value.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')) {
        return Err(I18nError::InvalidInput(format!("{label} contains invalid characters")));
    }
    Ok(())
}

fn validate_optional_text(value: Option<&str>, label: &str, max_len: usize) -> I18nResult<()> {
    match value {
        Some(text) => validate_text(text, label, max_len),
        None => Ok(()),
    }
}

fn validate_text(value: &str, label: &str, max_len: usize) -> I18nResult<()> {
    if value.len() < MIN_KEY_LEN || value.len() > max_len {
        return Err(I18nError::InvalidInput(format!("{label} length is invalid")));
    }
    Ok(())
}

fn validate_limit(limit: u64) -> I18nResult<()> {
    if limit == 0 || limit > MAX_LIMIT {
        return Err(I18nError::InvalidInput(format!("limit must be between 1 and {MAX_LIMIT}")));
    }
    Ok(())
}
