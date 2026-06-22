use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use types::{
    operations::{
        Announcement, AnnouncementInput, AnnouncementListFilters, AnnouncementPatch, NotificationItem, NotificationListFilters, SupportTicket,
        SupportTicketCreateInput, SupportTicketDetail, SupportTicketEmailDelivery, SupportTicketListFilters, SupportTicketMessage, SupportTicketMessageInput,
        SupportTicketPatch,
    },
    pagination::{Page, PageRequest},
};

use super::{OperationsRepository, OperationsResult, OperationsService, OperationsUseCase, TicketCaptchaVerifier, TicketEmail, TicketMailer};
use crate::application::OperationsError;

#[tokio::test]
async fn create_ticket_verifies_support_ticket_captcha_before_persisting() {
    let service = OperationsService::new(TestRepository::default(), TestMailer, RejectingCaptcha, "admin@example.test".into());
    let result = service.create_ticket(ticket_input()).await;

    assert_eq!(result.unwrap_err().to_string(), "invalid input: captcha verification is required");
}

#[tokio::test]
async fn delete_read_notifications_uses_current_user_visibility_scope() {
    let repository = TestRepository::default();
    let calls = repository.delete_read_calls.clone();
    let service = OperationsService::new(repository, TestMailer, RejectingCaptcha, "admin@example.test".into());

    service.delete_read_notifications("user-1", true).await.unwrap();

    let recorded = calls.lock().expect("delete read calls").clone();
    assert_eq!(
        recorded,
        vec![DeleteReadCall {
            user_id: "user-1".into(),
            is_admin: true,
        }]
    );
}

fn ticket_input() -> SupportTicketCreateInput {
    SupportTicketCreateInput {
        user_id: "user-1".into(),
        subject: "Help".into(),
        body_markdown: "Need support".into(),
        contact_email: Some("user@example.test".into()),
        captcha_token: None,
    }
}

struct RejectingCaptcha;

#[async_trait]
impl TicketCaptchaVerifier for RejectingCaptcha {
    async fn verify_support_ticket(&self, _token: Option<&str>) -> OperationsResult<()> {
        Err(OperationsError::InvalidInput("captcha verification is required".into()))
    }
}

struct TestMailer;

#[async_trait]
impl TicketMailer for TestMailer {
    async fn send_ticket_email(&self, _email: TicketEmail) -> SupportTicketEmailDelivery {
        SupportTicketEmailDelivery {
            status: "disabled".into(),
            error_code: None,
            error_message: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DeleteReadCall {
    user_id: String,
    is_admin: bool,
}

#[derive(Clone, Default)]
struct TestRepository {
    delete_read_calls: Arc<Mutex<Vec<DeleteReadCall>>>,
}

#[async_trait]
impl OperationsRepository for TestRepository {
    async fn create_announcement(&self, _operator_id: &str, _input: AnnouncementInput) -> OperationsResult<Announcement> {
        unimplemented!("not needed for ticket captcha tests")
    }

    async fn update_announcement(&self, _id: &str, _operator_id: &str, _input: AnnouncementPatch) -> OperationsResult<Announcement> {
        unimplemented!("not needed for ticket captcha tests")
    }

    async fn delete_announcement(&self, _id: &str) -> OperationsResult<()> {
        unimplemented!("not needed for ticket captcha tests")
    }

    async fn get_announcement(&self, _id: &str) -> OperationsResult<Option<Announcement>> {
        unimplemented!("not needed for ticket captcha tests")
    }

    async fn list_announcements(&self, _page: PageRequest, _filters: AnnouncementListFilters) -> OperationsResult<Page<Announcement>> {
        unimplemented!("not needed for ticket captcha tests")
    }

    async fn unread_announcements(&self, _user_id: &str) -> OperationsResult<Vec<Announcement>> {
        unimplemented!("not needed for ticket captcha tests")
    }

    async fn user_email(&self, _user_id: &str) -> OperationsResult<Option<String>> {
        unimplemented!("captcha must fail before user email lookup")
    }

    async fn create_ticket(&self, _input: SupportTicketCreateInput) -> OperationsResult<(SupportTicket, SupportTicketMessage)> {
        unimplemented!("captcha must fail before ticket persistence")
    }

    async fn add_ticket_message(&self, _input: SupportTicketMessageInput) -> OperationsResult<(SupportTicket, SupportTicketMessage)> {
        unimplemented!("not needed for ticket captcha tests")
    }

    async fn update_ticket(&self, _id: &str, _operator_id: &str, _input: SupportTicketPatch) -> OperationsResult<SupportTicket> {
        unimplemented!("not needed for ticket captcha tests")
    }

    async fn ticket_detail(&self, _id: &str) -> OperationsResult<Option<SupportTicketDetail>> {
        unimplemented!("not needed for ticket captcha tests")
    }

    async fn list_tickets(&self, _user_id: Option<&str>, _page: PageRequest, _filters: SupportTicketListFilters) -> OperationsResult<Page<SupportTicket>> {
        unimplemented!("not needed for ticket captcha tests")
    }

    async fn record_email_delivery(
        &self,
        _ticket: &SupportTicket,
        _message_id: Option<&str>,
        _email: &TicketEmail,
        _delivery: &SupportTicketEmailDelivery,
    ) -> OperationsResult<()> {
        unimplemented!("not needed for ticket captcha tests")
    }

    async fn list_notifications(
        &self,
        _user_id: &str,
        _is_admin: bool,
        _page: PageRequest,
        _filters: NotificationListFilters,
    ) -> OperationsResult<Page<NotificationItem>> {
        unimplemented!("not needed for ticket captcha tests")
    }

    async fn mark_notification_read(&self, _user_id: &str, _source_type: &str, _source_id: &str) -> OperationsResult<()> {
        unimplemented!("not needed for ticket captcha tests")
    }

    async fn mark_all_notifications_read(&self, _user_id: &str, _is_admin: bool) -> OperationsResult<()> {
        unimplemented!("not needed for ticket captcha tests")
    }

    async fn delete_notification(&self, _user_id: &str, _source_type: &str, _source_id: &str) -> OperationsResult<()> {
        unimplemented!("not needed for ticket captcha tests")
    }

    async fn delete_read_notifications(&self, user_id: &str, is_admin: bool) -> OperationsResult<()> {
        self.delete_read_calls.lock().expect("delete read calls").push(DeleteReadCall {
            user_id: user_id.into(),
            is_admin,
        });
        Ok(())
    }
}
