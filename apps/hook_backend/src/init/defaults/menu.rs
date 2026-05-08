use types::rbac::{MenuItemInput, MenuSectionInput};

pub(in crate::init) const USER_MENU_CODES: &[&str] = &["dashboard_home", "dashboard_models"];

const OVERVIEW_SORT_ORDER: i64 = -10;
const RESOURCES_SORT_ORDER: i64 = -5;
const SYSTEM_SORT_ORDER: i64 = 0;

#[derive(Clone, Copy)]
struct MenuItemTemplate {
    section: DefaultSection,
    code: &'static str,
    title: &'static str,
    path: &'static str,
    icon: &'static str,
    sort_order: i64,
    deep_match: bool,
}

#[derive(Clone, Copy)]
enum DefaultSection {
    Overview,
    Resources,
    System,
}

struct DefaultSectionIds<'a> {
    overview: &'a str,
    resources: &'a str,
    system: &'a str,
}

const MENU_TEMPLATES: &[MenuItemTemplate] = &[
    MenuItemTemplate {
        section: DefaultSection::Overview,
        code: "dashboard_home",
        title: "Dashboard",
        path: "/dashboard",
        icon: "icon.dashboard",
        sort_order: 0,
        deep_match: false,
    },
    MenuItemTemplate {
        section: DefaultSection::Resources,
        code: "dashboard_models",
        title: "Model Catalog",
        path: "/dashboard/models",
        icon: "icon.model",
        sort_order: 0,
        deep_match: true,
    },
    MenuItemTemplate {
        section: DefaultSection::System,
        code: "admin_users",
        title: "User Management",
        path: "/dashboard/admin/users",
        icon: "icon.user",
        sort_order: 0,
        deep_match: true,
    },
    MenuItemTemplate {
        section: DefaultSection::System,
        code: "admin_roles",
        title: "Role Management",
        path: "/dashboard/admin/roles",
        icon: "icon.lock",
        sort_order: 10,
        deep_match: true,
    },
    MenuItemTemplate {
        section: DefaultSection::System,
        code: "admin_apis",
        title: "API Management",
        path: "/dashboard/admin/apis",
        icon: "icon.menu",
        sort_order: 20,
        deep_match: true,
    },
    MenuItemTemplate {
        section: DefaultSection::System,
        code: "admin_menus",
        title: "Menu Management",
        path: "/dashboard/admin/menus",
        icon: "icon.menu",
        sort_order: 30,
        deep_match: true,
    },
    MenuItemTemplate {
        section: DefaultSection::System,
        code: "admin_models",
        title: "Model Management",
        path: "/dashboard/admin/models",
        icon: "icon.model",
        sort_order: 40,
        deep_match: true,
    },
];

pub(in crate::init) fn overview_section() -> MenuSectionInput {
    menu_section("overview", "Overview", OVERVIEW_SORT_ORDER)
}

pub(in crate::init) fn resources_section() -> MenuSectionInput {
    menu_section("resources", "Resources", RESOURCES_SORT_ORDER)
}

pub(in crate::init) fn system_section() -> MenuSectionInput {
    menu_section("system_management", "System Management", SYSTEM_SORT_ORDER)
}

pub(in crate::init) fn default_menu_items(overview_section_id: &str, resources_section_id: &str, system_section_id: &str) -> Vec<MenuItemInput> {
    let sections = DefaultSectionIds {
        overview: overview_section_id,
        resources: resources_section_id,
        system: system_section_id,
    };
    MENU_TEMPLATES.iter().map(|template| menu_item(template, &sections)).collect()
}

fn menu_section(code: &str, subheader: &str, sort_order: i64) -> MenuSectionInput {
    MenuSectionInput {
        code: code.into(),
        subheader: subheader.into(),
        sort_order,
        enabled: true,
    }
}

fn menu_item(template: &MenuItemTemplate, sections: &DefaultSectionIds<'_>) -> MenuItemInput {
    MenuItemInput {
        section_id: section_id(template.section, sections).into(),
        parent_id: None,
        code: template.code.into(),
        title: template.title.into(),
        path: template.path.into(),
        icon: Some(template.icon.into()),
        caption: None,
        deep_match: template.deep_match,
        sort_order: template.sort_order,
        enabled: true,
    }
}

fn section_id<'a>(section: DefaultSection, ids: &'a DefaultSectionIds<'a>) -> &'a str {
    match section {
        DefaultSection::Overview => ids.overview,
        DefaultSection::Resources => ids.resources,
        DefaultSection::System => ids.system,
    }
}
