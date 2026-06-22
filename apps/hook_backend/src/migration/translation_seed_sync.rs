use sea_orm_migration::{prelude::*, sea_orm::ConnectionTrait};

use super::baseline::seed_domain::TranslationSeed;

pub async fn seed_missing_translations(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for seed in super::baseline::seed_domain::translation_seeds()? {
        insert_if_missing(manager, &seed).await?;
    }
    Ok(())
}

async fn insert_if_missing(manager: &SchemaManager<'_>, seed: &TranslationSeed) -> Result<(), DbErr> {
    if translation_entry_exists(manager, seed).await? {
        return Ok(());
    }
    insert_translation_entry(manager, seed).await
}

async fn insert_translation_entry(manager: &SchemaManager<'_>, seed: &TranslationSeed) -> Result<(), DbErr> {
    manager
        .execute(
            Query::insert()
                .into_table(TranslationEntries::Table)
                .columns(translation_columns())
                .values_panic(translation_values(seed))
                .to_owned(),
        )
        .await
}

async fn translation_entry_exists(manager: &SchemaManager<'_>, seed: &TranslationSeed) -> Result<bool, DbErr> {
    let query = Query::select()
        .expr(Expr::val(1))
        .from(TranslationEntries::Table)
        .and_where(seed_filter(seed))
        .limit(1)
        .to_owned();
    manager.get_connection().query_one(&query).await.map(|row| row.is_some())
}

fn seed_filter(seed: &TranslationSeed) -> SimpleExpr {
    Expr::col(TranslationEntries::Namespace)
        .eq(seed.namespace)
        .and(Expr::col(TranslationEntries::GroupKey).eq(seed.group_key.as_str()))
        .and(Expr::col(TranslationEntries::ItemKey).eq(seed.item_key.as_str()))
        .and(Expr::col(TranslationEntries::LangCode).eq(seed.lang_code))
}

fn translation_columns() -> [TranslationEntries; 10] {
    [
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
    ]
}

fn translation_values(seed: &TranslationSeed) -> [Expr; 10] {
    [
        new_id().into(),
        seed.namespace.into(),
        seed.group_key.clone().into(),
        seed.item_key.clone().into(),
        seed.lang_code.into(),
        seed.value.clone().into(),
        Option::<String>::None.into(),
        true.into(),
        Expr::current_timestamp(),
        Expr::current_timestamp(),
    ]
}

fn new_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

#[derive(Clone, Copy, DeriveIden)]
enum TranslationEntries {
    Table,
    Id,
    Namespace,
    GroupKey,
    ItemKey,
    LangCode,
    Value,
    Description,
    Enabled,
    CreatedAt,
    UpdatedAt,
}

#[cfg(test)]
mod tests {
    use sea_orm_migration::sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, Transaction, sea_query::Value};
    use std::collections::{BTreeMap, BTreeSet};

    use super::*;
    use crate::migration::baseline::seed_domain::ADMIN_NAMESPACE;

    const RECHARGE_GROUP: &str = "adminRecharges";
    const FIELDS_GROUP: &str = "fields";
    const ALL_PRESET_ITEM: &str = "filters.datePresets.all";
    const TODAY_PRESET_ITEM: &str = "filters.datePresets.today";
    const EMAIL_ITEM: &str = "email";
    const PASSWORD_ITEM: &str = "password";

    #[test]
    fn translation_seeds_include_recharge_date_presets() {
        let entries = admin_seed_entries(RECHARGE_GROUP);

        assert!(entries.contains(&("cn", ALL_PRESET_ITEM.to_owned(), "全部".to_owned())));
        assert!(entries.contains(&("cn", TODAY_PRESET_ITEM.to_owned(), "今天".to_owned())));
        assert!(entries.contains(&("en", ALL_PRESET_ITEM.to_owned(), "All".to_owned())));
        assert!(entries.contains(&("en", TODAY_PRESET_ITEM.to_owned(), "Today".to_owned())));
    }

    #[test]
    fn translation_seeds_include_admin_email_and_password_fields() {
        let entries = admin_seed_entries(FIELDS_GROUP);

        assert!(entries.contains(&("cn", EMAIL_ITEM.to_owned(), "邮箱地址".to_owned())));
        assert!(entries.contains(&("cn", PASSWORD_ITEM.to_owned(), "密码".to_owned())));
        assert!(entries.contains(&("en", EMAIL_ITEM.to_owned(), "Email address".to_owned())));
        assert!(entries.contains(&("en", PASSWORD_ITEM.to_owned(), "Password".to_owned())));
    }

    #[tokio::test]
    async fn insert_if_missing_skips_existing_translation_key() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([one_query_row()])
            .into_connection();
        let manager = SchemaManager::new(&db);

        insert_if_missing(&manager, &test_seed()).await.unwrap();

        let log = db.into_transaction_log();
        assert_eq!(log.len(), 1);
        assert_transaction_log_contains(&log, "SELECT");
        assert_transaction_log_excludes(&log, "INSERT INTO");
    }

    #[tokio::test]
    async fn insert_if_missing_inserts_absent_translation_key() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([empty_query_rows()])
            .append_exec_results([MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();
        let manager = SchemaManager::new(&db);

        insert_if_missing(&manager, &test_seed()).await.unwrap();

        let log = db.into_transaction_log();
        assert_eq!(log.len(), 2);
        assert_transaction_log_contains(&log, "SELECT");
        assert_transaction_log_contains(&log, "INSERT INTO");
        assert_transaction_log_contains(&log, "translation_entries");
    }

    fn test_seed() -> TranslationSeed {
        TranslationSeed {
            namespace: ADMIN_NAMESPACE,
            lang_code: "cn",
            group_key: RECHARGE_GROUP.to_owned(),
            item_key: TODAY_PRESET_ITEM.to_owned(),
            value: "今天".to_owned(),
        }
    }

    fn admin_seed_entries(group_key: &str) -> BTreeSet<(&'static str, String, String)> {
        super::super::baseline::seed_domain::translation_seeds()
            .unwrap()
            .into_iter()
            .filter(|seed| seed.namespace == ADMIN_NAMESPACE && seed.group_key == group_key)
            .map(|seed| (seed.lang_code, seed.item_key, seed.value))
            .collect()
    }

    fn one_query_row() -> Vec<BTreeMap<&'static str, Value>> {
        let mut row = BTreeMap::new();
        row.insert("exists", 1.into());
        vec![row]
    }

    fn empty_query_rows() -> Vec<BTreeMap<&'static str, Value>> {
        Vec::new()
    }

    fn assert_transaction_log_contains(log: &[Transaction], pattern: &str) {
        assert!(format!("{log:?}").contains(pattern), "transaction log should contain {pattern}: {log:?}");
    }

    fn assert_transaction_log_excludes(log: &[Transaction], pattern: &str) {
        assert!(!format!("{log:?}").contains(pattern), "transaction log should not contain {pattern}: {log:?}");
    }
}
