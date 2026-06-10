use sea_orm::{DatabaseBackend, MockDatabase};
use storage::{
    Database,
    model::provider_models,
    provider::{
        ProviderStore,
        record::{provider_endpoints, provider_group_providers, provider_groups, providers},
    },
};
use types::provider::ProviderListRequest;

#[tokio::test]
async fn provider_list_filters_by_status_search_format_and_model() {
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([provider_records()])
            .append_query_results([endpoint_records()])
            .append_query_results([provider_model_records()])
            .append_query_results([Vec::<provider_groups::Model>::new()])
            .append_query_results([Vec::<provider_group_providers::Model>::new()])
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
async fn provider_list_paginates_after_group_and_priority_sorting() {
    let database = Database::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([grouped_provider_records()])
            .append_query_results([provider_group_records()])
            .append_query_results([provider_group_member_records()])
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
    assert_eq!(ids, ["group-one-fast", "group-one-slow", "group-two-fast"]);
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
        max_retries: Some(2),
        request_timeout_seconds: Some(300.0),
        stream_first_byte_timeout_seconds: Some(60.0),
        stream_idle_timeout_seconds: Some(30.0),
        priority,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active,
        created_at: now(),
        updated_at: now(),
    }
}

fn provider_group_records() -> Vec<provider_groups::Model> {
    vec![
        provider_group_record("group-two", "Group Two", 2),
        provider_group_record("group-one", "Group One", 1),
    ]
}

fn provider_group_record(id: &str, name: &str, sort_order: i64) -> provider_groups::Model {
    provider_groups::Model {
        id: id.into(),
        name: name.into(),
        description: None,
        sort_order,
        created_at: now(),
        updated_at: now(),
    }
}

fn provider_group_member_records() -> Vec<provider_group_providers::Model> {
    vec![
        provider_group_member_record("group-two", "group-two-fast", 1),
        provider_group_member_record("group-two", "group-two-slow", 9),
        provider_group_member_record("group-one", "group-one-fast", 2),
        provider_group_member_record("group-one", "group-one-slow", 5),
    ]
}

fn provider_group_member_record(group_id: &str, provider_id: &str, priority: i32) -> provider_group_providers::Model {
    provider_group_providers::Model {
        id: format!("{group_id}-{provider_id}"),
        provider_group_id: group_id.into(),
        provider_id: provider_id.into(),
        priority,
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
        provider_model_name: model_id.into(),
        provider_model_mappings: None,
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
