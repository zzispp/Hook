use std::sync::Arc;

use captcha::application::CaptchaUseCase;

use crate::{
    api::TokenService,
    application::{AdminAffiliateUseCase, AffiliateUseCase, UserGroupUseCase, UserUseCase},
};

#[derive(Clone)]
pub struct ApiState {
    pub users: Arc<dyn UserUseCase>,
    pub affiliates: Arc<dyn AffiliateUseCase>,
    pub admin_affiliates: Arc<dyn AdminAffiliateUseCase>,
    pub user_groups: Arc<dyn UserGroupUseCase>,
    pub tokens: TokenService,
    pub captcha: Arc<dyn CaptchaUseCase>,
}

impl ApiState {
    pub fn new(
        users: Arc<dyn UserUseCase>,
        affiliates: Arc<dyn AffiliateUseCase>,
        admin_affiliates: Arc<dyn AdminAffiliateUseCase>,
        user_groups: Arc<dyn UserGroupUseCase>,
        tokens: TokenService,
        captcha: Arc<dyn CaptchaUseCase>,
    ) -> Self {
        Self {
            users,
            affiliates,
            admin_affiliates,
            user_groups,
            tokens,
            captcha,
        }
    }
}
