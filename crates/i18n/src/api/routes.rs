use axum::{
    Router,
    routing::{get, patch, put},
};

use crate::api::{
    I18nApiState,
    handlers::{
        create_entry, create_language, delete_entry, delete_language, list_entries, list_languages, resource_bundle, update_entry, update_language,
        upsert_bundle,
    },
};

pub fn create_router(state: I18nApiState) -> Router {
    Router::new()
        .route("/i18n/resources", get(resource_bundle))
        .route("/admin/i18n/languages", get(list_languages).post(create_language))
        .route("/admin/i18n/languages/{code}", patch(update_language).delete(delete_language))
        .route("/admin/i18n/translations", get(list_entries).post(create_entry))
        .route("/admin/i18n/translations/{id}", patch(update_entry).delete(delete_entry))
        .route("/admin/i18n/translations/{namespace}/{group_key}/{item_key}", put(upsert_bundle))
        .with_state(state)
}
