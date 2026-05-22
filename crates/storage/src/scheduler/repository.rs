use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use types::{
    pagination::{Page, PageSliceRequest},
    scheduler::{ScheduledTask, ScheduledTaskDefinition, ScheduledTaskRun},
};

use crate::{Database, StorageError, StorageResult, json};

use super::{
    ScheduledTaskRecordPatch, ScheduledTaskRunRecordInput, ScheduledTaskRunRecordPatch,
    record::{ScheduledTaskRecord, ScheduledTaskRunRecord, scheduled_task_runs, scheduled_tasks},
};

#[derive(Clone)]
pub struct SchedulerStore {
    database: Database,
}

impl SchedulerStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn ensure_registered_tasks(&self, definitions: &[ScheduledTaskDefinition]) -> StorageResult<()> {
        for definition in definitions {
            self.ensure_task(definition).await?;
        }
        Ok(())
    }

    pub async fn task_record(&self, code: &str) -> StorageResult<Option<ScheduledTaskRecord>> {
        scheduled_tasks::Entity::find_by_id(code.to_owned())
            .one(self.database.connection())
            .await
            .map_err(Into::into)
    }

    pub async fn list_tasks(&self, definitions: &[ScheduledTaskDefinition]) -> StorageResult<Vec<ScheduledTask>> {
        let records = scheduled_tasks::Entity::find()
            .order_by_asc(scheduled_tasks::Column::Code)
            .all(self.database.connection())
            .await?;
        records.into_iter().map(|record| task_response(record, definitions)).collect()
    }

    pub async fn update_task(
        &self,
        definition: &ScheduledTaskDefinition,
        patch: ScheduledTaskRecordPatch,
    ) -> StorageResult<ScheduledTaskRecord> {
        let record = self.task_record(&definition.code).await?.ok_or(StorageError::NotFound)?;
        let mut active: scheduled_tasks::ActiveModel = record.into();
        apply_task_patch(&mut active, patch)?;
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await.map_err(Into::into)
    }

    pub async fn start_run(&self, input: ScheduledTaskRunRecordInput) -> StorageResult<String> {
        let id = self.database.next_id();
        let active = scheduled_task_runs::ActiveModel {
            id: Set(id.clone()),
            task_code: Set(input.task_code),
            status: Set(input.status.as_str().into()),
            started_at: Set(input.started_at),
            finished_at: Set(None),
            duration_ms: Set(None),
            message: Set(input.message),
            error: Set(input.error),
        };
        active.insert(self.database.connection()).await?;
        Ok(id)
    }

    pub async fn finish_run(&self, id: &str, patch: ScheduledTaskRunRecordPatch) -> StorageResult<()> {
        let record = scheduled_task_runs::Entity::find_by_id(id.to_owned())
            .one(self.database.connection())
            .await?
            .ok_or(StorageError::NotFound)?;
        let mut active: scheduled_task_runs::ActiveModel = record.into();
        active.status = Set(patch.status.as_str().into());
        active.finished_at = Set(Some(patch.finished_at));
        active.duration_ms = Set(Some(patch.duration_ms));
        active.message = Set(patch.message);
        active.error = Set(patch.error);
        active.update(self.database.connection()).await?;
        Ok(())
    }

    pub async fn update_task_last_run(&self, code: &str, patch: ScheduledTaskRunRecordPatch) -> StorageResult<()> {
        let record = self.task_record(code).await?.ok_or(StorageError::NotFound)?;
        let mut active: scheduled_tasks::ActiveModel = record.into();
        active.last_finished_at = Set(Some(patch.finished_at));
        active.last_status = Set(Some(patch.status.as_str().into()));
        active.last_duration_ms = Set(Some(patch.duration_ms));
        active.last_error = Set(patch.error);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?;
        Ok(())
    }

    pub async fn mark_task_started(&self, code: &str, started_at: time::OffsetDateTime) -> StorageResult<()> {
        let record = self.task_record(code).await?.ok_or(StorageError::NotFound)?;
        let mut active: scheduled_tasks::ActiveModel = record.into();
        active.last_started_at = Set(Some(started_at));
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?;
        Ok(())
    }

    pub async fn page_runs(
        &self,
        request: PageSliceRequest,
        task_code: Option<&str>,
        status: Option<&str>,
    ) -> StorageResult<Page<ScheduledTaskRun>> {
        let query = run_filters(scheduled_task_runs::Entity::find(), task_code, status);
        let total = query.clone().count(self.database.connection()).await?;
        let records = query
            .order_by_desc(scheduled_task_runs::Column::StartedAt)
            .offset(request.offset)
            .limit(request.limit)
            .all(self.database.connection())
            .await?;
        let items = records.into_iter().map(ScheduledTaskRunRecord::response).collect::<StorageResult<Vec<_>>>()?;
        Ok(Page {
            items,
            total,
            page: request.page,
            page_size: request.page_size,
        })
    }

    async fn ensure_task(&self, definition: &ScheduledTaskDefinition) -> StorageResult<()> {
        if self.task_record(&definition.code).await?.is_some() {
            return Ok(());
        }
        insert_task(self, definition).await
    }
}

async fn insert_task(store: &SchedulerStore, definition: &ScheduledTaskDefinition) -> StorageResult<()> {
    let now = time::OffsetDateTime::now_utc();
    let active = scheduled_tasks::ActiveModel {
        code: Set(definition.code.clone()),
        enabled: Set(definition.default_enabled),
        interval_seconds: Set(definition.default_interval_seconds),
        config: Set(json::encode_required(&definition.default_config)?),
        last_started_at: Set(None),
        last_finished_at: Set(None),
        last_status: Set(None),
        last_duration_ms: Set(None),
        last_error: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };
    active.insert(store.database.connection()).await?;
    Ok(())
}

fn task_response(record: ScheduledTaskRecord, definitions: &[ScheduledTaskDefinition]) -> StorageResult<ScheduledTask> {
    let definition = definitions
        .iter()
        .find(|item| item.code == record.code)
        .ok_or_else(|| StorageError::Database(format!("scheduled task definition missing: {}", record.code)))?;
    record.response(definition)
}

fn apply_task_patch(active: &mut scheduled_tasks::ActiveModel, patch: ScheduledTaskRecordPatch) -> StorageResult<()> {
    if let Some(value) = patch.enabled {
        active.enabled = Set(value);
    }
    if let Some(value) = patch.interval_seconds {
        active.interval_seconds = Set(value);
    }
    if let Some(value) = patch.config {
        active.config = Set(json::encode_required(&value)?);
    }
    Ok(())
}

fn run_filters(
    mut query: sea_orm::Select<scheduled_task_runs::Entity>,
    task_code: Option<&str>,
    status: Option<&str>,
) -> sea_orm::Select<scheduled_task_runs::Entity> {
    if let Some(value) = task_code.filter(|value| !value.trim().is_empty()) {
        query = query.filter(scheduled_task_runs::Column::TaskCode.eq(value.trim()));
    }
    if let Some(value) = status.filter(|value| !value.trim().is_empty()) {
        query = query.filter(scheduled_task_runs::Column::Status.eq(value.trim()));
    }
    query
}
