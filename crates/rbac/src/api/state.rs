use std::sync::Arc;

use user::{api::TokenService, application::UserUseCase};

use crate::application::{AuthorizationConfig, RbacAdminUseCase, RbacUseCase};

#[derive(Clone)]
pub struct RbacApiState {
    pub users: Arc<dyn UserUseCase>,
    pub tokens: TokenService,
    pub rbac: Arc<dyn RbacUseCase>,
    pub rbac_admin: Arc<dyn RbacAdminUseCase>,
    pub authorization: AuthorizationConfig,
}

impl RbacApiState {
    pub fn new(
        users: Arc<dyn UserUseCase>,
        tokens: TokenService,
        rbac: Arc<dyn RbacUseCase>,
        rbac_admin: Arc<dyn RbacAdminUseCase>,
        authorization: AuthorizationConfig,
    ) -> Self {
        Self {
            users,
            tokens,
            rbac,
            rbac_admin,
            authorization,
        }
    }
}
