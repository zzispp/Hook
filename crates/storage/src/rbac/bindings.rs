use toasty::stmt::CreateMany;

use crate::{StorageError, StorageResult};

use super::{RbacStore, RoleApiBindingRecordInput, RoleApiPermissionRecord, RoleMenuBindingRecordInput, RoleMenuPermissionRecord};

impl RbacStore {
    pub async fn replace_role_apis(&self, role_code: &str, inputs: Vec<RoleApiBindingRecordInput>) -> StorageResult<()> {
        let mut db = self.database.connection();
        let mut tx = db.transaction().await?;
        RoleApiPermissionRecord::filter(RoleApiPermissionRecord::fields().role_code().eq(role_code))
            .delete()
            .exec(&mut tx)
            .await?;
        create_role_api_bindings(inputs).exec(&mut tx).await?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn replace_role_menus(&self, role_code: &str, inputs: Vec<RoleMenuBindingRecordInput>) -> StorageResult<()> {
        let mut db = self.database.connection();
        let mut tx = db.transaction().await?;
        RoleMenuPermissionRecord::filter(RoleMenuPermissionRecord::fields().role_code().eq(role_code))
            .delete()
            .exec(&mut tx)
            .await?;
        create_role_menu_bindings(inputs).exec(&mut tx).await?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn list_role_api_bindings(&self) -> StorageResult<Vec<RoleApiBindingRecordInput>> {
        let mut db = self.database.connection();
        RoleApiPermissionRecord::all()
            .exec(&mut db)
            .await
            .map(role_api_binding_records)
            .map_err(StorageError::from)
    }

    pub async fn role_api_ids(&self, role_code: &str) -> StorageResult<Vec<String>> {
        let bindings = self.list_role_api_bindings().await?;
        Ok(bindings
            .into_iter()
            .filter(|binding| binding.role_code == role_code)
            .map(|binding| binding.api_permission_id)
            .collect())
    }

    pub async fn role_has_api_bindings(&self, role_code: &str) -> StorageResult<bool> {
        let mut db = self.database.connection();
        RoleApiPermissionRecord::filter(RoleApiPermissionRecord::fields().role_code().eq(role_code))
            .first()
            .exec(&mut db)
            .await
            .map(|record| record.is_some())
            .map_err(StorageError::from)
    }

    pub async fn api_has_role_bindings(&self, api_permission_id: &str) -> StorageResult<bool> {
        let mut db = self.database.connection();
        RoleApiPermissionRecord::filter(RoleApiPermissionRecord::fields().api_permission_id().eq(api_permission_id))
            .first()
            .exec(&mut db)
            .await
            .map(|record| record.is_some())
            .map_err(StorageError::from)
    }

    pub async fn list_role_menu_bindings(&self) -> StorageResult<Vec<RoleMenuBindingRecordInput>> {
        let mut db = self.database.connection();
        RoleMenuPermissionRecord::all()
            .exec(&mut db)
            .await
            .map(role_menu_binding_records)
            .map_err(StorageError::from)
    }

    pub async fn role_menu_item_ids(&self, role_code: &str) -> StorageResult<Vec<String>> {
        let bindings = self.list_role_menu_bindings().await?;
        Ok(bindings
            .into_iter()
            .filter(|binding| binding.role_code == role_code)
            .map(|binding| binding.menu_item_id)
            .collect())
    }

    pub async fn role_has_menu_bindings(&self, role_code: &str) -> StorageResult<bool> {
        let mut db = self.database.connection();
        RoleMenuPermissionRecord::filter(RoleMenuPermissionRecord::fields().role_code().eq(role_code))
            .first()
            .exec(&mut db)
            .await
            .map(|record| record.is_some())
            .map_err(StorageError::from)
    }

    pub async fn menu_item_has_role_bindings(&self, menu_item_id: &str) -> StorageResult<bool> {
        let mut db = self.database.connection();
        RoleMenuPermissionRecord::filter(RoleMenuPermissionRecord::fields().menu_item_id().eq(menu_item_id))
            .first()
            .exec(&mut db)
            .await
            .map(|record| record.is_some())
            .map_err(StorageError::from)
    }
}

fn create_role_api_bindings(inputs: Vec<RoleApiBindingRecordInput>) -> CreateMany<RoleApiPermissionRecord> {
    inputs.into_iter().fold(CreateMany::new(), |records, input| {
        records.with_item(|record| record.role_code(input.role_code).api_permission_id(input.api_permission_id))
    })
}

fn create_role_menu_bindings(inputs: Vec<RoleMenuBindingRecordInput>) -> CreateMany<RoleMenuPermissionRecord> {
    inputs.into_iter().fold(CreateMany::new(), |records, input| {
        records.with_item(|record| record.role_code(input.role_code).menu_item_id(input.menu_item_id))
    })
}

fn role_api_binding_records(records: Vec<RoleApiPermissionRecord>) -> Vec<RoleApiBindingRecordInput> {
    records
        .into_iter()
        .map(|record| RoleApiBindingRecordInput {
            role_code: record.role_code,
            api_permission_id: record.api_permission_id,
        })
        .collect()
}

fn role_menu_binding_records(records: Vec<RoleMenuPermissionRecord>) -> Vec<RoleMenuBindingRecordInput> {
    records
        .into_iter()
        .map(|record| RoleMenuBindingRecordInput {
            role_code: record.role_code,
            menu_item_id: record.menu_item_id,
        })
        .collect()
}
