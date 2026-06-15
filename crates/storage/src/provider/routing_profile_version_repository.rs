use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, Value};
use types::provider::RoutingMetricWindow;

use crate::{StorageResult, json};

use super::routing_repository::RoutingProfileVersionSnapshot;

pub(super) async fn latest_profile_version<C>(connection: &C, profile_id: &str) -> StorageResult<Option<RoutingProfileVersionSnapshot>>
where
    C: ConnectionTrait,
{
    let sql = "SELECT profile_id, profile_version, admin_weights, learned_weights, effective_weights, reward_window, sample_count, created_at \
               FROM routing_profile_versions WHERE profile_id = $1 ORDER BY created_at DESC LIMIT 1";
    let row = RoutingProfileVersionRow::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        sql.to_owned(),
        vec![Value::from(profile_id.to_owned())],
    ))
    .one(connection)
    .await?;
    row.map(TryInto::try_into).transpose()
}

pub(super) async fn insert_profile_version<C>(connection: &C, record: &RoutingProfileVersionSnapshot) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let values = vec![
        Value::from(uuid::Uuid::now_v7().to_string()),
        Value::from(record.profile_id.clone()),
        Value::from(record.profile_version.clone()),
        Value::from(json::encode_required(&record.admin_weights)?),
        Value::from(record.learned_weights.as_ref().map(json::encode_required).transpose()?),
        Value::from(json::encode_required(&record.effective_weights)?),
        Value::from(record.reward_window.as_str().to_owned()),
        Value::from(record.sample_count as i64),
        Value::from(record.created_at),
    ];
    let sql = "INSERT INTO routing_profile_versions \
               (id, profile_id, profile_version, admin_weights, learned_weights, effective_weights, reward_window, sample_count, created_at) \
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)";
    connection
        .execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, sql.to_owned(), values))
        .await?;
    Ok(())
}

#[derive(Clone, Debug, FromQueryResult)]
struct RoutingProfileVersionRow {
    profile_id: String,
    profile_version: String,
    admin_weights: String,
    learned_weights: Option<String>,
    effective_weights: String,
    reward_window: String,
    sample_count: i64,
    created_at: time::OffsetDateTime,
}

impl TryFrom<RoutingProfileVersionRow> for RoutingProfileVersionSnapshot {
    type Error = crate::StorageError;

    fn try_from(value: RoutingProfileVersionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            profile_id: value.profile_id,
            profile_version: value.profile_version,
            admin_weights: json::decode_required(value.admin_weights)?,
            learned_weights: value.learned_weights.map(json::decode_required).transpose()?,
            effective_weights: json::decode_required(value.effective_weights)?,
            reward_window: RoutingMetricWindow::from(value.reward_window.as_str()),
            sample_count: value.sample_count.max(0) as u64,
            created_at: value.created_at,
        })
    }
}
