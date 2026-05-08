use std::sync::Arc;

use crate::application::ModelUseCase;

#[derive(Clone)]
pub struct ModelApiState {
    pub models: Arc<dyn ModelUseCase>,
}

impl ModelApiState {
    pub fn new(models: Arc<dyn ModelUseCase>) -> Self {
        Self { models }
    }
}
