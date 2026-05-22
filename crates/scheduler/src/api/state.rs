use std::sync::Arc;

use crate::runtime::SchedulerUseCase;

#[derive(Clone)]
pub struct SchedulerApiState {
    pub scheduler: Arc<dyn SchedulerUseCase>,
}

impl SchedulerApiState {
    pub fn new(scheduler: Arc<dyn SchedulerUseCase>) -> Self {
        Self { scheduler }
    }
}
