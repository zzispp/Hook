use std::collections::HashSet;

use storage::model::{GlobalModelUsageRecord, GlobalModelUserUsageRecord};

pub(super) struct ModelProcessingBatch {
    pub(super) id: String,
    pub(super) records: Vec<GlobalModelUsageRecord>,
    pub(super) user_records: Vec<GlobalModelUserUsageRecord>,
}

pub(super) struct ModelProcessingOrphan {
    pub(super) batch_id: Option<String>,
    pub(super) reason: &'static str,
    pub(super) model_record_count: usize,
    pub(super) user_record_count: usize,
    pub(super) missing_user_model_ids: Vec<String>,
}

pub(super) enum ModelProcessingState {
    Empty,
    Ready(ModelProcessingBatch),
    Orphan(ModelProcessingOrphan),
}

pub(super) fn model_processing_batch(
    id: Option<String>,
    records: Vec<GlobalModelUsageRecord>,
    user_records: Vec<GlobalModelUserUsageRecord>,
) -> ModelProcessingState {
    if records.is_empty() && user_records.is_empty() && id.is_none() {
        return ModelProcessingState::Empty;
    }
    if records.is_empty() {
        return orphan(id, "platform_records_missing", &records, &user_records);
    }
    if id.is_none() {
        return orphan(id, "batch_id_missing", &records, &user_records);
    }
    let missing_user_model_ids = missing_user_model_ids(&records, &user_records);
    if !missing_user_model_ids.is_empty() {
        return ModelProcessingState::Orphan(ModelProcessingOrphan {
            batch_id: id,
            reason: "user_records_missing_platform_records",
            model_record_count: records.len(),
            user_record_count: user_records.len(),
            missing_user_model_ids,
        });
    }
    let Some(id) = id else {
        unreachable!("batch id checked above");
    };
    ModelProcessingState::Ready(ModelProcessingBatch { id, records, user_records })
}

fn orphan(id: Option<String>, reason: &'static str, records: &[GlobalModelUsageRecord], user_records: &[GlobalModelUserUsageRecord]) -> ModelProcessingState {
    ModelProcessingState::Orphan(ModelProcessingOrphan {
        batch_id: id,
        reason,
        model_record_count: records.len(),
        user_record_count: user_records.len(),
        missing_user_model_ids: missing_user_model_ids(records, user_records),
    })
}

fn missing_user_model_ids(records: &[GlobalModelUsageRecord], user_records: &[GlobalModelUserUsageRecord]) -> Vec<String> {
    let model_ids = records.iter().map(|record| record.model_id.as_str()).collect::<HashSet<_>>();
    user_records
        .iter()
        .filter(|record| !model_ids.contains(record.model_id.as_str()))
        .map(|record| record.model_id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ModelProcessingState, model_processing_batch};
    use storage::model::{GlobalModelUsageRecord, GlobalModelUserUsageRecord};

    #[test]
    fn model_processing_batch_returns_empty_for_absent_processing_state() {
        let state = model_processing_batch(None, Vec::new(), Vec::new());

        assert!(matches!(state, ModelProcessingState::Empty));
    }

    #[test]
    fn model_processing_batch_marks_user_only_processing_as_orphan() {
        let state = model_processing_batch(Some("batch-1".into()), Vec::new(), vec![user_usage_record("model-1")]);

        match state {
            ModelProcessingState::Orphan(orphan) => {
                assert_eq!(orphan.reason, "platform_records_missing");
                assert_eq!(orphan.batch_id.as_deref(), Some("batch-1"));
                assert_eq!(orphan.user_record_count, 1);
                assert_eq!(orphan.missing_user_model_ids, vec!["model-1"]);
            }
            _ => panic!("expected orphan"),
        }
    }

    #[test]
    fn model_processing_batch_marks_missing_batch_id_as_orphan() {
        let state = model_processing_batch(None, vec![usage_record("model-1")], vec![]);

        match state {
            ModelProcessingState::Orphan(orphan) => {
                assert_eq!(orphan.reason, "batch_id_missing");
                assert_eq!(orphan.model_record_count, 1);
            }
            _ => panic!("expected orphan"),
        }
    }

    #[test]
    fn model_processing_batch_marks_mismatched_user_models_as_orphan() {
        let state = model_processing_batch(Some("batch-1".into()), vec![usage_record("model-1")], vec![user_usage_record("model-2")]);

        match state {
            ModelProcessingState::Orphan(orphan) => {
                assert_eq!(orphan.reason, "user_records_missing_platform_records");
                assert_eq!(orphan.missing_user_model_ids, vec!["model-2"]);
            }
            _ => panic!("expected orphan"),
        }
    }

    #[test]
    fn model_processing_batch_returns_ready_for_consistent_records() {
        let state = model_processing_batch(Some("batch-1".into()), vec![usage_record("model-1")], vec![user_usage_record("model-1")]);

        match state {
            ModelProcessingState::Ready(batch) => {
                assert_eq!(batch.id, "batch-1");
                assert_eq!(batch.records.len(), 1);
                assert_eq!(batch.user_records.len(), 1);
            }
            _ => panic!("expected ready"),
        }
    }

    fn usage_record(model_id: &str) -> GlobalModelUsageRecord {
        GlobalModelUsageRecord {
            model_id: model_id.into(),
            count: 1,
            user_id: None,
        }
    }

    fn user_usage_record(model_id: &str) -> GlobalModelUserUsageRecord {
        GlobalModelUserUsageRecord {
            user_id: "user-1".into(),
            model_id: model_id.into(),
            count: 1,
        }
    }
}
