use std::sync::Arc;

use crate::application::RechargeUseCase;

#[derive(Clone)]
pub struct RechargeApiState {
    pub recharge: Arc<dyn RechargeUseCase>,
}

impl RechargeApiState {
    pub fn new(recharge: Arc<dyn RechargeUseCase>) -> Self {
        Self { recharge }
    }
}
