use types::{
    pagination::{Page, PageSliceRequest},
    rbac::ApiPermission,
};

use crate::{StorageError, StorageResult};

use super::{ApiPermissionRecord, ApiPermissionRecordInput, RbacStore, repository::rbac_page};

impl RbacStore {
    pub async fn create_api(&self, input: ApiPermissionRecordInput) -> StorageResult<ApiPermission> {
        let mut db = self.database.connection();
        toasty::create!(ApiPermissionRecord {
            id: self.database.next_id(),
            code: input.code,
            method: input.method,
            path_pattern: input.path_pattern,
            name: input.name,
            group: input.group,
            enabled: input.enabled,
            system: input.system,
        })
        .exec(&mut db)
        .await
        .map(ApiPermission::from)
        .map_err(StorageError::from)
    }

    pub async fn replace_api(&self, id: &str, input: ApiPermissionRecordInput) -> StorageResult<ApiPermission> {
        let mut db = self.database.connection();
        let mut record = self.find_api_record(id).await?.ok_or(StorageError::NotFound)?;
        record
            .update()
            .code(input.code)
            .method(input.method)
            .path_pattern(input.path_pattern)
            .name(input.name)
            .group(input.group)
            .enabled(input.enabled)
            .system(input.system)
            .exec(&mut db)
            .await?;
        self.find_api(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_api(&self, id: &str) -> StorageResult<()> {
        let mut db = self.database.connection();
        let record = self.find_api_record(id).await?.ok_or(StorageError::NotFound)?;
        record.delete().exec(&mut db).await?;
        Ok(())
    }

    pub async fn find_api(&self, id: &str) -> StorageResult<Option<ApiPermission>> {
        self.find_api_record(id).await.map(|record| record.map(ApiPermission::from))
    }

    pub async fn list_apis(&self) -> StorageResult<Vec<ApiPermission>> {
        let mut db = self.database.connection();
        ApiPermissionRecord::all()
            .order_by(ApiPermissionRecord::fields().id().asc())
            .exec(&mut db)
            .await
            .map(|records| records.into_iter().map(ApiPermission::from).collect())
            .map_err(StorageError::from)
    }

    pub async fn page_apis(&self, request: PageSliceRequest) -> StorageResult<Page<ApiPermission>> {
        let mut db = self.database.connection();
        let total = ApiPermissionRecord::all().count().exec(&mut db).await?;
        let items = ApiPermissionRecord::all()
            .order_by(ApiPermissionRecord::fields().id().asc())
            .limit(request.limit as usize)
            .offset(request.offset as usize)
            .exec(&mut db)
            .await?;
        Ok(rbac_page(items.into_iter().map(ApiPermission::from).collect(), total, request))
    }

    async fn find_api_record(&self, id: &str) -> StorageResult<Option<ApiPermissionRecord>> {
        let mut db = self.database.connection();
        ApiPermissionRecord::filter(ApiPermissionRecord::fields().id().eq(id))
            .first()
            .exec(&mut db)
            .await
            .map_err(StorageError::from)
    }
}
