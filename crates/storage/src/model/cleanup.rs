use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

use crate::{
    StorageResult,
    api_token::api_token_records,
    group::billing_group_models,
    json,
    provider::record::provider_api_keys,
    user::{UserActiveModel, UserEntity},
};

use super::provider_models;

pub(super) async fn delete_model_bindings<C>(connection: &C, model_id: &str) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    provider_models::Entity::delete_many()
        .filter(provider_models::Column::GlobalModelId.eq(model_id))
        .exec(connection)
        .await?;
    billing_group_models::Entity::delete_many()
        .filter(billing_group_models::Column::GlobalModelId.eq(model_id))
        .exec(connection)
        .await?;
    Ok(())
}

pub(super) async fn prune_model_references<C>(connection: &C, model_id: &str) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    prune_provider_api_key_model_ids(connection, model_id).await?;
    prune_api_token_model_ids(connection, model_id).await?;
    prune_user_model_ids(connection, model_id).await?;
    Ok(())
}

async fn prune_provider_api_key_model_ids<C>(connection: &C, model_id: &str) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let records = provider_api_keys::Entity::find().all(connection).await?;
    let now = time::OffsetDateTime::now_utc();
    for record in records {
        let allowed_model_ids: Vec<String> = json::decode_required(record.allowed_model_ids.clone())?;
        let pruned_model_ids = remove_model_id(allowed_model_ids.clone(), model_id);
        if pruned_model_ids.len() == allowed_model_ids.len() {
            continue;
        }
        let mut active: provider_api_keys::ActiveModel = record.into();
        active.allowed_model_ids = Set(json::encode_required(&pruned_model_ids)?);
        active.updated_at = Set(now);
        active.update(connection).await?;
    }
    Ok(())
}

async fn prune_api_token_model_ids<C>(connection: &C, model_id: &str) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let records = api_token_records::Entity::find().all(connection).await?;
    let now = time::OffsetDateTime::now_utc();
    for record in records {
        let allowed_model_ids: Vec<String> = json::decode_required(record.allowed_model_ids.clone())?;
        let pruned_model_ids = remove_model_id(allowed_model_ids.clone(), model_id);
        if pruned_model_ids.len() == allowed_model_ids.len() {
            continue;
        }
        let mut active: api_token_records::ActiveModel = record.into();
        active.allowed_model_ids = Set(json::encode_required(&pruned_model_ids)?);
        active.updated_at = Set(now);
        active.update(connection).await?;
    }
    Ok(())
}

async fn prune_user_model_ids<C>(connection: &C, model_id: &str) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let records = UserEntity::find().all(connection).await?;
    let now = time::OffsetDateTime::now_utc();
    for record in records {
        let allowed_model_ids: Vec<String> = json::decode_required(record.allowed_model_ids.clone())?;
        let pruned_model_ids = remove_model_id(allowed_model_ids.clone(), model_id);
        if pruned_model_ids.len() == allowed_model_ids.len() {
            continue;
        }
        let mut active: UserActiveModel = record.into();
        active.allowed_model_ids = Set(json::encode_required(&pruned_model_ids)?);
        active.updated_at = Set(now);
        active.update(connection).await?;
    }
    Ok(())
}

fn remove_model_id(model_ids: Vec<String>, model_id: &str) -> Vec<String> {
    model_ids.into_iter().filter(|value| value != model_id).collect()
}

#[cfg(test)]
mod tests {
    use super::{delete_model_bindings, prune_api_token_model_ids, prune_provider_api_key_model_ids, prune_user_model_ids, remove_model_id};
    use crate::{api_token::api_token_records, provider::record::provider_api_keys, user::UserRecord};
    use constants::user_group::DEFAULT_USER_GROUP_CODE;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    #[test]
    fn remove_model_id_drops_all_matching_entries() {
        let model_ids = vec!["model-a".to_owned(), "model-b".to_owned(), "model-a".to_owned()];

        let pruned = remove_model_id(model_ids, "model-a");

        assert_eq!(pruned, vec!["model-b".to_owned()]);
    }

    #[tokio::test]
    async fn delete_model_bindings_removes_provider_and_group_bindings() {
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_exec_results([
                MockExecResult {
                    last_insert_id: 0,
                    rows_affected: 1,
                },
                MockExecResult {
                    last_insert_id: 0,
                    rows_affected: 1,
                },
            ])
            .into_connection();

        delete_model_bindings(&connection, "model-a").await.unwrap();

        let logs = connection.into_transaction_log();
        let statements = logs.iter().flat_map(|entry| entry.statements()).collect::<Vec<_>>();
        assert!(
            statements.iter().any(|statement| statement.sql.contains("DELETE FROM \"provider_models\"")),
            "{statements:?}"
        );
        assert!(
            statements
                .iter()
                .any(|statement| statement.sql.contains("DELETE FROM \"billing_group_models\"")),
            "{statements:?}"
        );
    }

