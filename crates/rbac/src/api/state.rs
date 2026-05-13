use std::sync::Arc;

use crate::application::{AuthorizationConfig, RbacAdminUseCase, RbacUseCase};

#[derive(Clone)]
pub struct RbacApiState {
    pub authorization: AuthorizationConfig,
    pub rbac: Arc<dyn RbacUseCase>,
    pub rbac_admin: Arc<dyn RbacAdminUseCase>,
}

impl RbacApiState {
    pub fn new(authorization: AuthorizationConfig, rbac: Arc<dyn RbacUseCase>, rbac_admin: Arc<dyn RbacAdminUseCase>) -> Self {
        Self {
            authorization,
            rbac,
            rbac_admin,
        }
    }
}
