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

    pub async fn list_role_menu_bindings(&self) -> StorageResult<Vec<RoleMenuBindingRecordInput>> {
        let mut db = self.database.connection();
        RoleMenuPermissionRecord::all()
            .exec(&mut db)
            .await
            .map(role_menu_binding_records)
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
