pub mod api;
pub mod menu;

pub const ADMIN_ROLE: &str = "admin";
pub const USER_ROLE: &str = constants::auth::DEFAULT_USER_ROLE;

#[cfg(test)]
pub const AUTHENTICATED_API_CODES: &[&str] = &[
    "auth_me",
    "navbar_read",
    "i18n_resources_read",
    "notifications_read",
    "notifications_read_all",
    "notification_read",
    "notification_delete",
];

pub const ADMIN_MENU_CODES: &[&str] = &[
    "dashboard_home",
    "admin_performance_monitoring",
    "admin_user_stats",
    "admin_cost_analysis",
    "dashboard_model_status",
    "admin_wallets",
    "admin_card_codes",
    "admin_recharges",
    "admin_tokens",
    "admin_model_status_checks",
    "admin_announcements",
    "admin_tickets",
    "admin_user_groups",
    "admin_groups",
    "admin_users",
    "admin_roles",
    "admin_apis",
    "admin_menus",
    "admin_settings",
    "admin_scheduled_tasks",
    "admin_translations",
    "admin_models",
    "admin_providers",
    "admin_cache_monitoring",
    "admin_request_records",
];

pub const USER_MENU_CODES: &[&str] = &[
    "dashboard_home",
    "dashboard_profile",
    "dashboard_model_status",
    "announcements",
    "support_tickets",
    "dashboard_models",
    "dashboard_groups",
    "wallet_center",
    "api_tokens",
    "usage_records",
];

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
        menu_code: "dashboard_home",
        api_codes: &["dashboard_overview_read", "dashboard_activity_read", "dashboard_filter_options_read"],
    },
    MenuApiBindingDefinition {
        menu_code: "dashboard_profile",
        api_codes: &[
            "account_profile_read",
            "account_password_email_code",
            "account_password_change",
            "account_verify_email",
            "account_identities_read",
            "account_identity_delete",
            "account_oauth_start",
            "account_oauth_callback",
            "account_wallet_link",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_user_stats",
        api_codes: &[
            "admin_user_stats_leaderboard_read",
            "admin_user_usage_stats_read",
            "admin_user_stats_time_series_read",
            "users_read",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_cost_analysis",
        api_codes: &[
            "admin_cost_forecast_read",
            "admin_cost_savings_read",
            "admin_api_key_leaderboard_read",
            "admin_provider_usage_aggregation_read",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "dashboard_model_status",
        api_codes: &["model_status_checks_read"],
    },
    MenuApiBindingDefinition {
        menu_code: "announcements",
        api_codes: &["announcements_read", "announcements_detail"],
    },
    MenuApiBindingDefinition {
        menu_code: "support_tickets",
        api_codes: &["tickets_read", "tickets_create", "tickets_detail", "tickets_reply"],
    },
    MenuApiBindingDefinition {
        menu_code: "dashboard_models",
        api_codes: &["models_public_catalog_read"],
    },
    MenuApiBindingDefinition {
        menu_code: "dashboard_groups",
        api_codes: &["groups_available_read", "models_public_catalog_read"],
    },
    MenuApiBindingDefinition {
        menu_code: "wallet_center",
        api_codes: &[
            "wallet_balance_read",
            "wallet_transactions_read",
            "wallet_ledger_entries_read",
            "wallet_daily_model_usage_read",
            "card_codes_redeem",
            "recharge_packages_read",
            "recharge_orders_read",
            "recharge_orders_create",
            "payment_channels_read",
        ],
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
        menu_code: "usage_records",
        api_codes: &["usage_records_read", "models_public_catalog_read"],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_performance_monitoring",
        api_codes: &[
            "performance_monitoring_overview_read",
            "performance_monitoring_realtime_read",
            "performance_monitoring_analytics_read",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_announcements",
        api_codes: &[
            "admin_announcements_read",
            "admin_announcements_create",
            "admin_announcements_detail",
            "admin_announcements_update",
            "admin_announcements_delete",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_tickets",
        api_codes: &["admin_tickets_read", "admin_tickets_detail", "admin_tickets_update", "admin_tickets_reply"],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_users",
        api_codes: &[
            "users_read",
            "users_detail",
            "users_create",
            "users_update",
            "users_delete",
            "users_identity_delete",
            "user_groups_read",
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
            "groups_read",
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
            "provider_keys_batch_priorities",
            "provider_keys_delete",
            "provider_models_read",
            "provider_models_create",
            "provider_models_batch_update",
            "provider_models_update",
            "provider_models_delete",
            "provider_models_test",
            "provider_model_costs_read",
            "provider_model_costs_upsert",
            "provider_model_costs_delete",
            "models_global_read",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_cache_monitoring",
        api_codes: &[
            "cache_monitoring_affinities_read",
            "cache_monitoring_affinity_delete",
            "cache_monitoring_affinities_clear",
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
            "user_groups_read",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_scheduled_tasks",
        api_codes: &["scheduled_tasks_read", "scheduled_tasks_update", "scheduled_task_runs_read"],
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
            "admin_wallet_ledger_entries_global_read",
            "admin_wallet_user_balance_read",
            "admin_wallet_transactions_read",
            "admin_wallet_ledger_entries_read",
            "admin_wallet_daily_model_usage_read",
            "admin_wallet_adjust",
            "admin_wallet_recharge",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_card_codes",
        api_codes: &[
            "admin_card_codes_read",
            "admin_card_codes_generate",
            "admin_card_codes_batch_status",
            "admin_card_code_types_read",
            "admin_card_code_types_create",
            "admin_card_code_types_update",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_recharges",
        api_codes: &[
            "admin_recharge_packages_read",
            "admin_recharge_packages_create",
            "admin_recharge_packages_update",
            "admin_recharge_orders_read",
            "admin_payment_channels_read",
            "admin_payment_channels_update",
            "system_settings_read",
            "system_settings_update",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_tokens",
        api_codes: &[
            "groups_read",
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
        menu_code: "admin_model_status_checks",
        api_codes: &[
            "admin_model_status_checks_read",
            "admin_model_status_checks_create",
            "admin_model_status_checks_batch_create",
            "admin_model_status_checks_update",
            "admin_model_status_checks_delete",
            "admin_model_status_checks_batch_delete",
            "admin_model_status_checks_batch_update",
            "admin_model_status_runs_read",
            "models_global_read",
            "admin_api_tokens_read",
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
            "user_groups_read",
            "models_global_read",
            "providers_read",
        ],
    },
    MenuApiBindingDefinition {
        menu_code: "admin_user_groups",
        api_codes: &[
            "user_groups_read",
            "user_groups_create",
            "user_groups_detail",
            "user_groups_update",
            "user_groups_delete",
            "user_groups_members_read",
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_defaults_exclude_user_only_menus() {
        for user_only_code in ["dashboard_profile", "dashboard_models", "wallet_center", "api_tokens", "usage_records"] {
            assert!(!ADMIN_MENU_CODES.contains(&user_only_code));
        }

        assert!(ADMIN_MENU_CODES.contains(&"dashboard_home"));
        assert!(ADMIN_MENU_CODES.contains(&"admin_models"));
        assert!(ADMIN_MENU_CODES.contains(&"admin_providers"));
        assert!(ADMIN_MENU_CODES.contains(&"admin_cache_monitoring"));
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

        assert!(USER_MENU_CODES.contains(&"dashboard_profile"));
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
