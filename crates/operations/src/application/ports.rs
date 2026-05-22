use async_trait::async_trait;
use types::{
    operations::{
        Announcement, AnnouncementInput, AnnouncementListFilters, AnnouncementPatch, NotificationItem, NotificationListFilters, SupportTicket,
        SupportTicketCreateInput, SupportTicketDetail, SupportTicketEmailDelivery, SupportTicketListFilters, SupportTicketMessage, SupportTicketMessageInput,
        SupportTicketMutationResponse, SupportTicketPatch,
    },
    pagination::{Page, PageRequest},
};

use super::OperationsResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TicketEmail {
    pub recipient_email: String,
    pub subject: String,
    pub body_markdown: String,
}

#[async_trait]
pub trait TicketMailer: Send + Sync + 'static {
    async fn send_ticket_email(&self, email: TicketEmail) -> SupportTicketEmailDelivery;
}

/// Reads and consumes CAPTCHA verification for user-facing ticket submissions.
#[async_trait]
pub trait TicketCaptchaVerifier: Send + Sync + 'static {
    async fn verify_support_ticket(&self, token: Option<&str>) -> OperationsResult<()>;
}

#[async_trait]
pub trait OperationsRepository: Send + Sync + 'static {
    async fn create_announcement(&self, operator_id: &str, input: AnnouncementInput) -> OperationsResult<Announcement>;
    async fn update_announcement(&self, id: &str, operator_id: &str, input: AnnouncementPatch) -> OperationsResult<Announcement>;
    async fn delete_announcement(&self, id: &str) -> OperationsResult<()>;
    async fn get_announcement(&self, id: &str) -> OperationsResult<Option<Announcement>>;
    async fn list_announcements(&self, page: PageRequest, filters: AnnouncementListFilters) -> OperationsResult<Page<Announcement>>;
    async fn user_email(&self, user_id: &str) -> OperationsResult<Option<String>>;
    async fn create_ticket(&self, input: SupportTicketCreateInput) -> OperationsResult<(SupportTicket, SupportTicketMessage)>;
    async fn add_ticket_message(&self, input: SupportTicketMessageInput) -> OperationsResult<(SupportTicket, SupportTicketMessage)>;
    async fn update_ticket(&self, id: &str, operator_id: &str, input: SupportTicketPatch) -> OperationsResult<SupportTicket>;
    async fn ticket_detail(&self, id: &str) -> OperationsResult<Option<SupportTicketDetail>>;
    async fn list_tickets(&self, user_id: Option<&str>, page: PageRequest, filters: SupportTicketListFilters) -> OperationsResult<Page<SupportTicket>>;
    async fn record_email_delivery(
        &self,
        ticket: &SupportTicket,
        message_id: Option<&str>,
        email: &TicketEmail,
        delivery: &SupportTicketEmailDelivery,
    ) -> OperationsResult<()>;
    async fn list_notifications(
        &self,
        user_id: &str,
        is_admin: bool,
        page: PageRequest,
        filters: NotificationListFilters,
    ) -> OperationsResult<Page<NotificationItem>>;
    async fn mark_notification_read(&self, user_id: &str, source_type: &str, source_id: &str) -> OperationsResult<()>;
    async fn mark_all_notifications_read(&self, user_id: &str, is_admin: bool) -> OperationsResult<()>;
    async fn delete_notification(&self, user_id: &str, source_type: &str, source_id: &str) -> OperationsResult<()>;
}

#[async_trait]
pub trait OperationsUseCase: Send + Sync + 'static {
    async fn create_announcement(&self, operator_id: &str, input: AnnouncementInput) -> OperationsResult<Announcement>;
    async fn update_announcement(&self, id: &str, operator_id: &str, input: AnnouncementPatch) -> OperationsResult<Announcement>;
    async fn delete_announcement(&self, id: &str) -> OperationsResult<()>;
    async fn get_announcement(&self, id: &str, admin: bool) -> OperationsResult<Announcement>;
    async fn list_announcements(&self, page: PageRequest, filters: AnnouncementListFilters, admin: bool) -> OperationsResult<Page<Announcement>>;
    async fn create_ticket(&self, input: SupportTicketCreateInput) -> OperationsResult<SupportTicketMutationResponse>;
    async fn user_reply_ticket(&self, input: SupportTicketMessageInput) -> OperationsResult<SupportTicketMutationResponse>;
    async fn admin_reply_ticket(&self, input: SupportTicketMessageInput) -> OperationsResult<SupportTicketMutationResponse>;
    async fn update_ticket(&self, id: &str, operator_id: &str, input: SupportTicketPatch) -> OperationsResult<SupportTicketMutationResponse>;
    async fn ticket_detail(&self, id: &str, user_id: Option<&str>) -> OperationsResult<SupportTicketDetail>;
    async fn list_tickets(&self, user_id: Option<&str>, page: PageRequest, filters: SupportTicketListFilters) -> OperationsResult<Page<SupportTicket>>;
    async fn list_notifications(
        &self,
        user_id: &str,
        is_admin: bool,
        page: PageRequest,
        filters: NotificationListFilters,
    ) -> OperationsResult<Page<NotificationItem>>;
    async fn mark_notification_read(&self, user_id: &str, source_type: &str, source_id: &str) -> OperationsResult<()>;
    async fn mark_all_notifications_read(&self, user_id: &str, is_admin: bool) -> OperationsResult<()>;
    async fn delete_notification(&self, user_id: &str, source_type: &str, source_id: &str) -> OperationsResult<()>;
}
