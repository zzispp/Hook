use async_trait::async_trait;
use types::{
    operations::{
        Announcement, AnnouncementInput, AnnouncementListFilters, AnnouncementPatch, NotificationItem, NotificationListFilters, SupportTicket,
        SupportTicketCreateInput, SupportTicketDetail, SupportTicketListFilters, SupportTicketMessageInput, SupportTicketMutationResponse, SupportTicketPatch,
    },
    pagination::{Page, PageRequest},
};

use crate::application::{
    OperationsRepository, OperationsResult, OperationsUseCase, TicketCaptchaVerifier, TicketEmail, TicketMailer,
    validation::{
        sanitize_announcement, sanitize_announcement_patch, sanitize_ticket, sanitize_ticket_message, sanitize_ticket_patch, validate_announcement,
        validate_announcement_patch, validate_email, validate_page, validate_source_type, validate_ticket, validate_ticket_filters, validate_ticket_message,
        validate_ticket_patch,
    },
};

use super::OperationsError;

const ADMIN_ROLE: &str = "admin";

pub struct OperationsService<R, M, C> {
    repository: R,
    mailer: M,
    captcha: C,
    admin_email: String,
}

impl<R, M, C> OperationsService<R, M, C>
where
    R: OperationsRepository,
    M: TicketMailer,
    C: TicketCaptchaVerifier,
{
    pub fn new(repository: R, mailer: M, captcha: C, admin_email: String) -> Self {
        Self {
            repository,
            mailer,
            captcha,
            admin_email,
        }
    }

    async fn send_and_record(
        &self,
        ticket: &SupportTicket,
        message_id: Option<&str>,
        email: TicketEmail,
    ) -> OperationsResult<types::operations::SupportTicketEmailDelivery> {
        let delivery = self.mailer.send_ticket_email(email.clone()).await;
        self.repository.record_email_delivery(ticket, message_id, &email, &delivery).await?;
        Ok(delivery)
    }

    async fn resolved_contact_email(&self, input: &SupportTicketCreateInput) -> OperationsResult<String> {
        let email = match input.contact_email.as_deref() {
            Some(value) if !value.is_empty() => value.to_owned(),
            _ => self.repository.user_email(&input.user_id).await?.ok_or(OperationsError::NotFound)?,
        };
        validate_email(&email)?;
        Ok(email)
    }

    async fn owner_ticket_detail(&self, ticket_id: &str, user_id: Option<&str>) -> OperationsResult<SupportTicketDetail> {
        let detail = self.repository.ticket_detail(ticket_id).await?.ok_or(OperationsError::NotFound)?;
        if user_id.is_some_and(|owner| detail.ticket.user_id != owner) {
            return Err(OperationsError::Forbidden);
        }
        Ok(detail)
    }
}

