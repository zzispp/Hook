use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbBackend, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect, Set, Statement,
    TransactionTrait,
};
use types::{
    api_token::ApiToken,
    model_status::{
        ModelStatusCheckCreate, ModelStatusCheckListResponse, ModelStatusCheckResponse, ModelStatusCheckUpdate, ModelStatusListRequest,
        ModelStatusRunListRequest, ModelStatusRunListResponse,
    },
};

use crate::{
    Database, StorageError, StorageResult,
    api_token::api_token_records,
    model::global_models,
    model_status::{
        ModelStatusDueRecord, ModelStatusRetentionReport, ModelStatusRunRecordInput,
        entities::{checks, runs},
        hourly_stats::upsert_hourly_stat,
        list_query::{availability_statement, check_rows_statement},
        query::{AvailabilityRow, CheckRow, availability_for, empty_availability, list_response, range_bounds, response, timeline_point},
        retention, run_query,
    },
};

const TIMELINE_LIMIT: i64 = 60;
const LOCK_SECONDS: i64 = 120;
pub(super) const DUE_CHECKS_SQL: &str = "WITH due AS (SELECT id FROM model_status_checks WHERE enabled = TRUE AND next_due_at <= $1 AND (locked_until IS NULL OR locked_until <= $1) ORDER BY next_due_at ASC LIMIT {limit} FOR UPDATE SKIP LOCKED), \
             updated AS (UPDATE model_status_checks c SET locked_until = $2, updated_at = $1 FROM due WHERE c.id = due.id RETURNING c.*) \
             SELECT c.id, c.name, c.global_model_id, g.name AS model_name, c.api_format, c.api_token_id, t.name AS api_token_name, c.interval_seconds, c.enabled, c.next_due_at, c.last_status, c.last_checked_at, c.last_latency_ms, c.last_message, c.created_at, c.updated_at \
             FROM updated c JOIN global_models g ON g.id = c.global_model_id JOIN api_tokens t ON t.id = c.api_token_id";

#[derive(Clone)]
pub struct ModelStatusStore {
    database: Database,
}

