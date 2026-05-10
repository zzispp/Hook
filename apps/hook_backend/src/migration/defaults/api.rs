mod admin_wallet;
mod auth;
mod group;
mod model;
mod rbac;
mod setting;
mod token;
mod user;
mod wallet;

pub struct ApiDefinition {
    pub code: &'static str,
    pub method: &'static str,
    pub path_pattern: &'static str,
    pub name: &'static str,
}

const API_GROUPS: &[&[ApiDefinition]] = &[
    auth::AUTH_APIS,
    user::USER_APIS,
    rbac::RBAC_APIS,
    model::MODEL_APIS,
    group::GROUP_APIS,
    token::TOKEN_APIS,
    wallet::WALLET_APIS,
    admin_wallet::ADMIN_WALLET_APIS,
    setting::SETTING_APIS,
];

pub fn iter_definitions() -> impl Iterator<Item = &'static ApiDefinition> {
    API_GROUPS.iter().flat_map(|group| group.iter())
}

pub fn position_by_code(code: &str) -> Option<usize> {
    iter_definitions().position(|item| item.code == code)
}
