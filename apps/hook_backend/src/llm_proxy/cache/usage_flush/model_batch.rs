use std::collections::HashSet;

use storage::model::{GlobalModelUsageRecord, GlobalModelUserUsageRecord};

use super::super::LlmProxyError;

pub(super) struct ModelProcessingBatch {
    pub(super) id: String,
    pub(super) records: Vec<GlobalModelUsageRecord>,
    pub(super) user_records: Vec<GlobalModelUserUsageRecord>,
}

pub(super) fn model_processing_batch(
    id: Option<String>,
    records: Vec<GlobalModelUsageRecord>,
    user_records: Vec<GlobalModelUserUsageRecord>,
) -> Result<Option<ModelProcessingBatch>, LlmProxyError> {
    if records.is_empty() && user_records.is_empty() && id.is_none() {
        return Ok(None);
    }
    if records.is_empty() {
        return Err(LlmProxyError::Infrastructure("model usage processing records are missing".into()));
    }
    ensure_user_model_records_have_platform_records(&records, &user_records)?;
    let id = id.ok_or_else(|| LlmProxyError::Infrastructure("model usage processing batch id is missing".into()))?;
    Ok(Some(ModelProcessingBatch { id, records, user_records }))
}

fn ensure_user_model_records_have_platform_records(
    records: &[GlobalModelUsageRecord],
    user_records: &[GlobalModelUserUsageRecord],
) -> Result<(), LlmProxyError> {
    let model_ids = records.iter().map(|record| record.model_id.as_str()).collect::<HashSet<_>>();
    let missing = user_records.iter().find(|record| !model_ids.contains(record.model_id.as_str()));
    if let Some(record) = missing {
        return Err(LlmProxyError::Infrastructure(format!(
            "user model usage record missing platform model usage for model {}",
            record.model_id
        )));
    }
    Ok(())
}
