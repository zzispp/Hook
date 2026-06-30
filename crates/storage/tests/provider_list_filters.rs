use sea_orm::{DatabaseBackend, MockDatabase};
use storage::{
    Database,
    model::provider_models,
    provider::{
        ProviderStore,
        record::{provider_endpoints, providers},
    },
};
use types::provider::ProviderListRequest;

#[tokio::test]
async fn provider_list_filters_by_status_search_format_and_model() {
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([provider_records()])
            .append_query_results([Vec::<storage::provider::record::provider_quick_import_sources::Model>::new()])
            .append_query_results([endpoint_records()])
            .append_query_results([provider_model_records()])
            .into_connection(),
    );
    let store = ProviderStore::new(database);

    let response = store
        .list_providers(ProviderListRequest {
            is_active: Some(true),
            search: Some("alpha".into()),
            api_format: Some("openai:chat".into()),
            model_id: Some("gpt-5".into()),
            limit: 100,
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(response.total, 1);
    assert_eq!(response.providers[0].id, "provider-alpha");
}

#[tokio::test]
async fn provider_list_paginates_after_priority_sorting() {
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([grouped_provider_records()])
            .append_query_results([Vec::<storage::provider::record::provider_quick_import_sources::Model>::new()])
            .into_connection(),
    );
    let store = ProviderStore::new(database);

    let response = store
        .list_providers(ProviderListRequest {
            limit: 3,
            ..Default::default()
        })
        .await
        .unwrap();

    let ids = response.providers.iter().map(|provider| provider.id.as_str()).collect::<Vec<_>>();
    assert_eq!(response.total, 5);
    assert_eq!(ids, ["group-two-fast", "unbound-fast", "group-one-fast"]);
}

fn provider_records() -> Vec<providers::Model> {
    vec![
        provider_record("provider-alpha", "Alpha Paid", true, 1),
        provider_record("provider-beta", "Beta Paid", true, 2),
        provider_record("provider-disabled", "Alpha Disabled", false, 3),
    ]
}

fn grouped_provider_records() -> Vec<providers::Model> {
    vec![
        provider_record("group-two-fast", "Group Two Fast", true, 1),
        provider_record("unbound-fast", "Unbound Fast", true, 1),
        provider_record("group-one-fast", "Group One Fast", true, 2),
        provider_record("group-one-slow", "Group One Slow", true, 5),
        provider_record("group-two-slow", "Group Two Slow", true, 9),
    ]
}

fn provider_record(id: &str, name: &str, is_active: bool, priority: i32) -> providers::Model {
    providers::Model {
        id: id.into(),
        name: name.into(),
        provider_type: "custom".into(),
        provider_origin: "manual".into(),
        max_retries: Some(2),
        request_timeout_seconds: Some(300.0),
        stream_response_headers_timeout_seconds: Some(60.0),
        stream_first_byte_timeout_seconds: Some(60.0),
        stream_first_token_timeout_seconds: Some(45.0),
        stream_idle_timeout_seconds: Some(30.0),
        priority,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active,
        created_at: now(),
        updated_at: now(),
    }
}

fn endpoint_records() -> Vec<provider_endpoints::Model> {
    vec![
        endpoint_record("endpoint-alpha", "provider-alpha", "openai:chat"),
        endpoint_record("endpoint-beta", "provider-beta", "gemini:chat"),
        endpoint_record("endpoint-disabled", "provider-disabled", "openai:chat"),
    ]
}

fn endpoint_record(id: &str, provider_id: &str, api_format: &str) -> provider_endpoints::Model {
    provider_endpoints::Model {
        id: id.into(),
        provider_id: provider_id.into(),
        api_format: api_format.into(),
        base_url: "https://example.test".into(),
        custom_path: None,
        max_retries: None,
        is_active: true,
        format_acceptance_config: None,
        header_rules: None,
        body_rules: None,
        created_at: now(),
        updated_at: now(),
    }
}

fn provider_model_records() -> Vec<provider_models::Model> {
    vec![
        provider_model_record("model-alpha", "provider-alpha", "gpt-5"),
        provider_model_record("model-beta", "provider-beta", "gpt-5"),
        provider_model_record("model-disabled", "provider-disabled", "gpt-5"),
        provider_model_record("model-inactive", "provider-alpha", "claude"),
    ]
}

fn provider_model_record(id: &str, provider_id: &str, model_id: &str) -> provider_models::Model {
    provider_models::Model {
        id: id.into(),
        provider_id: provider_id.into(),
        global_model_id: model_id.into(),
        is_active: true,
        config: None,
        created_at: now(),
        updated_at: now(),
    }
}

fn now() -> time::OffsetDateTime {
    time::Date::from_calendar_date(2026, time::Month::May, 11)
        .unwrap()
        .with_hms(12, 0, 0)
        .unwrap()
        .assume_utc()
}
