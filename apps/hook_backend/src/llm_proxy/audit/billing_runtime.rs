mod collectors;

use std::collections::BTreeMap;

use provider::application::billing::{
    BillingRule, BillingRuleLookup, BillingRuleScope, BillingService, BillingServiceInput, BillingSnapshotStatus, CostResult, RequestBillingAmount,
    effective_rule_task_type,
};
use serde_json::{Value, json};
use storage::{api_token::ApiTokenUsageRecord, model::GlobalModelUsageRecord, provider::ProviderStore, provider::record::BillingRuleRecord};
use time::OffsetDateTime;
use types::model::PatchField;

use super::{AttemptAuditInput, TokenUsage};
use crate::llm_proxy::{LlmProxyError, billing::WalletSettlementInput};

#[derive(Clone, Debug)]
pub(crate) struct BillingAttempt {
    pub(super) amount: RequestBillingAmount,
    pub(super) snapshot: Value,
    pub(super) status: BillingSnapshotStatus,
}

impl BillingAttempt {
    fn from_result(request_id: &str, result: CostResult) -> Result<Self, LlmProxyError> {
        let status = result.status.clone();
        let amount = RequestBillingAmount::from(result);
        let snapshot = serde_json::to_value(&amount.snapshot)
            .map_err(|error| LlmProxyError::Infrastructure(format!("billing snapshot serialization failed for {request_id}: {error}")))?;
        Ok(Self { amount, snapshot, status })
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.status == BillingSnapshotStatus::Complete
    }
}

pub(crate) async fn attempt_billing(store: &ProviderStore, request_id: &str, input: &AttemptAuditInput) -> Result<Option<BillingAttempt>, LlmProxyError> {
    if input.status != "success" {
        return Ok(None);
    }
    let task_type = effective_rule_task_type(&request_task_type(input));
    let result = BillingService::calculate_from_response(BillingServiceInput {
        task_type: task_type.clone(),
        model_name: input.candidate.trace.model_name_snapshot.clone(),
        global_model_id: input.candidate.trace.global_model_id.clone(),
        provider_model_id: input.candidate.trace.provider_model_id.clone(),
        provider_id: input.candidate.trace.provider_id.clone(),
        api_format: input.candidate.trace.provider_api_format.clone(),
        request: patch_value(&input.provider_request_body),
        response: patch_value(&input.provider_response_body),
        metadata: None,
        base_dimensions: input.usage.map(usage_dimensions).unwrap_or_default(),
        group_code: input.candidate.trace.group_code.clone(),
        billing_multiplier: input.candidate.billing_multiplier,
        price_per_request: input.candidate.price_per_request,
        tiered_pricing: input.candidate.tiered_pricing.clone(),
        explicit_rule: billing_rule_lookup(store, input, &task_type).await?,
        collectors: collectors::billing_collectors(store, input, &task_type).await?,
    });
    Ok(Some(BillingAttempt::from_result(request_id, result)?))
}

pub(crate) fn request_billing_status(input: &AttemptAuditInput, billing: Option<&BillingAttempt>) -> &'static str {
    match input.status.as_str() {
        "success" if billing.is_some_and(BillingAttempt::is_complete) => "settled",
        "success" => "billing_incomplete",
        status => billing_status(status),
    }
}

pub(crate) fn token_usage_record(
    request_id: &str,
    input: &AttemptAuditInput,
    billing: Option<&BillingAttempt>,
    used_at: OffsetDateTime,
) -> Result<Option<ApiTokenUsageRecord>, LlmProxyError> {
    if !should_record_successful_attempt(input) {
        return Ok(None);
    }
    let Some(token_id) = input.candidate.trace.token_id.clone() else {
        return Ok(None);
    };
    let amount = complete_amount(request_id, billing)?;
    Ok(Some(ApiTokenUsageRecord {
        cost: amount.total_cost,
        token_id,
        request_count: 1,
        used_at,
    }))
}

pub(crate) fn model_usage_record(input: &AttemptAuditInput, billing: Option<&BillingAttempt>) -> Option<GlobalModelUsageRecord> {
    if !should_record_successful_attempt(input) || !billing.is_some_and(BillingAttempt::is_complete) {
        return None;
    }
    Some(GlobalModelUsageRecord {
        count: 1,
        model_id: input.candidate.trace.global_model_id.clone(),
    })
}

pub(crate) fn wallet_settlement_input<'a>(
    request_id: &'a str,
    input: &'a AttemptAuditInput,
    billing: Option<&BillingAttempt>,
) -> Result<Option<WalletSettlementInput<'a>>, LlmProxyError> {
    if !should_record_successful_attempt(input) {
        return Ok(None);
    }
    let amount = complete_amount(request_id, billing)?.clone();
    Ok(Some(WalletSettlementInput {
        request_id,
        candidate: &input.candidate,
        amount,
    }))
}

