use async_trait::async_trait;
use storage::{
    Database,
    rbac::{MenuApiBindingRecordInput, RbacStore, RoleApiBindingRecordInput, RoleMenuBindingRecordInput},
};
use types::{
    pagination::Page,
    rbac::{
        ApiMenuBindingInput, ApiPermission, ApiPermissionInput, MenuApiBindingInput, MenuItem, MenuItemInput, MenuSection, MenuSectionInput,
        PermissionSnapshot, RbacListRequest, Role, RoleInput, RolePermissionBindingInput,
    },
};

use crate::application::{RbacRepository, RbacResult};

use super::{
    mapper::{
        api_record_with_menu_inputs, menu_item_record_input, menu_section_record_input, page_request, rbac_record_filters, role_record_input, storage_error,
    },
    snapshot::{api_snapshots, menu_snapshots},
};

#[derive(Clone)]
pub struct StorageRbacRepository {
    store: RbacStore,
}

impl StorageRbacRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: RbacStore::new(database),
        }
    }
}

#[async_trait]
impl RbacRepository for StorageRbacRepository {
    async fn create_role(&self, input: RoleInput) -> RbacResult<Role> {
        self.store.create_role(role_record_input(input, false)).await.map_err(storage_error)
    }

    async fn create_system_role(&self, input: RoleInput) -> RbacResult<Role> {
        self.store.create_role(role_record_input(input, true)).await.map_err(storage_error)
    }

    async fn replace_role(&self, code: &str, input: RoleInput) -> RbacResult<Role> {
        self.store.replace_role(code, role_record_input(input, false)).await.map_err(storage_error)
    }

    async fn replace_system_role(&self, code: &str, input: RoleInput) -> RbacResult<Role> {
        self.store.replace_role(code, role_record_input(input, true)).await.map_err(storage_error)
    }

    async fn delete_role(&self, code: &str) -> RbacResult<()> {
        self.store.delete_role(code).await.map_err(storage_error)
    }

    async fn find_role(&self, code: &str) -> RbacResult<Option<Role>> {
        self.store.find_role(code).await.map_err(storage_error)
    }

    async fn role_has_menu_bindings(&self, code: &str) -> RbacResult<bool> {
        self.store.role_has_menu_bindings(code).await.map_err(storage_error)
    }

    async fn role_has_api_bindings(&self, code: &str) -> RbacResult<bool> {
        self.store.role_has_api_bindings(code).await.map_err(storage_error)
    }

    async fn role_has_users(&self, code: &str) -> RbacResult<bool> {
        self.store.role_has_users(code).await.map_err(storage_error)
    }

    async fn list_roles(&self) -> RbacResult<Vec<Role>> {
        self.store.list_roles().await.map_err(storage_error)
    }

    async fn page_roles(&self, request: RbacListRequest) -> RbacResult<Page<Role>> {
        self.store
            .page_roles(page_request(request.page), rbac_record_filters(request.filters))
            .await
            .map_err(storage_error)
    }

    async fn create_api(&self, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        let (record_input, menu_inputs) = api_record_with_menu_inputs(input, false);
        self.store.create_api_with_menus(record_input, menu_inputs).await.map_err(storage_error)
    }

    async fn replace_api(&self, id: &str, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        let (record_input, menu_inputs) = api_record_with_menu_inputs(input, false);
        self.store.replace_api_with_menus(id, record_input, menu_inputs).await.map_err(storage_error)
    }

    async fn delete_api(&self, id: &str) -> RbacResult<()> {
        self.store.delete_api(id).await.map_err(storage_error)
    }

    async fn find_api(&self, id: &str) -> RbacResult<Option<ApiPermission>> {
        self.store.find_api(id).await.map_err(storage_error)
    }

    async fn api_has_menu_bindings(&self, id: &str) -> RbacResult<bool> {
        self.store.api_has_menu_bindings(id).await.map_err(storage_error)
    }

    async fn api_has_role_bindings(&self, id: &str) -> RbacResult<bool> {
        self.store.api_has_role_bindings(id).await.map_err(storage_error)
    }

    async fn list_apis(&self) -> RbacResult<Vec<ApiPermission>> {
        self.store.list_apis().await.map_err(storage_error)
    }

    async fn page_apis(&self, request: RbacListRequest) -> RbacResult<Page<ApiPermission>> {
        self.store
            .page_apis(page_request(request.page), rbac_record_filters(request.filters))
            .await
            .map_err(storage_error)
    }

    async fn page_unbound_apis(&self, request: RbacListRequest) -> RbacResult<Page<ApiPermission>> {
        self.store
            .page_unbound_apis(page_request(request.page), rbac_record_filters(request.filters))
            .await
            .map_err(storage_error)
    }

    async fn create_menu_section(&self, input: MenuSectionInput) -> RbacResult<MenuSection> {
        self.store.create_menu_section(menu_section_record_input(input)).await.map_err(storage_error)
    }

    async fn replace_menu_section(&self, id: &str, input: MenuSectionInput) -> RbacResult<MenuSection> {
        self.store
            .replace_menu_section(id, menu_section_record_input(input))
            .await
            .map_err(storage_error)
    }

    async fn delete_menu_section(&self, id: &str) -> RbacResult<()> {
        self.store.delete_menu_section(id).await.map_err(storage_error)
    }

    async fn find_menu_section(&self, id: &str) -> RbacResult<Option<MenuSection>> {
        self.store.find_menu_section(id).await.map_err(storage_error)
    }

