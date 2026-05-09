use std::sync::Arc;

use crate::application::GroupUseCase;

#[derive(Clone)]
pub struct GroupApiState {
    pub groups: Arc<dyn GroupUseCase>,
}

impl GroupApiState {
    pub fn new(groups: Arc<dyn GroupUseCase>) -> Self {
        Self { groups }
    }
}
