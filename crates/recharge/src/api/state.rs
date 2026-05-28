use std::sync::Arc;

use captcha::application::CaptchaUseCase;

use crate::application::RechargeUseCase;

#[derive(Clone)]
pub struct RechargeApiState {
    pub recharge: Arc<dyn RechargeUseCase>,
    pub captcha: Arc<dyn CaptchaUseCase>,
}

impl RechargeApiState {
    pub fn new(recharge: Arc<dyn RechargeUseCase>, captcha: Arc<dyn CaptchaUseCase>) -> Self {
        Self { recharge, captcha }
    }
}
