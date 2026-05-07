use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use types::{
    pagination::{Page, PageRequest},
    rbac::{
        ApiPermission, ApiPermissionInput, MenuItem, MenuItemInput, MenuSection, MenuSectionInput, NavResponse, PermissionSnapshot, Role, RoleInput,
        RoleMenuBindingInput,
    },
};

use crate::application::{RbacCache, RbacError, RbacRepository, RbacResult};

use super::test_fixtures::{api_permission, menu_item, menu_section, page_items, permission_snapshot, role_from_input};

#[derive(Clone, Default)]
pub(super) struct MemoryRbacRepository {
    state: Arc<Mutex<RepositoryState>>,
}

#[derive(Default)]
struct RepositoryState {
    roles: Vec<Role>,
    apis: Vec<ApiPermission>,
    menu_sections: Vec<MenuSection>,
    menu_items: Vec<MenuItem>,
}

#[derive(Clone, Default)]
pub(super) struct MemoryRbacCache {
    state: Arc<Mutex<CacheState>>,
}

#[derive(Default)]
struct CacheState {
    snapshot: Option<PermissionSnapshot>,
    write_count: usize,
}

impl MemoryRbacRepository {
    pub(super) fn with_role(role: Role) -> Self {
        Self {
            state: Arc::new(Mutex::new(RepositoryState {
                roles: vec![role],
                ..RepositoryState::default()
            })),
        }
    }

    pub(super) fn with_apis(apis: Vec<ApiPermission>) -> Self {
        Self {
            state: Arc::new(Mutex::new(RepositoryState {
                apis,
                ..RepositoryState::default()
            })),
        }
    }
}

impl MemoryRbacCache {
    pub(super) fn with_snapshot(snapshot: PermissionSnapshot) -> Self {
        Self {
            state: Arc::new(Mutex::new(CacheState {
                snapshot: Some(snapshot),
                write_count: 0,
            })),
        }
    }

    pub(super) fn write_count(&self) -> usize {
        self.state.lock().unwrap().write_count
    }
}

#[async_trait]
impl RbacRepository for MemoryRbacRepository {
    async fn create_role(&self, input: RoleInput) -> RbacResult<Role> {
        let role = role_from_input(input);
        self.state.lock().unwrap().roles.push(role.clone());
        Ok(role)
    }

    async fn replace_role(&self, code: &str, input: RoleInput) -> RbacResult<Role> {
        let mut state = self.state.lock().unwrap();
        let role = state.roles.iter_mut().find(|role| role.code == code).ok_or(RbacError::NotFound)?;
        *role = role_from_input(input);
        Ok(role.clone())
    }

    async fn delete_role(&self, code: &str) -> RbacResult<()> {
        self.state.lock().unwrap().roles.retain(|role| role.code != code);
        Ok(())
    }

    async fn find_role(&self, code: &str) -> RbacResult<Option<Role>> {
        Ok(self.state.lock().unwrap().roles.iter().find(|role| role.code == code).cloned())
    }

    async fn list_roles(&self) -> RbacResult<Vec<Role>> {
        Ok(self.state.lock().unwrap().roles.clone())
    }

    async fn page_roles(&self, page: PageRequest) -> RbacResult<Page<Role>> {
        Ok(page_items(self.state.lock().unwrap().roles.clone(), page))
    }

    async fn create_api(&self, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        let mut state = self.state.lock().unwrap();
        let api = api_permission(state.apis.len() as u64 + 1, input);
        state.apis.push(api.clone());
        Ok(api)
    }

    async fn replace_api(&self, id: &str, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        let mut state = self.state.lock().unwrap();
        let api = state.apis.iter_mut().find(|api| api.id == id).ok_or(RbacError::NotFound)?;
        *api = api_permission(id_number(id), input);
        Ok(api.clone())
    }

    async fn delete_api(&self, id: &str) -> RbacResult<()> {
        self.state.lock().unwrap().apis.retain(|api| api.id != id);
        Ok(())
    }

