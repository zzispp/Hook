use sea_orm_migration::{prelude::*, schema::*};

use super::iden::*;

pub(super) fn operations_tables() -> Vec<TableCreateStatement> {
    vec![
        announcements_table(),
        support_tickets_table(),
        support_ticket_messages_table(),
        support_ticket_email_events_table(),
        notification_states_table(),
    ]
}

fn announcements_table() -> TableCreateStatement {
    Table::create()
        .table(Announcements::Table)
        .if_not_exists()
        .col(string_len(Announcements::Id, 36).primary_key())
        .col(string_len(Announcements::Title, 200))
        .col(text(Announcements::ContentMarkdown))
        .col(string_len(Announcements::AnnouncementType, 20))
        .col(boolean(Announcements::Pinned))
        .col(boolean(Announcements::Enabled))
        .col(string_len(Announcements::CreatedBy, 36))
        .col(string_len(Announcements::UpdatedBy, 36))
        .col(timestamp_tz(Announcements::CreatedAt))
        .col(timestamp_tz(Announcements::UpdatedAt))
        .to_owned()
}

fn support_tickets_table() -> TableCreateStatement {
    Table::create()
        .table(SupportTickets::Table)
        .if_not_exists()
        .col(string_len(SupportTickets::Id, 36).primary_key())
        .col(string_len(SupportTickets::UserId, 36))
        .col(string_len(SupportTickets::Subject, 200))
        .col(string_len(SupportTickets::ContactEmail, 255))
        .col(string_len(SupportTickets::Status, 20))
        .col(string_len(SupportTickets::Priority, 20))
        .col(timestamp_tz(SupportTickets::LastMessageAt))
        .col(string_len(SupportTickets::LastMessageSenderRole, 20))
        .col(timestamp_tz_null(SupportTickets::LastUserActivityAt))
        .col(timestamp_tz_null(SupportTickets::LastAdminActivityAt))
        .col(timestamp_tz(SupportTickets::CreatedAt))
        .col(timestamp_tz(SupportTickets::UpdatedAt))
        .to_owned()
}

fn support_ticket_messages_table() -> TableCreateStatement {
    Table::create()
        .table(SupportTicketMessages::Table)
        .if_not_exists()
        .col(string_len(SupportTicketMessages::Id, 36).primary_key())
        .col(string_len(SupportTicketMessages::TicketId, 36))
        .col(string_len(SupportTicketMessages::SenderUserId, 36))
        .col(string_len(SupportTicketMessages::SenderRole, 20))
        .col(string_len(SupportTicketMessages::MessageKind, 20))
        .col(text(SupportTicketMessages::BodyMarkdown))
        .col(timestamp_tz(SupportTicketMessages::CreatedAt))
        .to_owned()
}

fn support_ticket_email_events_table() -> TableCreateStatement {
    Table::create()
        .table(SupportTicketEmailEvents::Table)
        .if_not_exists()
        .col(string_len(SupportTicketEmailEvents::Id, 36).primary_key())
        .col(string_len(SupportTicketEmailEvents::TicketId, 36))
        .col(string_len_null(SupportTicketEmailEvents::MessageId, 36))
        .col(string_len(SupportTicketEmailEvents::RecipientEmail, 255))
        .col(string_len(SupportTicketEmailEvents::Subject, 200))
        .col(string_len(SupportTicketEmailEvents::Status, 20))
        .col(text_null(SupportTicketEmailEvents::ErrorMessage))
        .col(timestamp_tz(SupportTicketEmailEvents::CreatedAt))
        .to_owned()
}

fn notification_states_table() -> TableCreateStatement {
    Table::create()
        .table(NotificationStates::Table)
        .if_not_exists()
        .col(string_len(NotificationStates::Id, 36).primary_key())
        .col(string_len(NotificationStates::UserId, 36))
        .col(string_len(NotificationStates::SourceType, 30))
        .col(string_len(NotificationStates::SourceId, 36))
        .col(timestamp_tz_null(NotificationStates::ReadAt))
        .col(timestamp_tz_null(NotificationStates::DeletedAt))
        .col(timestamp_tz(NotificationStates::CreatedAt))
        .col(timestamp_tz(NotificationStates::UpdatedAt))
        .to_owned()
}

fn timestamp_tz<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).timestamp_with_time_zone().not_null().take()
}

fn timestamp_tz_null<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).timestamp_with_time_zone().null().take()
}
