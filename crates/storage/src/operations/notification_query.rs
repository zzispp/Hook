use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use types::{
    operations::{NotificationItem, NotificationListFilters},
    pagination::{Page, PageSliceRequest},
};

use crate::StorageResult;

use super::{
    AnnouncementColumn, AnnouncementEntity, NotificationSourceRecord, NotificationStateActiveModel, NotificationStateRecord, OperationsStore, TicketColumn,
    TicketEntity, time_format::format_timestamp,
};

const CATEGORY_ANNOUNCEMENT: &str = "announcement";
const CATEGORY_TICKET: &str = "ticket";
const FILTER_READ: &str = "read";
const FILTER_UNREAD: &str = "unread";
const SOURCE_ANNOUNCEMENT: &str = "announcement";
const SOURCE_TICKET: &str = "ticket";

impl OperationsStore {
    pub async fn page_notifications(
        &self,
        user_id: &str,
        is_admin: bool,
        request: PageSliceRequest,
        filters: NotificationListFilters,
    ) -> StorageResult<Page<NotificationItem>> {
        let sources = self.notification_sources(user_id, is_admin).await?;
        let mut items = self.notification_items(user_id, sources).await?;
        items.retain(|item| matches_filter(item.is_unread, filters.status.as_deref()));
        let total = items.len() as u64;
        let items = items.into_iter().skip(request.offset as usize).take(request.limit as usize).collect();
        Ok(Page {
            items,
            total,
            page: request.page,
            page_size: request.page_size,
        })
    }

    pub async fn mark_notification_read(&self, user_id: &str, source_type: &str, source_id: &str) -> StorageResult<()> {
        self.upsert_notification_state(user_id, source_type, source_id, StatePatch::read()).await
    }

    pub async fn delete_notification(&self, user_id: &str, source_type: &str, source_id: &str) -> StorageResult<()> {
        self.upsert_notification_state(user_id, source_type, source_id, StatePatch::deleted()).await
    }

    pub async fn mark_all_notifications_read(&self, user_id: &str, is_admin: bool) -> StorageResult<()> {
        for source in self.notification_sources(user_id, is_admin).await? {
            self.mark_notification_read(user_id, &source.source_type, &source.source_id).await?;
        }
        Ok(())
    }

    async fn notification_sources(&self, user_id: &str, is_admin: bool) -> StorageResult<Vec<NotificationSourceRecord>> {
        if is_admin {
            return self.admin_ticket_sources().await;
        }
        let mut sources = self.announcement_sources().await?;
        sources.extend(self.user_ticket_sources(user_id).await?);
        sources.sort_by_key(|source| std::cmp::Reverse(source.event_at));
        Ok(sources)
    }

    async fn notification_items(&self, user_id: &str, sources: Vec<NotificationSourceRecord>) -> StorageResult<Vec<NotificationItem>> {
        let mut items = Vec::with_capacity(sources.len());
        for source in sources {
            if let Some(item) = self.notification_item(user_id, source).await? {
                items.push(item);
            }
        }
        Ok(items)
    }

    async fn notification_item(&self, user_id: &str, source: NotificationSourceRecord) -> StorageResult<Option<NotificationItem>> {
        let state = self.notification_state(user_id, &source.source_type, &source.source_id).await?;
        if deleted_after_event(state.as_ref(), source.event_at) {
            return Ok(None);
        }
        Ok(Some(NotificationItem {
            source_type: source.source_type,
            source_id: source.source_id,
            title: source.title,
            category: source.category,
            is_unread: unread_after_event(state.as_ref(), source.event_at),
            created_at: format_timestamp(source.event_at),
            link_path: source.link_path,
        }))
    }

    async fn upsert_notification_state(&self, user_id: &str, source_type: &str, source_id: &str, patch: StatePatch) -> StorageResult<()> {
        let now = time::OffsetDateTime::now_utc();
        match self.notification_state(user_id, source_type, source_id).await? {
            Some(record) => update_notification_state(self, record, patch, now).await,
            None => insert_notification_state(self, user_id, source_type, source_id, patch, now).await,
        }
    }
}

async fn update_notification_state(
    store: &OperationsStore,
    record: NotificationStateRecord,
    patch: StatePatch,
    now: time::OffsetDateTime,
) -> StorageResult<()> {
    let mut active: NotificationStateActiveModel = record.into();
    apply_state_patch(&mut active, patch, now);
    active.updated_at = Set(now);
    active.update(store.connection()).await?;
    Ok(())
}

