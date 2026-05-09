use types::rbac::MenuItemInput;

use crate::application::{RbacError, RbacRepository, RbacResult};

pub(super) async fn reject_system_role_update<R: RbacRepository>(repository: &R, code: &str) -> RbacResult<()> {
    let role = repository.find_role(code).await?.ok_or(RbacError::NotFound)?;
    if role.system {
        return Err(RbacError::Conflict("system role cannot be changed".into()));
    }
    Ok(())
}

pub(super) async fn reject_bound_role_delete<R: RbacRepository>(repository: &R, code: &str) -> RbacResult<()> {
    if repository.role_has_menu_bindings(code).await? {
        return Err(RbacError::Conflict("role is still bound to menu items".into()));
    }
    if repository.role_has_api_bindings(code).await? {
        return Err(RbacError::Conflict("role is still bound to API permissions".into()));
    }
    if repository.role_has_users(code).await? {
        return Err(RbacError::Conflict("role is still assigned to users".into()));
    }
    Ok(())
}

pub(super) async fn reject_bound_api_delete<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    if repository.api_has_menu_bindings(id).await? {
        return Err(RbacError::Conflict("API permission is still bound to menu items".into()));
    }
    if repository.api_has_role_bindings(id).await? {
        return Err(RbacError::Conflict("API permission is still bound to roles".into()));
    }
    Ok(())
}

pub(super) async fn reject_non_empty_menu_section_delete<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    if repository.menu_section_has_items(id).await? {
        return Err(RbacError::Conflict("menu section still contains menu items".into()));
    }
    Ok(())
}

pub(super) async fn reject_menu_item_delete_with_dependents<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    if repository.menu_item_has_children(id).await? {
        return Err(RbacError::Conflict("menu item still has child menu items".into()));
    }
    if repository.menu_item_has_role_bindings(id).await? {
        return Err(RbacError::Conflict("menu item is still bound to roles".into()));
    }
    if repository.menu_item_has_api_bindings(id).await? {
        return Err(RbacError::Conflict("menu item is still bound to API permissions".into()));
    }
    Ok(())
}

pub(super) async fn ensure_role_exists<R: RbacRepository>(repository: &R, code: &str) -> RbacResult<()> {
    repository.find_role(code).await?.map(|_| ()).ok_or(RbacError::NotFound)
}

pub(super) async fn ensure_api_permission_exists<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    repository.find_api(id).await?.map(|_| ()).ok_or(RbacError::NotFound)
}

pub(super) async fn ensure_api_permissions_exist<R: RbacRepository>(repository: &R, ids: &[String]) -> RbacResult<()> {
    for id in unique_ids(ids) {
        if repository.find_api(id).await?.is_none() {
            return Err(RbacError::InvalidInput(format!("api permission does not exist: {id}")));
        }
    }
    Ok(())
}

pub(super) async fn ensure_menu_items_exist<R: RbacRepository>(repository: &R, ids: &[String]) -> RbacResult<()> {
    for id in unique_ids(ids) {
        if repository.find_menu_item(id).await?.is_none() {
            return Err(RbacError::InvalidInput(format!("menu item does not exist: {id}")));
        }
    }
    Ok(())
}

pub(super) async fn ensure_menu_section_exists<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    repository
        .find_menu_section(id)
        .await?
        .map(|_| ())
        .ok_or_else(|| RbacError::InvalidInput(format!("menu section does not exist: {id}")))
}

pub(super) async fn ensure_menu_item_exists<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    repository.find_menu_item(id).await?.map(|_| ()).ok_or(RbacError::NotFound)
}

pub(super) async fn ensure_menu_parent_is_valid<R: RbacRepository>(repository: &R, current_id: Option<&str>, input: &MenuItemInput) -> RbacResult<()> {
    let Some(parent_id) = input.parent_id.as_deref() else {
        return Ok(());
    };
    ensure_parent_is_not_self(current_id, parent_id)?;
    ensure_parent_belongs_to_section(repository, parent_id, &input.section_id).await?;
    if let Some(current_id) = current_id {
        ensure_menu_parent_does_not_create_cycle(repository, current_id, parent_id).await?;
    }
    Ok(())
}

fn ensure_parent_is_not_self(current_id: Option<&str>, parent_id: &str) -> RbacResult<()> {
    if current_id == Some(parent_id) {
        return Err(RbacError::InvalidInput("menu item cannot be its own parent".into()));
    }
    Ok(())
}

async fn ensure_parent_belongs_to_section<R: RbacRepository>(repository: &R, parent_id: &str, section_id: &str) -> RbacResult<()> {
    let parent = repository
        .find_menu_item(parent_id)
        .await?
        .ok_or_else(|| RbacError::InvalidInput(format!("parent menu item does not exist: {parent_id}")))?;
    if parent.section_id != section_id {
        return Err(RbacError::InvalidInput("parent menu item must belong to the same section".into()));
    }
    Ok(())
}

async fn ensure_menu_parent_does_not_create_cycle<R: RbacRepository>(repository: &R, current_id: &str, parent_id: &str) -> RbacResult<()> {
    let items = repository.list_menu_items().await?;
    let mut cursor = Some(parent_id);
    while let Some(id) = cursor {
        if id == current_id {
            return Err(RbacError::InvalidInput("menu parent cannot be a descendant of itself".into()));
        }
        cursor = items.iter().find(|item| item.id == id).and_then(|item| item.parent_id.as_deref());
    }
    Ok(())
}

fn unique_ids(ids: &[String]) -> Vec<&str> {
    let mut ids = ids.iter().map(String::as_str).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    ids
}
