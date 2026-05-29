use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::api::{
    ApiState,
    handlers::{
        account_identities, account_password_change, account_password_email_code, account_profile, account_unlink_identity, admin_unlink_identity, auth_config,
        bind_oauth_existing, create_user, delete_user, get_user, list_users, me, oauth_callback, oauth_start, refresh, replace_user, request_password_reset,
        request_registration_email_code, reset_password, sign_in, sign_up, wallet_complete, wallet_email_code, wallet_nonce, wallet_sign_in,
    },
    user_group_handlers::{create_user_group, delete_user_group, get_user_group, list_user_group_members, list_user_groups, update_user_group},
};

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/auth/config", get(auth_config))
        .route("/auth/registration-email-code", post(request_registration_email_code))
        .route("/auth/sign-up", post(sign_up))
        .route("/auth/sign-in", post(sign_in))
        .route("/auth/refresh", post(refresh))
        .route("/auth/oauth/{provider}/start", get(oauth_start))
        .route("/auth/oauth/{provider}/callback", get(oauth_callback))
        .route("/auth/oauth/{provider}/bind-existing", post(bind_oauth_existing))
        .route("/auth/wallet/nonce", post(wallet_nonce))
        .route("/auth/wallet/sign-in", post(wallet_sign_in))
        .route("/auth/wallet/email-code", post(wallet_email_code))
        .route("/auth/wallet/complete", post(wallet_complete))
        .route("/auth/password-reset/request", post(request_password_reset))
        .route("/auth/password-reset/confirm", post(reset_password))
        .route("/auth/me", get(me))
        .route("/account/profile", get(account_profile))
        .route("/account/password/email-code", post(account_password_email_code))
        .route("/account/password/change", post(account_password_change))
        .route("/account/identities", get(account_identities))
        .route("/account/identities/{identity_id}", delete(account_unlink_identity))
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", get(get_user).put(replace_user).delete(delete_user))
        .route("/users/{id}/identities/{identity_id}", delete(admin_unlink_identity))
        .route("/admin/user-groups", get(list_user_groups).post(create_user_group))
        .route(
            "/admin/user-groups/{code}",
            get(get_user_group).patch(update_user_group).delete(delete_user_group),
        )
        .route("/admin/user-groups/{code}/users", get(list_user_group_members))
        .with_state(state)
}

#[cfg(test)]
mod tests;
