use std::sync::Arc;

use crate::application::OperationsUseCase;

#[derive(Clone)]
pub struct OperationsApiState {
    pub operations: Arc<dyn OperationsUseCase>,
}

impl OperationsApiState {
    pub fn new(operations: Arc<dyn OperationsUseCase>) -> Self {
        Self { operations }
    }
}
