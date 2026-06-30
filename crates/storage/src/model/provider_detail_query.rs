use std::collections::HashMap;

use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use types::model::{ModelCatalogProviderDetail, ModelCatalogProviderPriceRange};

use crate::{
    StorageError, StorageResult,
    provider::record::{ProviderModelCostRecord, ProviderRecord, provider_model_costs, providers},
};

use super::{GlobalModelRecord, ModelRecord, ProviderDetailPriceSummary};

pub async fn provider_details(
    connection: &DatabaseConnection,
    global_model: &GlobalModelRecord,
    bindings: Vec<ModelRecord>,
) -> StorageResult<Vec<ModelCatalogProviderDetail>> {
    if bindings.is_empty() {
        return Ok(Vec::new());
    }

    let provider_records = provider_records_by_id(connection, &bindings).await?;
    let cost_summaries = cost_summaries_by_model_id(connection, &bindings).await?;
    let default_summary = global_default_summary(global_model)?;

    bindings
        .into_iter()
        .map(|binding| {
            let provider = provider_records
                .get(&binding.provider_id)
                .ok_or_else(|| StorageError::Database(format!("provider missing for model binding: {}", binding.provider_id)))?;
            let pricing = cost_summaries.get(&binding.id).unwrap_or(&default_summary);
            binding.provider_detail(global_model, provider, pricing)
        })
        .collect()
}

async fn provider_records_by_id(connection: &DatabaseConnection, bindings: &[ModelRecord]) -> StorageResult<HashMap<String, ProviderRecord>> {
    let provider_ids = bindings.iter().map(|binding| binding.provider_id.clone()).collect::<Vec<_>>();
    let records = providers::Entity::find()
        .filter(providers::Column::Id.is_in(provider_ids))
        .all(connection)
        .await?;
    Ok(records.into_iter().map(|record| (record.id.clone(), record)).collect())
}

async fn cost_summaries_by_model_id(connection: &DatabaseConnection, bindings: &[ModelRecord]) -> StorageResult<HashMap<String, ProviderDetailPriceSummary>> {
    let model_ids = bindings.iter().map(|binding| binding.id.clone()).collect::<Vec<_>>();
    let records = provider_model_costs::Entity::find()
        .filter(provider_model_costs::Column::ProviderModelId.is_in(model_ids))
        .all(connection)
        .await?;

    let mut builders = HashMap::<String, PriceSummaryBuilder>::new();
    for record in records {
        builders.entry(record.provider_model_id.clone()).or_default().push(&record);
    }

    Ok(builders.into_iter().map(|(model_id, builder)| (model_id, builder.finish())).collect())
}

fn global_default_summary(global_model: &GlobalModelRecord) -> StorageResult<ProviderDetailPriceSummary> {
    let tiered = global_model.default_tiered_pricing()?;
    let tier = tiered.tiers.first();
    let input_price_per_1m = tier.map(|item| item.input_price_per_1m);
    let output_price_per_1m = tier.map(|item| item.output_price_per_1m);
    let cache_creation_price_per_1m = tier.and_then(|item| item.cache_creation_price_per_1m);
    let cache_read_price_per_1m = tier.and_then(|item| item.cache_read_price_per_1m);
    let price_per_request = global_model.default_price_per_request;

    Ok(ProviderDetailPriceSummary {
        configured_cost_count: 0,
        input_price_per_1m,
        input_price_range: single_value_range(input_price_per_1m),
        output_price_per_1m,
        output_price_range: single_value_range(output_price_per_1m),
        cache_creation_price_per_1m,
        cache_creation_price_range: single_value_range(cache_creation_price_per_1m),
        cache_read_price_per_1m,
        cache_read_price_range: single_value_range(cache_read_price_per_1m),
        price_per_request,
        price_per_request_range: single_value_range(price_per_request),
    })
}

fn single_value_range(value: Option<Decimal>) -> ModelCatalogProviderPriceRange {
    ModelCatalogProviderPriceRange { min: value, max: value }
}

