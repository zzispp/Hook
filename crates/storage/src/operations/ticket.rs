use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, ExprTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
    sea_query::{Expr, Order},
};
use types::{
    operations::{SupportTicket, SupportTicketDetail, SupportTicketEmailEvent, SupportTicketListFilters, SupportTicketMessage},
    pagination::{Page, PageSliceRequest},
};

use crate::{StorageError, StorageResult, user::UserEntity};

use super::{
    EmailEventRecordInput, OperationsStore, TicketActiveModel, TicketColumn, TicketEmailEventActiveModel, TicketEmailEventColumn, TicketEmailEventEntity,
    TicketEntity, TicketMessageActiveModel, TicketMessageColumn, TicketMessageEntity, TicketMessageRecordInput, TicketRecord, TicketRecordInput,
    TicketRecordPatch,
};

const MESSAGE_KIND_MESSAGE: &str = "message";
const PRIORITY_NORMAL: &str = "normal";
const ROLE_ADMIN: &str = "admin";
const ROLE_USER: &str = "user";
const STATUS_IN_PROGRESS: &str = "in_progress";
const STATUS_OPEN: &str = "open";
const STATUS_WAITING_USER: &str = "waiting_user";

impl OperationsStore {
    pub async fn create_ticket(&self, input: TicketRecordInput) -> StorageResult<(SupportTicket, SupportTicketMessage)> {
        let tx = self.connection().begin().await?;
        let now = time::OffsetDateTime::now_utc();
        let ticket = insert_ticket(&tx, self.next_id(), &input, now).await?;
        let message = insert_message(&tx, self.next_id(), first_message_input(&ticket.id, &input), now).await?;
        tx.commit().await?;
        Ok((ticket.into(), message.into()))
    }

    pub async fn add_ticket_message(&self, input: TicketMessageRecordInput) -> StorageResult<(SupportTicket, SupportTicketMessage)> {
        let tx = self.connection().begin().await?;
        let now = time::OffsetDateTime::now_utc();
        let record = TicketEntity::find_by_id(input.ticket_id.clone())
            .one(&tx)
            .await?
            .ok_or(StorageError::NotFound)?;
        let mut active: TicketActiveModel = record.into();
        apply_message_activity(&mut active, &input.sender_role, now);
        let ticket = active.update(&tx).await?;
        let message = insert_message(&tx, self.next_id(), input, now).await?;
        tx.commit().await?;
        Ok((ticket.into(), message.into()))
    }

    pub async fn update_ticket(&self, id: &str, input: TicketRecordPatch) -> StorageResult<SupportTicket> {
        let record = self.ticket_record(id).await?.ok_or(StorageError::NotFound)?;
        let mut active: TicketActiveModel = record.into();
        apply_ticket_patch(&mut active, input);
        let record = active.update(self.connection()).await?;
        Ok(record.into())
    }

    pub async fn ticket_detail(&self, id: &str) -> StorageResult<Option<SupportTicketDetail>> {
        let Some(ticket) = self.ticket_record(id).await? else {
            return Ok(None);
        };
        let messages = self.ticket_messages(id).await?;
        let email_events = self.ticket_email_events(id).await?;
        Ok(Some(SupportTicketDetail {
            ticket: ticket.into(),
            messages,
            email_events,
        }))
    }