#[async_trait]
impl<R, M, C> OperationsUseCase for OperationsService<R, M, C>
where
    R: OperationsRepository,
    M: TicketMailer,
    C: TicketCaptchaVerifier,
{
    async fn create_announcement(&self, operator_id: &str, input: AnnouncementInput) -> OperationsResult<Announcement> {
        let input = sanitize_announcement(input);
        validate_announcement(&input)?;
        self.repository.create_announcement(operator_id, input).await
    }

    async fn update_announcement(&self, id: &str, operator_id: &str, input: AnnouncementPatch) -> OperationsResult<Announcement> {
        let input = sanitize_announcement_patch(input);
        validate_announcement_patch(&input)?;
        self.repository.update_announcement(id, operator_id, input).await
    }

    async fn delete_announcement(&self, id: &str) -> OperationsResult<()> {
        self.repository.delete_announcement(id).await
    }

    async fn get_announcement(&self, id: &str, admin: bool) -> OperationsResult<Announcement> {
        let announcement = self.repository.get_announcement(id).await?.ok_or(OperationsError::NotFound)?;
        if !admin && !announcement.enabled {
            return Err(OperationsError::NotFound);
        }
        Ok(announcement)
    }

    async fn list_announcements(&self, page: PageRequest, mut filters: AnnouncementListFilters, admin: bool) -> OperationsResult<Page<Announcement>> {
        validate_page(page)?;
        if !admin {
            filters.enabled = Some(true);
        }
        self.repository.list_announcements(page, filters).await
    }

    async fn unread_announcements(&self, user_id: &str) -> OperationsResult<Vec<Announcement>> {
        self.repository.unread_announcements(user_id).await
    }

    async fn create_ticket(&self, input: SupportTicketCreateInput) -> OperationsResult<SupportTicketMutationResponse> {
        let mut input = sanitize_ticket(input);
        validate_ticket(&input)?;
        self.captcha.verify_support_ticket(input.captcha_token.as_deref()).await?;
        input.contact_email = Some(self.resolved_contact_email(&input).await?);
        let (ticket, message) = self.repository.create_ticket(input).await?;
        let email = admin_email(&self.admin_email, &ticket, &message.body_markdown);
        let delivery = self.send_and_record(&ticket, Some(&message.id), email).await?;
        Ok(SupportTicketMutationResponse {
            ticket,
            message: Some(message),
            email_delivery: delivery,
        })
    }

    async fn user_reply_ticket(&self, input: SupportTicketMessageInput) -> OperationsResult<SupportTicketMutationResponse> {
        let input = sanitize_ticket_message(input);
        validate_ticket_message(&input)?;
        self.owner_ticket_detail(&input.ticket_id, Some(&input.sender_user_id)).await?;
        let (ticket, message) = self.repository.add_ticket_message(input).await?;
        let email = admin_email(&self.admin_email, &ticket, &message.body_markdown);
        let delivery = self.send_and_record(&ticket, Some(&message.id), email).await?;
        Ok(SupportTicketMutationResponse {
            ticket,
            message: Some(message),
            email_delivery: delivery,
        })
    }

    async fn admin_reply_ticket(&self, input: SupportTicketMessageInput) -> OperationsResult<SupportTicketMutationResponse> {
        let input = sanitize_ticket_message(input);
        validate_ticket_message(&input)?;
        let (ticket, message) = self.repository.add_ticket_message(input).await?;
        let email = user_email(&ticket, &message.body_markdown);
        let delivery = self.send_and_record(&ticket, Some(&message.id), email).await?;
        Ok(SupportTicketMutationResponse {
            ticket,
            message: Some(message),
            email_delivery: delivery,
        })
    }

    async fn update_ticket(&self, id: &str, operator_id: &str, input: SupportTicketPatch) -> OperationsResult<SupportTicketMutationResponse> {
        let input = sanitize_ticket_patch(input);
        validate_ticket_patch(&input)?;
        let ticket = self.repository.update_ticket(id, operator_id, input).await?;
        let email = user_email(&ticket, &format!("工单状态已更新：{}", ticket.status));
        let delivery = self.send_and_record(&ticket, None, email).await?;
        Ok(SupportTicketMutationResponse {
            ticket,
            message: None,
            email_delivery: delivery,
        })
    }

    async fn ticket_detail(&self, id: &str, user_id: Option<&str>) -> OperationsResult<SupportTicketDetail> {
        self.owner_ticket_detail(id, user_id).await
    }

    async fn list_tickets(&self, user_id: Option<&str>, page: PageRequest, filters: SupportTicketListFilters) -> OperationsResult<Page<SupportTicket>> {
        validate_page(page)?;
        validate_ticket_filters(&filters)?;
        self.repository.list_tickets(user_id, page, filters).await
    }

    async fn list_notifications(
        &self,
        user_id: &str,
        is_admin: bool,
        page: PageRequest,
        filters: NotificationListFilters,
    ) -> OperationsResult<Page<NotificationItem>> {
        validate_page(page)?;
        self.repository.list_notifications(user_id, is_admin, page, filters).await
    }

    async fn mark_notification_read(&self, user_id: &str, source_type: &str, source_id: &str) -> OperationsResult<()> {
        validate_source_type(source_type)?;
        self.repository.mark_notification_read(user_id, source_type, source_id).await
    }

    async fn mark_all_notifications_read(&self, user_id: &str, is_admin: bool) -> OperationsResult<()> {
        self.repository.mark_all_notifications_read(user_id, is_admin).await
    }

    async fn delete_notification(&self, user_id: &str, source_type: &str, source_id: &str) -> OperationsResult<()> {
        validate_source_type(source_type)?;
        self.repository.delete_notification(user_id, source_type, source_id).await
    }

    async fn delete_read_notifications(&self, user_id: &str, is_admin: bool) -> OperationsResult<()> {
        self.repository.delete_read_notifications(user_id, is_admin).await
    }
}

fn admin_email(admin_email: &str, ticket: &SupportTicket, body_markdown: &str) -> TicketEmail {
    TicketEmail {
        recipient_email: admin_email.into(),
        subject: format!("[Support Ticket] {}", ticket.subject),
        body_markdown: body_markdown.into(),
    }
}

fn user_email(ticket: &SupportTicket, body_markdown: &str) -> TicketEmail {
    TicketEmail {
        recipient_email: ticket.contact_email.clone(),
        subject: format!("[Support Ticket] {}", ticket.subject),
        body_markdown: body_markdown.into(),
    }
}

pub fn is_admin_role(role: &str) -> bool {
    role == ADMIN_ROLE
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
