use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_TRANSLATION_LIMIT: u64 = 100;
const DEFAULT_LANGUAGE_LIMIT: u64 = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranslationLanguage {
    pub code: String,
    pub name: String,
    pub native_name: String,
    pub enabled: bool,
    pub system: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranslationEntry {
    pub id: String,
    pub namespace: String,
    pub group_key: String,
    pub item_key: String,
    pub lang_code: String,
    pub value: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct TranslationLanguageListRequest {
    #[serde(default)]
    pub skip: u64,
    #[serde(default = "default_language_limit")]
    pub limit: u64,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub search: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct TranslationEntryListRequest {
    #[serde(default)]
    pub skip: u64,
    #[serde(default = "default_translation_limit")]
    pub limit: u64,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub group_key: Option<String>,
    #[serde(default)]
    pub lang_code: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub search: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct TranslationLanguageCreate {
    pub code: String,
    pub name: String,
    pub native_name: String,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct TranslationLanguageUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub native_name: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct TranslationEntryCreate {
    pub namespace: String,
    pub group_key: String,
    pub item_key: String,
    pub lang_code: String,
    pub value: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct TranslationEntryUpdate {
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub description: Option<Option<String>>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct TranslationBundleUpsert {
    #[serde(default)]
    pub values: BTreeMap<String, String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct I18nResourceResponse {
    pub lang: String,
    pub namespace: String,
    pub resources: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TranslationLanguageResponse {
    pub code: String,
    pub name: String,
    pub native_name: String,
    pub enabled: bool,
    pub system: bool,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TranslationEntryResponse {
    pub id: String,
    pub namespace: String,
    pub group_key: String,
    pub item_key: String,
    pub lang_code: String,
    pub value: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TranslationLanguageListResponse {
    pub languages: Vec<TranslationLanguageResponse>,
    pub total: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TranslationEntryListResponse {
    pub translations: Vec<TranslationEntryResponse>,
    pub total: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TranslationBundleResponse {
    pub namespace: String,
    pub group_key: String,
    pub item_key: String,
    pub entries: Vec<TranslationEntryResponse>,
}

impl TranslationLanguageUpdate {
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.native_name.is_none() && self.enabled.is_none() && self.sort_order.is_none()
    }
}

impl TranslationEntryUpdate {
    pub fn is_empty(&self) -> bool {
        self.value.is_none() && self.description.is_none() && self.enabled.is_none()
    }
}

impl From<TranslationLanguage> for TranslationLanguageResponse {
    fn from(value: TranslationLanguage) -> Self {
        Self {
            code: value.code,
            name: value.name,
            native_name: value.native_name,
            enabled: value.enabled,
            system: value.system,
            sort_order: value.sort_order,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<TranslationEntry> for TranslationEntryResponse {
    fn from(value: TranslationEntry) -> Self {
        Self {
            id: value.id,
            namespace: value.namespace,
            group_key: value.group_key,
            item_key: value.item_key,
            lang_code: value.lang_code,
            value: value.value,
            description: value.description,
            enabled: value.enabled,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

fn default_translation_limit() -> u64 {
    DEFAULT_TRANSLATION_LIMIT
}

fn default_language_limit() -> u64 {
    DEFAULT_LANGUAGE_LIMIT
}
