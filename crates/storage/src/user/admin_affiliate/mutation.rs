use sea_orm::{ActiveModelTrait, ColumnTrait, DbBackend, FromQueryResult, QueryFilter, QuerySelect, Set, Statement, TransactionTrait};
use types::user::AffiliateRelationChangeRecord;

use crate::{StorageError, StorageResult};

use super::super::{AffiliateRelationChangeActiveModel, UserActiveModel, UserColumn, UserRecord, query::active_users};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AffiliateRelationUpdateInput {
    pub referrer_aff_code: Option<String>,
    pub clear_referrer: bool,
    pub reason: String,
    pub operator_user_id: Option<String>,
}

pub(super) async fn update_relation(
    store: &super::super::UserStore,
    user_id: &str,
    input: AffiliateRelationUpdateInput,
) -> StorageResult<AffiliateRelationChangeRecord> {
    let reason = normalized_required(&input.reason, "reason")?;
    let tx = store.database.connection().begin().await?;
    let user = locked_user(user_id, &tx).await?;
    let new_referrer_id = relation_target_id(&tx, &user, &input).await?;
    ensure_no_relation_cycle(&tx, &user.id, new_referrer_id.as_deref()).await?;
    let changed = apply_relation_change(&tx, store.database.next_id(), user, new_referrer_id, input.operator_user_id, reason).await?;
    tx.commit().await?;
    Ok(changed.into())
}

async fn locked_user<C>(user_id: &str, connection: &C) -> StorageResult<UserRecord>
where
    C: sea_orm::ConnectionTrait,
{
    active_users()
        .filter(UserColumn::Id.eq(user_id))
        .lock_exclusive()
        .one(connection)
        .await?
        .ok_or(StorageError::NotFound)
}

async fn relation_target_id<C>(connection: &C, user: &UserRecord, input: &AffiliateRelationUpdateInput) -> StorageResult<Option<String>>
where
    C: sea_orm::ConnectionTrait,
{
    if input.clear_referrer {
        return Ok(None);
    }
    let code = normalized_required(input.referrer_aff_code.as_deref().unwrap_or_default(), "referrer_aff_code")?;
    let target = active_users()
        .filter(UserColumn::AffiliateCode.eq(code))
        .one(connection)
        .await?
        .ok_or_else(|| StorageError::Conflict("referrer affiliate code does not exist".into()))?;
    reject_invalid_referrer(user, &target)?;
    Ok(Some(target.id))
}

fn reject_invalid_referrer(user: &UserRecord, target: &UserRecord) -> StorageResult<()> {
    if user.id == target.id {
        return Err(StorageError::Conflict("user cannot refer themselves".into()));
    }
    if target.id == "system" || target.affiliate_code == "system" {
        return Err(StorageError::Conflict("system user cannot be referrer".into()));
    }
    if target.role != constants::auth::DEFAULT_USER_ROLE {
        return Err(StorageError::Conflict("only regular users can be referrers".into()));
    }
    Ok(())
}

async fn ensure_no_relation_cycle<C>(connection: &C, user_id: &str, target_id: Option<&str>) -> StorageResult<()>
where
    C: sea_orm::ConnectionTrait,
{
    let Some(target_id) = target_id else {
        return Ok(());
    };
    let row = CycleRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        cycle_sql(),
        vec![user_id.into(), target_id.into()],
    ))
    .one(connection)
    .await?
    .ok_or_else(|| StorageError::Database("affiliate cycle query returned no rows".into()))?;
    if row.found.unwrap_or(false) {
        return Err(StorageError::Conflict("affiliate relation cycle is not allowed".into()));
    }
    Ok(())
}

async fn apply_relation_change<C>(
    connection: &C,
    change_id: String,
    user: UserRecord,
    new_referrer_id: Option<String>,
    operator_user_id: Option<String>,
    reason: String,
) -> StorageResult<super::super::affiliate_relation_changes::Model>
where
    C: sea_orm::ConnectionTrait,
{
    let now = time::OffsetDateTime::now_utc();
    let user_id = user.id.clone();
    let old_referrer_id = user.referred_by_user_id.clone();
    update_user_referrer(connection, user, new_referrer_id.clone(), now).await?;
    AffiliateRelationChangeActiveModel {
        id: Set(change_id),
        user_id: Set(user_id),
        old_referrer_user_id: Set(old_referrer_id),
        new_referrer_user_id: Set(new_referrer_id),
        operator_user_id: Set(operator_user_id),
        reason: Set(reason),
        created_at: Set(now),
    }
    .insert(connection)
    .await
    .map_err(Into::into)
}

async fn update_user_referrer<C>(connection: &C, user: UserRecord, new_referrer_id: Option<String>, now: time::OffsetDateTime) -> StorageResult<()>
where
    C: sea_orm::ConnectionTrait,
{
    let referred_at = new_referrer_id.as_ref().map(|_| now);
    let mut active: UserActiveModel = user.into();
    active.referred_by_user_id = Set(new_referrer_id);
    active.referred_at = Set(referred_at);
    active.updated_at = Set(now);
    active.update(connection).await?;
    Ok(())
}

fn normalized_required(value: &str, field: &str) -> StorageResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(StorageError::Conflict(format!("{field} is required")));
    }
    Ok(trimmed.to_owned())
}

fn cycle_sql() -> String {
    "WITH RECURSIVE ancestors(id, referred_by_user_id) AS ( \
        SELECT id, referred_by_user_id FROM users WHERE id = $2 AND is_deleted = FALSE \
        UNION ALL SELECT u.id, u.referred_by_user_id FROM users u JOIN ancestors a ON u.id = a.referred_by_user_id WHERE u.is_deleted = FALSE \
    ) SELECT EXISTS(SELECT 1 FROM ancestors WHERE id = $1) AS found"
        .into()
}

#[derive(Debug, FromQueryResult)]
struct CycleRow {
    found: Option<bool>,
}
