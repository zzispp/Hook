use types::{
    pagination::{Page, PageSliceRequest},
    rbac::Role,
};

use crate::{StorageError, StorageResult};

use super::{RbacStore, RoleRecord, RoleRecordInput, repository::rbac_page};

impl RbacStore {
    pub async fn create_role(&self, input: RoleRecordInput) -> StorageResult<Role> {
        let mut db = self.database.connection();
        toasty::create!(RoleRecord {
            code: input.code,
            name: input.name,
            description: input.description,
            enabled: input.enabled,
            system: input.system,
            sort_order: input.sort_order,
        })
        .exec(&mut db)
        .await
        .map(Role::from)
        .map_err(StorageError::from)
    }

    pub async fn replace_role(&self, code: &str, input: RoleRecordInput) -> StorageResult<Role> {
        let mut db = self.database.connection();
        let mut record = self.find_role_record(code).await?.ok_or(StorageError::NotFound)?;
        record
            .update()
            .name(input.name)
            .description(input.description)
            .enabled(input.enabled)
            .system(input.system)
            .sort_order(input.sort_order)
            .exec(&mut db)
            .await?;
        self.find_role(code).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_role(&self, code: &str) -> StorageResult<()> {
        let mut db = self.database.connection();
        let record = self.find_role_record(code).await?.ok_or(StorageError::NotFound)?;
        record.delete().exec(&mut db).await?;
        Ok(())
    }

    pub async fn find_role(&self, code: &str) -> StorageResult<Option<Role>> {
        self.find_role_record(code).await.map(|record| record.map(Role::from))
    }

    pub async fn list_roles(&self) -> StorageResult<Vec<Role>> {
        let mut db = self.database.connection();
        RoleRecord::all()
            .order_by(RoleRecord::fields().sort_order().asc())
            .exec(&mut db)
            .await
            .map(|records| records.into_iter().map(Role::from).collect())
            .map_err(StorageError::from)
    }

    pub async fn page_roles(&self, request: PageSliceRequest) -> StorageResult<Page<Role>> {
        let mut db = self.database.connection();
        let total = RoleRecord::all().count().exec(&mut db).await?;
        let items = RoleRecord::all()
            .order_by(RoleRecord::fields().sort_order().asc())
            .limit(request.limit as usize)
            .offset(request.offset as usize)
            .exec(&mut db)
            .await?;
        Ok(rbac_page(items.into_iter().map(Role::from).collect(), total, request))
    }

    pub(super) async fn find_role_record(&self, code: &str) -> StorageResult<Option<RoleRecord>> {
        let mut db = self.database.connection();
        RoleRecord::filter(RoleRecord::fields().code().eq(code))
            .first()
            .exec(&mut db)
            .await
            .map_err(StorageError::from)
    }
}
