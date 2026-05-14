use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use types::{
    operations::{Announcement, AnnouncementListFilters},
    pagination::{Page, PageSliceRequest},
};

use crate::{StorageError, StorageResult};

use super::{
    AnnouncementActiveModel, AnnouncementColumn, AnnouncementEntity, AnnouncementRecord, AnnouncementRecordInput, AnnouncementRecordPatch, OperationsStore,
};

impl OperationsStore {
    pub async fn create_announcement(&self, input: AnnouncementRecordInput) -> StorageResult<Announcement> {
        let now = time::OffsetDateTime::now_utc();
        let record = AnnouncementActiveModel {
            id: Set(self.next_id()),
            title: Set(input.title),
            content_markdown: Set(input.content_markdown),
            announcement_type: Set(input.announcement_type),
            pinned: Set(input.pinned),
            priority: Set(input.priority),
            enabled: Set(input.enabled),
            created_by: Set(input.operator_id.clone()),
            updated_by: Set(input.operator_id),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(self.connection())
        .await?;
        Ok(record.into())
    }

    pub async fn update_announcement(&self, id: &str, input: AnnouncementRecordPatch) -> StorageResult<Announcement> {
        let record = self.announcement_record(id).await?.ok_or(StorageError::NotFound)?;
        let mut active: AnnouncementActiveModel = record.into();
        apply_announcement_patch(&mut active, input);
        let record = active.update(self.connection()).await?;
        Ok(record.into())
    }

    pub async fn delete_announcement(&self, id: &str) -> StorageResult<()> {
        let record = self.announcement_record(id).await?.ok_or(StorageError::NotFound)?;
        let active: AnnouncementActiveModel = record.into();
        active.delete(self.connection()).await?;
        Ok(())
    }

    pub async fn get_announcement(&self, id: &str) -> StorageResult<Option<Announcement>> {
        self.announcement_record(id).await.map(|record| record.map(Into::into))
    }

    pub async fn page_announcements(&self, request: PageSliceRequest, filters: AnnouncementListFilters) -> StorageResult<Page<Announcement>> {
        let query = filter_announcements(AnnouncementEntity::find(), filters);
        let total = query.clone().count(self.connection()).await?;
        let items = query
            .order_by_desc(AnnouncementColumn::Pinned)
            .order_by_desc(AnnouncementColumn::Priority)
            .order_by_desc(AnnouncementColumn::CreatedAt)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.connection())
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(Page {
            items,
            total,
            page: request.page,
            page_size: request.page_size,
        })
    }

    async fn announcement_record(&self, id: &str) -> StorageResult<Option<AnnouncementRecord>> {
        AnnouncementEntity::find_by_id(id.to_owned()).one(self.connection()).await.map_err(Into::into)
    }
}

fn filter_announcements(mut query: sea_orm::Select<AnnouncementEntity>, filters: AnnouncementListFilters) -> sea_orm::Select<AnnouncementEntity> {
    if let Some(search) = filters.search.filter(|value| !value.is_empty()) {
        query = query.filter(
            Condition::any()
                .add(AnnouncementColumn::Title.contains(search.clone()))
                .add(AnnouncementColumn::ContentMarkdown.contains(search)),
        );
    }
    if let Some(value) = filters.announcement_type.filter(|value| !value.is_empty()) {
        query = query.filter(AnnouncementColumn::AnnouncementType.eq(value));
    }
    if let Some(value) = filters.enabled {
        query = query.filter(AnnouncementColumn::Enabled.eq(value));
    }
    query
}

fn apply_announcement_patch(active: &mut AnnouncementActiveModel, input: AnnouncementRecordPatch) {
    apply_announcement_flags(active, &input);
    if let Some(value) = input.title {
        active.title = Set(value);
    }
    if let Some(value) = input.content_markdown {
        active.content_markdown = Set(value);
    }
    active.updated_by = Set(input.operator_id);
    active.updated_at = Set(time::OffsetDateTime::now_utc());
}

fn apply_announcement_flags(active: &mut AnnouncementActiveModel, input: &AnnouncementRecordPatch) {
    if let Some(value) = &input.announcement_type {
        active.announcement_type = Set(value.clone());
    }
    if let Some(value) = input.pinned {
        active.pinned = Set(value);
    }
    if let Some(value) = input.priority {
        active.priority = Set(value);
    }
    if let Some(value) = input.enabled {
        active.enabled = Set(value);
    }
}