    async fn list_apis(&self) -> RbacResult<Vec<ApiPermission>> {
        Ok(self.state.lock().unwrap().apis.clone())
    }

    async fn page_apis(&self, page: PageRequest) -> RbacResult<Page<ApiPermission>> {
        Ok(page_items(self.state.lock().unwrap().apis.clone(), page))
    }

    async fn create_menu_section(&self, input: MenuSectionInput) -> RbacResult<MenuSection> {
        let mut state = self.state.lock().unwrap();
        let section = menu_section(state.menu_sections.len() as u64 + 1, input);
        state.menu_sections.push(section.clone());
        Ok(section)
    }

    async fn replace_menu_section(&self, id: &str, input: MenuSectionInput) -> RbacResult<MenuSection> {
        let mut state = self.state.lock().unwrap();
        let section = state.menu_sections.iter_mut().find(|section| section.id == id).ok_or(RbacError::NotFound)?;
        *section = menu_section(id_number(id), input);
        Ok(section.clone())
    }

    async fn delete_menu_section(&self, id: &str) -> RbacResult<()> {
        self.state.lock().unwrap().menu_sections.retain(|section| section.id != id);
        Ok(())
    }

    async fn page_menu_sections(&self, page: PageRequest) -> RbacResult<Page<MenuSection>> {
        Ok(page_items(self.state.lock().unwrap().menu_sections.clone(), page))
    }

    async fn create_menu_item(&self, input: MenuItemInput) -> RbacResult<MenuItem> {
        let mut state = self.state.lock().unwrap();
        let item = menu_item(state.menu_items.len() as u64 + 1, input);
        state.menu_items.push(item.clone());
        Ok(item)
    }

    async fn replace_menu_item(&self, id: &str, input: MenuItemInput) -> RbacResult<MenuItem> {
        let mut state = self.state.lock().unwrap();
        let item = state.menu_items.iter_mut().find(|item| item.id == id).ok_or(RbacError::NotFound)?;
        *item = menu_item(id_number(id), input);
        Ok(item.clone())
    }

    async fn delete_menu_item(&self, id: &str) -> RbacResult<()> {
        self.state.lock().unwrap().menu_items.retain(|item| item.id != id);
        Ok(())
    }

    async fn page_menu_items(&self, page: PageRequest) -> RbacResult<Page<MenuItem>> {
        Ok(page_items(self.state.lock().unwrap().menu_items.clone(), page))
    }

    async fn replace_role_apis(&self, _role_code: &str, _api_permission_ids: Vec<String>) -> RbacResult<()> {
        Ok(())
    }

    async fn replace_role_menus(&self, _role_code: &str, _input: RoleMenuBindingInput) -> RbacResult<()> {
        Ok(())
    }

    async fn permission_snapshot(&self) -> RbacResult<PermissionSnapshot> {
        Ok(permission_snapshot())
    }
}

fn id_number(id: &str) -> u64 {
    id.strip_prefix("018f0000-0000-7000-9000-").and_then(|suffix| suffix.parse().ok()).unwrap_or(0)
}

#[async_trait]
impl RbacCache for MemoryRbacCache {
    async fn write_snapshot(&self, snapshot: &PermissionSnapshot) -> RbacResult<()> {
        let mut state = self.state.lock().unwrap();
        state.snapshot = Some(snapshot.clone());
        state.write_count += 1;
        Ok(())
    }

    async fn read_snapshot(&self) -> RbacResult<PermissionSnapshot> {
        self.state
            .lock()
            .unwrap()
            .snapshot
            .clone()
            .ok_or_else(|| RbacError::Infrastructure("rbac cache snapshot is missing".into()))
    }

    async fn read_nav(&self, role_code: &str) -> RbacResult<NavResponse> {
        let snapshot = self.read_snapshot().await?;
        let role_menu = snapshot
            .menus
            .into_iter()
            .find(|menu| menu.role_code == role_code)
            .ok_or_else(|| RbacError::Infrastructure(format!("rbac menu cache is missing for role {role_code}")))?;
        Ok(NavResponse { nav_items: role_menu.sections })
    }
}
