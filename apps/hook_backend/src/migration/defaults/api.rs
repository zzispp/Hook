mod admin_affiliate;
mod admin_wallet;
mod auth;
mod cache_monitoring;
mod card_code;
mod dashboard;
mod group;
mod i18n;
mod model;
mod model_status;
mod operations;
mod performance_monitoring;
mod provider;
mod rbac;
mod recharge;
mod routing;
mod scheduler;
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
    dashboard::DASHBOARD_APIS,
    rbac::RBAC_APIS,
    model::MODEL_APIS,
    model_status::MODEL_STATUS_APIS,
    operations::OPERATIONS_APIS,
    provider::PROVIDER_APIS,
    recharge::RECHARGE_APIS,
    group::GROUP_APIS,
    i18n::I18N_APIS,
    token::TOKEN_APIS,
    wallet::WALLET_APIS,
    admin_affiliate::ADMIN_AFFILIATE_APIS,
    admin_wallet::ADMIN_WALLET_APIS,
    card_code::CARD_CODE_APIS,
    setting::SETTING_APIS,
    scheduler::SCHEDULER_APIS,
    performance_monitoring::PERFORMANCE_MONITORING_APIS,
    cache_monitoring::CACHE_MONITORING_APIS,
    routing::ROUTING_APIS,
];

pub fn iter_definitions() -> impl Iterator<Item = &'static ApiDefinition> {
    API_GROUPS.iter().flat_map(|group| group.iter())
}

pub fn position_by_code(code: &str) -> Option<usize> {
    iter_definitions().position(|item| item.code == code)
}
