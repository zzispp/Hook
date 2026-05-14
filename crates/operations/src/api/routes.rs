use axum::{
    Router,
    routing::{get, patch},
};

use crate::api::{
    OperationsApiState,
    handlers::{
        admin_create_announcement, admin_delete_announcement, admin_get_announcement, admin_list_announcements, admin_list_tickets, admin_reply_ticket,
        admin_ticket_detail, admin_update_announcement, create_ticket, delete_notification, get_announcement, list_announcements, list_notifications,
        list_tickets, mark_all_notifications_read, mark_notification_read, reply_ticket, ticket_detail, update_ticket,
    },
};

pub fn create_router(state: OperationsApiState) -> Router {
    Router::new()
        .route("/announcements", get(list_announcements))
        .route("/announcements/{id}", get(get_announcement))
        .route("/admin/announcements", get(admin_list_announcements).post(admin_create_announcement))
        .route(
            "/admin/announcements/{id}",
            get(admin_get_announcement).patch(admin_update_announcement).delete(admin_delete_announcement),
        )
        .route("/tickets", get(list_tickets).post(create_ticket))
        .route("/tickets/{id}", get(ticket_detail))
        .route("/tickets/{id}/messages", patch(reply_ticket))
        .route("/admin/tickets", get(admin_list_tickets))
        .route("/admin/tickets/{id}", get(admin_ticket_detail).patch(update_ticket))
        .route("/admin/tickets/{id}/messages", patch(admin_reply_ticket))
        .route("/notifications", get(list_notifications))
        .route("/notifications/read-all", patch(mark_all_notifications_read))
        .route("/notifications/{source_type}/{source_id}/read", patch(mark_notification_read))
        .route("/notifications/{source_type}/{source_id}", axum::routing::delete(delete_notification))
        .with_state(state)
}
