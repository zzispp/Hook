use provider::application::billing::RequestBillingAmount;
use storage::provider::{RequestBillingRecordPatch, RequestBillingRecordValues};
use types::model::PatchField;

pub(super) fn billing_values(service_tier: Option<String>, amount: Option<&RequestBillingAmount>) -> RequestBillingRecordValues {
    RequestBillingRecordValues {
        service_tier,
        cost_currency: amount.map(|amount| amount.currency.clone()),
        input_cost: amount.map(|amount| amount.input_cost),
        output_cost: amount.map(|amount| amount.output_cost),
        cache_creation_cost: amount.map(|amount| amount.cache_creation_cost),
        cache_read_cost: amount.map(|amount| amount.cache_read_cost),
        request_cost: amount.map(|amount| amount.request_cost),
        token_cost: amount.map(|amount| amount.token_cost),
        base_cost: amount.map(|amount| amount.base_cost),
        total_cost: amount.map(|amount| amount.total_cost),
        billing_multiplier: amount.map(|amount| amount.billing_multiplier),
        input_price_per_million: amount.and_then(|amount| amount.input_price_per_1m),
        output_price_per_million: amount.and_then(|amount| amount.output_price_per_1m),
        cache_creation_price_per_million: amount.and_then(|amount| amount.cache_creation_price_per_1m),
        cache_read_price_per_million: amount.and_then(|amount| amount.cache_read_price_per_1m),
    }
}

pub(super) fn billing_patch(amount: Option<&RequestBillingAmount>) -> RequestBillingRecordPatch {
    RequestBillingRecordPatch {
        service_tier: PatchField::Missing,
        cost_currency: option_patch(amount.map(|amount| amount.currency.clone())),
        input_cost: option_patch(amount.map(|amount| amount.input_cost)),
        output_cost: option_patch(amount.map(|amount| amount.output_cost)),
        cache_creation_cost: option_patch(amount.map(|amount| amount.cache_creation_cost)),
        cache_read_cost: option_patch(amount.map(|amount| amount.cache_read_cost)),
        request_cost: option_patch(amount.map(|amount| amount.request_cost)),
        token_cost: option_patch(amount.map(|amount| amount.token_cost)),
        base_cost: option_patch(amount.map(|amount| amount.base_cost)),
        total_cost: option_patch(amount.map(|amount| amount.total_cost)),
        billing_multiplier: option_patch(amount.map(|amount| amount.billing_multiplier)),
        input_price_per_million: option_patch(amount.and_then(|amount| amount.input_price_per_1m)),
        output_price_per_million: option_patch(amount.and_then(|amount| amount.output_price_per_1m)),
        cache_creation_price_per_million: option_patch(amount.and_then(|amount| amount.cache_creation_price_per_1m)),
        cache_read_price_per_million: option_patch(amount.and_then(|amount| amount.cache_read_price_per_1m)),
    }
}

fn option_patch<T>(value: Option<T>) -> PatchField<T> {
    match value {
        Some(value) => PatchField::Value(value),
        None => PatchField::Null,
    }
}
