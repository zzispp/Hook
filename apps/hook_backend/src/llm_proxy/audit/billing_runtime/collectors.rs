use provider::application::billing::{CollectorSource, DimensionCollector, DimensionValueType};
use storage::{provider::ProviderStore, provider::record::DimensionCollectorRecord};

use crate::llm_proxy::{LlmProxyError, audit::AttemptRecordInput};

pub(super) async fn billing_collectors(
    store: &ProviderStore,
    input: &AttemptRecordInput<'_>,
    task_type: &str,
) -> Result<Vec<DimensionCollector>, LlmProxyError> {
    let mut collectors = builtin_collectors(&input.candidate.trace.provider_api_format, task_type);
    for record in store
        .enabled_dimension_collectors(&input.candidate.trace.provider_api_format, task_type)
        .await?
    {
        collectors.push(collector_from_record(record)?);
    }
    Ok(collectors)
}

fn collector_from_record(record: DimensionCollectorRecord) -> Result<DimensionCollector, LlmProxyError> {
    Ok(DimensionCollector {
        api_format: record.api_format,
        task_type: record.task_type,
        dimension_name: record.dimension_name,
        source_type: collector_source(&record.source_type)?,
        source_path: record.source_path,
        value_type: dimension_value_type(&record.value_type)?,
        transform_expression: record.transform_expression,
        default_value: record.default_value,
        priority: record.priority,
        is_enabled: record.is_enabled,
    })
}

fn builtin_collectors(api_format: &str, task_type: &str) -> Vec<DimensionCollector> {
    match api_format {
        "openai:chat" | "openai_completion" | "openai:cli" | "openai:compact" => openai_collectors(api_format, task_type),
        "claude:chat" => claude_collectors(api_format, task_type),
        "gemini:chat" | "gemini_embedding" => gemini_collectors(api_format, task_type),
        _ => Vec::new(),
    }
}

fn openai_collectors(api_format: &str, task_type: &str) -> Vec<DimensionCollector> {
    vec![
        response_int_collector(api_format, task_type, "input_tokens", "usage.prompt_tokens", 100),
        response_int_collector(api_format, task_type, "input_tokens", "usage.input_tokens", 90),
        response_int_collector(api_format, task_type, "output_tokens", "usage.completion_tokens", 100),
        response_int_collector(api_format, task_type, "output_tokens", "usage.output_tokens", 90),
        response_int_collector(api_format, task_type, "cache_read_tokens", "usage.prompt_tokens_details.cached_tokens", 100),
        response_int_collector(api_format, task_type, "cache_read_tokens", "usage.input_tokens_details.cached_tokens", 90),
        response_int_collector(
            api_format,
            task_type,
            "cache_creation_tokens",
            "usage.prompt_tokens_details.cache_creation_tokens",
            100,
        ),
        response_int_collector(
            api_format,
            task_type,
            "reasoning_tokens",
            "usage.completion_tokens_details.reasoning_tokens",
            100,
        ),
    ]
}

fn claude_collectors(api_format: &str, task_type: &str) -> Vec<DimensionCollector> {
    vec![
        response_int_collector(api_format, task_type, "input_tokens", "usage.input_tokens", 100),
        response_int_collector(api_format, task_type, "output_tokens", "usage.output_tokens", 100),
        response_int_collector(api_format, task_type, "cache_read_tokens", "usage.cache_read_input_tokens", 100),
        response_int_collector(api_format, task_type, "cache_creation_tokens", "usage.cache_creation_input_tokens", 100),
    ]
}

fn gemini_collectors(api_format: &str, task_type: &str) -> Vec<DimensionCollector> {
    vec![
        response_int_collector(api_format, task_type, "input_tokens", "usageMetadata.promptTokenCount", 100),
        response_int_collector(api_format, task_type, "output_tokens", "usageMetadata.candidatesTokenCount", 100),
        response_int_collector(api_format, task_type, "cache_read_tokens", "usageMetadata.cachedContentTokenCount", 100),
        response_int_collector(api_format, task_type, "reasoning_tokens", "usageMetadata.thoughtsTokenCount", 100),
    ]
}

fn response_int_collector(api_format: &str, task_type: &str, dimension_name: &str, source_path: &str, priority: i32) -> DimensionCollector {
    DimensionCollector {
        api_format: api_format.into(),
        task_type: task_type.into(),
        dimension_name: dimension_name.into(),
        source_type: CollectorSource::Response,
        source_path: Some(source_path.into()),
        value_type: DimensionValueType::Int,
        transform_expression: None,
        default_value: None,
        priority,
        is_enabled: true,
    }
}

fn collector_source(value: &str) -> Result<CollectorSource, LlmProxyError> {
    match value {
        "request" => Ok(CollectorSource::Request),
        "response" => Ok(CollectorSource::Response),
        "metadata" => Ok(CollectorSource::Metadata),
        "computed" => Ok(CollectorSource::Computed),
        _ => Err(LlmProxyError::Infrastructure(format!("unsupported dimension collector source_type: {value}"))),
    }
}

fn dimension_value_type(value: &str) -> Result<DimensionValueType, LlmProxyError> {
    match value {
        "float" => Ok(DimensionValueType::Float),
        "int" => Ok(DimensionValueType::Int),
        "string" => Ok(DimensionValueType::String),
        _ => Err(LlmProxyError::Infrastructure(format!("unsupported dimension collector value_type: {value}"))),
    }
}
