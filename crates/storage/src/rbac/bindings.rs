use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};

use crate::{StorageError, StorageResult};

use super::{
    MenuApiBindingRecordInput, MenuApiPermissionRecord, RbacStore, RoleApiBindingRecordInput, RoleApiPermissionRecord, RoleMenuBindingRecordInput,
    RoleMenuPermissionRecord, menu_api_permission_records, menu_api_permission_records::ActiveModel as MenuApiActiveModel, role_api_permission_records,
    role_api_permission_records::ActiveModel as RoleApiActiveModel, role_menu_permission_records,
    role_menu_permission_records::ActiveModel as RoleMenuActiveModel,
};

impl RbacStore {
    pub async fn replace_menu_apis(&self, menu_item_id: &str, inputs: Vec<MenuApiBindingRecordInput>) -> StorageResult<()> {
        let tx = self.database.connection().begin().await?;
        menu_api_permission_records::Entity::delete_many()
            .filter(menu_api_permission_records::Column::MenuItemId.eq(menu_item_id))
            .exec(&tx)
            .await?;
        insert_menu_api_bindings(inputs, &tx).await?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn replace_api_menus(&self, api_permission_id: &str, inputs: Vec<MenuApiBindingRecordInput>) -> StorageResult<()> {
        let tx = self.database.connection().begin().await?;
        menu_api_permission_records::Entity::delete_many()
            .filter(menu_api_permission_records::Column::ApiPermissionId.eq(api_permission_id))
            .exec(&tx)
            .await?;
        insert_menu_api_bindings(inputs, &tx).await?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn replace_role_permissions(
        &self,
        role_code: &str,
        menu_inputs: Vec<RoleMenuBindingRecordInput>,
        api_inputs: Vec<RoleApiBindingRecordInput>,
    ) -> StorageResult<()> {
        let tx = self.database.connection().begin().await?;
        role_menu_permission_records::Entity::delete_many()
            .filter(role_menu_permission_records::Column::RoleCode.eq(role_code))
            .exec(&tx)
            .await?;
        role_api_permission_records::Entity::delete_many()
            .filter(role_api_permission_records::Column::RoleCode.eq(role_code))
            .exec(&tx)
            .await?;
        insert_role_menu_bindings(menu_inputs, &tx).await?;
        insert_role_api_bindings(api_inputs, &tx).await?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn list_menu_api_bindings(&self) -> StorageResult<Vec<MenuApiBindingRecordInput>> {
        menu_api_permission_records::Entity::find()
            .all(self.database.connection())
            .await
            .map(menu_api_binding_records)
            .map_err(StorageError::from)
    }

    pub async fn menu_api_ids(&self, menu_item_id: &str) -> StorageResult<Vec<String>> {
        menu_api_permission_records::Entity::find()
            .filter(menu_api_permission_records::Column::MenuItemId.eq(menu_item_id))
            .all(self.database.connection())
            .await
            .map(|records| records.into_iter().map(|record| record.api_permission_id).collect())
            .map_err(StorageError::from)
    }

    pub async fn api_menu_ids(&self, api_permission_id: &str) -> StorageResult<Vec<String>> {
        menu_api_permission_records::Entity::find()
            .filter(menu_api_permission_records::Column::ApiPermissionId.eq(api_permission_id))
            .all(self.database.connection())
            .await
            .map(|records| records.into_iter().map(|record| record.menu_item_id).collect())
            .map_err(StorageError::from)
    }

    pub async fn menu_item_has_api_bindings(&self, menu_item_id: &str) -> StorageResult<bool> {
        binding_exists(
            menu_api_permission_records::Entity::find().filter(menu_api_permission_records::Column::MenuItemId.eq(menu_item_id)),
            self.database.connection(),
        )
        .await
    }

    pub async fn api_has_menu_bindings(&self, api_permission_id: &str) -> StorageResult<bool> {
        binding_exists(
            menu_api_permission_records::Entity::find().filter(menu_api_permission_records::Column::ApiPermissionId.eq(api_permission_id)),
            self.database.connection(),
        )
        .await
    }

    pub async fn list_role_menu_bindings(&self) -> StorageResult<Vec<RoleMenuBindingRecordInput>> {
        role_menu_permission_records::Entity::find()
            .all(self.database.connection())
            .await
            .map(role_menu_binding_records)
            .map_err(StorageError::from)
    }

    pub async fn list_role_api_bindings(&self) -> StorageResult<Vec<RoleApiBindingRecordInput>> {
        role_api_permission_records::Entity::find()
            .all(self.database.connection())
            .await
            .map(role_api_binding_records)
            .map_err(StorageError::from)
    }

    pub async fn role_menu_item_ids(&self, role_code: &str) -> StorageResult<Vec<String>> {
        role_menu_permission_records::Entity::find()
            .filter(role_menu_permission_records::Column::RoleCode.eq(role_code))
            .all(self.database.connection())
            .await
            .map(|records| records.into_iter().map(|record| record.menu_item_id).collect())
            .map_err(StorageError::from)
    }

    pub async fn role_api_ids(&self, role_code: &str) -> StorageResult<Vec<String>> {
        role_api_permission_records::Entity::find()
            .filter(role_api_permission_records::Column::RoleCode.eq(role_code))
            .all(self.database.connection())
            .await
            .map(|records| records.into_iter().map(|record| record.api_permission_id).collect())
            .map_err(StorageError::from)
    }

    pub async fn role_has_menu_bindings(&self, role_code: &str) -> StorageResult<bool> {
        binding_exists(
            role_menu_permission_records::Entity::find().filter(role_menu_permission_records::Column::RoleCode.eq(role_code)),
            self.database.connection(),
        )
        .await
    }

    pub async fn role_has_api_bindings(&self, role_code: &str) -> StorageResult<bool> {
        binding_exists(
            role_api_permission_records::Entity::find().filter(role_api_permission_records::Column::RoleCode.eq(role_code)),
            self.database.connection(),
        )
        .await
    }

    pub async fn api_has_role_bindings(&self, api_permission_id: &str) -> StorageResult<bool> {
        binding_exists(
            role_api_permission_records::Entity::find().filter(role_api_permission_records::Column::ApiPermissionId.eq(api_permission_id)),
            self.database.connection(),
        )
        .await
    }

    pub async fn menu_item_has_role_bindings(&self, menu_item_id: &str) -> StorageResult<bool> {
        binding_exists(
            role_menu_permission_records::Entity::find().filter(role_menu_permission_records::Column::MenuItemId.eq(menu_item_id)),
            self.database.connection(),
        )
        .await
    }
}

pub(super) async fn insert_menu_api_bindings(inputs: Vec<MenuApiBindingRecordInput>, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    if inputs.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = inputs.into_iter().map(|input| menu_api_active_model(input, now));
    menu_api_permission_records::Entity::insert_many(records).exec(tx).await?;
    Ok(())
}

async fn insert_role_menu_bindings(inputs: Vec<RoleMenuBindingRecordInput>, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    if inputs.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = inputs.into_iter().map(|input| role_menu_active_model(input, now));
    role_menu_permission_records::Entity::insert_many(records).exec(tx).await?;
    Ok(())
}

async fn insert_role_api_bindings(inputs: Vec<RoleApiBindingRecordInput>, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    if inputs.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();
    let records = inputs.into_iter().map(|input| role_api_active_model(input, now));
    role_api_permission_records::Entity::insert_many(records).exec(tx).await?;
    Ok(())
}

async fn binding_exists<E>(select: sea_orm::Select<E>, db: &sea_orm::DatabaseConnection) -> StorageResult<bool>
where
    E: EntityTrait,
{
    select.one(db).await.map(|record| record.is_some()).map_err(StorageError::from)
}

fn menu_api_active_model(input: MenuApiBindingRecordInput, now: time::OffsetDateTime) -> MenuApiActiveModel {
    MenuApiActiveModel {
        menu_item_id: Set(input.menu_item_id),
        api_permission_id: Set(input.api_permission_id),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

fn role_menu_active_model(input: RoleMenuBindingRecordInput, now: time::OffsetDateTime) -> RoleMenuActiveModel {
    RoleMenuActiveModel {
        role_code: Set(input.role_code),
        menu_item_id: Set(input.menu_item_id),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

fn role_api_active_model(input: RoleApiBindingRecordInput, now: time::OffsetDateTime) -> RoleApiActiveModel {
    RoleApiActiveModel {
        role_code: Set(input.role_code),
        api_permission_id: Set(input.api_permission_id),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

fn menu_api_binding_records(records: Vec<MenuApiPermissionRecord>) -> Vec<MenuApiBindingRecordInput> {
    records
        .into_iter()
        .map(|record| MenuApiBindingRecordInput {
            menu_item_id: record.menu_item_id,
            api_permission_id: record.api_permission_id,
        })
        .collect()
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