#[derive(Default)]
struct PriceSummaryBuilder {
    configured_cost_count: u64,
    input_price_range: DecimalRangeBuilder,
    output_price_range: DecimalRangeBuilder,
    cache_creation_price_range: DecimalRangeBuilder,
    cache_read_price_range: DecimalRangeBuilder,
    price_per_request_range: DecimalRangeBuilder,
}

impl PriceSummaryBuilder {
    fn push(&mut self, record: &ProviderModelCostRecord) {
        self.configured_cost_count += 1;
        self.input_price_range.push(record.input_price_per_million);
        self.output_price_range.push(record.output_price_per_million);
        self.cache_creation_price_range.push(record.cache_creation_price_per_million);
        self.cache_read_price_range.push(record.cache_read_price_per_million);
        self.price_per_request_range.push(record.price_per_request);
    }

    fn finish(self) -> ProviderDetailPriceSummary {
        let input_price_range = self.input_price_range.finish();
        let output_price_range = self.output_price_range.finish();
        let cache_creation_price_range = self.cache_creation_price_range.finish();
        let cache_read_price_range = self.cache_read_price_range.finish();
        let price_per_request_range = self.price_per_request_range.finish();

        ProviderDetailPriceSummary {
            configured_cost_count: self.configured_cost_count,
            input_price_per_1m: input_price_range.min,
            input_price_range,
            output_price_per_1m: output_price_range.min,
            output_price_range,
            cache_creation_price_per_1m: cache_creation_price_range.min,
            cache_creation_price_range,
            cache_read_price_per_1m: cache_read_price_range.min,
            cache_read_price_range,
            price_per_request: price_per_request_range.min,
            price_per_request_range,
        }
    }
}

#[derive(Default)]
struct DecimalRangeBuilder {
    min: Option<Decimal>,
    max: Option<Decimal>,
}

impl DecimalRangeBuilder {
    fn push(&mut self, value: Option<Decimal>) {
        let Some(value) = value else {
            return;
        };

        self.min = Some(self.min.map_or(value, |current| current.min(value)));
        self.max = Some(self.max.map_or(value, |current| current.max(value)));
    }

    fn finish(self) -> ModelCatalogProviderPriceRange {
        ModelCatalogProviderPriceRange { min: self.min, max: self.max }
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::{DatabaseBackend, MockDatabase};

    use super::*;
    use crate::model::record::{global_models, provider_models};

    #[tokio::test]
    async fn global_model_providers_use_provider_names_and_configured_cost_ranges() {
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[
                provider_record("provider-a", "Provider A", true),
                provider_record("provider-b", "Provider B", false),
            ]])
            .append_query_results([[
                cost_record(
                    "cost-1",
                    "model-a",
                    Some(dec("0.1")),
                    Some(dec("0.6")),
                    Some(dec("0.125")),
                    Some(dec("0.01")),
                    None,
                ),
                cost_record(
                    "cost-2",
                    "model-a",
                    Some(dec("0.3")),
                    Some(dec("1.8")),
                    Some(dec("0.375")),
                    Some(dec("0.03")),
                    None,
                ),
            ]])
            .into_connection();

        let details = provider_details(
            &connection,
            &global_model_record(),
            vec![
                model_binding("model-a", "provider-a", "gpt-5.4", true),
                model_binding("model-b", "provider-b", "gpt-5.4", true),
            ],
        )
        .await
        .unwrap();

        assert_eq!(details[0].provider_name, "Provider A");
        assert!(details[0].is_active);
        assert_eq!(details[0].configured_cost_count, 2);
        assert_eq!(details[0].input_price_per_1m, Some(dec("0.1")));
        assert_eq!(details[0].input_price_range.min, Some(dec("0.1")));
        assert_eq!(details[0].input_price_range.max, Some(dec("0.3")));
        assert_eq!(details[0].cache_read_price_range.max, Some(dec("0.03")));