async fn insert_notification_state(
    store: &OperationsStore,
    user_id: &str,
    source_type: &str,
    source_id: &str,
    patch: StatePatch,
    now: time::OffsetDateTime,
) -> StorageResult<()> {
    let mut active = NotificationStateActiveModel {
        id: Set(store.next_id()),
        user_id: Set(user_id.to_owned()),
        source_type: Set(source_type.to_owned()),
        source_id: Set(source_id.to_owned()),
        read_at: Set(None),
        deleted_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };
    apply_state_patch(&mut active, patch, now);
    active.insert(store.connection()).await?;
    Ok(())
}

fn apply_state_patch(active: &mut NotificationStateActiveModel, patch: StatePatch, now: time::OffsetDateTime) {
    if patch.read {
        active.read_at = Set(Some(now));
    }
    if patch.deleted {
        active.deleted_at = Set(Some(now));
    }
}

fn deleted_after_event(state: Option<&NotificationStateRecord>, event_at: time::OffsetDateTime) -> bool {
    state.and_then(|value| value.deleted_at).is_some_and(|deleted_at| deleted_at >= event_at)
}

fn unread_after_event(state: Option<&NotificationStateRecord>, event_at: time::OffsetDateTime) -> bool {
    state.and_then(|value| value.read_at).is_none_or(|read_at| read_at < event_at)
}

fn matches_filter(unread: bool, status: Option<&str>) -> bool {
    match status {
        Some(FILTER_UNREAD) => unread,
        Some(FILTER_READ) => !unread,
        _ => true,
    }
}

#[derive(Clone, Copy)]
struct StatePatch {
    read: bool,
    deleted: bool,
}

impl StatePatch {
    const fn read() -> Self {
        Self { read: true, deleted: false }
    }

    const fn deleted() -> Self {
        Self { read: false, deleted: true }
    }
}

impl OperationsStore {
    async fn announcement_sources(&self) -> StorageResult<Vec<NotificationSourceRecord>> {
        let records = AnnouncementEntity::find()
            .filter(AnnouncementColumn::Enabled.eq(true))
            .order_by_desc(AnnouncementColumn::CreatedAt)
            .all(self.connection())
            .await?;
        Ok(records
            .into_iter()
            .map(|record| NotificationSourceRecord {
                source_type: SOURCE_ANNOUNCEMENT.into(),
                source_id: record.id.clone(),
                title: record.title,
                category: format!("{CATEGORY_ANNOUNCEMENT}.{}", record.announcement_type),
                event_at: record.updated_at,
                link_path: format!("/dashboard/announcements/{}", record.id),
            })
            .collect())
    }

    async fn admin_ticket_sources(&self) -> StorageResult<Vec<NotificationSourceRecord>> {
        let records = TicketEntity::find()
            .filter(TicketColumn::LastUserActivityAt.is_not_null())
            .order_by_desc(TicketColumn::LastUserActivityAt)
            .all(self.connection())
            .await?;
        Ok(records.into_iter().filter_map(|record| ticket_source(record, true)).collect())
    }

    async fn user_ticket_sources(&self, user_id: &str) -> StorageResult<Vec<NotificationSourceRecord>> {
        let records = TicketEntity::find()
            .filter(TicketColumn::UserId.eq(user_id))
            .filter(TicketColumn::LastAdminActivityAt.is_not_null())
            .order_by_desc(TicketColumn::LastAdminActivityAt)
            .all(self.connection())
            .await?;
        Ok(records.into_iter().filter_map(|record| ticket_source(record, false)).collect())
    }
}

fn ticket_source(record: super::TicketRecord, admin: bool) -> Option<NotificationSourceRecord> {
    let event_at = if admin { record.last_user_activity_at } else { record.last_admin_activity_at }?;
    let link_path = if admin {
        format!("/dashboard/admin/tickets?id={}", record.id)
    } else {
        format!("/dashboard/tickets?id={}", record.id)
    };
    Some(NotificationSourceRecord {
        source_type: SOURCE_TICKET.into(),
        source_id: record.id,
        title: record.subject,
        category: CATEGORY_TICKET.into(),
        event_at,
        link_path,
    })
}
