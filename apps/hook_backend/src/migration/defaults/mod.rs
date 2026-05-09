pub mod api;
pub mod menu;

pub const ADMIN_ROLE: &str = "admin";
pub const USER_ROLE: &str = constants::auth::DEFAULT_USER_ROLE;

#[cfg(test)]
pub const AUTHENTICATED_API_CODES: &[&str] = &["auth_me", "navbar_read"];

pub const USER_MENU_CODES: &[&str] = &["dashboard_home", "dashboard_models", "wallet_center"];
pub const ADMIN_MENU_EXCLUDED_CODES: &[&str] = &["dashboard_models", "wallet_center"];

pub struct MenuApiBindingDefinition {
    pub menu_code: &'static str,
    pub api_codes: &'static [&'static str],
}

pub struct RoleApiBindingDefinition {
    pub role_code: &'static str,
    pub api_codes: &'static [&'static str],
}

pub const ROLE_API_BINDINGS: &[RoleApiBindingDefinition] = &[];

pub const MENU_API_BINDINGS: &[MenuApiBindingDefinition] = &[
    MenuApiBindingDefinition {
        menu_code: "dashboard_models",
        api_codes: &["models_public_catalog_read"],
    },
    MenuApiBindingDefinition {
        menu_code: "wallet_center",
        api_codes: &["wallet_balance_read", "wallet_transactions_read"],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_users",
        api_codes: &["users_read", "users_create", "users_update", "users_delete", "roles_read"],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_roles",
        api_codes: &[
            "roles_read",
            "roles_create",
            "roles_update",
            "roles_delete",
            "role_permissions_read",
            "role_permissions_update",
            "menu_items_read",
            "apis_read",
            "apis_unbound_read",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_apis",
        api_codes: &[
            "apis_read",
            "apis_create",
            "apis_update",
            "apis_delete",
            "api_menus_read",
            "api_menus_update",
            "menu_items_read",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_menus",
        api_codes: &[
            "menu_sections_read",
            "menu_sections_create",
            "menu_sections_update",
            "menu_sections_delete",
            "menu_items_read",
            "menu_items_create",
            "menu_items_update",
            "menu_items_delete",
            "menu_item_apis_read",
            "menu_item_apis_update",
            "apis_read",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_models",
        api_codes: &[
            "models_global_read",
            "models_global_create",
            "models_global_detail",
            "models_global_update",
            "models_global_delete",
            "models_global_batch_delete",
            "models_global_providers",
            "models_catalog_read",
            "models_external_read",
        ],
    },
];

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
    fn admin_defaults_exclude_user_surfaces() {
        let admin_menu_codes: Vec<_> = admin_menu_codes().collect();

        assert!(!admin_menu_codes.contains(&"dashboard_models"));
        assert!(!admin_menu_codes.contains(&"wallet_center"));
        assert!(admin_menu_codes.contains(&"admin_models"));
    }

    #[test]
    fn authenticated_api_codes_are_not_menu_bound() {
        let menu_api_codes: Vec<_> = MENU_API_BINDINGS.iter().flat_map(|binding| binding.api_codes.iter().copied()).collect();

        for code in AUTHENTICATED_API_CODES {
            assert!(!menu_api_codes.contains(code));
        }
    }

    #[test]
    fn authenticated_api_codes_are_not_role_bound() {
        let role_api_codes: Vec<_> = ROLE_API_BINDINGS.iter().flat_map(|binding| binding.api_codes.iter().copied()).collect();

        for code in AUTHENTICATED_API_CODES {
            assert!(!role_api_codes.contains(code));
        }
    }
}
