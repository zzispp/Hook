use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait, Select, Set, TransactionTrait,
};
use types::{
    pagination::{Page, PageSliceRequest},
    rbac::ApiPermission,
};

use crate::{
    StorageError, StorageResult,
    rbac::{api_permission_records, api_permission_records::ActiveModel as ApiActiveModel},
};

use super::{
    ApiPermissionRecord, ApiPermissionRecordInput, MenuApiBindingRecordInput, RbacRecordFilters, RbacStore, bindings::insert_menu_api_bindings,
    menu_api_permission_records, repository::rbac_page,
};

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

    pub async fn create_api_with_menus(&self, input: ApiPermissionRecordInput, menu_inputs: Vec<MenuApiBindingRecordInput>) -> StorageResult<ApiPermission> {
        let tx = self.database.connection().begin().await?;
        let api = api_active_model(self.database.next_id(), input).insert(&tx).await?;
        insert_menu_api_bindings(menu_inputs_for_api(&api.id, menu_inputs), &tx).await?;
        tx.commit().await?;
        Ok(ApiPermission::from(api))
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

    pub async fn replace_api_with_menus(
        &self,
        id: &str,
        input: ApiPermissionRecordInput,
        menu_inputs: Vec<MenuApiBindingRecordInput>,
    ) -> StorageResult<ApiPermission> {
        let tx = self.database.connection().begin().await?;
        let record = self.find_api_record(id).await?.ok_or(StorageError::NotFound)?;
        let mut active: ApiActiveModel = record.into();
        set_api_fields(&mut active, input);
        active.update(&tx).await?;
        super::menu_api_permission_records::Entity::delete_many()
            .filter(super::menu_api_permission_records::Column::ApiPermissionId.eq(id))
            .exec(&tx)
            .await?;
        insert_menu_api_bindings(menu_inputs_for_api(id, menu_inputs), &tx).await?;
        tx.commit().await?;
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

    pub async fn page_apis(&self, request: PageSliceRequest, filters: RbacRecordFilters) -> StorageResult<Page<ApiPermission>> {
        let query = filtered_apis(filters);
        let total = query.clone().count(self.database.connection()).await?;
        let items = query
            .order_by_asc(api_permission_records::Column::Id)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?;
        Ok(rbac_page(items.into_iter().map(ApiPermission::from).collect(), total, request))
    }

    pub async fn page_unbound_apis(&self, request: PageSliceRequest, filters: RbacRecordFilters) -> StorageResult<Page<ApiPermission>> {
        let query = unbound_api_query(filters);
        let total = query.clone().count(self.database.connection()).await?;
        let items = query
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

fn unbound_api_query(filters: RbacRecordFilters) -> Select<api_permission_records::Entity> {
    let bound_api_ids = menu_api_permission_records::Entity::find()
        .select_only()
        .column(menu_api_permission_records::Column::ApiPermissionId)
        .into_query();

    filtered_apis(filters).filter(api_permission_records::Column::Id.not_in_subquery(bound_api_ids))
}

fn filtered_apis(filters: RbacRecordFilters) -> Select<api_permission_records::Entity> {
    let mut query = api_permission_records::Entity::find();
    if let Some(enabled) = filters.enabled {
        query = query.filter(api_permission_records::Column::Enabled.eq(enabled));
    }
    match filters.search {
        Some(search) if !search.is_empty() => query.filter(api_search_condition(&search)),
        _ => query,
    }
}

fn api_search_condition(search: &str) -> Condition {
    Condition::any()
        .add(api_permission_records::Column::Code.contains(search))
        .add(api_permission_records::Column::Method.contains(search))
        .add(api_permission_records::Column::PathPattern.contains(search))
        .add(api_permission_records::Column::Name.contains(search))
        .add(api_permission_records::Column::Group.contains(search))
}

fn api_active_model(id: String, input: ApiPermissionRecordInput) -> ApiActiveModel {
    let now = time::OffsetDateTime::now_utc();
    ApiActiveModel {
        id: Set(id),
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
}

fn menu_inputs_for_api(api_id: &str, inputs: Vec<MenuApiBindingRecordInput>) -> Vec<MenuApiBindingRecordInput> {
    inputs
        .into_iter()
        .map(|input| MenuApiBindingRecordInput {
            api_permission_id: api_id.into(),
            ..input
        })
        .collect()
}

fn set_api_fields(active: &mut ApiActiveModel, input: ApiPermissionRecordInput) {
    active.code = Set(input.code);
    active.method = Set(input.method);
    active.path_pattern = Set(input.path_pattern);
    active.name = Set(input.name);
    active.group = Set(input.group);
    active.enabled = Set(input.enabled);
    active.system = Set(input.system);
    active.updated_at = Set(time::OffsetDateTime::now_utc());
}
