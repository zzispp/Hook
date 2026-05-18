use std::sync::Arc;

use crate::application::DashboardUseCase;

#[derive(Clone)]
pub struct DashboardApiState {
    pub dashboard: Arc<dyn DashboardUseCase>,
}

impl DashboardApiState {
    pub fn new(dashboard: Arc<dyn DashboardUseCase>) -> Self {
        Self { dashboard }
    }
}