    async fn menu_section_has_items(&self, id: &str) -> RbacResult<bool> {
        self.store.menu_section_has_items(id).await.map_err(storage_error)
    }

    async fn page_menu_sections(&self, request: RbacListRequest) -> RbacResult<Page<MenuSection>> {
        self.store
            .page_menu_sections(page_request(request.page), rbac_record_filters(request.filters))
            .await
            .map_err(storage_error)
    }

    async fn create_menu_item(&self, input: MenuItemInput) -> RbacResult<MenuItem> {
        self.store.create_menu_item(menu_item_record_input(input)).await.map_err(storage_error)
    }

    async fn replace_menu_item(&self, id: &str, input: MenuItemInput) -> RbacResult<MenuItem> {
        self.store.replace_menu_item(id, menu_item_record_input(input)).await.map_err(storage_error)
    }

    async fn delete_menu_item(&self, id: &str) -> RbacResult<()> {
        self.store.delete_menu_item(id).await.map_err(storage_error)
    }

    async fn find_menu_item(&self, id: &str) -> RbacResult<Option<MenuItem>> {
        self.store.find_menu_item(id).await.map_err(storage_error)
    }

    async fn menu_item_has_children(&self, id: &str) -> RbacResult<bool> {
        self.store.menu_item_has_children(id).await.map_err(storage_error)
    }

    async fn menu_item_has_role_bindings(&self, id: &str) -> RbacResult<bool> {
        self.store.menu_item_has_role_bindings(id).await.map_err(storage_error)
    }

    async fn menu_item_has_api_bindings(&self, id: &str) -> RbacResult<bool> {
        self.store.menu_item_has_api_bindings(id).await.map_err(storage_error)
    }

    async fn list_menu_items(&self) -> RbacResult<Vec<MenuItem>> {
        self.store.list_menu_items().await.map_err(storage_error)
    }

    async fn page_menu_items(&self, request: RbacListRequest) -> RbacResult<Page<MenuItem>> {
        self.store
            .page_menu_items(page_request(request.page), rbac_record_filters(request.filters))
            .await
            .map_err(storage_error)
    }

    async fn replace_menu_apis(&self, menu_item_id: &str, input: MenuApiBindingInput) -> RbacResult<()> {
        let inputs = input
            .api_permission_ids
            .into_iter()
            .map(|api_permission_id| MenuApiBindingRecordInput {
                menu_item_id: menu_item_id.into(),
                api_permission_id,
            })
            .collect();
        self.store.replace_menu_apis(menu_item_id, inputs).await.map_err(storage_error)
    }

    async fn replace_api_menus(&self, api_permission_id: &str, input: ApiMenuBindingInput) -> RbacResult<()> {
        let inputs = input
            .menu_item_ids
            .into_iter()
            .map(|menu_item_id| MenuApiBindingRecordInput {
                menu_item_id,
                api_permission_id: api_permission_id.into(),
            })
            .collect();
        self.store.replace_api_menus(api_permission_id, inputs).await.map_err(storage_error)
    }

    async fn replace_role_permissions(&self, role_code: &str, input: RolePermissionBindingInput) -> RbacResult<()> {
        let menu_inputs = input
            .menu_item_ids
            .into_iter()
            .map(|menu_item_id| RoleMenuBindingRecordInput {
                role_code: role_code.into(),
                menu_item_id,
            })
            .collect();
        let api_inputs = input
            .api_permission_ids
            .into_iter()
            .map(|api_permission_id| RoleApiBindingRecordInput {
                role_code: role_code.into(),
                api_permission_id,
            })
            .collect();
        self.store
            .replace_role_permissions(role_code, menu_inputs, api_inputs)
            .await
            .map_err(storage_error)
    }

    async fn menu_api_ids(&self, menu_item_id: &str) -> RbacResult<Vec<String>> {
        self.store.menu_api_ids(menu_item_id).await.map_err(storage_error)
    }

    async fn api_menu_ids(&self, api_permission_id: &str) -> RbacResult<Vec<String>> {
        self.store.api_menu_ids(api_permission_id).await.map_err(storage_error)
    }

    async fn role_menu_item_ids(&self, role_code: &str) -> RbacResult<Vec<String>> {
        self.store.role_menu_item_ids(role_code).await.map_err(storage_error)
    }

    async fn role_api_ids(&self, role_code: &str) -> RbacResult<Vec<String>> {
        self.store.role_api_ids(role_code).await.map_err(storage_error)
    }

    async fn permission_snapshot(&self) -> RbacResult<PermissionSnapshot> {
        let roles = self.store.list_roles().await.map_err(storage_error)?;
        let apis = self.store.list_apis().await.map_err(storage_error)?;
        let sections = self.store.list_menu_sections().await.map_err(storage_error)?;
        let items = self.store.list_menu_items().await.map_err(storage_error)?;
        let api_bindings = self.store.list_menu_api_bindings().await.map_err(storage_error)?;
        let menu_bindings = self.store.list_role_menu_bindings().await.map_err(storage_error)?;
        let role_api_bindings = self.store.list_role_api_bindings().await.map_err(storage_error)?;
        Ok(PermissionSnapshot {
            api_permissions: api_snapshots(apis, items.clone(), roles.clone(), api_bindings, menu_bindings.clone(), role_api_bindings),
            menus: menu_snapshots(sections, items, menu_bindings),
        })
    }
}
