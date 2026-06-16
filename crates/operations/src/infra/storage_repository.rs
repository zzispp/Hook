use async_trait::async_trait;
use storage::{
    Database, StorageError,
    operations::{
        AnnouncementRecordInput, AnnouncementRecordPatch, EmailEventRecordInput, OperationsStore, TicketMessageRecordInput, TicketRecordInput,
        TicketRecordPatch,
    },
};
use types::{
    operations::{
        Announcement, AnnouncementInput, AnnouncementListFilters, AnnouncementPatch, NotificationItem, NotificationListFilters, SupportTicket,
        SupportTicketCreateInput, SupportTicketDetail, SupportTicketEmailDelivery, SupportTicketListFilters, SupportTicketMessage, SupportTicketMessageInput,
        SupportTicketPatch,
    },
    pagination::{Page, PageRequest, PageSliceRequest},
};

use crate::application::{OperationsError, OperationsRepository, OperationsResult, TicketEmail};

#[derive(Clone)]
pub struct StorageOperationsRepository {
    store: OperationsStore,
}

impl StorageOperationsRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: OperationsStore::new(database),
        }
    }
}

#[async_trait]
impl OperationsRepository for StorageOperationsRepository {
    async fn create_announcement(&self, operator_id: &str, input: AnnouncementInput) -> OperationsResult<Announcement> {
        self.store
            .create_announcement(announcement_input(operator_id, input))
            .await
            .map_err(storage_error)
    }

    async fn update_announcement(&self, id: &str, operator_id: &str, input: AnnouncementPatch) -> OperationsResult<Announcement> {
        self.store
            .update_announcement(id, announcement_patch(operator_id, input))
            .await
            .map_err(storage_error)
    }

    async fn delete_announcement(&self, id: &str) -> OperationsResult<()> {
        self.store.delete_announcement(id).await.map_err(storage_error)
    }

    async fn get_announcement(&self, id: &str) -> OperationsResult<Option<Announcement>> {
        self.store.get_announcement(id).await.map_err(storage_error)
    }

    async fn list_announcements(&self, page: PageRequest, filters: AnnouncementListFilters) -> OperationsResult<Page<Announcement>> {
        self.store.page_announcements(page_slice(page), filters).await.map_err(storage_error)
    }

    async fn user_email(&self, user_id: &str) -> OperationsResult<Option<String>> {
        self.store.user_email(user_id).await.map_err(storage_error)
    }

    async fn create_ticket(&self, input: SupportTicketCreateInput) -> OperationsResult<(SupportTicket, SupportTicketMessage)> {
        self.store.create_ticket(ticket_input(input)).await.map_err(storage_error)
    }

    async fn add_ticket_message(&self, input: SupportTicketMessageInput) -> OperationsResult<(SupportTicket, SupportTicketMessage)> {
        self.store.add_ticket_message(message_input(input)).await.map_err(storage_error)
    }

    async fn update_ticket(&self, id: &str, operator_id: &str, input: SupportTicketPatch) -> OperationsResult<SupportTicket> {
        self.store.update_ticket(id, ticket_patch(operator_id, input)).await.map_err(storage_error)
    }

    async fn ticket_detail(&self, id: &str) -> OperationsResult<Option<SupportTicketDetail>> {
        self.store.ticket_detail(id).await.map_err(storage_error)
    }

    async fn list_tickets(&self, user_id: Option<&str>, page: PageRequest, filters: SupportTicketListFilters) -> OperationsResult<Page<SupportTicket>> {
        self.store.page_tickets(user_id, page_slice(page), filters).await.map_err(storage_error)
    }

    async fn record_email_delivery(
        &self,
        ticket: &SupportTicket,
        message_id: Option<&str>,
        email: &TicketEmail,
        delivery: &SupportTicketEmailDelivery,
    ) -> OperationsResult<()> {
        self.store
            .record_email_event(email_event(ticket, message_id, email, delivery))
            .await
            .map(|_| ())
            .map_err(storage_error)
    }

