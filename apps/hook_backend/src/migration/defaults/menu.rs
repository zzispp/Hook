pub struct MenuSectionDefinition {
    pub id: &'static str,
    pub code: &'static str,
    pub subheader: &'static str,
    pub sort_order: i64,
}

pub struct MenuItemDefinition {
    pub id: &'static str,
    pub section_id: &'static str,
    pub code: &'static str,
    pub title: &'static str,
    pub path: &'static str,
    pub icon: &'static str,
    pub sort_order: i64,
    pub deep_match: bool,
}

pub const OVERVIEW_SECTION_ID: &str = "00000000-0000-7000-8000-000000000101";
pub const RESOURCES_SECTION_ID: &str = "00000000-0000-7000-8000-000000000102";
pub const SYSTEM_SECTION_ID: &str = "00000000-0000-7000-8000-000000000103";
pub const ACCOUNT_SECTION_ID: &str = "00000000-0000-7000-8000-000000000104";

pub const MENU_SECTIONS: &[MenuSectionDefinition] = &[
    MenuSectionDefinition {
        id: OVERVIEW_SECTION_ID,
        code: "overview",
        subheader: "Overview",
        sort_order: -10,
    },
    MenuSectionDefinition {
        id: RESOURCES_SECTION_ID,
        code: "resources",
        subheader: "Resources",
        sort_order: -5,
    },
    MenuSectionDefinition {
        id: ACCOUNT_SECTION_ID,
        code: "account",
        subheader: "Account",
        sort_order: -3,
    },
    MenuSectionDefinition {
        id: SYSTEM_SECTION_ID,
        code: "system_management",
        subheader: "System Management",
        sort_order: 0,
    },
];

pub const MENU_ITEMS: &[MenuItemDefinition] = &[
    MenuItemDefinition {
        id: "00000000-0000-7000-8000-000000000201",
        section_id: OVERVIEW_SECTION_ID,
        code: "dashboard_home",
        title: "Dashboard",
        path: "/dashboard",
        icon: "icon.dashboard",
        sort_order: 0,
        deep_match: false,
    },
    MenuItemDefinition {
        id: "00000000-0000-7000-8000-000000000202",
        section_id: RESOURCES_SECTION_ID,
        code: "dashboard_models",
        title: "Model Catalog",
        path: "/dashboard/models",
        icon: "icon.model",
        sort_order: 0,
        deep_match: true,
    },
    MenuItemDefinition {
        id: "00000000-0000-7000-8000-000000000208",
        section_id: ACCOUNT_SECTION_ID,
        code: "wallet_center",
        title: "Wallet Center",
        path: "/dashboard/wallet",
        icon: "icon.wallet",
        sort_order: 0,
        deep_match: true,
    },
    MenuItemDefinition {
        id: "00000000-0000-7000-8000-000000000203",
        section_id: SYSTEM_SECTION_ID,
        code: "admin_users",
        title: "User Management",
        path: "/dashboard/admin/users",
        icon: "icon.user",
        sort_order: 0,
        deep_match: true,
    },
    MenuItemDefinition {
        id: "00000000-0000-7000-8000-000000000204",
        section_id: SYSTEM_SECTION_ID,
        code: "admin_roles",
        title: "Role Management",
        path: "/dashboard/admin/roles",
        icon: "icon.lock",
        sort_order: 10,
        deep_match: true,
    },
    MenuItemDefinition {
        id: "00000000-0000-7000-8000-000000000205",
        section_id: SYSTEM_SECTION_ID,
        code: "admin_apis",
        title: "API Management",
        path: "/dashboard/admin/apis",
        icon: "icon.menu",
        sort_order: 20,
        deep_match: true,
    },
    MenuItemDefinition {
        id: "00000000-0000-7000-8000-000000000206",
        section_id: SYSTEM_SECTION_ID,
        code: "admin_menus",
        title: "Menu Management",
        path: "/dashboard/admin/menus",
        icon: "icon.menu",
        sort_order: 30,
        deep_match: true,
    },
    MenuItemDefinition {
        id: "00000000-0000-7000-8000-000000000207",
        section_id: SYSTEM_SECTION_ID,
        code: "admin_models",
        title: "Model Management",
        path: "/dashboard/admin/models",
        icon: "icon.model",
        sort_order: 40,
        deep_match: true,
    },
];
