use storage::{
    StorageError,
    rbac::{ApiPermissionRecordInput, MenuApiBindingRecordInput, MenuItemRecordInput, MenuSectionRecordInput, RbacRecordFilters, RoleRecordInput},
};
use types::{
    pagination::{PageRequest, PageSliceRequest},
    rbac::{ApiPermissionInput, MenuItemInput, MenuSectionInput, RbacListFilters, RoleInput},
};

use crate::application::RbacError;

pub(super) fn page_request(page: PageRequest) -> PageSliceRequest {
    PageSliceRequest {
        offset: (page.page - 1) * page.page_size,
        limit: page.page_size,
        page: page.page,
        page_size: page.page_size,
    }
}

pub(super) fn role_record_input(input: RoleInput, system: bool) -> RoleRecordInput {
    RoleRecordInput {
        code: input.code,
        name: input.name,
        description: input.description,
        enabled: input.enabled,
        system,
        sort_order: input.sort_order,
    }
}

pub(super) fn rbac_record_filters(filters: RbacListFilters) -> RbacRecordFilters {
    RbacRecordFilters {
        search: filters.search,
        enabled: filters.enabled,
    }
}

pub(super) fn api_record_with_menu_inputs(input: ApiPermissionInput, system: bool) -> (ApiPermissionRecordInput, Vec<MenuApiBindingRecordInput>) {
    let menu_inputs = input
        .menu_item_ids
        .into_iter()
        .map(|menu_item_id| MenuApiBindingRecordInput {
            menu_item_id,
            api_permission_id: String::new(),
        })
        .collect();
    let record_input = ApiPermissionRecordInput {
        code: input.code,
        method: input.method,
        path_pattern: input.path_pattern,
        name: input.name,
        group: input.group,
        enabled: input.enabled,
        system,
    };
    (record_input, menu_inputs)
}

pub(super) fn storage_error(error: StorageError) -> RbacError {
    match error {
        StorageError::NotFound => RbacError::NotFound,
        StorageError::Conflict(message) => RbacError::Conflict(message),
        StorageError::Database(message) => RbacError::Infrastructure(message),
    }
}

pub(super) fn menu_section_record_input(input: MenuSectionInput) -> MenuSectionRecordInput {
    MenuSectionRecordInput {
        code: input.code,
        subheader: input.subheader,
        sort_order: input.sort_order,
        enabled: input.enabled,
    }
}

pub(super) fn menu_item_record_input(input: MenuItemInput) -> MenuItemRecordInput {
    MenuItemRecordInput {
        section_id: input.section_id,
        parent_id: input.parent_id,
        code: input.code,
        title: input.title,
        path: input.path,
        icon: input.icon,
        caption: input.caption,
        deep_match: input.deep_match,
        sort_order: input.sort_order,
        enabled: input.enabled,
    }
}
