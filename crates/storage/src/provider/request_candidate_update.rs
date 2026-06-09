use rust_decimal::Decimal;
use sea_orm::{EntityTrait, UpdateMany, sea_query::Expr};
use types::model::PatchField;

use crate::{StorageResult, json};

use super::{
    RequestBillingRecordValues, RequestCandidateRecordPatch, record::request_candidates, request_record_write::ensure_accounting_cost_currency,
    request_upstream_cost,
};

pub(super) fn candidate_update(
    input: RequestCandidateRecordPatch,
    now: time::OffsetDateTime,
    was_started: bool,
) -> StorageResult<UpdateMany<request_candidates::Entity>> {
    let mut update = request_candidates::Entity::update_many()
        .col_expr(request_candidates::Column::Status, Expr::val(input.status.clone()))
        .col_expr(request_candidates::Column::SkipReason, Expr::val(input.skip_reason.clone()))
        .col_expr(request_candidates::Column::StatusCode, Expr::val(input.status_code))
        .col_expr(request_candidates::Column::PromptTokens, Expr::val(input.prompt_tokens))
        .col_expr(request_candidates::Column::CompletionTokens, Expr::val(input.completion_tokens))
        .col_expr(request_candidates::Column::TotalTokens, Expr::val(input.total_tokens))
        .col_expr(
            request_candidates::Column::CacheCreationInputTokens,
            Expr::val(input.cache_creation_input_tokens),
        )
        .col_expr(request_candidates::Column::CacheReadInputTokens, Expr::val(input.cache_read_input_tokens))
        .col_expr(request_candidates::Column::InputTextTokens, Expr::val(input.input_text_tokens))
        .col_expr(request_candidates::Column::InputAudioTokens, Expr::val(input.input_audio_tokens))
        .col_expr(request_candidates::Column::InputImageTokens, Expr::val(input.input_image_tokens))
        .col_expr(request_candidates::Column::OutputTextTokens, Expr::val(input.output_text_tokens))
        .col_expr(request_candidates::Column::OutputAudioTokens, Expr::val(input.output_audio_tokens))
        .col_expr(request_candidates::Column::OutputImageTokens, Expr::val(input.output_image_tokens))
        .col_expr(request_candidates::Column::ReasoningTokens, Expr::val(input.reasoning_tokens))
        .col_expr(
            request_candidates::Column::CacheCreation5mInputTokens,
            Expr::val(input.cache_creation_5m_input_tokens),
        )
        .col_expr(
            request_candidates::Column::CacheCreation1hInputTokens,
            Expr::val(input.cache_creation_1h_input_tokens),
        )
        .col_expr(request_candidates::Column::UsageSource, Expr::val(input.usage_source.clone()))
        .col_expr(request_candidates::Column::UsageSemantic, Expr::val(input.usage_semantic.clone()));
    update = apply_candidate_upstream_cost_patch(update, &input.upstream_cost);
    update = apply_candidate_billing_values(update, &input.billing)?;
    update = apply_json_expr(update, request_candidates::Column::BillingSnapshot, &input.billing_snapshot)?;
    update = apply_error_and_timing(update, input, now, was_started);
    Ok(update)
}

fn apply_error_and_timing(
    update: UpdateMany<request_candidates::Entity>,
    input: RequestCandidateRecordPatch,
    now: time::OffsetDateTime,
    was_started: bool,
) -> UpdateMany<request_candidates::Entity> {
    let mut update = update
        .col_expr(request_candidates::Column::LatencyMs, Expr::val(input.latency_ms))
        .col_expr(request_candidates::Column::FirstByteTimeMs, Expr::val(input.first_byte_time_ms))
        .col_expr(request_candidates::Column::ErrorType, Expr::val(input.error_type))
        .col_expr(request_candidates::Column::ErrorMessage, Expr::val(input.error_message))
        .col_expr(request_candidates::Column::ErrorCode, Expr::val(input.error_code))
        .col_expr(request_candidates::Column::ErrorParam, Expr::val(input.error_param));
    update = apply_legacy_payload_expr(update, request_candidates::Column::ProviderRequestHeaders, input.provider_request_headers);
    update = apply_legacy_payload_expr(update, request_candidates::Column::ProviderRequestBody, input.provider_request_body);
    update = apply_legacy_payload_expr(update, request_candidates::Column::ProviderResponseHeaders, input.provider_response_headers);
    update = apply_legacy_payload_expr(update, request_candidates::Column::ProviderResponseBody, input.provider_response_body);
    apply_timestamps(update, input.finished, now, was_started)
}

