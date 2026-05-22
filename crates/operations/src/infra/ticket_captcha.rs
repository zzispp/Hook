use std::sync::Arc;

use async_trait::async_trait;
use captcha::application::{CaptchaError, CaptchaUseCase};

use crate::application::{OperationsError, OperationsResult, TicketCaptchaVerifier};

#[derive(Clone)]
pub struct CaptchaTicketVerifier {
    captcha: Arc<dyn CaptchaUseCase>,
}

impl CaptchaTicketVerifier {
    pub fn new(captcha: Arc<dyn CaptchaUseCase>) -> Self {
        Self { captcha }
    }
}

#[async_trait]
impl TicketCaptchaVerifier for CaptchaTicketVerifier {
    async fn verify_support_ticket(&self, token: Option<&str>) -> OperationsResult<()> {
        self.captcha.verify_support_ticket(token).await.map_err(captcha_error)
    }
}

fn captcha_error(error: CaptchaError) -> OperationsError {
    match error {
        CaptchaError::InvalidInput(message) => OperationsError::InvalidInput(message),
        CaptchaError::Infrastructure(message) => OperationsError::Infrastructure(message),
    }
}
