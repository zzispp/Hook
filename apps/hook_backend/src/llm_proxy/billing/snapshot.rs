use serde_json::{Value, json};
use types::{
    api_token::{ApiToken, ApiTokenType},
    wallet::Wallet,
};

use super::{BalanceChange, REASON_LLM_MODEL_USAGE, WalletSettlementInput};
use crate::llm_proxy::{LlmProxyError, cache::snapshot::CachedUserAccess, candidate::ProxyCandidate};

pub(super) struct DescriptionInput<'a, 'b> {
    pub(super) input: &'a WalletSettlementInput<'b>,
    pub(super) token: &'a ApiToken,
    pub(super) user: &'a CachedUserAccess,
    pub(super) wallet: &'a Wallet,
    pub(super) change: &'a BalanceChange,
}

pub(super) fn settlement_description(input: DescriptionInput<'_, '_>) -> Result<String, LlmProxyError> {
    serde_json::to_string(&json!({
        "kind": REASON_LLM_MODEL_USAGE,
        "request": request_snapshot(input.input),
        "user": user_snapshot(input.user),
        "token": token_snapshot(input.token),
        "group": group_snapshot(input.input.candidate),
        "model": model_snapshot(input.input.candidate),
        "provider": provider_snapshot(input.input.candidate),
        "endpoint": endpoint_snapshot(input.input.candidate),
        "key": key_snapshot(input.input.candidate),
        "usage": usage_snapshot(input.input.usage),
        "pricing": pricing_snapshot(input.input.candidate),
        "amounts": amount_snapshot(input),
    }))
    .map_err(|error| LlmProxyError::Infrastructure(error.to_string()))
}

fn request_snapshot(input: &WalletSettlementInput<'_>) -> Value {
    json!({
        "request_id": input.request_id,
        "client_api_format": input.candidate.trace.client_api_format,
        "provider_api_format": input.candidate.trace.provider_api_format,
        "is_stream": input.candidate.trace.is_stream,
    })
}

fn user_snapshot(user: &CachedUserAccess) -> Value {
    json!({ "id": user.id, "username": user.username, "quota_mode": user.quota_mode })
}

fn token_snapshot(token: &ApiToken) -> Value {
    json!({ "id": token.id, "name": token.name, "prefix": token.token_prefix, "type": token_type(token.token_type) })
}

fn group_snapshot(candidate: &ProxyCandidate) -> Value {
    json!({ "code": candidate.trace.group_code, "billing_multiplier": candidate.billing_multiplier.to_string() })
}

fn model_snapshot(candidate: &ProxyCandidate) -> Value {
    json!({
        "global_model_id": candidate.trace.global_model_id,
        "model_name": candidate.trace.model_name_snapshot,
        "requested_model_name": candidate.requested_model_name,
        "provider_model_name": candidate.provider_model_name,
    })
}

fn provider_snapshot(candidate: &ProxyCandidate) -> Value {
    json!({ "id": candidate.trace.provider_id, "name": candidate.trace.provider_name_snapshot })
}

fn endpoint_snapshot(candidate: &ProxyCandidate) -> Value {
    json!({ "id": candidate.trace.endpoint_id, "name": candidate.trace.endpoint_name_snapshot })
}

fn key_snapshot(candidate: &ProxyCandidate) -> Value {
    json!({
        "id": candidate.trace.key_id,
        "name": candidate.trace.key_name_snapshot,
        "preview": candidate.trace.key_preview_snapshot,
    })
}

fn usage_snapshot(usage: crate::llm_proxy::audit::TokenUsage) -> Value {
    json!({
        "prompt_tokens": usage.prompt_tokens,
        "completion_tokens": usage.completion_tokens,
        "total_tokens": usage.total_tokens,
        "cache_creation_input_tokens": usage.cache_creation_input_tokens,
        "cache_read_input_tokens": usage.cache_read_input_tokens,
    })
}

fn pricing_snapshot(candidate: &ProxyCandidate) -> Value {
    json!({
        "price_per_request": candidate.price_per_request.map(|value| value.to_string()),
        "tiered_pricing": candidate.tiered_pricing,
    })
}

fn amount_snapshot(input: DescriptionInput<'_, '_>) -> Value {
    json!({
        "billing_currency": input.input.amount.currency,
        "wallet_currency": input.wallet.currency,
        "token_cost": input.input.amount.token_cost.to_string(),
        "base_cost": input.input.amount.base_cost.to_string(),
        "total_cost": input.input.amount.total_cost.to_string(),
        "deducted_amount": input.input.amount.total_cost.to_string(),
        "balance_before": input.change.before_total().to_string(),
        "balance_after": input.change.after_total().to_string(),
    })
}

fn token_type(value: ApiTokenType) -> &'static str {
    match value {
        ApiTokenType::User => "user",
        ApiTokenType::Independent => "independent",
    }
}