        assert_eq!(details[1].provider_name, "Provider B");
        assert!(!details[1].is_active);
        assert!(details[1].binding_is_active);
        assert!(!details[1].provider_is_active);
    }

    #[tokio::test]
    async fn global_model_providers_fall_back_to_default_pricing_without_configured_costs() {
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([[provider_record("provider-a", "Provider A", true)]])
            .append_query_results([Vec::<ProviderModelCostRecord>::new()])
            .into_connection();

        let details = provider_details(
            &connection,
            &global_model_record(),
            vec![model_binding("model-a", "provider-a", "gpt-5.4", true)],
        )
        .await
        .unwrap();

        let detail = &details[0];
        assert_eq!(detail.configured_cost_count, 0);
        assert_eq!(detail.input_price_per_1m, Some(dec("2.5")));
        assert_eq!(detail.input_price_range.min, Some(dec("2.5")));
        assert_eq!(detail.input_price_range.max, Some(dec("2.5")));
        assert_eq!(detail.cache_creation_price_per_1m, Some(dec("3.125")));
        assert_eq!(detail.cache_read_price_per_1m, Some(dec("0.25")));
    }

    fn global_model_record() -> GlobalModelRecord {
        global_models::Model {
            id: "global-model-1".into(),
            name: "gpt-5.4".into(),
            display_name: "GPT-5.4".into(),
            default_price_per_request: None,
            default_tiered_pricing: "{\"tiers\":[{\"up_to\":null,\"input_price_per_1m\":2.5,\"output_price_per_1m\":15.0,\"cache_creation_price_per_1m\":3.125,\"cache_read_price_per_1m\":0.25}]}".into(),
            supported_capabilities: None,
            config: Some("{\"streaming\":true}".into()),
            routing_profile_id: None,
            is_active: true,
            usage_count: 0,
            created_at: now(),
            updated_at: now(),
        }
    }

    fn model_binding(id: &str, provider_id: &str, _provider_model_name: &str, is_active: bool) -> ModelRecord {
        provider_models::Model {
            id: id.into(),
            provider_id: provider_id.into(),
            global_model_id: "global-model-1".into(),
            is_active,
            config: None,
            created_at: now(),
            updated_at: now(),
        }
    }

    fn provider_record(id: &str, name: &str, is_active: bool) -> ProviderRecord {
        ProviderRecord {
            id: id.into(),
            name: name.into(),
            provider_type: "custom".into(),
            provider_origin: "manual".into(),
            max_retries: None,
            request_timeout_seconds: None,
            stream_response_headers_timeout_seconds: None,
            stream_first_byte_timeout_seconds: None,
            stream_first_token_timeout_seconds: None,
            stream_idle_timeout_seconds: None,
            priority: 0,
            keep_priority_on_conversion: false,
            enable_format_conversion: true,
            is_active,
            created_at: now(),
            updated_at: now(),
        }
    }

    fn cost_record(
        id: &str,
        provider_model_id: &str,
        input: Option<Decimal>,
        output: Option<Decimal>,
        cache_creation: Option<Decimal>,
        cache_read: Option<Decimal>,
        price_per_request: Option<Decimal>,
    ) -> ProviderModelCostRecord {
        ProviderModelCostRecord {
            id: id.into(),
            provider_id: "provider-a".into(),
            key_id: format!("key-{id}"),
            provider_model_id: provider_model_id.into(),
            cost_mode: if price_per_request.is_some() {
                "per_request".into()
            } else {
                "per_token".into()
            },
            price_per_request,
            input_price_per_million: input,
            output_price_per_million: output,
            cache_creation_price_per_million: cache_creation,
            cache_read_price_per_million: cache_read,
            created_at: now(),
            updated_at: now(),
        }
    }

    fn dec(value: &str) -> Decimal {
        value.parse().unwrap()
    }

    fn now() -> time::OffsetDateTime {
        time::Date::from_calendar_date(2026, time::Month::June, 16)
            .unwrap()
            .with_hms(10, 30, 0)
            .unwrap()
            .assume_utc()
    }
}
