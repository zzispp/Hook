use std::sync::Arc;

use captcha::application::CaptchaUseCase;

use crate::{api::TokenService, application::UserUseCase};

#[derive(Clone)]
pub struct ApiState {
    pub users: Arc<dyn UserUseCase>,
    pub tokens: TokenService,
    pub captcha: Arc<dyn CaptchaUseCase>,
}

impl ApiState {
    pub fn new(users: Arc<dyn UserUseCase>, tokens: TokenService, captcha: Arc<dyn CaptchaUseCase>) -> Self {
        Self { users, tokens, captcha }
    }
}
