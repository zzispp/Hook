use std::sync::Arc;

use crate::application::UserUseCase;

#[derive(Clone)]
pub struct ApiState {
    pub users: Arc<dyn UserUseCase>,
}

impl ApiState {
    pub const fn new(users: Arc<dyn UserUseCase>) -> Self {
        Self { users }
    }
}