fn apply_candidate_upstream_cost_patch(
    mut update: UpdateMany<request_candidates::Entity>,
    patch: &super::RequestUpstreamCostRecordPatch,
) -> UpdateMany<request_candidates::Entity> {
    if !patch.upstream_cost_mode.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamCostMode,
            patch_string_value(&patch.upstream_cost_mode, request_upstream_cost::mode_value),
        );
    }
    if !patch.upstream_cost_source.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamCostSource,
            patch_string_value(&patch.upstream_cost_source, request_upstream_cost::source_value),
        );
    }
    update = apply_candidate_upstream_prices(update, patch);
    apply_candidate_upstream_costs(update, patch)
}

fn apply_candidate_upstream_prices(
    mut update: UpdateMany<request_candidates::Entity>,
    patch: &super::RequestUpstreamCostRecordPatch,
) -> UpdateMany<request_candidates::Entity> {
    if !patch.upstream_price_per_request.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamPricePerRequest,
            patch_decimal_value(&patch.upstream_price_per_request),
        );
    }
    if !patch.upstream_input_price_per_million.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamInputPricePerMillion,
            patch_decimal_value(&patch.upstream_input_price_per_million),
        );
    }
    if !patch.upstream_output_price_per_million.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamOutputPricePerMillion,
            patch_decimal_value(&patch.upstream_output_price_per_million),
        );
    }
    if !patch.upstream_cache_creation_price_per_million.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamCacheCreationPricePerMillion,
            patch_decimal_value(&patch.upstream_cache_creation_price_per_million),
        );
    }
    if !patch.upstream_cache_read_price_per_million.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamCacheReadPricePerMillion,
            patch_decimal_value(&patch.upstream_cache_read_price_per_million),
        );
    }
    update
}

fn apply_candidate_upstream_costs(
    mut update: UpdateMany<request_candidates::Entity>,
    patch: &super::RequestUpstreamCostRecordPatch,
) -> UpdateMany<request_candidates::Entity> {
    if !patch.upstream_request_cost.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamRequestCost,
            patch_decimal_value(&patch.upstream_request_cost),
        );
    }
    if !patch.upstream_input_cost.is_missing() {
        update = update.col_expr(request_candidates::Column::UpstreamInputCost, patch_decimal_value(&patch.upstream_input_cost));
    }
    if !patch.upstream_output_cost.is_missing() {
        update = update.col_expr(request_candidates::Column::UpstreamOutputCost, patch_decimal_value(&patch.upstream_output_cost));
    }
    if !patch.upstream_cache_creation_cost.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamCacheCreationCost,
            patch_decimal_value(&patch.upstream_cache_creation_cost),
        );
    }
    if !patch.upstream_cache_read_cost.is_missing() {
        update = update.col_expr(
            request_candidates::Column::UpstreamCacheReadCost,
            patch_decimal_value(&patch.upstream_cache_read_cost),
        );
    }
    if !patch.upstream_total_cost.is_missing() {
        update = update.col_expr(request_candidates::Column::UpstreamTotalCost, patch_decimal_value(&patch.upstream_total_cost));
    }
    update
}