pub(crate) fn total_tokens(usage: Option<TokenUsage>) -> Option<i64> {
    usage.and_then(|item| item.total_tokens.or_else(|| Some(item.prompt_tokens? + item.completion_tokens?)))
}

async fn billing_rule_lookup(store: &ProviderStore, input: &AttemptAuditInput, task_type: &str) -> Result<Option<BillingRuleLookup>, LlmProxyError> {
    if let Some(rule) = store
        .enabled_billing_rule_for_model(&input.candidate.trace.provider_model_id, task_type)
        .await?
    {
        return Ok(Some(rule_lookup(rule, BillingRuleScope::Model, task_type)?));
    }
    let rule = store
        .enabled_billing_rule_for_global_model(&input.candidate.trace.global_model_id, task_type)
        .await?;
    rule.map(|rule| rule_lookup(rule, BillingRuleScope::Global, task_type)).transpose()
}

fn rule_lookup(rule: BillingRuleRecord, scope: BillingRuleScope, task_type: &str) -> Result<BillingRuleLookup, LlmProxyError> {
    Ok(BillingRuleLookup {
        rule: BillingRule {
            id: rule.id,
            name: rule.name,
            task_type: rule.task_type,
            expression: rule.expression,
            variables: serde_json::from_str(&rule.variables).map_err(billing_config_error)?,
            dimension_mappings: serde_json::from_str(&rule.dimension_mappings).map_err(billing_config_error)?,
        },
        scope,
        effective_task_type: task_type.to_owned(),
    })
}

fn usage_dimensions(usage: TokenUsage) -> BTreeMap<String, Value> {
    let mut dimensions = BTreeMap::new();
    insert_i64(&mut dimensions, "input_tokens", usage.prompt_tokens);
    insert_i64(&mut dimensions, "output_tokens", usage.completion_tokens);
    insert_i64(&mut dimensions, "total_tokens", total_tokens(Some(usage)));
    insert_i64(&mut dimensions, "cache_creation_input_tokens", usage.cache_creation_input_tokens);
    insert_i64(&mut dimensions, "cache_read_input_tokens", usage.cache_read_input_tokens);
    insert_i64(&mut dimensions, "input_text_tokens", usage.input_text_tokens);
    insert_i64(&mut dimensions, "input_audio_tokens", usage.input_audio_tokens);
    insert_i64(&mut dimensions, "input_image_tokens", usage.input_image_tokens);
    insert_i64(&mut dimensions, "output_text_tokens", usage.output_text_tokens);
    insert_i64(&mut dimensions, "output_audio_tokens", usage.output_audio_tokens);
    insert_i64(&mut dimensions, "output_image_tokens", usage.output_image_tokens);
    insert_i64(&mut dimensions, "reasoning_tokens", usage.reasoning_tokens);
    insert_i64(&mut dimensions, "cache_creation_5m_input_tokens", usage.cache_creation_5m_input_tokens);
    insert_i64(&mut dimensions, "cache_creation_1h_input_tokens", usage.cache_creation_1h_input_tokens);
    dimensions
}

fn complete_amount<'a>(request_id: &str, billing: Option<&'a BillingAttempt>) -> Result<&'a RequestBillingAmount, LlmProxyError> {
    let Some(billing) = billing else {
        return Err(LlmProxyError::Infrastructure(format!(
            "successful request missing billing snapshot: {request_id}"
        )));
    };
    if billing.is_complete() {
        return Ok(&billing.amount);
    }
    Err(LlmProxyError::Infrastructure(format!(
        "successful request has incomplete billing snapshot: {request_id}"
    )))
}

fn should_record_successful_attempt(input: &AttemptAuditInput) -> bool {
    input.status == "success" && input.finished
}

fn billing_status(status: &str) -> &'static str {
    match status {
        "failed" | "cancelled" | "skipped" => "void",
        _ => "pending",
    }
}

fn request_task_type(input: &AttemptAuditInput) -> String {
    if input.candidate.trace.client_api_format == "openai:cli" {
        return "cli".into();
    }
    "chat".into()
}

fn patch_value(value: &PatchField<Value>) -> Option<Value> {
    match value {
        PatchField::Value(value) => Some(value.clone()),
        PatchField::Null | PatchField::Missing => None,
    }
}

fn insert_i64(dimensions: &mut BTreeMap<String, Value>, key: &str, value: Option<i64>) {
    if let Some(value) = value {
        dimensions.insert(key.into(), json!(value));
    }
}

fn billing_config_error(error: serde_json::Error) -> LlmProxyError {
    LlmProxyError::Infrastructure(format!("billing config json decode error: {error}"))
}
