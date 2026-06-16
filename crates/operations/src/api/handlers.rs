use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use rbac::api::CurrentUser;
use serde::Deserialize;
use types::{
    operations::{
        Announcement, AnnouncementInput, AnnouncementListFilters, AnnouncementPatch, NotificationItem, NotificationListFilters, SupportTicket,
        SupportTicketCreateInput, SupportTicketCreatePayload, SupportTicketDetail, SupportTicketListFilters, SupportTicketMessageInput,
        SupportTicketMessagePayload, SupportTicketMutationResponse, SupportTicketPatch,
    },
    pagination::{Page, PageRequest},
    response::ApiResponse,
};

use crate::{
    api::{OperationsApiError, OperationsApiState},
    application::is_admin_role,
};

type ApiJson<T> = Json<ApiResponse<T>>;
type ApiResult<T> = Result<T, OperationsApiError>;

#[derive(Debug, Deserialize)]
pub struct AnnouncementListQuery {
    pub page: u64,
    pub page_size: u64,
    pub search: Option<String>,
    pub announcement_type: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TicketListQuery {
    pub page: u64,
    pub page_size: u64,
    pub search: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationListQuery {
    pub page: u64,
    pub page_size: u64,
    pub status: Option<String>,
}

pub async fn list_announcements(State(state): State<OperationsApiState>, Query(query): Query<AnnouncementListQuery>) -> ApiResult<ApiJson<Page<Announcement>>> {
    Ok(ok(state.operations.list_announcements(query.page(), query.filters(), false).await?))
}

pub async fn get_announcement(State(state): State<OperationsApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<Announcement>> {
    Ok(ok(state.operations.get_announcement(&id, false).await?))
}

pub async fn admin_list_announcements(
    State(state): State<OperationsApiState>,
    Query(query): Query<AnnouncementListQuery>,
) -> ApiResult<ApiJson<Page<Announcement>>> {
    Ok(ok(state.operations.list_announcements(query.page(), query.filters(), true).await?))
}

pub async fn admin_get_announcement(State(state): State<OperationsApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<Announcement>> {
    Ok(ok(state.operations.get_announcement(&id, true).await?))
}

pub async fn admin_create_announcement(
    State(state): State<OperationsApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<AnnouncementInput>,
) -> ApiResult<ApiJson<Announcement>> {
    Ok(ok(state.operations.create_announcement(&current_user.id, payload).await?))
}

pub async fn admin_update_announcement(
    State(state): State<OperationsApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(payload): Json<AnnouncementPatch>,
) -> ApiResult<ApiJson<Announcement>> {
    Ok(ok(state.operations.update_announcement(&id, &current_user.id, payload).await?))
}

pub async fn admin_delete_announcement(State(state): State<OperationsApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.operations.delete_announcement(&id).await?;
    Ok(ok(()))
}

pub async fn create_ticket(
    State(state): State<OperationsApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<SupportTicketCreatePayload>,
) -> ApiResult<ApiJson<SupportTicketMutationResponse>> {
    Ok(ok(state.operations.create_ticket(create_input(current_user.id, payload)).await?))
}

pub async fn reply_ticket(
    State(state): State<OperationsApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(payload): Json<SupportTicketMessagePayload>,
) -> ApiResult<ApiJson<SupportTicketMutationResponse>> {
    Ok(ok(state
        .operations
        .user_reply_ticket(message_input(id, current_user.id, "user", payload))
        .await?))
}

pub async fn admin_reply_ticket(
    State(state): State<OperationsApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(payload): Json<SupportTicketMessagePayload>,
) -> ApiResult<ApiJson<SupportTicketMutationResponse>> {
    Ok(ok(state
        .operations
        .admin_reply_ticket(message_input(id, current_user.id, "admin", payload))
        .await?))
}

pub async fn update_ticket(
    State(state): State<OperationsApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(payload): Json<SupportTicketPatch>,
) -> ApiResult<ApiJson<SupportTicketMutationResponse>> {
    Ok(ok(state.operations.update_ticket(&id, &current_user.id, payload).await?))
}

pub async fn ticket_detail(
    State(state): State<OperationsApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> ApiResult<ApiJson<SupportTicketDetail>> {
    Ok(ok(state.operations.ticket_detail(&id, Some(&current_user.id)).await?))
}

pub async fn admin_ticket_detail(State(state): State<OperationsApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<SupportTicketDetail>> {
    Ok(ok(state.operations.ticket_detail(&id, None).await?))
}

pub async fn list_tickets(
    State(state): State<OperationsApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<TicketListQuery>,
) -> ApiResult<ApiJson<Page<SupportTicket>>> {
    Ok(ok(state.operations.list_tickets(Some(&current_user.id), query.page(), query.filters()).await?))
}

pub async fn admin_list_tickets(State(state): State<OperationsApiState>, Query(query): Query<TicketListQuery>) -> ApiResult<ApiJson<Page<SupportTicket>>> {
    Ok(ok(state.operations.list_tickets(None, query.page(), query.filters()).await?))
}

pub async fn list_notifications(
    State(state): State<OperationsApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<NotificationListQuery>,
) -> ApiResult<ApiJson<Page<NotificationItem>>> {
    let admin = is_admin_role(&current_user.role);
    Ok(ok(state
        .operations
        .list_notifications(&current_user.id, admin, query.page(), query.filters())
        .await?))
}

pub async fn mark_all_notifications_read(State(state): State<OperationsApiState>, Extension(current_user): Extension<CurrentUser>) -> ApiResult<ApiJson<()>> {
    state
        .operations
        .mark_all_notifications_read(&current_user.id, is_admin_role(&current_user.role))
        .await?;
    Ok(ok(()))
}

pub async fn mark_notification_read(
    State(state): State<OperationsApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((source_type, source_id)): Path<(String, String)>,
) -> ApiResult<ApiJson<()>> {
    state.operations.mark_notification_read(&current_user.id, &source_type, &source_id).await?;
    Ok(ok(()))
}

pub async fn delete_notification(
    State(state): State<OperationsApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((source_type, source_id)): Path<(String, String)>,
) -> ApiResult<ApiJson<()>> {
    state.operations.delete_notification(&current_user.id, &source_type, &source_id).await?;
    Ok(ok(()))
}

pub async fn delete_read_notifications(State(state): State<OperationsApiState>, Extension(current_user): Extension<CurrentUser>) -> ApiResult<ApiJson<()>> {
    state
        .operations
        .delete_read_notifications(&current_user.id, is_admin_role(&current_user.role))
        .await?;
    Ok(ok(()))
}

impl AnnouncementListQuery {
    fn page(&self) -> PageRequest {
        PageRequest {
            page: self.page,
            page_size: self.page_size,
        }
    }

    fn filters(self) -> AnnouncementListFilters {
        AnnouncementListFilters {
            search: self.search,
            announcement_type: self.announcement_type,
            enabled: self.enabled,
        }
    }
}

impl TicketListQuery {
    fn page(&self) -> PageRequest {
        PageRequest {
            page: self.page,
            page_size: self.page_size,
        }
    }

    fn filters(self) -> SupportTicketListFilters {
        SupportTicketListFilters {
            search: self.search,
            status: self.status,
            priority: self.priority,
        }
    }
}

impl NotificationListQuery {
    fn page(&self) -> PageRequest {
        PageRequest {
            page: self.page,
            page_size: self.page_size,
        }
    }

    fn filters(self) -> NotificationListFilters {
        NotificationListFilters { status: self.status }
    }
}

fn create_input(user_id: String, payload: SupportTicketCreatePayload) -> SupportTicketCreateInput {
    SupportTicketCreateInput {
        user_id,
        subject: payload.subject,
        body_markdown: payload.body_markdown,
        contact_email: payload.contact_email,
        captcha_token: payload.captcha_token,
    }
}

fn message_input(ticket_id: String, sender_user_id: String, sender_role: &str, payload: SupportTicketMessagePayload) -> SupportTicketMessageInput {
    SupportTicketMessageInput {
        ticket_id,
        sender_user_id,
        sender_role: sender_role.into(),
        body_markdown: payload.body_markdown,
    }
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(ApiResponse::new(data))
}