    #[tokio::test]
    async fn prune_model_references_removes_deleted_model_from_provider_api_keys() {
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[provider_api_key("key-1", r#"["model-a","model-b"]"#)]])
            .append_query_results([[provider_api_key("key-1", r#"["model-b"]"#)]])
            .append_exec_results([MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();

        prune_provider_api_key_model_ids(&connection, "model-a").await.unwrap();

        let logs = connection.into_transaction_log();
        let statements = logs.iter().flat_map(|entry| entry.statements()).collect::<Vec<_>>();
        let update = statements
            .iter()
            .find(|statement| statement.sql.contains("UPDATE \"provider_api_keys\" SET"))
            .unwrap_or_else(|| panic!("provider_api_keys update should exist"));
        let update = format!("{update:?}");
        assert!(update.contains("model-b"), "{update}");
        assert!(!update.contains("model-a"), "{update}");
    }

    #[tokio::test]
    async fn prune_model_references_removes_deleted_model_from_api_tokens() {
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[api_token("token-1", r#"["model-a","model-b"]"#)]])
            .append_query_results([[api_token("token-1", r#"["model-b"]"#)]])
            .append_exec_results([MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();

        prune_api_token_model_ids(&connection, "model-a").await.unwrap();

        let logs = connection.into_transaction_log();
        let statements = logs.iter().flat_map(|entry| entry.statements()).collect::<Vec<_>>();
        let update = statements
            .iter()
            .find(|statement| statement.sql.contains("UPDATE \"api_tokens\" SET"))
            .unwrap_or_else(|| panic!("api_tokens update should exist"));
        let update = format!("{update:?}");
        assert!(update.contains("model-b"), "{update}");
        assert!(!update.contains("model-a"), "{update}");
    }

    #[tokio::test]
    async fn prune_model_references_removes_deleted_model_from_users() {
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[user("user-1", r#"["model-a","model-b"]"#)]])
            .append_query_results([[user("user-1", r#"["model-b"]"#)]])
            .append_exec_results([MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();

        prune_user_model_ids(&connection, "model-a").await.unwrap();

        let logs = connection.into_transaction_log();
        let statements = logs.iter().flat_map(|entry| entry.statements()).collect::<Vec<_>>();
        let update = statements
            .iter()
            .find(|statement| statement.sql.contains("UPDATE \"users\" SET"))
            .unwrap_or_else(|| panic!("users update should exist"));
        let update = format!("{update:?}");
        assert!(update.contains("model-b"), "{update}");
        assert!(!update.contains("model-a"), "{update}");
    }

    fn provider_api_key(id: &str, allowed_model_ids: &str) -> provider_api_keys::Model {
        provider_api_keys::Model {
            id: id.to_owned(),
            provider_id: "provider-a".to_owned(),
            name: "key-a".to_owned(),
            api_formats: r#"["openai:chat"]"#.to_owned(),
            allowed_model_ids: allowed_model_ids.to_owned(),
            encrypted_api_key: "encrypted".to_owned(),
            note: None,
            internal_priority: 0,
            rpm_limit: None,
            learned_rpm_limit: None,
            cache_ttl_minutes: 60,
            max_probe_interval_minutes: 15,
            time_range_enabled: false,
            time_range_start: None,
            time_range_end: None,
            health_by_format: None,
            circuit_breaker_by_format: None,
            is_active: true,
            created_at: now(),
            updated_at: now(),
        }
    }

    fn api_token(id: &str, allowed_model_ids: &str) -> api_token_records::Model {
        api_token_records::Model {
            id: id.to_owned(),
            user_id: None,
            token_type: "user".to_owned(),
            name: "token-a".to_owned(),
            token_value: "raw-token".to_owned(),
            token_hash: "hash".to_owned(),
            token_prefix: "prefix".to_owned(),
            group_code: "group-a".to_owned(),
            expires_at: None,
            model_access_mode: "limited".to_owned(),
            allowed_model_ids: allowed_model_ids.to_owned(),
            rate_limit_rpm: None,
            quota_limit: None,
            used_quota: rust_decimal::Decimal::ZERO,
            request_count: 0,
            is_active: true,
            last_used_at: None,
            created_at: now(),
            updated_at: now(),
        }
    }

    fn user(id: &str, allowed_model_ids: &str) -> UserRecord {
        UserRecord {
            id: id.to_owned(),
            username: "user-a".to_owned(),
            password_hash: "hash".to_owned(),
            email: "user@example.com".to_owned(),
            group_code: DEFAULT_USER_GROUP_CODE.to_owned(),
            role: "user".to_owned(),
            is_active: true,
            is_deleted: false,
            allowed_model_ids: allowed_model_ids.to_owned(),
            allowed_provider_ids: "[]".to_owned(),
            created_at: now(),
            updated_at: now(),
            last_login_at: None,
            auth_source: UserRecord::local_auth_source(),
            email_verified: false,
            rate_limit_rpm: None,
            quota_mode: types::user::USER_QUOTA_MODE_WALLET.to_owned(),
        }
    }

    fn now() -> time::OffsetDateTime {
        time::Date::from_calendar_date(2026, time::Month::May, 20)
            .unwrap()
            .with_hms(10, 30, 0)
            .unwrap()
            .assume_utc()
    }
}
