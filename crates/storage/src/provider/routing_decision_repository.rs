use sea_orm::{ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, QueryFilter, Statement, Value};
use types::provider::{RouteIdentity, RouteScoreExplanation, RoutingDecisionResponse};

use crate::{StorageResult, json};

use super::{record::routing_decision_samples, routing_repository::DecisionSamplePayload};

const DECISION_RETENTION_SECONDS: i64 = 86_400;

pub(super) async fn upsert_decision_sample<C>(
    connection: &C,
    request_id: &str,
    profile_id: &str,
    profile_version: &str,
    selected: Option<&RouteIdentity>,
    candidates: &[RouteScoreExplanation],
) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let selected_route = selected.map(json::encode_required).transpose()?;
    let payload = DecisionSamplePayload {
        candidates: candidates.to_vec(),
    };
    let values = decision_values(request_id, profile_id, profile_version, selected_route, &payload)?;
    connection
        .execute_raw(Statement::from_sql_and_values(DbBackend::Postgres, upsert_sql().to_owned(), values))
        .await?;
    prune_decision_samples(connection, time::OffsetDateTime::now_utc()).await
}

pub(super) async fn get_decision_sample<C>(connection: &C, request_id: &str) -> StorageResult<Option<RoutingDecisionResponse>>
where
    C: ConnectionTrait,
{
    let Some(record) = routing_decision_samples::Entity::find_by_id(request_id.to_owned()).one(connection).await? else {
        return Ok(None);
    };
    let selected = record.selected_route.map(json::decode_required::<RouteIdentity>).transpose()?;
    let payload: DecisionSamplePayload = json::decode_required(record.candidate_scores)?;
    Ok(Some(RoutingDecisionResponse {
        request_id: record.request_id,
        profile_id: types::provider::RoutingProfileId::from(record.profile_id.as_str()),
        profile_version: record.profile_version,
        selected,
        candidates: payload.candidates,
        created_at: super::record::format_timestamp(record.created_at),
    }))
}

async fn prune_decision_samples<C>(connection: &C, now: time::OffsetDateTime) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let cutoff = now - time::Duration::seconds(DECISION_RETENTION_SECONDS);
    routing_decision_samples::Entity::delete_many()
        .filter(routing_decision_samples::Column::CreatedAt.lt(cutoff))
        .exec(connection)
        .await?;
    Ok(())
}

fn decision_values(
    request_id: &str,
    profile_id: &str,
    profile_version: &str,
    selected_route: Option<String>,
    payload: &DecisionSamplePayload,
) -> StorageResult<Vec<Value>> {
    let candidates_json = json::encode_required(payload)?;
    let exclusion_json = json::encode_required(&exclusions(&payload.candidates))?;
    Ok(vec![
        Value::from(request_id.to_owned()),
        Value::from(profile_id.to_owned()),
        Value::from(profile_version.to_owned()),
        Value::from(selected_route),
        Value::from(candidates_json),
        Value::from(exclusion_json),
        Value::from(time::OffsetDateTime::now_utc()),
    ])
}

fn exclusions(candidates: &[RouteScoreExplanation]) -> Vec<(&RouteIdentity, &String)> {
    candidates
        .iter()
        .filter_map(|item| item.exclusion_reason.as_ref().map(|reason| (&item.route, reason)))
        .collect()
}

fn upsert_sql() -> &'static str {
    "INSERT INTO routing_decision_samples (request_id, profile_id, profile_version, selected_route, candidate_scores, exclusion_reasons, created_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (request_id) DO UPDATE SET profile_id = EXCLUDED.profile_id, profile_version = EXCLUDED.profile_version, \
     selected_route = EXCLUDED.selected_route, candidate_scores = EXCLUDED.candidate_scores, exclusion_reasons = EXCLUDED.exclusion_reasons, created_at = EXCLUDED.created_at"
}
