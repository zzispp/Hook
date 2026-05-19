use sea_orm_migration::prelude::*;
use serde_json::Value;

use super::iden::*;

const DEFAULT_GROUP_ID: &str = "00000000-0000-7000-8000-000000000401";
pub(in crate::migration) const ADMIN_NAMESPACE: &str = "admin";
pub(in crate::migration) const AUTH_NAMESPACE: &str = "auth";
pub(in crate::migration) const CN_ADMIN_TRANSLATIONS: &str = include_str!("../defaults/i18n/admin.cn.json");
pub(in crate::migration) const EN_ADMIN_TRANSLATIONS: &str = include_str!("../defaults/i18n/admin.en.json");
pub(in crate::migration) const CN_AUTH_TRANSLATIONS: &str = include_str!("../defaults/i18n/auth.cn.json");
pub(in crate::migration) const EN_AUTH_TRANSLATIONS: &str = include_str!("../defaults/i18n/auth.en.json");

pub(super) async fn seed_domain_defaults(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    seed_default_group(manager).await?;
    super::setting_seed::seed_system_settings(manager).await?;
    seed_translation_languages(manager).await?;
    seed_admin_translations(manager).await
}

async fn seed_default_group(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::insert()
                .into_table(BillingGroups::Table)
                .columns([
                    BillingGroups::Id,
                    BillingGroups::Code,
                    BillingGroups::Name,
                    BillingGroups::Description,
                    BillingGroups::BillingMultiplier,
                    BillingGroups::IsActive,
                    BillingGroups::IsSystem,
                    BillingGroups::SortOrder,
                    BillingGroups::CreatedAt,
                    BillingGroups::UpdatedAt,
                ])
                .values_panic([
                    DEFAULT_GROUP_ID.into(),
                    constants::billing::DEFAULT_SYSTEM_GROUP_CODE.into(),
                    "System Group".into(),
                    Some("Built-in billing group used when a token does not choose a group").into(),
                    1.into(),
                    true.into(),
                    true.into(),
                    0.into(),
                    Expr::current_timestamp(),
                    Expr::current_timestamp(),
                ])
                .to_owned(),
        )
        .await
}

async fn seed_translation_languages(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::insert()
                .into_table(TranslationLanguages::Table)
                .columns([
                    TranslationLanguages::Code,
                    TranslationLanguages::Name,
                    TranslationLanguages::NativeName,
                    TranslationLanguages::Enabled,
                    TranslationLanguages::System,
                    TranslationLanguages::SortOrder,
                    TranslationLanguages::CreatedAt,
                    TranslationLanguages::UpdatedAt,
                ])
                .values_panic(language_values("cn", "Chinese", "中文", 0))
                .values_panic(language_values("en", "English", "English", 10))
                .to_owned(),
        )
        .await
}

async fn seed_admin_translations(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let mut insert = Query::insert();
    insert.into_table(TranslationEntries::Table).columns([
        TranslationEntries::Id,
        TranslationEntries::Namespace,
        TranslationEntries::GroupKey,
        TranslationEntries::ItemKey,
        TranslationEntries::LangCode,
        TranslationEntries::Value,
        TranslationEntries::Description,
        TranslationEntries::Enabled,
        TranslationEntries::CreatedAt,
        TranslationEntries::UpdatedAt,
    ]);
    for (index, entry) in translation_seeds()?.into_iter().enumerate() {
        insert.values_panic(translation_values(index, entry));
    }
    manager.execute(insert.to_owned()).await
}

fn language_values(code: &'static str, name: &'static str, native_name: &'static str, sort_order: i64) -> [Expr; 8] {
    [
        code.into(),
        name.into(),
        native_name.into(),
        true.into(),
        true.into(),
        sort_order.into(),
        Expr::current_timestamp(),
        Expr::current_timestamp(),
    ]
}

fn translation_values(index: usize, entry: TranslationSeed) -> [Expr; 10] {
    [
        default_translation_id(index).into(),
        entry.namespace.into(),
        entry.group_key.into(),
        entry.item_key.into(),
        entry.lang_code.into(),
        entry.value.into(),
        Option::<String>::None.into(),
        true.into(),
        Expr::current_timestamp(),
        Expr::current_timestamp(),
    ]
}

