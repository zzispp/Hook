use super::LlmProxyCache;

impl LlmProxyCache {
    pub(super) fn pending_token_keys(&self) -> [String; 3] {
        [
            self.pending_token_cost_key(),
            self.pending_token_count_key(),
            self.pending_token_last_used_at_key(),
        ]
    }

    pub(super) fn processing_token_keys(&self) -> [String; 3] {
        [
            self.processing_token_cost_key(),
            self.processing_token_count_key(),
            self.processing_token_last_used_at_key(),
        ]
    }

    pub(super) fn pending_token_cost_key(&self) -> String {
        format!("{}:llm_proxy:usage:pending:token:cost", self.key_prefix)
    }

    pub(super) fn pending_token_count_key(&self) -> String {
        format!("{}:llm_proxy:usage:pending:token:count", self.key_prefix)
    }

    pub(super) fn pending_token_last_used_at_key(&self) -> String {
        format!("{}:llm_proxy:usage:pending:token:last_used_at", self.key_prefix)
    }

    pub(super) fn processing_token_cost_key(&self) -> String {
        format!("{}:llm_proxy:usage:processing:token:cost", self.key_prefix)
    }

    pub(super) fn processing_token_count_key(&self) -> String {
        format!("{}:llm_proxy:usage:processing:token:count", self.key_prefix)
    }

    pub(super) fn processing_token_last_used_at_key(&self) -> String {
        format!("{}:llm_proxy:usage:processing:token:last_used_at", self.key_prefix)
    }

    pub(super) fn processing_token_batch_id_key(&self) -> String {
        format!("{}:llm_proxy:usage:processing:token:batch_id", self.key_prefix)
    }

    pub(super) fn pending_model_count_key(&self) -> String {
        format!("{}:llm_proxy:usage:pending:model:count", self.key_prefix)
    }

    pub(super) fn processing_model_count_key(&self) -> String {
        format!("{}:llm_proxy:usage:processing:model:count", self.key_prefix)
    }

    pub(super) fn processing_model_batch_id_key(&self) -> String {
        format!("{}:llm_proxy:usage:processing:model:batch_id", self.key_prefix)
    }

    pub(super) fn usage_flush_lock_key(&self) -> String {
        format!("{}:llm_proxy:usage:flush_lock", self.key_prefix)
    }
}
