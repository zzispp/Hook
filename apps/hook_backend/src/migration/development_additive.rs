use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Schema, Set},
    seaql_migrations,
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};

use storage::provider::record::{
    billing_group_provider_groups, billing_group_provider_key_groups, billing_group_provider_keys, billing_group_providers, provider_group_providers,
    provider_groups, provider_key_group_keys, provider_key_groups,
};

use super::baseline::seed_domain::TranslationSeed;

const ADDITIVE_VERSION: &str = "m20260608_000001_provider_group_additive";
const MIGRATION_TABLE: &str = "seaql_migrations";

pub async fn apply(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    super::baseline::apply_schema_without_seed(manager).await?;
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    seed_missing_translations(manager).await?;
    migrate_legacy_bindings(manager.get_connection()).await?;
    mark_additive_applied(manager).await
}

async fn seed_missing_translations(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for seed in super::baseline::seed_domain::translation_seeds()? {
        ensure_translation_entry(manager, &seed).await?;
    }
    Ok(())
}

async fn ensure_translation_entry(manager: &SchemaManager<'_>, seed: &TranslationSeed) -> Result<(), DbErr> {
    if translation_entry_exists(manager, seed).await? {
        return Ok(());
    }
    manager
        .execute(
            Query::insert()
                .into_table(TranslationEntries::Table)
                .columns([
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
                ])
                .values_panic(translation_values(seed))
                .to_owned(),
        )
        .await
}

