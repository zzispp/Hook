pub mod api;
pub mod menu;

pub const ADMIN_ROLE: &str = "admin";
pub const USER_ROLE: &str = constants::auth::DEFAULT_USER_ROLE;

pub const USER_API_CODES: &[&str] = &["auth_me", "navbar_read", "models_public_catalog_read"];
pub const USER_MENU_CODES: &[&str] = &["dashboard_home", "dashboard_models"];
