use super::{
    LlmProxyCache, LlmProxyError,
    model_batch::{ModelProcessingBatch, ModelProcessingState},
    report::log_orphan_model_usage,
};

impl LlmProxyCache {
    pub(super) async fn recover_model_processing_state(&self, state: ModelProcessingState) -> Result<Option<ModelProcessingBatch>, LlmProxyError> {
        match state {
            ModelProcessingState::Empty => Ok(None),
            ModelProcessingState::Ready(batch) => Ok(Some(batch)),
            ModelProcessingState::Orphan(orphan) => {
                log_orphan_model_usage(&orphan);
                self.clear_model_processing_usage().await?;
                if let Some(batch_id) = orphan.batch_id.as_deref() {
                    self.delete_usage_flush_batch(batch_id).await?;
                }
                Ok(None)
            }
        }
    }
}