async fn translation_entry_exists(manager: &SchemaManager<'_>, seed: &TranslationSeed) -> Result<bool, DbErr> {
    let query = Query::select()
        .expr(Expr::val(1))
        .from(TranslationEntries::Table)
        .and_where(Expr::col(TranslationEntries::Namespace).eq(seed.namespace))
        .and_where(Expr::col(TranslationEntries::GroupKey).eq(seed.group_key.as_str()))
        .and_where(Expr::col(TranslationEntries::ItemKey).eq(seed.item_key.as_str()))
        .and_where(Expr::col(TranslationEntries::LangCode).eq(seed.lang_code))
        .limit(1)
        .to_owned();
    manager.get_connection().query_one(&query).await.map(|row| row.is_some())
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

async fn migrate_legacy_bindings(db: &impl ConnectionTrait) -> Result<(), DbErr> {
    let key_bindings = legacy_key_bindings(db).await?;
    let provider_bindings = legacy_provider_bindings(db).await?;
    for (group_code, key_ids) in &key_bindings {
        migrate_key_binding(db, group_code, key_ids).await?;
    }
    for (group_code, provider_ids) in provider_bindings {
        if !key_bindings.contains_key(&group_code) {
            migrate_provider_binding(db, &group_code, &provider_ids).await?;
        }
    }
    Ok(())
}

async fn migrate_key_binding(db: &impl ConnectionTrait, group_code: &str, key_ids: &[String]) -> Result<(), DbErr> {
    let group_id = ensure_provider_key_group(db, &format!("Migrated key group: {group_code}")).await?;
    for key_id in key_ids {
        ensure_provider_key_group_key(db, &group_id, key_id).await?;
    }
    ensure_billing_key_group(db, group_code, &group_id).await
}

async fn migrate_provider_binding(db: &impl ConnectionTrait, group_code: &str, provider_ids: &[String]) -> Result<(), DbErr> {
    let group_id = ensure_provider_group(db, &format!("Migrated provider group: {group_code}")).await?;
    for provider_id in provider_ids {
        ensure_provider_group_provider(db, &group_id, provider_id).await?;
    }
    ensure_billing_provider_group(db, group_code, &group_id).await
}

async fn legacy_key_bindings(db: &impl ConnectionTrait) -> Result<BTreeMap<String, Vec<String>>, DbErr> {
    let records = billing_group_provider_keys::Entity::find()
        .order_by_asc(billing_group_provider_keys::Column::GroupCode)
        .order_by_asc(billing_group_provider_keys::Column::ProviderKeyId)
        .all(db)
        .await?;
    Ok(bindings_by_group(records.into_iter().map(|record| (record.group_code, record.provider_key_id))))
}

async fn legacy_provider_bindings(db: &impl ConnectionTrait) -> Result<BTreeMap<String, Vec<String>>, DbErr> {
    let records = billing_group_providers::Entity::find()
        .order_by_asc(billing_group_providers::Column::GroupCode)
        .order_by_asc(billing_group_providers::Column::ProviderId)
        .all(db)
        .await?;
    Ok(bindings_by_group(records.into_iter().map(|record| (record.group_code, record.provider_id))))
}

fn bindings_by_group(records: impl Iterator<Item = (String, String)>) -> BTreeMap<String, Vec<String>> {
    let mut bindings = BTreeMap::<String, BTreeSet<String>>::new();
    for (group_code, id) in records {
        bindings.entry(group_code).or_default().insert(id);
    }
    bindings.into_iter().map(|(group_code, ids)| (group_code, ids.into_iter().collect())).collect()
}

async fn ensure_provider_group(db: &impl ConnectionTrait, name: &str) -> Result<String, DbErr> {
    if let Some(record) = provider_groups::Entity::find().filter(provider_groups::Column::Name.eq(name)).one(db).await? {
        return Ok(record.id);
    }
    let now = time::OffsetDateTime::now_utc();
    let record = provider_groups::ActiveModel {
        id: Set(new_id()),
        name: Set(name.to_owned()),
        description: Set(None),
        sort_order: Set(0),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await?;
    Ok(record.id)
}

async fn ensure_provider_key_group(db: &impl ConnectionTrait, name: &str) -> Result<String, DbErr> {
    if let Some(record) = provider_key_groups::Entity::find()
        .filter(provider_key_groups::Column::Name.eq(name))
        .one(db)
        .await?
    {
        return Ok(record.id);
    }
    let now = time::OffsetDateTime::now_utc();
    let record = provider_key_groups::ActiveModel {
        id: Set(new_id()),
        name: Set(name.to_owned()),
        description: Set(None),
        sort_order: Set(0),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await?;
    Ok(record.id)
}

async fn ensure_provider_group_provider(db: &impl ConnectionTrait, group_id: &str, provider_id: &str) -> Result<(), DbErr> {
    let exists = provider_group_providers::Entity::find()
        .filter(provider_group_providers::Column::ProviderGroupId.eq(group_id))
        .filter(provider_group_providers::Column::ProviderId.eq(provider_id))
        .one(db)
        .await?
        .is_some();
    if exists {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    provider_group_providers::ActiveModel {
        id: Set(new_id()),
        provider_group_id: Set(group_id.to_owned()),
        provider_id: Set(provider_id.to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await?;
    Ok(())
}

async fn ensure_provider_key_group_key(db: &impl ConnectionTrait, group_id: &str, key_id: &str) -> Result<(), DbErr> {
    let exists = provider_key_group_keys::Entity::find()
        .filter(provider_key_group_keys::Column::ProviderKeyGroupId.eq(group_id))
        .filter(provider_key_group_keys::Column::ProviderKeyId.eq(key_id))
        .one(db)
        .await?
        .is_some();
    if exists {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    provider_key_group_keys::ActiveModel {
        id: Set(new_id()),
        provider_key_group_id: Set(group_id.to_owned()),
        provider_key_id: Set(key_id.to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await?;
    Ok(())
}

async fn ensure_billing_provider_group(db: &impl ConnectionTrait, group_code: &str, provider_group_id: &str) -> Result<(), DbErr> {
    if billing_provider_group_exists(db, group_code, provider_group_id).await? {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    billing_group_provider_groups::ActiveModel {
        id: Set(new_id()),
        group_code: Set(group_code.to_owned()),
        provider_group_id: Set(provider_group_id.to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await?;
    Ok(())
}

async fn ensure_billing_key_group(db: &impl ConnectionTrait, group_code: &str, provider_key_group_id: &str) -> Result<(), DbErr> {
    if billing_key_group_exists(db, group_code, provider_key_group_id).await? {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    billing_group_provider_key_groups::ActiveModel {
        id: Set(new_id()),
        group_code: Set(group_code.to_owned()),
        provider_key_group_id: Set(provider_key_group_id.to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await?;
    Ok(())
}

async fn billing_provider_group_exists(db: &impl ConnectionTrait, group_code: &str, provider_group_id: &str) -> Result<bool, DbErr> {
    billing_group_provider_groups::Entity::find()
        .filter(billing_group_provider_groups::Column::GroupCode.eq(group_code))
        .filter(billing_group_provider_groups::Column::ProviderGroupId.eq(provider_group_id))
        .one(db)
        .await
        .map(|record| record.is_some())
}

async fn billing_key_group_exists(db: &impl ConnectionTrait, group_code: &str, provider_key_group_id: &str) -> Result<bool, DbErr> {
    billing_group_provider_key_groups::Entity::find()
        .filter(billing_group_provider_key_groups::Column::GroupCode.eq(group_code))
        .filter(billing_group_provider_key_groups::Column::ProviderKeyGroupId.eq(provider_key_group_id))
        .one(db)
        .await
        .map(|record| record.is_some())
}

async fn additive_marker_exists(manager: &SchemaManager<'_>) -> Result<bool, DbErr> {
    if !manager.has_table(MIGRATION_TABLE).await? {
        return Ok(false);
    }
    seaql_migrations::Entity::find()
        .filter(seaql_migrations::Column::Version.eq(ADDITIVE_VERSION))
        .one(manager.get_connection())
        .await
        .map(|record| record.is_some())
}

async fn mark_additive_applied(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    create_migration_table(manager).await?;
    if additive_marker_exists(manager).await? {
        return Ok(());
    }
    seaql_migrations::Entity::insert(seaql_migrations::ActiveModel {
        version: ActiveValue::Set(ADDITIVE_VERSION.to_owned()),
        applied_at: ActiveValue::Set(current_timestamp()?),
    })
    .exec(manager.get_connection())
    .await?;
    Ok(())
}

async fn create_migration_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let schema = Schema::new(manager.get_database_backend());
    let mut statement = schema.create_table_from_entity(seaql_migrations::Entity);
    statement.if_not_exists();
    manager.create_table(statement).await
}

fn current_timestamp() -> Result<i64, DbErr> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .map_err(|error| DbErr::Migration(format!("system time is before UNIX epoch: {error}")))
}

fn new_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

#[derive(DeriveIden)]
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
