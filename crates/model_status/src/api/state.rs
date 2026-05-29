use std::sync::Arc;

use crate::application::ModelStatusUseCase;

#[derive(Clone)]
pub struct ModelStatusApiState {
    pub model_status: Arc<dyn ModelStatusUseCase>,
}

impl ModelStatusApiState {
    pub fn new(model_status: Arc<dyn ModelStatusUseCase>) -> Self {
        Self { model_status }
    }
}