    async fn list_notifications(
        &self,
        user_id: &str,
        is_admin: bool,
        page: PageRequest,
        filters: NotificationListFilters,
    ) -> OperationsResult<Page<NotificationItem>> {
        self.store
            .page_notifications(user_id, is_admin, page_slice(page), filters)
            .await
            .map_err(storage_error)
    }

    async fn mark_notification_read(&self, user_id: &str, source_type: &str, source_id: &str) -> OperationsResult<()> {
        self.store.mark_notification_read(user_id, source_type, source_id).await.map_err(storage_error)
    }

    async fn mark_all_notifications_read(&self, user_id: &str, is_admin: bool) -> OperationsResult<()> {
        self.store.mark_all_notifications_read(user_id, is_admin).await.map_err(storage_error)
    }

    async fn delete_notification(&self, user_id: &str, source_type: &str, source_id: &str) -> OperationsResult<()> {
        self.store.delete_notification(user_id, source_type, source_id).await.map_err(storage_error)
    }

    async fn delete_read_notifications(&self, user_id: &str, is_admin: bool) -> OperationsResult<()> {
        self.store.delete_read_notifications(user_id, is_admin).await.map_err(storage_error)
    }
}

fn announcement_input(operator_id: &str, input: AnnouncementInput) -> AnnouncementRecordInput {
    AnnouncementRecordInput {
        title: input.title,
        content_markdown: input.content_markdown,
        announcement_type: input.announcement_type,
        pinned: input.pinned,
        priority: input.priority,
        enabled: input.enabled,
        operator_id: operator_id.into(),
    }
}

fn announcement_patch(operator_id: &str, input: AnnouncementPatch) -> AnnouncementRecordPatch {
    AnnouncementRecordPatch {
        title: input.title,
        content_markdown: input.content_markdown,
        announcement_type: input.announcement_type,
        pinned: input.pinned,
        priority: input.priority,
        enabled: input.enabled,
        operator_id: operator_id.into(),
    }
}

fn ticket_input(input: SupportTicketCreateInput) -> TicketRecordInput {
    TicketRecordInput {
        user_id: input.user_id,
        subject: input.subject,
        contact_email: input.contact_email.unwrap_or_default(),
        body_markdown: input.body_markdown,
    }
}

fn message_input(input: SupportTicketMessageInput) -> TicketMessageRecordInput {
    TicketMessageRecordInput {
        ticket_id: input.ticket_id,
        sender_user_id: input.sender_user_id,
        sender_role: input.sender_role,
        body_markdown: input.body_markdown,
    }
}

fn ticket_patch(operator_id: &str, input: SupportTicketPatch) -> TicketRecordPatch {
    TicketRecordPatch {
        status: input.status,
        priority: input.priority,
        operator_id: operator_id.into(),
    }
}

fn email_event(ticket: &SupportTicket, message_id: Option<&str>, email: &TicketEmail, delivery: &SupportTicketEmailDelivery) -> EmailEventRecordInput {
    EmailEventRecordInput {
        ticket_id: ticket.id.clone(),
        message_id: message_id.map(str::to_owned),
        recipient_email: email.recipient_email.clone(),
        subject: email.subject.clone(),
        status: delivery.status.clone(),
        error_message: delivery.error_message.clone(),
    }
}

fn page_slice(page: PageRequest) -> PageSliceRequest {
    PageSliceRequest {
        offset: (page.page - 1) * page.page_size,
        limit: page.page_size,
        page: page.page,
        page_size: page.page_size,
    }
}

fn storage_error(error: StorageError) -> OperationsError {
    match error {
        StorageError::NotFound => OperationsError::NotFound,
        StorageError::Conflict(message) => OperationsError::InvalidInput(message),
        StorageError::Database(message) => OperationsError::Infrastructure(message),
    }
}
