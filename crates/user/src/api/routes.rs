use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use crate::api::{
    ApiState,
    handlers::{
        account_affiliate_commissions, account_affiliate_referrals, account_affiliate_summary, account_identities, account_oauth_callback, account_oauth_start,
        account_password_change, account_password_email_code, account_profile, account_unlink_identity, account_verify_email, account_wallet_link,
        admin_affiliates_commissions, admin_affiliates_overview, admin_affiliates_relation_changes, admin_affiliates_relations, admin_affiliates_reports,
        admin_unlink_identity, auth_config, bind_oauth_existing, create_user, delete_user, export_account_affiliate_commissions,
        export_admin_affiliates_report, get_user, list_users, me, oauth_callback, oauth_start, refresh, replace_user, request_password_reset,
        request_registration_email_code, reset_password, sign_in, sign_up, update_admin_affiliate_relation, wallet_nonce, wallet_register, wallet_sign_in,
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
        .route("/auth/wallet/register", post(wallet_register))
        .route("/auth/password-reset/request", post(request_password_reset))
        .route("/auth/password-reset/confirm", post(reset_password))
        .route("/auth/me", get(me))
        .route("/account/profile", get(account_profile))
        .route("/account/affiliate-summary", get(account_affiliate_summary))
        .route("/account/affiliate/referrals", get(account_affiliate_referrals))
        .route("/account/affiliate/commissions", get(account_affiliate_commissions))
        .route("/account/affiliate/commissions/export", get(export_account_affiliate_commissions))
        .route("/account/password/email-code", post(account_password_email_code))
        .route("/account/password/change", post(account_password_change))
        .route("/account/email/verify", post(account_verify_email))
        .route("/account/identities", get(account_identities))
        .route("/account/identities/{identity_id}", delete(account_unlink_identity))
        .route("/account/oauth/{provider}/start", get(account_oauth_start))
        .route("/account/oauth/{provider}/callback", get(account_oauth_callback))
        .route("/account/wallet/link", post(account_wallet_link))
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", get(get_user).put(replace_user).delete(delete_user))
        .route("/users/{id}/identities/{identity_id}", delete(admin_unlink_identity))
        .route("/admin/user-groups", get(list_user_groups).post(create_user_group))
        .route(
            "/admin/user-groups/{code}",
            get(get_user_group).patch(update_user_group).delete(delete_user_group),
        )
        .route("/admin/user-groups/{code}/users", get(list_user_group_members))
        .route("/admin/affiliates/overview", get(admin_affiliates_overview))
        .route("/admin/affiliates/relations", get(admin_affiliates_relations))
        .route("/admin/affiliates/relations/{user_id}", patch(update_admin_affiliate_relation))
        .route("/admin/affiliates/relation-changes", get(admin_affiliates_relation_changes))
        .route("/admin/affiliates/commissions", get(admin_affiliates_commissions))
        .route("/admin/affiliates/reports", get(admin_affiliates_reports))
        .route("/admin/affiliates/reports/export", get(export_admin_affiliates_report))
        .with_state(state)
}

#[cfg(test)]
mod tests;
