use std::sync::Arc;

use crate::application::{ProviderModelTester, ProviderUseCase};

#[derive(Clone)]
pub struct ProviderApiState {
    pub providers: Arc<dyn ProviderUseCase>,
    pub model_tester: Arc<dyn ProviderModelTester>,
}

impl ProviderApiState {
    pub fn new(providers: Arc<dyn ProviderUseCase>, model_tester: Arc<dyn ProviderModelTester>) -> Self {
        Self { providers, model_tester }
    }
}
