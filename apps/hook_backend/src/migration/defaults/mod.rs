pub mod api;
pub mod menu;

pub const ADMIN_ROLE: &str = "admin";
pub const USER_ROLE: &str = constants::auth::DEFAULT_USER_ROLE;

#[cfg(test)]
pub const AUTHENTICATED_API_CODES: &[&str] = &["auth_me", "navbar_read", "i18n_resources_read", "system_display_currency_read"];

pub const ADMIN_MENU_CODES: &[&str] = &[
    "dashboard_home",
    "admin_wallets",
    "admin_tokens",
    "admin_groups",
    "admin_users",
    "admin_roles",
    "admin_apis",
    "admin_menus",
    "admin_settings",
    "admin_translations",
    "admin_models",
    "admin_providers",
    "admin_request_records",
];

pub const USER_MENU_CODES: &[&str] = &["dashboard_home", "dashboard_models", "wallet_center", "api_tokens"];

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
        api_codes: &["models_public_catalog_read", "groups_available_read"],
    },
    MenuApiBindingDefinition {
        menu_code: "wallet_center",
        api_codes: &["wallet_balance_read", "wallet_transactions_read"],
    },
    MenuApiBindingDefinition {
        menu_code: "api_tokens",
        api_codes: &[
            "groups_available_read",
            "models_public_catalog_read",
            "api_tokens_read",
            "api_tokens_create",
            "api_tokens_detail",
            "api_tokens_update",
            "api_tokens_delete",
            "api_tokens_secret_read",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_users",
        api_codes: &[
            "users_read",
            "users_create",
            "users_update",
            "users_delete",
            "roles_read",
            "admin_wallet_user_balance_read",
            "admin_wallet_transactions_read",
            "admin_wallet_adjust",
            "admin_wallet_recharge",
            "admin_api_tokens_read",
            "admin_api_tokens_create",
            "admin_api_tokens_detail",
            "admin_api_tokens_update",
            "admin_api_tokens_delete",
            "admin_api_tokens_secret_read",
            "groups_available_read",
            "models_public_catalog_read",
        ],
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
    MenuApiBindingDefinition {
        menu_code: "admin_providers",
        api_codes: &[
            "providers_read",
            "providers_create",
            "providers_detail",
            "providers_update",
            "providers_delete",
            "provider_endpoints_read",
            "provider_endpoints_create",
            "provider_keys_read",
            "provider_keys_create",
            "provider_keys_update",
            "provider_keys_delete",
            "provider_models_read",
            "provider_models_create",
            "provider_models_update",
            "provider_models_delete",
            "models_global_read",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_request_records",
        api_codes: &["request_records_read", "request_records_active_read", "request_records_detail"],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_settings",
        api_codes: &[
            "system_settings_read",
            "system_settings_update",
            "system_settings_smtp_test",
            "system_exchange_rate_read",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_translations",
        api_codes: &[
            "admin_i18n_languages_read",
            "admin_i18n_languages_create",
            "admin_i18n_languages_update",
            "admin_i18n_languages_delete",
            "admin_i18n_translations_read",
            "admin_i18n_translations_create",
            "admin_i18n_translations_update",
            "admin_i18n_translations_delete",
            "admin_i18n_bundle_update",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_wallets",
        api_codes: &[
            "admin_wallets_read",
            "admin_wallet_ledger_read",
            "admin_wallet_user_balance_read",
            "admin_wallet_transactions_read",
            "admin_wallet_adjust",
            "admin_wallet_recharge",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_tokens",
        api_codes: &[
            "groups_available_read",
            "models_public_catalog_read",
            "users_read",
            "admin_api_tokens_read",
            "admin_api_tokens_create",
            "admin_api_tokens_detail",
            "admin_api_tokens_update",
            "admin_api_tokens_delete",
            "admin_api_tokens_secret_read",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_groups",
        api_codes: &[
            "groups_read",
            "groups_create",
            "groups_detail",
            "groups_update",
            "groups_delete",
            "models_global_read",
            "providers_read",
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_defaults_exclude_user_only_menus() {
        for user_only_code in ["dashboard_models", "wallet_center", "api_tokens"] {
            assert!(!ADMIN_MENU_CODES.contains(&user_only_code));
        }

        assert!(ADMIN_MENU_CODES.contains(&"dashboard_home"));
        assert!(ADMIN_MENU_CODES.contains(&"admin_models"));
        assert!(ADMIN_MENU_CODES.contains(&"admin_providers"));
        assert!(ADMIN_MENU_CODES.contains(&"admin_request_records"));
        assert!(ADMIN_MENU_CODES.contains(&"admin_wallets"));
        assert!(ADMIN_MENU_CODES.contains(&"admin_tokens"));
    }

    #[test]
    fn default_role_menu_codes_exist() {
        let menu_codes: Vec<_> = menu::MENU_ITEMS.iter().map(|item| item.code).collect();

        for code in ADMIN_MENU_CODES.iter().chain(USER_MENU_CODES) {
            assert!(menu_codes.contains(code), "default menu code does not exist: {code}");
        }
    }

    #[test]
    fn authenticated_api_codes_are_not_menu_bound() {
        let menu_api_codes: Vec<_> = MENU_API_BINDINGS.iter().flat_map(|binding| binding.api_codes.iter().copied()).collect();

        for code in AUTHENTICATED_API_CODES {
            assert!(!menu_api_codes.contains(code));
        }
    }

    #[test]
    fn menu_api_codes_exist_in_default_api_definitions() {
        for code in MENU_API_BINDINGS.iter().flat_map(|binding| binding.api_codes.iter().copied()) {
            assert!(api::position_by_code(code).is_some(), "default API code does not exist: {code}");
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
