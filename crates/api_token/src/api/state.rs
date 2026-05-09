use std::sync::Arc;

use crate::application::ApiTokenUseCase;

#[derive(Clone)]
pub struct ApiTokenApiState {
    pub tokens: Arc<dyn ApiTokenUseCase>,
}

impl ApiTokenApiState {
    pub fn new(tokens: Arc<dyn ApiTokenUseCase>) -> Self {
        Self { tokens }
    }
}
