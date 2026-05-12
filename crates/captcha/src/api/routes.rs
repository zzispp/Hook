use axum::{
    Router,
    routing::{get, post},
};

use crate::api::{
    CaptchaApiState,
    handlers::{challenge, config, redeem},
};

pub fn create_router(state: CaptchaApiState) -> Router {
    Router::new()
        .route("/captcha/config", get(config))
        .route("/captcha/challenge", post(challenge))
        .route("/captcha/redeem", post(redeem))
        .with_state(state)
}
