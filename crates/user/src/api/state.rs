use std::sync::Arc;

use captcha::application::CaptchaUseCase;

use crate::{
    api::TokenService,
    application::{UserGroupUseCase, UserUseCase},
};

#[derive(Clone)]
pub struct ApiState {
    pub users: Arc<dyn UserUseCase>,
    pub user_groups: Arc<dyn UserGroupUseCase>,
    pub tokens: TokenService,
    pub captcha: Arc<dyn CaptchaUseCase>,
}

impl ApiState {
    pub fn new(users: Arc<dyn UserUseCase>, user_groups: Arc<dyn UserGroupUseCase>, tokens: TokenService, captcha: Arc<dyn CaptchaUseCase>) -> Self {
        Self {
            users,
            user_groups,
            tokens,
            captcha,
        }
    }
}
