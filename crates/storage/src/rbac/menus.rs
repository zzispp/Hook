use types::{
    pagination::{Page, PageSliceRequest},
    rbac::{MenuItem, MenuSection},
};

use crate::{StorageError, StorageResult};

use super::{MenuItemRecord, MenuItemRecordInput, MenuSectionRecord, MenuSectionRecordInput, RbacStore, repository::rbac_page};

impl RbacStore {
    pub async fn create_menu_section(&self, input: MenuSectionRecordInput) -> StorageResult<MenuSection> {
        let mut db = self.database.connection();
        toasty::create!(MenuSectionRecord {
            id: self.database.next_id(),
            code: input.code,
            subheader: input.subheader,
            sort_order: input.sort_order,
            enabled: input.enabled,
        })
        .exec(&mut db)
        .await
        .map(MenuSection::from)
        .map_err(StorageError::from)
    }

    pub async fn replace_menu_section(&self, id: &str, input: MenuSectionRecordInput) -> StorageResult<MenuSection> {
        let mut db = self.database.connection();
        let mut record = self.find_menu_section_record(id).await?.ok_or(StorageError::NotFound)?;
        record
            .update()
            .code(input.code)
            .subheader(input.subheader)
            .sort_order(input.sort_order)
            .enabled(input.enabled)
            .exec(&mut db)
            .await?;
        self.find_menu_section(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_menu_section(&self, id: &str) -> StorageResult<()> {
        let mut db = self.database.connection();
        let record = self.find_menu_section_record(id).await?.ok_or(StorageError::NotFound)?;
        record.delete().exec(&mut db).await?;
        Ok(())
    }

    pub async fn find_menu_section(&self, id: &str) -> StorageResult<Option<MenuSection>> {
        self.find_menu_section_record(id).await.map(|record| record.map(MenuSection::from))
    }

    pub async fn list_menu_sections(&self) -> StorageResult<Vec<MenuSection>> {
        let mut db = self.database.connection();
        MenuSectionRecord::all()
            .order_by(MenuSectionRecord::fields().sort_order().asc())
            .exec(&mut db)
            .await
            .map(|records| records.into_iter().map(MenuSection::from).collect())
            .map_err(StorageError::from)
    }

    pub async fn page_menu_sections(&self, request: PageSliceRequest) -> StorageResult<Page<MenuSection>> {
        let mut db = self.database.connection();
        let total = MenuSectionRecord::all().count().exec(&mut db).await?;
        let items = MenuSectionRecord::all()
            .order_by(MenuSectionRecord::fields().sort_order().asc())
            .limit(request.limit as usize)
            .offset(request.offset as usize)
            .exec(&mut db)
            .await?;
        Ok(rbac_page(items.into_iter().map(MenuSection::from).collect(), total, request))
    }

    pub async fn menu_section_has_items(&self, id: &str) -> StorageResult<bool> {
        let mut db = self.database.connection();
        MenuItemRecord::filter(MenuItemRecord::fields().section_id().eq(id))
            .first()
            .exec(&mut db)
            .await
            .map(|record| record.is_some())
            .map_err(StorageError::from)
    }

    async fn find_menu_section_record(&self, id: &str) -> StorageResult<Option<MenuSectionRecord>> {
        let mut db = self.database.connection();
        MenuSectionRecord::filter(MenuSectionRecord::fields().id().eq(id))
            .first()
            .exec(&mut db)
            .await
            .map_err(StorageError::from)
    }

    pub async fn create_menu_item(&self, input: MenuItemRecordInput) -> StorageResult<MenuItem> {
        let mut db = self.database.connection();
        toasty::create!(MenuItemRecord {
            id: self.database.next_id(),
            section_id: input.section_id,
            parent_id: input.parent_id,
            code: input.code,
            title: input.title,
            route_path: input.path,
            icon: input.icon,
            caption: input.caption,
            deep_match: input.deep_match,
            sort_order: input.sort_order,
            enabled: input.enabled,
        })
        .exec(&mut db)
        .await
        .map(MenuItem::from)
        .map_err(StorageError::from)
    }

    pub async fn replace_menu_item(&self, id: &str, input: MenuItemRecordInput) -> StorageResult<MenuItem> {
        let mut db = self.database.connection();
        let mut record = self.find_menu_item_record(id).await?.ok_or(StorageError::NotFound)?;
        record
            .update()
            .section_id(input.section_id)
            .parent_id(input.parent_id)
            .code(input.code)
            .title(input.title)
            .route_path(input.path)
            .icon(input.icon)
            .caption(input.caption)
            .deep_match(input.deep_match)
            .sort_order(input.sort_order)
            .enabled(input.enabled)
            .exec(&mut db)
            .await?;
        self.find_menu_item(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_menu_item(&self, id: &str) -> StorageResult<()> {
        let mut db = self.database.connection();
        let record = self.find_menu_item_record(id).await?.ok_or(StorageError::NotFound)?;
        record.delete().exec(&mut db).await?;
        Ok(())
    }

    pub async fn find_menu_item(&self, id: &str) -> StorageResult<Option<MenuItem>> {
        self.find_menu_item_record(id).await.map(|record| record.map(MenuItem::from))
    }

    pub async fn list_menu_items(&self) -> StorageResult<Vec<MenuItem>> {
        let mut db = self.database.connection();
        MenuItemRecord::all()
            .order_by(MenuItemRecord::fields().sort_order().asc())
            .exec(&mut db)
            .await
            .map(|records| records.into_iter().map(MenuItem::from).collect())
            .map_err(StorageError::from)
    }

    pub async fn page_menu_items(&self, request: PageSliceRequest) -> StorageResult<Page<MenuItem>> {
        let mut db = self.database.connection();
        let total = MenuItemRecord::all().count().exec(&mut db).await?;
        let items = MenuItemRecord::all()
            .order_by(MenuItemRecord::fields().sort_order().asc())
            .limit(request.limit as usize)
            .offset(request.offset as usize)
            .exec(&mut db)
            .await?;
        Ok(rbac_page(items.into_iter().map(MenuItem::from).collect(), total, request))
    }

    pub async fn menu_item_has_children(&self, id: &str) -> StorageResult<bool> {
        let mut db = self.database.connection();
        MenuItemRecord::filter(MenuItemRecord::fields().parent_id().eq(Some(id.to_owned())))
            .first()
            .exec(&mut db)
            .await
            .map(|record| record.is_some())
            .map_err(StorageError::from)
    }

    async fn find_menu_item_record(&self, id: &str) -> StorageResult<Option<MenuItemRecord>> {
        let mut db = self.database.connection();
        MenuItemRecord::filter(MenuItemRecord::fields().id().eq(id))
            .first()
            .exec(&mut db)
            .await
            .map_err(StorageError::from)
    }
}
