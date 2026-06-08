#![allow(dead_code)]

use proxy::{
    format_conversion::ApiFormat,
    scheduler::{EndpointSnapshot, KeySnapshot, ModelAccessPolicy, ModelBindingSnapshot, PriorityMode, ProviderSnapshot, SchedulerInput, SchedulingMode},
};

pub fn base_input() -> SchedulerInput {
    SchedulerInput {
        group_code: "default".into(),
        group_is_active: true,
        group_allowed_model_ids: Vec::new(),
        group_allowed_provider_key_ids: None,
        user_allowed_model_ids: Vec::new(),
        user_allowed_provider_ids: Vec::new(),
        token_model_policy: ModelAccessPolicy::All,
        requested_model_id: "gpt-4o-mini".into(),
        client_format: ApiFormat::OpenAiChat,
        is_stream: false,
        affinity: None,
        load_balance_seed: None,
        scheduling_mode: SchedulingMode::FixedOrder,
        priority_mode: PriorityMode::Provider,
        global_keep_priority_on_conversion: false,
        global_format_conversion_enabled: true,
        providers: Vec::new(),
    }
}

pub fn provider_a() -> ProviderSnapshot {
    ProviderSnapshot {
        id: "provider-a".into(),
        name: "Provider A".into(),
        priority: 10,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
        endpoints: vec![endpoint("endpoint-a-openai", ApiFormat::OpenAiChat)],
        keys: vec![key("key-a-1", 10)],
        models: vec![model("gpt-4o-mini", "upstream-gpt-4o-mini")],
    }
}

pub fn provider_b() -> ProviderSnapshot {
    ProviderSnapshot {
        id: "provider-b".into(),
        name: "Provider B".into(),
        priority: 1,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
        endpoints: vec![endpoint("endpoint-b-openai", ApiFormat::OpenAiChat)],
        keys: vec![key("key-b-1", 10)],
        models: vec![model("other-model", "other-model")],
    }
}

pub fn provider_with_gemini_low_priority() -> ProviderSnapshot {
    ProviderSnapshot {
        id: "provider-gemini".into(),
        name: "Gemini Provider".into(),
        priority: 1,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
        endpoints: vec![endpoint("endpoint-gemini", ApiFormat::GeminiChat)],
        keys: vec![key_for_format("key-gemini", 10, ApiFormat::GeminiChat)],
        models: vec![model("gpt-4o-mini", "gemini-upstream")],
    }
}

pub fn provider_with_two_keys() -> ProviderSnapshot {
    ProviderSnapshot {
        keys: vec![key("key-a-1", 10), key("key-a-2", 10)],
        ..provider_a()
    }
}

pub fn provider_with_priority(provider_id: &str, provider_priority: i32, key_id: &str, key_priority: i32) -> ProviderSnapshot {
    ProviderSnapshot {
        id: provider_id.into(),
        name: provider_id.into(),
        priority: provider_priority,
        keep_priority_on_conversion: false,
        enable_format_conversion: true,
        is_active: true,
        endpoints: vec![endpoint(&format!("endpoint-{provider_id}"), ApiFormat::OpenAiChat)],
        keys: vec![key(key_id, key_priority)],
        models: vec![model("gpt-4o-mini", &format!("upstream-{provider_id}"))],
    }
}

fn endpoint(id: &str, api_format: ApiFormat) -> EndpointSnapshot {
    EndpointSnapshot {
        id: id.into(),
        api_format,
        is_active: true,
        accepts_format_conversion: true,
        supports_stream_conversion: true,
    }
}

fn key(id: &str, internal_priority: i32) -> KeySnapshot {
    key_for_format(id, internal_priority, ApiFormat::OpenAiChat)
}

fn key_for_format(id: &str, internal_priority: i32, api_format: ApiFormat) -> KeySnapshot {
    KeySnapshot {
        id: id.into(),
        api_formats: vec![api_format],
        internal_priority,
        global_priority_by_format: std::collections::BTreeMap::from([(api_format, internal_priority)]),
        cache_ttl_minutes: 5,
        is_active: true,
    }
}

fn model(global_model_id: &str, provider_model_name: &str) -> ModelBindingSnapshot {
    ModelBindingSnapshot {
        global_model_id: global_model_id.into(),
        provider_model_name: provider_model_name.into(),
    }
}
