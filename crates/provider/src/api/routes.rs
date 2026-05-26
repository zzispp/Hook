use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::api::{
    ProviderApiState,
    handlers::{
        create_api_key, create_endpoint, create_model_binding, create_provider, delete_api_key, delete_endpoint, delete_model_binding, delete_provider,
        delete_model_cost, fetch_upstream_models, get_provider, get_request_record, list_active_request_records, list_api_keys, list_endpoints,
        list_model_bindings, list_model_costs, list_provider_cooldowns, list_providers, list_request_records, list_usage_records, release_provider_cooldown,
        test_model_binding, update_api_key, update_endpoint, update_model_binding, update_provider, upsert_model_costs,
    },
};

pub fn create_router(state: ProviderApiState) -> Router {
    Router::new()
        .route("/admin/providers", get(list_providers).post(create_provider))
        .route("/admin/providers/{id}", get(get_provider).patch(update_provider).delete(delete_provider))
        .route("/admin/providers/{provider_id}/endpoints", get(list_endpoints).post(create_endpoint))
        .route(
            "/admin/providers/{provider_id}/endpoints/{endpoint_id}",
            patch(update_endpoint).delete(delete_endpoint),
        )
        .route("/admin/providers/{provider_id}/keys", get(list_api_keys).post(create_api_key))
        .route("/admin/providers/{provider_id}/keys/{key_id}", patch(update_api_key).delete(delete_api_key))
        .route("/admin/providers/{provider_id}/upstream-models", get(fetch_upstream_models))
        .route("/admin/providers/{provider_id}/models", get(list_model_bindings).post(create_model_binding))
        .route(
            "/admin/providers/{provider_id}/models/{model_id}",
            patch(update_model_binding).delete(delete_model_binding),
        )
        .route("/admin/providers/{provider_id}/models/{model_id}/test", post(test_model_binding))
        .route("/admin/providers/{provider_id}/model-costs", get(list_model_costs))
        .route("/admin/providers/{provider_id}/keys/{key_id}/model-costs", axum::routing::put(upsert_model_costs))
        .route(
            "/admin/providers/{provider_id}/keys/{key_id}/model-costs/{provider_model_id}",
            axum::routing::delete(delete_model_cost),
        )
        .route("/admin/provider-cooldowns", get(list_provider_cooldowns))
        .route("/admin/provider-cooldowns/{provider_id}/release", post(release_provider_cooldown))
        .route("/request-records", get(list_usage_records))
        .route("/admin/request-records", get(list_request_records))
        .route("/admin/request-records/active", post(list_active_request_records))
        .route("/admin/request-records/{request_id}", get(get_request_record))
        .with_state(state)
}
