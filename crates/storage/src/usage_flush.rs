use sea_orm::{ActiveModelTrait, ConnectionTrait, EntityTrait, Set};

use crate::{StorageError, StorageResult};

use crate::Database;

pub mod usage_flush_batches {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "usage_flush_batches")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub usage_kind: String,
        pub record_count: i64,
        pub created_at: TimeDateTimeWithTimeZone,
    }

    #[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[derive(Clone)]
pub struct UsageFlushStore {
    database: Database,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsageFlushKind {
    Token,
    Model,
}

pub(crate) struct UsageFlushBatch {
    pub id: String,
    pub usage_kind: UsageFlushKind,
    pub record_count: i64,
}

impl UsageFlushStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn delete_batch(&self, id: &str) -> StorageResult<()> {
        usage_flush_batches::Entity::delete_by_id(id.to_owned())
            .exec(self.database.connection())
            .await?;
        Ok(())
    }
}

pub(crate) async fn batch_exists<C>(connection: &C, id: &str) -> StorageResult<bool>
where
    C: ConnectionTrait,
{
    let record = usage_flush_batches::Entity::find_by_id(id.to_owned()).one(connection).await?;
    Ok(record.is_some())
}

pub(crate) fn token_usage_flush_batch(batch_id: &str, count: usize) -> StorageResult<UsageFlushBatch> {
    usage_flush_batch(batch_id, UsageFlushKind::Token, count)
}

pub(crate) fn model_usage_flush_batch(batch_id: &str, count: usize) -> StorageResult<UsageFlushBatch> {
    usage_flush_batch(batch_id, UsageFlushKind::Model, count)
}

pub(crate) async fn insert_batch<C>(connection: &C, batch: UsageFlushBatch) -> StorageResult<()>
where
    C: ConnectionTrait,
{
    let active = usage_flush_batches::ActiveModel {
        id: Set(batch.id),
        usage_kind: Set(batch.usage_kind.as_str().into()),
        record_count: Set(batch.record_count),
        created_at: Set(time::OffsetDateTime::now_utc()),
    };
    active.insert(connection).await?;
    Ok(())
}

impl UsageFlushKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Token => "token",
            Self::Model => "model",
        }
    }
}

fn usage_flush_batch(batch_id: &str, kind: UsageFlushKind, count: usize) -> StorageResult<UsageFlushBatch> {
    Ok(UsageFlushBatch {
        id: batch_id.to_owned(),
        usage_kind: kind,
        record_count: record_count(count)?,
    })
}

fn record_count(value: usize) -> StorageResult<i64> {
    i64::try_from(value).map_err(|_| StorageError::Database("usage flush batch record count exceeds i64 range".into()))
}
