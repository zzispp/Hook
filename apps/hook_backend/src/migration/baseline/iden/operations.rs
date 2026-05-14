use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum Announcements {
    Table,
    Id,
    Title,
    ContentMarkdown,
    AnnouncementType,
    Pinned,
    Priority,
    Enabled,
    CreatedBy,
    UpdatedBy,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum SupportTickets {
    Table,
    Id,
    UserId,
    Subject,
    ContactEmail,
    Status,
    Priority,
    LastMessageAt,
    LastMessageSenderRole,
    LastUserActivityAt,
    LastAdminActivityAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum SupportTicketMessages {
    Table,
    Id,
    TicketId,
    SenderUserId,
    SenderRole,
    MessageKind,
    BodyMarkdown,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum SupportTicketEmailEvents {
    Table,
    Id,
    TicketId,
    MessageId,
    RecipientEmail,
    Subject,
    Status,
    ErrorMessage,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum NotificationStates {
    Table,
    Id,
    UserId,
    SourceType,
    SourceId,
    ReadAt,
    DeletedAt,
    CreatedAt,
    UpdatedAt,
}
