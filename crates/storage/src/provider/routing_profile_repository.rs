use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use types::provider::RoutingProfile;

use crate::{StorageResult, json};

use super::{record::routing_profiles, routing_repository::RoutingProfileVersionSnapshot};

pub(super) async fn list_profiles<C>(connection: &C) -> StorageResult<Vec<RoutingProfile>>
where
    C: ConnectionTrait,
{
    let records = routing_profiles::Entity::find()
        .order_by_asc(routing_profiles::Column::ProfileId)
        .all(connection)
        .await?;
    records.into_iter().map(|record| json::decode_required(record.profile_config)).collect()
}

pub(super) async fn upsert_profile<C>(connection: &C, profile: RoutingProfile) -> StorageResult<RoutingProfile>
where
    C: ConnectionTrait,
{
    let active = active_model(&profile)?;
    if existing_profile(connection, profile.id.as_str()).await?.is_some() {
        active.update(connection).await?;
    } else {
        active.insert(connection).await?;
    }
    insert_profile_version(connection, &profile).await?;
    Ok(profile)
}

async fn existing_profile<C>(connection: &C, profile_id: &str) -> StorageResult<Option<routing_profiles::Model>>
where
    C: ConnectionTrait,
{
    Ok(routing_profiles::Entity::find()
        .filter(routing_profiles::Column::ProfileId.eq(profile_id))
        .one(connection)
        .await?)
}

fn active_model(profile: &RoutingProfile) -> StorageResult<routing_profiles::ActiveModel> {
    Ok(routing_profiles::ActiveModel {
        profile_id: Set(profile.id.as_str().to_owned()),
        profile_version: Set(profile.version.clone()),
        profile_config: Set(json::encode_required(profile)?),
        updated_at: Set(time::OffsetDateTime::now_utc()),
    })
}

async fn insert_profile_version<C>(connection: &C, profile: &RoutingProfile) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    super::routing_profile_version_repository::insert_profile_version(
        connection,
        &RoutingProfileVersionSnapshot {
            profile_id: profile.id.as_str().to_owned(),
            profile_version: profile.version.clone(),
            admin_weights: profile.weights.clone(),
            learned_weights: None,
            effective_weights: profile.weights.clone(),
            reward_window: types::provider::RoutingMetricWindow::SevenDays,
            sample_count: 0,
            created_at: time::OffsetDateTime::now_utc(),
        },
    )
    .await
}
