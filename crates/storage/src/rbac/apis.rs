use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, QueryOrder, QuerySelect, Set};
use types::{
    pagination::{Page, PageSliceRequest},
    rbac::ApiPermission,
};

use crate::{
    StorageError, StorageResult,
    rbac::{api_permission_records, api_permission_records::ActiveModel as ApiActiveModel},
};

use super::{ApiPermissionRecord, ApiPermissionRecordInput, RbacStore, repository::rbac_page};

impl RbacStore {
    pub async fn create_api(&self, input: ApiPermissionRecordInput) -> StorageResult<ApiPermission> {
        let now = time::OffsetDateTime::now_utc();
        ApiActiveModel {
            id: Set(self.database.next_id()),
            code: Set(input.code),
            method: Set(input.method),
            path_pattern: Set(input.path_pattern),
            name: Set(input.name),
            group: Set(input.group),
            enabled: Set(input.enabled),
            system: Set(input.system),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(self.database.connection())
        .await
        .map(ApiPermission::from)
        .map_err(StorageError::from)
    }

    pub async fn replace_api(&self, id: &str, input: ApiPermissionRecordInput) -> StorageResult<ApiPermission> {
        let record = self.find_api_record(id).await?.ok_or(StorageError::NotFound)?;
        let mut active: ApiActiveModel = record.into();
        active.code = Set(input.code);
        active.method = Set(input.method);
        active.path_pattern = Set(input.path_pattern);
        active.name = Set(input.name);
        active.group = Set(input.group);
        active.enabled = Set(input.enabled);
        active.system = Set(input.system);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?;
        self.find_api(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_api(&self, id: &str) -> StorageResult<()> {
        let record = self.find_api_record(id).await?.ok_or(StorageError::NotFound)?;
        let active: ApiActiveModel = record.into();
        active.delete(self.database.connection()).await?;
        Ok(())
    }

    pub async fn find_api(&self, id: &str) -> StorageResult<Option<ApiPermission>> {
        self.find_api_record(id).await.map(|record| record.map(ApiPermission::from))
    }

    pub async fn list_apis(&self) -> StorageResult<Vec<ApiPermission>> {
        api_permission_records::Entity::find()
            .order_by_asc(api_permission_records::Column::Id)
            .all(self.database.connection())
            .await
            .map(|records| records.into_iter().map(ApiPermission::from).collect())
            .map_err(StorageError::from)
    }

    pub async fn page_apis(&self, request: PageSliceRequest) -> StorageResult<Page<ApiPermission>> {
        let total = api_permission_records::Entity::find().count(self.database.connection()).await?;
        let items = api_permission_records::Entity::find()
            .order_by_asc(api_permission_records::Column::Id)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?;
        Ok(rbac_page(items.into_iter().map(ApiPermission::from).collect(), total, request))
    }

    async fn find_api_record(&self, id: &str) -> StorageResult<Option<ApiPermissionRecord>> {
        api_permission_records::Entity::find_by_id(id.to_owned())
            .one(self.database.connection())
            .await
            .map_err(StorageError::from)
    }
}
