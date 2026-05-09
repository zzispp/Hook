use storage::rbac::{MenuApiBindingRecordInput, RoleApiBindingRecordInput, RoleMenuBindingRecordInput};
use types::rbac::{ApiPermission, ApiPermissionSnapshot, MenuItem, MenuSection, NavItemResponse, NavSectionResponse, Role, RoleMenuSnapshot};

pub(super) fn api_snapshots(
    apis: Vec<ApiPermission>,
    items: Vec<MenuItem>,
    roles: Vec<Role>,
    api_bindings: Vec<MenuApiBindingRecordInput>,
    menu_bindings: Vec<RoleMenuBindingRecordInput>,
    role_api_bindings: Vec<RoleApiBindingRecordInput>,
) -> Vec<ApiPermissionSnapshot> {
    apis.into_iter()
        .filter(|api| api.enabled)
        .map(|api| ApiPermissionSnapshot {
            role_codes: role_codes_for_api(&api.id, &items, &roles, &api_bindings, &menu_bindings, &role_api_bindings),
            method: api.method,
            path_pattern: api.path_pattern,
        })
        .collect()
}

fn role_codes_for_api(
    id: &str,
    items: &[MenuItem],
    roles: &[Role],
    api_bindings: &[MenuApiBindingRecordInput],
    menu_bindings: &[RoleMenuBindingRecordInput],
    role_api_bindings: &[RoleApiBindingRecordInput],
) -> Vec<String> {
    let menu_ids = enabled_menu_ids_for_api(id, items, api_bindings);
    let mut codes = role_codes_from_menus(&menu_ids, roles, menu_bindings);
    codes.extend(role_codes_from_apis(id, roles, role_api_bindings));
    codes.sort();
    codes.dedup();
    codes
}

fn role_codes_from_menus(menu_ids: &[String], roles: &[Role], bindings: &[RoleMenuBindingRecordInput]) -> Vec<String> {
    bindings
        .iter()
        .filter(|binding| menu_ids.iter().any(|menu_id| menu_id == &binding.menu_item_id))
        .filter(|binding| role_enabled(roles, &binding.role_code))
        .map(|binding| binding.role_code.clone())
        .collect()
}

fn role_codes_from_apis(id: &str, roles: &[Role], bindings: &[RoleApiBindingRecordInput]) -> Vec<String> {
    bindings
        .iter()
        .filter(|binding| binding.api_permission_id == id)
        .filter(|binding| role_enabled(roles, &binding.role_code))
        .map(|binding| binding.role_code.clone())
        .collect()
}

fn enabled_menu_ids_for_api(id: &str, items: &[MenuItem], bindings: &[MenuApiBindingRecordInput]) -> Vec<String> {
    bindings
        .iter()
        .filter(|binding| binding.api_permission_id == id)
        .filter(|binding| menu_enabled(items, &binding.menu_item_id))
        .map(|binding| binding.menu_item_id.clone())
        .collect()
}

pub(super) fn menu_snapshots(sections: Vec<MenuSection>, items: Vec<MenuItem>, bindings: Vec<RoleMenuBindingRecordInput>) -> Vec<RoleMenuSnapshot> {
    role_codes_for_menus(&bindings)
        .into_iter()
        .map(|role_code| RoleMenuSnapshot {
            sections: sections_for_role(&role_code, &sections, &items, &bindings),
            role_code,
        })
        .collect()
}

fn role_codes_for_menus(bindings: &[RoleMenuBindingRecordInput]) -> Vec<String> {
    let mut role_codes = bindings.iter().map(|binding| binding.role_code.clone()).collect::<Vec<_>>();
    role_codes.sort();
    role_codes.dedup();
    role_codes
}

fn sections_for_role(role_code: &str, sections: &[MenuSection], items: &[MenuItem], bindings: &[RoleMenuBindingRecordInput]) -> Vec<NavSectionResponse> {
    sections
        .iter()
        .filter(|section| section.enabled)
        .filter_map(|section| nav_section_for_role(role_code, section, items, bindings))
        .collect()
}

fn nav_section_for_role(role_code: &str, section: &MenuSection, items: &[MenuItem], bindings: &[RoleMenuBindingRecordInput]) -> Option<NavSectionResponse> {
    let nav_items = child_items(role_code, &section.id, None, items, bindings);
    if nav_items.is_empty() {
        return None;
    }
    Some(NavSectionResponse {
        code: section.code.clone(),
        subheader: section.subheader.clone(),
        items: nav_items,
    })
}

fn child_items(
    role_code: &str,
    section_id: &str,
    parent_id: Option<&str>,
    items: &[MenuItem],
    bindings: &[RoleMenuBindingRecordInput],
) -> Vec<NavItemResponse> {
    items
        .iter()
        .filter(|item| menu_visible(role_code, section_id, parent_id, item, bindings))
        .map(|item| nav_item(role_code, item, items, bindings))
        .collect()
}

fn menu_visible(role_code: &str, section_id: &str, parent_id: Option<&str>, item: &MenuItem, bindings: &[RoleMenuBindingRecordInput]) -> bool {
    item.section_id == section_id && item.parent_id.as_deref() == parent_id && item.enabled && menu_bound(role_code, &item.id, bindings)
}

fn nav_item(role_code: &str, item: &MenuItem, items: &[MenuItem], bindings: &[RoleMenuBindingRecordInput]) -> NavItemResponse {
    NavItemResponse {
        code: item.code.clone(),
        title: item.title.clone(),
        path: item.path.clone(),
        icon: item.icon.clone(),
        caption: item.caption.clone(),
        deep_match: item.deep_match,
        children: child_items(role_code, &item.section_id, Some(item.id.as_str()), items, bindings),
    }
}

fn role_enabled(roles: &[Role], code: &str) -> bool {
    roles.iter().any(|role| role.code == code && role.enabled)
}

fn menu_enabled(items: &[MenuItem], id: &str) -> bool {
    items.iter().any(|item| item.id == id && item.enabled)
}

fn menu_bound(role_code: &str, item_id: &str, bindings: &[RoleMenuBindingRecordInput]) -> bool {
    bindings.iter().any(|binding| binding.role_code == role_code && binding.menu_item_id == item_id)
}
