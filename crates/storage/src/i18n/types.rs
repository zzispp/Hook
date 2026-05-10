#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranslationLanguageRecordInput {
    pub code: String,
    pub name: String,
    pub native_name: String,
    pub enabled: bool,
    pub system: bool,
    pub sort_order: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TranslationLanguageRecordPatch {
    pub name: Option<String>,
    pub native_name: Option<String>,
    pub enabled: Option<bool>,
    pub sort_order: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranslationEntryRecordInput {
    pub namespace: String,
    pub group_key: String,
    pub item_key: String,
    pub lang_code: String,
    pub value: String,
    pub description: Option<String>,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TranslationEntryRecordPatch {
    pub value: Option<String>,
    pub description: Option<Option<String>>,
    pub enabled: Option<bool>,
}