impl ModelStatusStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn list_public(&self, request: ModelStatusListRequest) -> StorageResult<ModelStatusCheckListResponse> {
        self.list_checks(request, true).await
    }

    pub async fn list_admin(&self, request: ModelStatusListRequest) -> StorageResult<ModelStatusCheckListResponse> {
        self.list_checks(request, false).await
    }

    pub async fn create_check(&self, input: ModelStatusCheckCreate) -> StorageResult<ModelStatusCheckResponse> {
        self.ensure_model_exists(&input.global_model_id).await?;
        self.ensure_independent_token(&input.api_token_id).await?;
        let now = time::OffsetDateTime::now_utc();
        let id = self.database.next_id();
        checks::ActiveModel {
            id: Set(id.clone()),
            name: Set(input.name),
            global_model_id: Set(input.global_model_id),
            api_format: Set(input.api_format),
            api_token_id: Set(input.api_token_id),
            interval_seconds: Set(input.interval_seconds),
            enabled: Set(input.enabled.unwrap_or(true)),
            next_due_at: Set(now),
            locked_until: Set(None),
            last_status: Set(None),
            last_checked_at: Set(None),
            last_latency_ms: Set(None),
            last_message: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(self.connection())
        .await?;
        self.find_response(&id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn batch_create_checks(
        &self,
        input: types::model_status::ModelStatusCheckBatchCreateRequest,
    ) -> StorageResult<types::model_status::ModelStatusCheckBatchCreateResponse> {
        super::batch_create::batch_create_checks(self, input).await
    }

    pub async fn update_check(&self, id: &str, input: ModelStatusCheckUpdate) -> StorageResult<ModelStatusCheckResponse> {
        let record = checks::Entity::find_by_id(id.to_owned())
            .one(self.connection())
            .await?
            .ok_or(StorageError::NotFound)?;
        if let Some(model_id) = input.global_model_id.as_deref() {
            self.ensure_model_exists(model_id).await?;
        }
        if let Some(token_id) = input.api_token_id.as_deref() {
            self.ensure_independent_token(token_id).await?;
        }
        let mut active: checks::ActiveModel = record.into();
        if let Some(value) = input.name {
            active.name = Set(value);
        }
        if let Some(value) = input.global_model_id {
            active.global_model_id = Set(value);
        }
        if let Some(value) = input.api_format {
            active.api_format = Set(value);
        }
        if let Some(value) = input.api_token_id {
            active.api_token_id = Set(value);
        }
        if let Some(value) = input.interval_seconds {
            active.interval_seconds = Set(value);
        }
        if let Some(value) = input.enabled {
            active.enabled = Set(value);
        }
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.connection()).await?;
        self.find_response(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_check(&self, id: &str) -> StorageResult<()> {
        let record = checks::Entity::find_by_id(id.to_owned())
            .one(self.connection())
            .await?
            .ok_or(StorageError::NotFound)?;
        checks::ActiveModel::from(record).delete(self.connection()).await?;
        Ok(())
    }

    pub async fn batch_update_checks(
        &self,
        input: types::model_status::ModelStatusCheckBatchUpdateRequest,
    ) -> StorageResult<types::model_status::ModelStatusCheckBatchUpdateResponse> {
        super::batch_update::batch_update_checks(self, input).await
    }

    pub async fn list_runs(&self, request: ModelStatusRunListRequest) -> StorageResult<ModelStatusRunListResponse> {
        run_query::list_runs(self, request).await
    }

    pub async fn delete_history_before(&self, cutoff: time::OffsetDateTime) -> StorageResult<ModelStatusRetentionReport> {
        retention::delete_history_before(self, cutoff).await
    }

    pub async fn token_has_checks(&self, token_id: &str) -> StorageResult<bool> {
        checks::Entity::find()
            .filter(checks::Column::ApiTokenId.eq(token_id))
            .one(self.connection())
            .await
            .map(|record| record.is_some())
            .map_err(Into::into)
    }

    pub async fn independent_token(&self, id: &str) -> StorageResult<Option<ApiToken>> {
        let Some(record) = api_token_records::Entity::find_by_id(id.to_owned())
            .filter(api_token_records::Column::TokenType.eq("independent"))
            .filter(api_token_records::Column::IsActive.eq(true))
            .one(self.connection())
            .await?
        else {
            return Ok(None);
        };
        record.into_domain().map(Some)
    }

    pub async fn due_checks(&self, limit: u64, now: time::OffsetDateTime) -> StorageResult<Vec<ModelStatusDueRecord>> {
        let lock_until = now + time::Duration::seconds(LOCK_SECONDS);
        let sql = DUE_CHECKS_SQL.replace("{limit}", &limit.to_string());
        let tx = self.connection().begin().await?;
        let rows = CheckRow::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, vec![now.into(), lock_until.into()]))
            .all(&tx)
            .await?;
        tx.commit().await?;
        let mut output = Vec::with_capacity(rows.len());
        for row in rows {
            let token = self.independent_token(&row.api_token_id).await?.ok_or(StorageError::NotFound)?;
            output.push(ModelStatusDueRecord {
                check_id: row.id,
                model_name: row.model_name,
                api_format: row.api_format,
                interval_seconds: row.interval_seconds,
                token,
            });
        }
        Ok(output)
    }

    pub async fn record_run(&self, record: ModelStatusRunRecordInput, interval_seconds: i64) -> StorageResult<()> {
        let now = time::OffsetDateTime::now_utc();
        let status = record.status.as_str().to_owned();
        let tx = self.connection().begin().await?;
        runs::ActiveModel {
            id: Set(self.database.next_id()),
            check_id: Set(record.check_id.clone()),
            status: Set(status.clone()),
            latency_ms: Set(record.latency_ms),
            status_code: Set(record.status_code),
            message: Set(record.message.clone()),
            checked_at: Set(record.checked_at),
            created_at: Set(now),
        }
        .insert(&tx)
        .await?;
        upsert_hourly_stat(&tx, &self.database.next_id(), &record, now).await?;
        let check = checks::Entity::find_by_id(record.check_id.clone())
            .one(&tx)
            .await?
            .ok_or(StorageError::NotFound)?;
        let mut active: checks::ActiveModel = check.into();
        active.last_status = Set(Some(status));
        active.last_checked_at = Set(Some(record.checked_at));
        active.last_latency_ms = Set(record.latency_ms);
        active.last_message = Set(record.message);
        active.locked_until = Set(None);
        active.next_due_at = Set(record.checked_at + time::Duration::seconds(interval_seconds));
        active.updated_at = Set(now);
        active.update(&tx).await?;
        tx.commit().await?;
        Ok(())
    }

    pub(crate) fn connection(&self) -> &DatabaseConnection {
        self.database.connection()
    }

    async fn list_checks(&self, request: ModelStatusListRequest, public_only: bool) -> StorageResult<ModelStatusCheckListResponse> {
        let rows = self.check_rows(&request, public_only).await?;
        let (started_at, ended_at) = range_bounds(&request)?;
        let check_ids = rows.iter().map(|row| row.id.clone()).collect::<Vec<_>>();
        let availability = self.availability(started_at, ended_at, &check_ids).await?;
        let mut checks = Vec::with_capacity(rows.len());
        for row in rows {
            let availability = availability_for(&availability, &row.id);
            let timeline = self.timeline(&row.id, started_at, ended_at).await?;
            checks.push(response(row, availability, timeline));
        }
        Ok(list_response(checks))
    }

    async fn check_rows(&self, request: &ModelStatusListRequest, public_only: bool) -> StorageResult<Vec<CheckRow>> {
        CheckRow::find_by_statement(check_rows_statement(request, public_only))
            .all(self.connection())
            .await
            .map_err(Into::into)
    }

    async fn find_response(&self, id: &str) -> StorageResult<Option<ModelStatusCheckResponse>> {
        let request = ModelStatusListRequest::default();
        let row = self.check_rows(&request, false).await?.into_iter().find(|row| row.id == id);
        Ok(row.map(|row| response(row, empty_availability(), Vec::new())))
    }

    async fn availability(
        &self,
        started_at: time::OffsetDateTime,
        ended_at: time::OffsetDateTime,
        check_ids: &[String],
    ) -> StorageResult<Vec<AvailabilityRow>> {
        if check_ids.is_empty() {
            return Ok(Vec::new());
        }
        AvailabilityRow::find_by_statement(availability_statement(started_at, ended_at, check_ids))
            .all(self.connection())
            .await
            .map_err(Into::into)
    }

    async fn timeline(
        &self,
        check_id: &str,
        started_at: time::OffsetDateTime,
        ended_at: time::OffsetDateTime,
    ) -> StorageResult<Vec<types::model_status::ModelStatusTimelinePoint>> {
        let rows = runs::Entity::find()
            .filter(runs::Column::CheckId.eq(check_id))
            .filter(runs::Column::CheckedAt.gte(started_at))
            .filter(runs::Column::CheckedAt.lt(ended_at))
            .order_by_desc(runs::Column::CheckedAt)
            .limit(TIMELINE_LIMIT as u64)
            .all(self.connection())
            .await?;
        rows.into_iter().map(timeline_point).collect()
    }

    async fn ensure_model_exists(&self, id: &str) -> StorageResult<()> {
        global_models::Entity::find_by_id(id.to_owned())
            .one(self.connection())
            .await?
            .ok_or(StorageError::NotFound)
            .map(|_| ())
    }

    async fn ensure_independent_token(&self, id: &str) -> StorageResult<()> {
        self.independent_token(id).await?.ok_or(StorageError::NotFound).map(|_| ())
    }
}
