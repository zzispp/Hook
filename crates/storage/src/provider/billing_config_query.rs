use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

use crate::{StorageError, StorageResult, json};

use super::{
    BillingRuleRecordInput, DimensionCollectorRecordInput,
    record::{billing_rules, dimension_collectors},
    repository::ProviderStore,
};

pub async fn create_billing_rule(store: &ProviderStore, input: BillingRuleRecordInput) -> StorageResult<billing_rules::Model> {
    let now = time::OffsetDateTime::now_utc();
    let record = billing_rules::ActiveModel {
        id: Set(store.next_id()),
        global_model_id: Set(input.global_model_id),
        model_id: Set(input.model_id),
        name: Set(input.name),
        task_type: Set(input.task_type),
        expression: Set(input.expression),
        variables: Set(json::encode_required(&input.variables)?),
        dimension_mappings: Set(json::encode_required(&input.dimension_mappings)?),
        is_enabled: Set(input.is_enabled),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(store.connection())
    .await?;
    Ok(record)
}

pub async fn enabled_rule_for_model(store: &ProviderStore, model_id: &str, task_type: &str) -> StorageResult<Option<billing_rules::Model>> {
    billing_rules::Entity::find()
        .filter(billing_rules::Column::ModelId.eq(model_id))
        .filter(billing_rules::Column::TaskType.eq(task_type))
        .filter(billing_rules::Column::IsEnabled.eq(true))
        .one(store.connection())
        .await
        .map_err(StorageError::from)
}

pub async fn enabled_rule_for_global_model(store: &ProviderStore, global_model_id: &str, task_type: &str) -> StorageResult<Option<billing_rules::Model>> {
    billing_rules::Entity::find()
        .filter(billing_rules::Column::GlobalModelId.eq(global_model_id))
        .filter(billing_rules::Column::TaskType.eq(task_type))
        .filter(billing_rules::Column::IsEnabled.eq(true))
        .one(store.connection())
        .await
        .map_err(StorageError::from)
}

pub async fn create_dimension_collector(store: &ProviderStore, input: DimensionCollectorRecordInput) -> StorageResult<dimension_collectors::Model> {
    let now = time::OffsetDateTime::now_utc();
    let record = dimension_collectors::ActiveModel {
        id: Set(store.next_id()),
        api_format: Set(input.api_format),
        task_type: Set(input.task_type),
        dimension_name: Set(input.dimension_name),
        source_type: Set(input.source_type),
        source_path: Set(input.source_path),
        value_type: Set(input.value_type),
        transform_expression: Set(input.transform_expression),
        default_value: Set(input.default_value),
        priority: Set(input.priority),
        is_enabled: Set(input.is_enabled),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(store.connection())
    .await?;
    Ok(record)
}

pub async fn enabled_collectors(store: &ProviderStore, api_format: &str, task_type: &str) -> StorageResult<Vec<dimension_collectors::Model>> {
    dimension_collectors::Entity::find()
        .filter(dimension_collectors::Column::ApiFormat.eq(api_format))
        .filter(dimension_collectors::Column::TaskType.eq(task_type))
        .filter(dimension_collectors::Column::IsEnabled.eq(true))
        .order_by_asc(dimension_collectors::Column::DimensionName)
        .order_by_desc(dimension_collectors::Column::Priority)
        .all(store.connection())
        .await
        .map_err(StorageError::from)
}
