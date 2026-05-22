use std::sync::Arc;

use storage::{scheduler::SchedulerStore, StorageResult};
use types::{
    pagination::{Page, PageSliceRequest},
    scheduler::{ScheduledTask, ScheduledTaskDefinition, ScheduledTaskRun, ScheduledTaskRunListRequest},
};

use crate::runtime::{SchedulerError, SchedulerRegistry, SchedulerResult};

const MIN_PAGE_NUMBER: u64 = 1;
const MIN_PAGE_SIZE: u64 = 1;
const MAX_PAGE_SIZE: u64 = 100;

pub async fn list_tasks(store: &SchedulerStore, registry: &Arc<SchedulerRegistry>) -> SchedulerResult<Vec<ScheduledTask>> {
    let definitions = registry.definitions();
    store.list_tasks(&definitions).await.map_err(Into::into)
}

pub async fn list_runs(store: &SchedulerStore, request: ScheduledTaskRunListRequest) -> SchedulerResult<Page<ScheduledTaskRun>> {
    let slice = slice_request(request.page, request.page_size)?;
    store
        .page_runs(slice, request.task_code.as_deref(), request.status.as_deref())
        .await
        .map_err(Into::into)
}

pub fn task_definition(registry: &Arc<SchedulerRegistry>, code: &str) -> SchedulerResult<ScheduledTaskDefinition> {
    registry
        .definitions()
        .into_iter()
        .find(|definition| definition.code == code)
        .ok_or_else(|| SchedulerError::NotFound(code.to_owned()))
}

pub fn slice_request(page: u64, page_size: u64) -> SchedulerResult<PageSliceRequest> {
    if page < MIN_PAGE_NUMBER {
        return Err(SchedulerError::InvalidInput("page must be greater than 0".into()));
    }
    if !(MIN_PAGE_SIZE..=MAX_PAGE_SIZE).contains(&page_size) {
        return Err(SchedulerError::InvalidInput(format!(
            "page_size must be between {MIN_PAGE_SIZE} and {MAX_PAGE_SIZE}"
        )));
    }
    Ok(PageSliceRequest {
        offset: (page - MIN_PAGE_NUMBER) * page_size,
        limit: page_size,
        page,
        page_size,
    })
}

pub fn next_runtime_config(
    current: &storage::scheduler::ScheduledTaskRecord,
    update_config: Option<serde_json::Value>,
) -> StorageResult<serde_json::Value> {
    match update_config {
        Some(value) => Ok(value),
        None => current.runtime_config(),
    }
}