fn apply_candidate_billing_values(
    mut update: UpdateMany<request_candidates::Entity>,
    billing: &RequestBillingRecordValues,
) -> StorageResult<UpdateMany<request_candidates::Entity>> {
    if let Some(service_tier) = billing.service_tier.clone() {
        update = update.col_expr(request_candidates::Column::ServiceTier, Expr::val(Some(service_tier)));
    }
    Ok(update
        .col_expr(
            request_candidates::Column::CostCurrency,
            Expr::val(ensure_accounting_cost_currency(billing.cost_currency.clone())?),
        )
        .col_expr(request_candidates::Column::InputCost, Expr::val(billing.input_cost))
        .col_expr(request_candidates::Column::OutputCost, Expr::val(billing.output_cost))
        .col_expr(request_candidates::Column::CacheCreationCost, Expr::val(billing.cache_creation_cost))
        .col_expr(request_candidates::Column::CacheReadCost, Expr::val(billing.cache_read_cost))
        .col_expr(request_candidates::Column::RequestCost, Expr::val(billing.request_cost))
        .col_expr(request_candidates::Column::TokenCost, Expr::val(billing.token_cost))
        .col_expr(request_candidates::Column::BaseCost, Expr::val(billing.base_cost))
        .col_expr(request_candidates::Column::TotalCost, Expr::val(billing.total_cost))
        .col_expr(request_candidates::Column::BillingMultiplier, Expr::val(billing.billing_multiplier))
        .col_expr(request_candidates::Column::InputPricePerMillion, Expr::val(billing.input_price_per_million))
        .col_expr(request_candidates::Column::OutputPricePerMillion, Expr::val(billing.output_price_per_million))
        .col_expr(
            request_candidates::Column::CacheCreationPricePerMillion,
            Expr::val(billing.cache_creation_price_per_million),
        )
        .col_expr(
            request_candidates::Column::CacheReadPricePerMillion,
            Expr::val(billing.cache_read_price_per_million),
        ))
}

fn apply_json_expr(
    update: UpdateMany<request_candidates::Entity>,
    column: request_candidates::Column,
    patch: &PatchField<serde_json::Value>,
) -> StorageResult<UpdateMany<request_candidates::Entity>> {
    Ok(match patch {
        PatchField::Value(value) => update.col_expr(column, Expr::val(Some(json::encode_required(value)?))),
        PatchField::Null => update.col_expr(column, Expr::val(Option::<String>::None)),
        PatchField::Missing => update,
    })
}

fn apply_legacy_payload_expr(
    update: UpdateMany<request_candidates::Entity>,
    column: request_candidates::Column,
    patch: PatchField<serde_json::Value>,
) -> UpdateMany<request_candidates::Entity> {
    if patch.is_missing() {
        return update;
    }
    update.col_expr(column, Expr::val(Option::<String>::None))
}

fn apply_timestamps(
    mut update: UpdateMany<request_candidates::Entity>,
    finished: bool,
    now: time::OffsetDateTime,
    was_started: bool,
) -> UpdateMany<request_candidates::Entity> {
    if !was_started {
        update = update.col_expr(request_candidates::Column::StartedAt, Expr::val(Some(now)));
    }
    if finished {
        update = update.col_expr(request_candidates::Column::FinishedAt, Expr::val(Some(now)));
    }
    update
}

fn patch_decimal_value(patch: &PatchField<Decimal>) -> sea_orm::sea_query::SimpleExpr {
    match patch {
        PatchField::Value(value) => Expr::val(Some(*value)),
        PatchField::Null | PatchField::Missing => Expr::val(Option::<Decimal>::None),
    }
}

fn patch_string_value<T>(patch: &PatchField<T>, render: fn(&T) -> &'static str) -> sea_orm::sea_query::SimpleExpr {
    match patch {
        PatchField::Value(value) => Expr::val(Some(render(value).to_owned())),
        PatchField::Null | PatchField::Missing => Expr::val(Option::<String>::None),
    }
}
