use std::collections::BTreeMap;

use configuration::Settings;
use rbac::application::{RbacCache, RbacError, RbacRepository, RbacService};
use types::{
    pagination::PageRequest,
    rbac::{ApiPermission, MenuItem, MenuItemInput, MenuSectionInput, RoleInput, RoleMenuBindingInput},
};

use crate::BackendResult;

mod defaults;

type IdByCode = BTreeMap<String, String>;

pub async fn ensure_default_rbac<R, C>(rbac: &RbacService<R, C>, settings: &Settings) -> BackendResult<()>
where
    R: RbacRepository,
    C: RbacCache,
{
    let admin_role = settings.admin.role.trim();
    rbac.ensure_system_role(RoleInput {
        code: admin_role.into(),
        name: "Administrator".into(),
        description: "Built-in administrator role".into(),
        enabled: true,
        sort_order: 0,
    })
    .await?;

    let user_role = constants::auth::DEFAULT_USER_ROLE;
    rbac.ensure_system_role(RoleInput {
        code: user_role.into(),
        name: "User".into(),
        description: "Default signed-up user role".into(),
        enabled: true,
        sort_order: 10,
    })
    .await?;

    let api_ids_by_code = ensure_default_apis(rbac).await?;
    let menu_item_ids_by_code = ensure_default_menus(rbac).await?;

    rbac.replace_role_apis(admin_role, all_ids(&api_ids_by_code)).await?;
    rbac.replace_role_menus(
        admin_role,
        RoleMenuBindingInput {
            menu_item_ids: all_ids(&menu_item_ids_by_code),
        },
    )
    .await?;
    rbac.replace_role_apis(user_role, ids_for_codes(&api_ids_by_code, defaults::USER_API_CODES, "API permission")?)
        .await?;
    rbac.replace_role_menus(
        user_role,
        RoleMenuBindingInput {
            menu_item_ids: ids_for_codes(&menu_item_ids_by_code, defaults::USER_MENU_CODES, "menu item")?,
        },
    )
    .await?;
    Ok(())
}

async fn ensure_default_apis<R, C>(rbac: &RbacService<R, C>) -> BackendResult<IdByCode>
where
    R: RbacRepository,
    C: RbacCache,
{
    let existing = rbac.list_apis().await?;
    for input in defaults::default_api_permissions() {
        if existing.iter().any(|api| api.code == input.code) {
            continue;
        }

        match rbac.create_api(input).await {
            Ok(_) => {}
            Err(RbacError::Infrastructure(message)) if message.contains("duplicate key") => {}
            Err(error) => return Err(error.into()),
        }
    }

    Ok(api_ids_by_code(rbac.list_apis().await?))
}

async fn ensure_default_menus<R, C>(rbac: &RbacService<R, C>) -> BackendResult<IdByCode>
where
    R: RbacRepository,
    C: RbacCache,
{
    let overview_section_id = ensure_default_menu_section(rbac, defaults::overview_section()).await?;
    let resources_section_id = ensure_default_menu_section(rbac, defaults::resources_section()).await?;
    let system_section_id = ensure_default_menu_section(rbac, defaults::system_section()).await?;

    let existing = rbac.page_menu_items(PageRequest { page: 1, page_size: 100 }).await?.items;
    for input in defaults::default_menu_items(&overview_section_id, &resources_section_id, &system_section_id) {
        if let Some(item) = existing.iter().find(|item| item.code == input.code) {
            sync_existing_default_menu_item(rbac, item, input).await?;
            continue;
        }

        match rbac.create_menu_item(input).await {
            Ok(_) => {}
            Err(RbacError::Infrastructure(message)) if message.contains("duplicate key") => {}
            Err(error) => return Err(error.into()),
        }
    }

    Ok(menu_item_ids_by_code(
        rbac.page_menu_items(PageRequest { page: 1, page_size: 100 }).await?.items,
    ))
}

async fn ensure_default_menu_section<R, C>(rbac: &RbacService<R, C>, input: MenuSectionInput) -> BackendResult<String>
where
    R: RbacRepository,
    C: RbacCache,
{
    if let Some(id) = find_menu_section_id(rbac, &input.code).await? {
        return Ok(id);
    }

    match rbac.create_menu_section(input.clone()).await {
        Ok(section) => Ok(section.id),
        Err(RbacError::Infrastructure(message)) if message.contains("duplicate key") => find_menu_section_id(rbac, &input.code)
            .await?
            .ok_or_else(|| format!("default menu section '{}' was not found after duplicate insert", input.code).into()),
        Err(error) => Err(error.into()),
    }
}

async fn find_menu_section_id<R, C>(rbac: &RbacService<R, C>, code: &str) -> BackendResult<Option<String>>
where
    R: RbacRepository,
    C: RbacCache,
{
    Ok(rbac
        .page_menu_sections(PageRequest { page: 1, page_size: 100 })
        .await?
        .items
        .into_iter()
        .find(|section| section.code == code)
        .map(|section| section.id))
}

async fn sync_existing_default_menu_item<R, C>(rbac: &RbacService<R, C>, item: &MenuItem, input: MenuItemInput) -> BackendResult<()>
where
    R: RbacRepository,
    C: RbacCache,
{
    if item.code != "admin_models" || item.icon == input.icon {
        return Ok(());
    }

    rbac.replace_menu_item(
        &item.id,
        MenuItemInput {
            icon: input.icon,
            ..menu_item_input_from(item)
        },
    )
    .await?;
    Ok(())
}

fn menu_item_input_from(item: &MenuItem) -> MenuItemInput {
    MenuItemInput {
        section_id: item.section_id.clone(),
        parent_id: item.parent_id.clone(),
        code: item.code.clone(),
        title: item.title.clone(),
        path: item.path.clone(),
        icon: item.icon.clone(),
        caption: item.caption.clone(),
        deep_match: item.deep_match,
        sort_order: item.sort_order,
        enabled: item.enabled,
    }
}

fn api_ids_by_code(apis: Vec<ApiPermission>) -> IdByCode {
    apis.into_iter().map(|api| (api.code, api.id)).collect()
}

fn menu_item_ids_by_code(items: Vec<MenuItem>) -> IdByCode {
    items.into_iter().map(|item| (item.code, item.id)).collect()
}

fn all_ids(ids_by_code: &IdByCode) -> Vec<String> {
    ids_by_code.values().cloned().collect()
}

fn ids_for_codes(ids_by_code: &IdByCode, codes: &[&str], kind: &str) -> BackendResult<Vec<String>> {
    let mut ids = Vec::with_capacity(codes.len());
    for code in codes {
        let id = ids_by_code.get(*code).ok_or_else(|| format!("default {kind} '{code}' was not created"))?;
        ids.push(id.clone());
    }
    Ok(ids)
}
