use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use types::{
    pagination::{Page, PageSliceRequest},
    rbac::{MenuItem, MenuSection},
};

use crate::{
    StorageError, StorageResult,
    rbac::{
        menu_item_records, menu_item_records::ActiveModel as MenuItemActiveModel, menu_section_records,
        menu_section_records::ActiveModel as MenuSectionActiveModel,
    },
};

use super::{MenuItemRecord, MenuItemRecordInput, MenuSectionRecord, MenuSectionRecordInput, RbacStore, repository::rbac_page};

impl RbacStore {
    pub async fn create_menu_section(&self, input: MenuSectionRecordInput) -> StorageResult<MenuSection> {
        let now = time::OffsetDateTime::now_utc();
        MenuSectionActiveModel {
            id: Set(self.database.next_id()),
            code: Set(input.code),
            subheader: Set(input.subheader),
            sort_order: Set(input.sort_order),
            enabled: Set(input.enabled),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(self.database.connection())
        .await
        .map(MenuSection::from)
        .map_err(StorageError::from)
    }

    pub async fn replace_menu_section(&self, id: &str, input: MenuSectionRecordInput) -> StorageResult<MenuSection> {
        let record = self.find_menu_section_record(id).await?.ok_or(StorageError::NotFound)?;
        let mut active: MenuSectionActiveModel = record.into();
        active.code = Set(input.code);
        active.subheader = Set(input.subheader);
        active.sort_order = Set(input.sort_order);
        active.enabled = Set(input.enabled);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?;
        self.find_menu_section(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_menu_section(&self, id: &str) -> StorageResult<()> {
        let record = self.find_menu_section_record(id).await?.ok_or(StorageError::NotFound)?;
        let active: MenuSectionActiveModel = record.into();
        active.delete(self.database.connection()).await?;
        Ok(())
    }

    pub async fn find_menu_section(&self, id: &str) -> StorageResult<Option<MenuSection>> {
        self.find_menu_section_record(id).await.map(|record| record.map(MenuSection::from))
    }

    pub async fn list_menu_sections(&self) -> StorageResult<Vec<MenuSection>> {
        menu_section_records::Entity::find()
            .order_by_asc(menu_section_records::Column::SortOrder)
            .all(self.database.connection())
            .await
            .map(|records| records.into_iter().map(MenuSection::from).collect())
            .map_err(StorageError::from)
    }

    pub async fn page_menu_sections(&self, request: PageSliceRequest) -> StorageResult<Page<MenuSection>> {
        let total = menu_section_records::Entity::find().count(self.database.connection()).await?;
        let items = menu_section_records::Entity::find()
            .order_by_asc(menu_section_records::Column::SortOrder)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?;
        Ok(rbac_page(items.into_iter().map(MenuSection::from).collect(), total, request))
    }

    pub async fn menu_section_has_items(&self, id: &str) -> StorageResult<bool> {
        menu_item_records::Entity::find()
            .filter(menu_item_records::Column::SectionId.eq(id))
            .one(self.database.connection())
            .await
            .map(|record| record.is_some())
            .map_err(StorageError::from)
    }

    async fn find_menu_section_record(&self, id: &str) -> StorageResult<Option<MenuSectionRecord>> {
        menu_section_records::Entity::find_by_id(id.to_owned())
            .one(self.database.connection())
            .await
            .map_err(StorageError::from)
    }

    pub async fn create_menu_item(&self, input: MenuItemRecordInput) -> StorageResult<MenuItem> {
        let now = time::OffsetDateTime::now_utc();
        MenuItemActiveModel {
            id: Set(self.database.next_id()),
            section_id: Set(input.section_id),
            parent_id: Set(input.parent_id),
            code: Set(input.code),
            title: Set(input.title),
            route_path: Set(input.path),
            icon: Set(input.icon),
            caption: Set(input.caption),
            deep_match: Set(input.deep_match),
            sort_order: Set(input.sort_order),
            enabled: Set(input.enabled),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(self.database.connection())
        .await
        .map(MenuItem::from)
        .map_err(StorageError::from)
    }

    pub async fn replace_menu_item(&self, id: &str, input: MenuItemRecordInput) -> StorageResult<MenuItem> {
        let record = self.find_menu_item_record(id).await?.ok_or(StorageError::NotFound)?;
        let mut active: MenuItemActiveModel = record.into();
        active.section_id = Set(input.section_id);
        active.parent_id = Set(input.parent_id);
        active.code = Set(input.code);
        active.title = Set(input.title);
        active.route_path = Set(input.path);
        active.icon = Set(input.icon);
        active.caption = Set(input.caption);
        active.deep_match = Set(input.deep_match);
        active.sort_order = Set(input.sort_order);
        active.enabled = Set(input.enabled);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        active.update(self.database.connection()).await?;
        self.find_menu_item(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_menu_item(&self, id: &str) -> StorageResult<()> {
        let record = self.find_menu_item_record(id).await?.ok_or(StorageError::NotFound)?;
        let active: MenuItemActiveModel = record.into();
        active.delete(self.database.connection()).await?;
        Ok(())
    }

    pub async fn find_menu_item(&self, id: &str) -> StorageResult<Option<MenuItem>> {
        self.find_menu_item_record(id).await.map(|record| record.map(MenuItem::from))
    }

    pub async fn list_menu_items(&self) -> StorageResult<Vec<MenuItem>> {
        menu_item_records::Entity::find()
            .order_by_asc(menu_item_records::Column::SortOrder)
            .all(self.database.connection())
            .await
            .map(|records| records.into_iter().map(MenuItem::from).collect())
            .map_err(StorageError::from)
    }

    pub async fn page_menu_items(&self, request: PageSliceRequest) -> StorageResult<Page<MenuItem>> {
        let total = menu_item_records::Entity::find().count(self.database.connection()).await?;
        let items = menu_item_records::Entity::find()
            .order_by_asc(menu_item_records::Column::SortOrder)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?;
        Ok(rbac_page(items.into_iter().map(MenuItem::from).collect(), total, request))
    }

    pub async fn menu_item_has_children(&self, id: &str) -> StorageResult<bool> {
        menu_item_records::Entity::find()
            .filter(menu_item_records::Column::ParentId.eq(Some(id.to_owned())))
            .one(self.database.connection())
            .await
            .map(|record| record.is_some())
            .map_err(StorageError::from)
    }

    async fn find_menu_item_record(&self, id: &str) -> StorageResult<Option<MenuItemRecord>> {
        menu_item_records::Entity::find_by_id(id.to_owned())
            .one(self.database.connection())
            .await
            .map_err(StorageError::from)
    }
}
