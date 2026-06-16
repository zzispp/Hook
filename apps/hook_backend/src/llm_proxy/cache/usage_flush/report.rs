use storage::usage_flush::UsageFlushApplyReport;

use super::super::LlmProxyError;
use super::model_batch::ModelProcessingOrphan;

#[derive(Default)]
pub(super) struct FlushReport {
    pub(super) token_records: usize,
    pub(super) model_records: usize,
}

impl FlushReport {
    pub(super) fn is_empty(&self) -> bool {
        self.token_records == 0 && self.model_records == 0
    }

    pub(super) fn add(&mut self, other: Self) {
        self.token_records += other.token_records;
        self.model_records += other.model_records;
    }
}

pub(super) struct ProcessingBatch<T> {
    pub(super) id: String,
    pub(super) records: Vec<T>,
}

pub(super) fn merge_flush_outcome(token: Result<usize, LlmProxyError>, model: Result<usize, LlmProxyError>) -> Result<FlushReport, LlmProxyError> {
    match (token, model) {
        (Ok(token_records), Ok(model_records)) => Ok(FlushReport { token_records, model_records }),
        (Err(error), Ok(_)) | (Ok(_), Err(error)) => Err(error),
        (Err(token_error), Err(model_error)) => Err(LlmProxyError::Infrastructure(format!(
            "token usage flush failed: {token_error}; model usage flush failed: {model_error}"
        ))),
    }
}

pub(super) fn log_skipped_usage(kind: &'static str, batch_id: &str, report: &UsageFlushApplyReport) {
    if report.skipped_missing_count() == 0 {
        return;
    }
    let missing_ids = report.skipped_missing_resource_ids.join(",");
    hook_tracing::warn_with_fields!(
        "llm proxy usage flush skipped missing resources",
        kind = kind,
        batch_id = batch_id,
        skipped_records = report.skipped_missing_count(),
        missing_ids = missing_ids,
    );
}

pub(super) fn log_orphan_model_usage(orphan: &ModelProcessingOrphan) {
    let batch_id = orphan.batch_id.as_deref().unwrap_or("-");
    let missing_model_ids = if orphan.missing_user_model_ids.is_empty() {
        "-".to_owned()
    } else {
        orphan.missing_user_model_ids.join(",")
    };
    hook_tracing::warn_with_fields!(
        "llm proxy usage flush dropped orphan model processing batch",
        batch_id = batch_id,
        reason = orphan.reason,
        model_record_count = orphan.model_record_count,
        user_record_count = orphan.user_record_count,
        missing_model_ids = missing_model_ids,
    );
}

pub(super) fn processing_batch<T>(id: Option<String>, records: Vec<T>, label: &str) -> Result<Option<ProcessingBatch<T>>, LlmProxyError> {
    if records.is_empty() && id.is_none() {
        return Ok(None);
    }
    if records.is_empty() {
        return Err(LlmProxyError::Infrastructure(format!("{label} usage processing records are missing")));
    }
    let id = id.ok_or_else(|| LlmProxyError::Infrastructure(format!("{label} usage processing batch id is missing")))?;
    Ok(Some(ProcessingBatch { id, records }))
}
