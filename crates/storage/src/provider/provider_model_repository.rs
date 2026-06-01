use crate::StorageResult;

use super::{ProviderModelRecordBatchUpdate, ProviderStore};

impl ProviderStore {
    pub async fn batch_update_model_bindings(&self, input: ProviderModelRecordBatchUpdate) -> StorageResult<Vec<types::provider::ProviderModelBinding>> {
        super::provider_model_query::batch_update_model_bindings(self, input).await
    }
}
