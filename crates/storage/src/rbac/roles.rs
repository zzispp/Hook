use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use types::{
    pagination::{Page, PageSliceRequest},
    rbac::Role,
};

use crate::{
    StorageError, StorageResult,
    rbac::{role_records, role_records::ActiveModel as RoleActiveModel},
    user::{UserColumn, UserEntity as Users},
};

use super::{RbacStore, RoleRecord, RoleRecordInput, repository::rbac_page};

impl RbacStore {
    pub async fn create_role(&self, input: RoleRecordInput) -> StorageResult<Role> {
        let now = time::OffsetDateTime::now_utc();
        RoleActiveModel {
            code: Set(input.code),
            name: Set(input.name),
            description: Set(input.description),
            enabled: Set(input.enabled),
            system: Set(input.system),
            sort_order: Set(input.sort_order),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(self.database.connection())
        .await
        .map(Role::from)
        .map_err(StorageError::from)
    }

    pub async fn replace_role(&self, code: &str, input: RoleRecordInput) -> StorageResult<Role> {
        let record = self.find_role_record(code).await?.ok_or(StorageError::NotFound)?;
        let mut active: RoleActiveModel = record.into();
        active.name = Set(input.name);
        active.description = Set(input.description);
        active.enabled = Set(input.enabled);
        active.system = Set(input.system);
        active.sort_order = Set(input.sort_order);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?;
        self.find_role(code).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_role(&self, code: &str) -> StorageResult<()> {
        let record = self.find_role_record(code).await?.ok_or(StorageError::NotFound)?;
        let active: RoleActiveModel = record.into();
        active.delete(self.database.connection()).await?;
        Ok(())
    }

    pub async fn find_role(&self, code: &str) -> StorageResult<Option<Role>> {
        self.find_role_record(code).await.map(|record| record.map(Role::from))
    }

    pub async fn list_roles(&self) -> StorageResult<Vec<Role>> {
        role_records::Entity::find()
            .order_by_asc(role_records::Column::SortOrder)
            .all(self.database.connection())
            .await
            .map(|records| records.into_iter().map(Role::from).collect())
            .map_err(StorageError::from)
    }

    pub async fn page_roles(&self, request: PageSliceRequest) -> StorageResult<Page<Role>> {
        let total = role_records::Entity::find().count(self.database.connection()).await?;
        let items = role_records::Entity::find()
            .order_by_asc(role_records::Column::SortOrder)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?;
        Ok(rbac_page(items.into_iter().map(Role::from).collect(), total, request))
    }

    pub async fn role_has_users(&self, code: &str) -> StorageResult<bool> {
        Users::find()
            .filter(UserColumn::Role.eq(code))
            .filter(UserColumn::IsDeleted.eq(false))
            .one(self.database.connection())
            .await
            .map(|record| record.is_some())
            .map_err(StorageError::from)
    }

    pub(super) async fn find_role_record(&self, code: &str) -> StorageResult<Option<RoleRecord>> {
        role_records::Entity::find_by_id(code.to_owned())
            .one(self.database.connection())
            .await
            .map_err(StorageError::from)
    }
}
