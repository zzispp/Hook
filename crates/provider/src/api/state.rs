use std::sync::Arc;

use crate::application::ProviderUseCase;

#[derive(Clone)]
pub struct ProviderApiState {
    pub providers: Arc<dyn ProviderUseCase>,
}

impl ProviderApiState {
    pub fn new(providers: Arc<dyn ProviderUseCase>) -> Self {
        Self { providers }
    }
}