pub(in crate::migration) fn translation_seeds() -> Result<Vec<TranslationSeed>, DbErr> {
    let mut seeds = Vec::new();
    for (namespace, lang, source) in [
        (ADMIN_NAMESPACE, "cn", CN_ADMIN_TRANSLATIONS),
        (ADMIN_NAMESPACE, "en", EN_ADMIN_TRANSLATIONS),
        (AUTH_NAMESPACE, "cn", CN_AUTH_TRANSLATIONS),
        (AUTH_NAMESPACE, "en", EN_AUTH_TRANSLATIONS),
    ] {
        seeds.extend(flatten_translations(namespace, lang, source)?);
    }
    Ok(seeds)
}

pub(in crate::migration) fn flatten_translations(namespace: &'static str, lang_code: &'static str, source: &str) -> Result<Vec<TranslationSeed>, DbErr> {
    let value = serde_json::from_str(source).map_err(|error| DbErr::Migration(error.to_string()))?;
    let mut entries = Vec::new();
    flatten_value(namespace, lang_code, &mut entries, Vec::new(), &value);
    Ok(entries)
}

fn flatten_value(namespace: &'static str, lang_code: &'static str, entries: &mut Vec<TranslationSeed>, path: Vec<String>, value: &Value) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let mut next_path = path.clone();
                next_path.push(key.clone());
                flatten_value(namespace, lang_code, entries, next_path, child);
            }
        }
        Value::String(text) if path.len() >= 2 => entries.push(TranslationSeed {
            namespace,
            lang_code,
            group_key: path[0].clone(),
            item_key: path[1..].join("."),
            value: text.clone(),
        }),
        Value::Array(items) => flatten_array(namespace, lang_code, entries, path, items),
        _ => {}
    }
}

fn flatten_array(namespace: &'static str, lang_code: &'static str, entries: &mut Vec<TranslationSeed>, path: Vec<String>, items: &[Value]) {
    for (index, item) in items.iter().enumerate() {
        let mut next_path = path.clone();
        next_path.push(index.to_string());
        flatten_value(namespace, lang_code, entries, next_path, item);
    }
}

fn default_translation_id(index: usize) -> String {
    format!("00000000-0000-7000-8000-{:012}", 1000 + index)
}

pub(in crate::migration) struct TranslationSeed {
    pub(in crate::migration) namespace: &'static str,
    pub(in crate::migration) lang_code: &'static str,
    pub(in crate::migration) group_key: String,
    pub(in crate::migration) item_key: String,
    pub(in crate::migration) value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_admin_translations_keeps_string_arrays() {
        let source = r#"{"dashboard":{"months":["Jan","Feb"],"welcome":"Welcome"}}"#;
        let entries = flatten_translations(ADMIN_NAMESPACE, "en", source).unwrap();
        let keys: Vec<_> = entries
            .iter()
            .map(|entry| (entry.group_key.as_str(), entry.item_key.as_str(), entry.value.as_str()))
            .collect();

        assert!(keys.contains(&("dashboard", "months.0", "Jan")));
        assert!(keys.contains(&("dashboard", "months.1", "Feb")));
        assert!(keys.contains(&("dashboard", "welcome", "Welcome")));
    }

    #[test]
    fn default_admin_translations_include_llm_wallet_labels() {
        let entries = translation_seeds().unwrap();
        let keys: Vec<_> = entries
            .iter()
            .map(|entry| {
                (
                    entry.namespace,
                    entry.lang_code,
                    entry.group_key.as_str(),
                    entry.item_key.as_str(),
                    entry.value.as_str(),
                )
            })
            .collect();

        assert!(keys.contains(&(ADMIN_NAMESPACE, "cn", "wallet", "reasonLabels.llm_model_usage", "模型调用消费")));
        assert!(keys.contains(&(ADMIN_NAMESPACE, "en", "wallet", "reasonLabels.llm_model_usage", "Model usage")));
        assert!(keys.contains(&(ADMIN_NAMESPACE, "cn", "wallet", "linkTypeLabels.llm_request_record", "模型调用记录")));
        assert!(keys.contains(&(ADMIN_NAMESPACE, "en", "wallet", "linkTypeLabels.llm_request_record", "LLM request record")));
    }
}
