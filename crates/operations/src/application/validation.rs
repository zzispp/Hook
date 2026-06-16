use types::{
    operations::{AnnouncementInput, AnnouncementPatch, SupportTicketCreateInput, SupportTicketListFilters, SupportTicketMessageInput, SupportTicketPatch},
    pagination::PageRequest,
};

use super::{OperationsError, OperationsResult};

const ANNOUNCEMENT_TYPES: &[&str] = &["info", "warning", "maintenance", "important"];
const EMAIL_MAX_LEN: usize = 255;
const MAX_PAGE_SIZE: u64 = 100;
const NOTIFICATION_SOURCE_TYPES: &[&str] = &["announcement", "ticket", "provider_quick_import_sync"];
const TEXT_MAX_LEN: usize = 20_000;
const PRIORITIES: &[&str] = &["normal", "high", "urgent"];
const STATUSES: &[&str] = &["open", "in_progress", "waiting_user", "resolved", "closed"];
const TITLE_MAX_LEN: usize = 200;

pub fn sanitize_announcement(input: AnnouncementInput) -> AnnouncementInput {
    AnnouncementInput {
        title: input.title.trim().into(),
        content_markdown: input.content_markdown.trim().into(),
        announcement_type: input.announcement_type.trim().into(),
        pinned: input.pinned,
        priority: input.priority,
        enabled: input.enabled,
    }
}

pub fn sanitize_announcement_patch(input: AnnouncementPatch) -> AnnouncementPatch {
    AnnouncementPatch {
        title: input.title.map(|value| value.trim().into()),
        content_markdown: input.content_markdown.map(|value| value.trim().into()),
        announcement_type: input.announcement_type.map(|value| value.trim().into()),
        pinned: input.pinned,
        priority: input.priority,
        enabled: input.enabled,
    }
}

pub fn sanitize_ticket(input: SupportTicketCreateInput) -> SupportTicketCreateInput {
    SupportTicketCreateInput {
        user_id: input.user_id.trim().into(),
        subject: input.subject.trim().into(),
        body_markdown: input.body_markdown.trim().into(),
        contact_email: input.contact_email.map(|value| value.trim().into()),
        captcha_token: input.captcha_token.map(|value| value.trim().into()),
    }
}

pub fn sanitize_ticket_message(input: SupportTicketMessageInput) -> SupportTicketMessageInput {
    SupportTicketMessageInput {
        ticket_id: input.ticket_id.trim().into(),
        sender_user_id: input.sender_user_id.trim().into(),
        sender_role: input.sender_role.trim().into(),
        body_markdown: input.body_markdown.trim().into(),
    }
}

pub fn sanitize_ticket_patch(input: SupportTicketPatch) -> SupportTicketPatch {
    SupportTicketPatch {
        status: input.status.map(|value| value.trim().into()),
        priority: input.priority.map(|value| value.trim().into()),
    }
}

pub fn validate_announcement(input: &AnnouncementInput) -> OperationsResult<()> {
    validate_required_text("title", &input.title, TITLE_MAX_LEN)?;
    validate_required_text("content_markdown", &input.content_markdown, TEXT_MAX_LEN)?;
    validate_enum("announcement_type", &input.announcement_type, ANNOUNCEMENT_TYPES)
}

pub fn validate_announcement_patch(input: &AnnouncementPatch) -> OperationsResult<()> {
    validate_optional_text("title", input.title.as_deref(), TITLE_MAX_LEN)?;
    validate_optional_text("content_markdown", input.content_markdown.as_deref(), TEXT_MAX_LEN)?;
    validate_optional_enum("announcement_type", input.announcement_type.as_deref(), ANNOUNCEMENT_TYPES)
}

pub fn validate_ticket(input: &SupportTicketCreateInput) -> OperationsResult<()> {
    validate_required_text("subject", &input.subject, TITLE_MAX_LEN)?;
    validate_required_text("body_markdown", &input.body_markdown, TEXT_MAX_LEN)?;
    validate_optional_email(input.contact_email.as_deref())
}

pub fn validate_ticket_message(input: &SupportTicketMessageInput) -> OperationsResult<()> {
    validate_required_text("body_markdown", &input.body_markdown, TEXT_MAX_LEN)
}

pub fn validate_ticket_patch(input: &SupportTicketPatch) -> OperationsResult<()> {
    validate_optional_enum("status", input.status.as_deref(), STATUSES)?;
    validate_optional_enum("priority", input.priority.as_deref(), PRIORITIES)
}

pub fn validate_ticket_filters(input: &SupportTicketListFilters) -> OperationsResult<()> {
    validate_optional_enum("status", input.status.as_deref(), STATUSES)?;
    validate_optional_enum("priority", input.priority.as_deref(), PRIORITIES)
}

pub fn validate_page(page: PageRequest) -> OperationsResult<()> {
    if page.page == 0 || page.page_size == 0 || page.page_size > MAX_PAGE_SIZE {
        return Err(OperationsError::InvalidInput("invalid page request".into()));
    }
    Ok(())
}

pub fn validate_source_type(value: &str) -> OperationsResult<()> {
    validate_enum("source_type", value, NOTIFICATION_SOURCE_TYPES)
}

pub fn validate_email(value: &str) -> OperationsResult<()> {
    if value.len() > EMAIL_MAX_LEN || !value.contains('@') || value.starts_with('@') || value.ends_with('@') {
        return Err(OperationsError::InvalidInput("contact_email must be a valid email address".into()));
    }
    Ok(())
}

fn validate_required_text(field: &str, value: &str, max_len: usize) -> OperationsResult<()> {
    if value.is_empty() {
        return Err(OperationsError::InvalidInput(format!("{field} cannot be empty")));
    }
    validate_len(field, value, max_len)
}

fn validate_optional_text(field: &str, value: Option<&str>, max_len: usize) -> OperationsResult<()> {
    match value {
        Some(text) => validate_required_text(field, text, max_len),
        None => Ok(()),
    }
}

fn validate_optional_email(value: Option<&str>) -> OperationsResult<()> {
    match value {
        Some(email) => validate_email(email),
        None => Ok(()),
    }
}

fn validate_enum(field: &str, value: &str, allowed: &[&str]) -> OperationsResult<()> {
    if allowed.contains(&value) {
        return Ok(());
    }
    Err(OperationsError::InvalidInput(format!("{field} has unsupported value")))
}

fn validate_optional_enum(field: &str, value: Option<&str>, allowed: &[&str]) -> OperationsResult<()> {
    match value {
        Some(text) => validate_enum(field, text, allowed),
        None => Ok(()),
    }
}

fn validate_len(field: &str, value: &str, max_len: usize) -> OperationsResult<()> {
    if value.len() > max_len {
        return Err(OperationsError::InvalidInput(format!("{field} is too long")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_quick_import_sync_is_valid_notification_source_type() {
        assert!(validate_source_type("provider_quick_import_sync").is_ok());
    }

    #[test]
    fn unknown_notification_source_type_is_rejected() {
        let error = validate_source_type("provider").unwrap_err();

        assert_eq!(error.to_string(), "invalid input: source_type has unsupported value");
    }
}
