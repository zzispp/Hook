use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbBackend, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, Statement, Value,
};
use types::{
    pagination::{Page, PageSliceRequest},
    scheduler::{ScheduledTask, ScheduledTaskDefinition, ScheduledTaskRun},
};

use crate::{Database, StorageError, StorageResult, json};

use super::{
    ScheduledTaskClaim, ScheduledTaskRecordPatch, ScheduledTaskRunRecordInput, ScheduledTaskRunRecordPatch,
    record::{ScheduledTaskRecord, ScheduledTaskRunRecord, scheduled_task_runs, scheduled_tasks},
};

const CLAIM_DUE_TASK_SQL: &str = "WITH due AS (\
    SELECT code FROM scheduled_tasks \
    WHERE code = $1 \
      AND enabled = TRUE \
      AND next_run_at <= $2 \
      AND (locked_until IS NULL OR locked_until <= $2) \
    FOR UPDATE SKIP LOCKED\
), updated AS (\
    UPDATE scheduled_tasks AS task \
    SET locked_until = $2 + (task.interval_seconds * INTERVAL '1 second'), \
        locked_by = $3, \
        last_started_at = $2, \
        next_run_at = $2 + (task.interval_seconds * INTERVAL '1 second'), \
        updated_at = $2 \
    FROM due \
    WHERE task.code = due.code \
    RETURNING task.*\
) SELECT * FROM updated";

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
        let codes = definitions.iter().map(|definition| definition.code.clone()).collect::<Vec<_>>();
        let records = scheduled_tasks::Entity::find()
            .filter(scheduled_tasks::Column::Code.is_in(codes))
            .order_by_asc(scheduled_tasks::Column::Code)
            .all(self.database.connection())
            .await?;
        records
            .into_iter()
            .filter(|record| definitions.iter().any(|definition| definition.code == record.code))
            .map(|record| task_response(record, definitions))
            .collect()
    }

    pub async fn update_task(&self, definition: &ScheduledTaskDefinition, patch: ScheduledTaskRecordPatch) -> StorageResult<ScheduledTaskRecord> {
        let record = self.task_record(&definition.code).await?.ok_or(StorageError::NotFound)?;
        let now = time::OffsetDateTime::now_utc();
        let reset_schedule = schedule_reset(&record, &patch, now);
        let mut active: scheduled_tasks::ActiveModel = record.into();
        apply_task_patch(&mut active, patch, reset_schedule)?;
        active.updated_at = Set(now);
        active.update(self.database.connection()).await.map_err(Into::into)
    }

    pub async fn claim_due_task(&self, code: &str, now: time::OffsetDateTime) -> StorageResult<Option<ScheduledTaskClaim>> {
        let lock_owner = self.database.next_id();
        let statement = claim_due_task_statement(code, now, &lock_owner);
        let record = scheduled_tasks::Model::find_by_statement(statement).one(self.database.connection()).await?;
        Ok(record.map(|record| ScheduledTaskClaim {
            record,
            lock_owner,
            started_at: now,
        }))
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

    pub async fn finish_claimed_task_run(&self, code: &str, lock_owner: &str, patch: ScheduledTaskRunRecordPatch) -> StorageResult<bool> {
        let result = scheduled_tasks::Entity::update_many()
            .col_expr(scheduled_tasks::Column::LastFinishedAt, sea_orm::sea_query::Expr::val(Some(patch.finished_at)))
            .col_expr(scheduled_tasks::Column::LastStatus, sea_orm::sea_query::Expr::val(Some(patch.status.as_str())))
            .col_expr(scheduled_tasks::Column::LastDurationMs, sea_orm::sea_query::Expr::val(Some(patch.duration_ms)))
            .col_expr(scheduled_tasks::Column::LastError, sea_orm::sea_query::Expr::val(patch.error))
            .col_expr(
                scheduled_tasks::Column::LockedUntil,
                sea_orm::sea_query::Expr::val(Option::<time::OffsetDateTime>::None),
            )
            .col_expr(scheduled_tasks::Column::LockedBy, sea_orm::sea_query::Expr::val(Option::<String>::None))
            .col_expr(
                scheduled_tasks::Column::UpdatedAt,
                sea_orm::sea_query::Expr::val(time::OffsetDateTime::now_utc()),
            )
            .filter(scheduled_tasks::Column::Code.eq(code))
            .filter(scheduled_tasks::Column::LockedBy.eq(lock_owner))
            .exec(self.database.connection())
            .await?;
        Ok(result.rows_affected > 0)
    }

    pub async fn page_runs(&self, request: PageSliceRequest, task_code: Option<&str>, status: Option<&str>) -> StorageResult<Page<ScheduledTaskRun>> {
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
        next_run_at: Set(next_run_at(now, definition.default_interval_seconds)),
        locked_until: Set(None),
        locked_by: Set(None),
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

fn apply_task_patch(
    active: &mut scheduled_tasks::ActiveModel,
    patch: ScheduledTaskRecordPatch,
    reset_schedule: Option<time::OffsetDateTime>,
) -> StorageResult<()> {
    if let Some(value) = patch.enabled {
        active.enabled = Set(value);
    }
    if let Some(value) = patch.interval_seconds {
        active.interval_seconds = Set(value);
    }
    if let Some(value) = patch.config {
        active.config = Set(json::encode_required(&value)?);
    }
    if let Some(value) = reset_schedule {
        active.next_run_at = Set(value);
        active.locked_until = Set(None);
        active.locked_by = Set(None);
    }
    Ok(())
}

fn claim_due_task_statement(code: &str, now: time::OffsetDateTime, lock_owner: &str) -> Statement {
    Statement::from_sql_and_values(
        DbBackend::Postgres,
        CLAIM_DUE_TASK_SQL,
        vec![Value::from(code.to_owned()), Value::from(now), Value::from(lock_owner.to_owned())],
    )
}

fn next_run_at(now: time::OffsetDateTime, interval_seconds: i64) -> time::OffsetDateTime {
    now + time::Duration::seconds(interval_seconds)
}

fn schedule_reset(record: &ScheduledTaskRecord, patch: &ScheduledTaskRecordPatch, now: time::OffsetDateTime) -> Option<time::OffsetDateTime> {
    let changed = patch.enabled == Some(true) || patch.interval_seconds.is_some();
    changed.then(|| next_run_at(now, patch.interval_seconds.unwrap_or(record.interval_seconds)))
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