    pub async fn page_tickets(
        &self,
        user_id: Option<&str>,
        request: PageSliceRequest,
        filters: SupportTicketListFilters,
    ) -> StorageResult<Page<SupportTicket>> {
        let query = filter_tickets(TicketEntity::find(), user_id, filters);
        let total = query.clone().count(self.connection()).await?;
        let items = ordered_tickets(query, user_id.is_some())
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

    pub async fn record_email_event(&self, input: EmailEventRecordInput) -> StorageResult<SupportTicketEmailEvent> {
        let record = TicketEmailEventActiveModel {
            id: Set(self.next_id()),
            ticket_id: Set(input.ticket_id),
            message_id: Set(input.message_id),
            recipient_email: Set(input.recipient_email),
            subject: Set(input.subject),
            status: Set(input.status),
            error_message: Set(input.error_message),
            created_at: Set(time::OffsetDateTime::now_utc()),
        }
        .insert(self.connection())
        .await?;
        Ok(record.into())
    }

    pub async fn user_email(&self, user_id: &str) -> StorageResult<Option<String>> {
        UserEntity::find_by_id(user_id.to_owned())
            .one(self.connection())
            .await
            .map(|record| record.map(|user| user.email))
            .map_err(Into::into)
    }

    async fn ticket_record(&self, id: &str) -> StorageResult<Option<TicketRecord>> {
        TicketEntity::find_by_id(id.to_owned()).one(self.connection()).await.map_err(Into::into)
    }

    async fn ticket_messages(&self, ticket_id: &str) -> StorageResult<Vec<SupportTicketMessage>> {
        let messages = TicketMessageEntity::find()
            .filter(TicketMessageColumn::TicketId.eq(ticket_id))
            .order_by_asc(TicketMessageColumn::CreatedAt)
            .all(self.connection())
            .await?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        Ok(messages)
    }

    async fn ticket_email_events(&self, ticket_id: &str) -> StorageResult<Vec<SupportTicketEmailEvent>> {
        let events = TicketEmailEventEntity::find()
            .filter(TicketEmailEventColumn::TicketId.eq(ticket_id))
            .order_by_desc(TicketEmailEventColumn::CreatedAt)
            .all(self.connection())
            .await?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        Ok(events)
    }
}

fn filter_tickets(mut query: sea_orm::Select<TicketEntity>, user_id: Option<&str>, filters: SupportTicketListFilters) -> sea_orm::Select<TicketEntity> {
    if let Some(value) = user_id {
        query = query.filter(TicketColumn::UserId.eq(value));
    }
    if let Some(search) = filters.search.filter(|value| !value.is_empty()) {
        query = query.filter(
            Condition::any()
                .add(TicketColumn::Subject.contains(search.clone()))
                .add(TicketColumn::ContactEmail.contains(search)),
        );
    }
    if let Some(value) = filters.status.filter(|value| !value.is_empty()) {
        query = query.filter(TicketColumn::Status.eq(value));
    }
    if let Some(value) = filters.priority.filter(|value| !value.is_empty()) {
        query = query.filter(TicketColumn::Priority.eq(value));
    }
    query
}

fn ordered_tickets(query: sea_orm::Select<TicketEntity>, user_scope: bool) -> sea_orm::Select<TicketEntity> {
    query
        .order_by(ticket_attention_rank(user_scope), Order::Asc)
        .order_by_desc(TicketColumn::LastMessageAt)
}

fn ticket_attention_rank(user_scope: bool) -> Expr {
    Expr::case(unread_ticket_condition(user_scope), 0)
        .case(unfinished_ticket_condition(), 1)
        .finally(2)
        .into()
}

fn unread_ticket_condition(user_scope: bool) -> Condition {
    if user_scope {
        return unread_after_counterparty_activity(TicketColumn::LastAdminActivityAt, TicketColumn::LastUserActivityAt);
    }
    unread_after_counterparty_activity(TicketColumn::LastUserActivityAt, TicketColumn::LastAdminActivityAt)
}

fn unread_after_counterparty_activity(counterparty: TicketColumn, own: TicketColumn) -> Condition {
    Condition::all()
        .add(Expr::col(counterparty).is_not_null())
        .add(Condition::any().add(Expr::col(own).is_null()).add(Expr::col(counterparty).gt(Expr::col(own))))
}

fn unfinished_ticket_condition() -> Expr {
    Expr::col(TicketColumn::Status).is_in([STATUS_OPEN, STATUS_IN_PROGRESS, STATUS_WAITING_USER])
}

async fn insert_ticket<C>(connection: &C, id: String, input: &TicketRecordInput, now: time::OffsetDateTime) -> StorageResult<TicketRecord>
where
    C: sea_orm::ConnectionTrait,
{
    TicketActiveModel {
        id: Set(id),
        user_id: Set(input.user_id.clone()),
        subject: Set(input.subject.clone()),
        contact_email: Set(input.contact_email.clone()),
        status: Set(STATUS_OPEN.into()),
        priority: Set(PRIORITY_NORMAL.into()),
        last_message_at: Set(now),
        last_message_sender_role: Set(ROLE_USER.into()),
        last_user_activity_at: Set(Some(now)),
        last_admin_activity_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(connection)
    .await
    .map_err(Into::into)
}

async fn insert_message<C>(connection: &C, id: String, input: TicketMessageRecordInput, now: time::OffsetDateTime) -> StorageResult<super::TicketMessageRecord>
where
    C: sea_orm::ConnectionTrait,
{
    TicketMessageActiveModel {
        id: Set(id),
        ticket_id: Set(input.ticket_id),
        sender_user_id: Set(input.sender_user_id),
        sender_role: Set(input.sender_role),
        message_kind: Set(MESSAGE_KIND_MESSAGE.into()),
        body_markdown: Set(input.body_markdown),
        created_at: Set(now),
    }
    .insert(connection)
    .await
    .map_err(Into::into)
}

fn first_message_input(ticket_id: &str, input: &TicketRecordInput) -> TicketMessageRecordInput {
    TicketMessageRecordInput {
        ticket_id: ticket_id.to_owned(),
        sender_user_id: input.user_id.clone(),
        sender_role: ROLE_USER.into(),
        body_markdown: input.body_markdown.clone(),
    }
}

fn apply_message_activity(active: &mut TicketActiveModel, sender_role: &str, now: time::OffsetDateTime) {
    active.last_message_at = Set(now);
    active.last_message_sender_role = Set(sender_role.to_owned());
    active.updated_at = Set(now);
    if sender_role == ROLE_ADMIN {
        active.last_admin_activity_at = Set(Some(now));
    } else {
        active.status = Set(STATUS_OPEN.into());
        active.last_user_activity_at = Set(Some(now));
    }
}

fn apply_ticket_patch(active: &mut TicketActiveModel, input: TicketRecordPatch) {
    let now = time::OffsetDateTime::now_utc();
    if let Some(value) = input.status {
        active.status = Set(value);
        active.last_admin_activity_at = Set(Some(now));
    }
    if let Some(value) = input.priority {
        active.priority = Set(value);
        active.last_admin_activity_at = Set(Some(now));
    }
    active.updated_at = Set(now);
}
