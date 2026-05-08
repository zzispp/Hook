pub mod api;
pub mod menu;

pub const ADMIN_ROLE: &str = "admin";
pub const USER_ROLE: &str = constants::auth::DEFAULT_USER_ROLE;

pub const USER_API_CODES: &[&str] = &["auth_me", "navbar_read", "models_public_catalog_read"];
pub const USER_MENU_CODES: &[&str] = &["dashboard_home", "dashboard_models"];
pub const ADMIN_API_EXCLUDED_CODES: &[&str] = &["models_public_catalog_read"];
pub const ADMIN_MENU_EXCLUDED_CODES: &[&str] = &["dashboard_models"];

pub fn admin_api_codes() -> impl Iterator<Item = &'static str> {
    api::API_DEFINITIONS
        .iter()
        .map(|definition| definition.code)
        .filter(|code| !ADMIN_API_EXCLUDED_CODES.contains(code))
}

pub fn admin_menu_codes() -> impl Iterator<Item = &'static str> {
    menu::MENU_ITEMS
        .iter()
        .map(|item| item.code)
        .filter(|code| !ADMIN_MENU_EXCLUDED_CODES.contains(code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_defaults_exclude_user_model_directory() {
        let admin_api_codes: Vec<_> = admin_api_codes().collect();
        let admin_menu_codes: Vec<_> = admin_menu_codes().collect();

        assert!(!admin_api_codes.contains(&"models_public_catalog_read"));
        assert!(admin_api_codes.contains(&"models_global_read"));
        assert!(!admin_menu_codes.contains(&"dashboard_models"));
        assert!(admin_menu_codes.contains(&"admin_models"));
    }
}
