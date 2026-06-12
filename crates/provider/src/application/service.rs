use async_trait::async_trait;
use types::provider::*;

use crate::application::{
    GlobalModelCatalog, ProviderError, ProviderQuickImportSyncRunOptions, ProviderQuickImportSyncRunReport, ProviderRepository, ProviderResult,
    ProviderUseCase, SecretCipher, UpstreamModelFetcher, UpstreamProviderImportSource,
};

mod key_endpoint_scope;
mod key_permissions;
mod model_bindings;
mod model_costs;
mod provider_core;
mod provider_key_groups;
mod quick_import;
mod quick_import_append;
mod quick_import_commit;
mod quick_import_commit_models;
#[cfg(test)]
mod quick_import_commit_tests;
mod quick_import_costs;
mod quick_import_preview;
mod quick_import_resolution;
mod quick_import_resolution_context;
mod quick_import_resolution_models;
mod quick_import_shared;
mod quick_import_sync;
mod quick_import_sync_bindings;
mod quick_import_sync_candidates;
mod quick_import_sync_events;
mod quick_import_sync_model_check;
#[cfg(test)]
mod quick_import_sync_model_rules_tests;
mod quick_import_sync_outcome;
#[cfg(test)]
mod quick_import_sync_outcome_tests;
#[cfg(test)]
mod quick_import_sync_policy_tests;
mod quick_import_sync_settings;
mod request_queries;
mod upstream_models;

use key_endpoint_scope::ensure_api_formats_bound;
use key_permissions::ensure_allowed_models_bound;
use model_bindings::{prepare_model_binding_batch_update, prepare_model_binding_create};
use model_costs::{ensure_model_cost_delete_scope, ensure_model_cost_scope};
use provider_core::{ensure_provider, prepare_provider_create, prepare_provider_list_request};
use provider_key_groups::{prepare_provider_key_group_create, prepare_provider_key_group_list_request, prepare_provider_key_group_update};
use quick_import::{QuickImportArgs, commit_quick_import, preview_quick_import};
use quick_import_append::{commit_quick_import_append, preview_quick_import_append};
use quick_import_resolution::{
    accept_quick_import_current, quick_import_model_associations, quick_import_resolution, relink_quick_import_key, update_quick_import_model_associations,
};
use quick_import_resolution_models::has_hard_quick_import_status;
use quick_import_sync::{SyncArgs, run_quick_import_sync};
use quick_import_sync_settings::{quick_import_sync_settings, update_quick_import_sync_settings};
use request_queries::{
    sanitize_active_request_record_request, sanitize_provider_cooldown_request, validate_provider_cooldown_request, validate_request_record_list_request,
};
use upstream_models::{FetchUpstreamModels, fetch_upstream_models};

use super::validation::{
    sanitize_api_key, sanitize_api_key_update, sanitize_endpoint, sanitize_endpoint_update, sanitize_model_binding_update, sanitize_model_cost_batch,
    sanitize_update, validate_api_key, validate_api_key_priority_batch, validate_api_key_update, validate_endpoint, validate_endpoint_update,
    validate_model_binding_update, validate_model_cost_batch, validate_update,
};

pub struct ProviderService<R, M, C, F, I> {
    repository: R,
    models: M,
    cipher: C,
    fetcher: F,
    importer: I,
}

impl<R, M, C, F, I> ProviderService<R, M, C, F, I>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    F: UpstreamModelFetcher,
    I: UpstreamProviderImportSource,
{
    pub const fn new(repository: R, models: M, cipher: C, fetcher: F, importer: I) -> Self {
        Self {
            repository,
            models,
            cipher,
            fetcher,
            importer,
        }
    }
}

#[async_trait]
impl<R, M, C, F, I> ProviderUseCase for ProviderService<R, M, C, F, I>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    F: UpstreamModelFetcher,
    I: UpstreamProviderImportSource,
{
    async fn create_provider(&self, input: ProviderCreate) -> ProviderResult<Provider> {
        let input = prepare_provider_create(&self.repository, input).await?;
        self.repository.create_provider(input).await
    }

    async fn update_provider(&self, id: &str, input: ProviderUpdate) -> ProviderResult<Provider> {
        let input = sanitize_update(input);
        validate_update(&input)?;
        self.repository.update_provider(id, input).await
    }

    async fn delete_provider(&self, id: &str) -> ProviderResult<()> {
        self.repository.delete_provider(id).await
    }

    async fn get_provider(&self, id: &str) -> ProviderResult<Provider> {
        self.repository.find_provider(id).await?.ok_or(ProviderError::NotFound)
    }

    async fn list_providers(&self, request: ProviderListRequest) -> ProviderResult<ProviderListResponse> {
        let request = prepare_provider_list_request(request)?;
        self.repository.list_providers(request).await
    }

    async fn create_provider_key_group(&self, input: ProviderKeyGroupCreate) -> ProviderResult<ProviderKeyGroup> {
        let input = prepare_provider_key_group_create(&self.repository, input).await?;
        self.repository.create_provider_key_group(input).await
    }

    async fn update_provider_key_group(&self, id: &str, input: ProviderKeyGroupUpdate) -> ProviderResult<ProviderKeyGroup> {
        let input = prepare_provider_key_group_update(&self.repository, id, input).await?;
        self.repository.update_provider_key_group(id, input).await
    }

    async fn delete_provider_key_group(&self, id: &str) -> ProviderResult<()> {
        self.repository.delete_provider_key_group(id).await
    }

    async fn get_provider_key_group(&self, id: &str) -> ProviderResult<ProviderKeyGroup> {
        self.repository.find_provider_key_group(id).await?.ok_or(ProviderError::NotFound)
    }

    async fn list_provider_key_groups(&self, request: ProviderKeyGroupListRequest) -> ProviderResult<ProviderKeyGroupListResponse> {
        let request = prepare_provider_key_group_list_request(request)?;
        self.repository.list_provider_key_groups(request).await
    }

    async fn create_endpoint(&self, provider_id: &str, input: ProviderEndpointCreate) -> ProviderResult<ProviderEndpoint> {
        ensure_provider(&self.repository, provider_id).await?;
        let input = sanitize_endpoint(input);
        validate_endpoint(&input)?;
        self.repository.create_endpoint(provider_id, input).await
    }

    async fn update_endpoint(&self, provider_id: &str, endpoint_id: &str, input: ProviderEndpointUpdate) -> ProviderResult<ProviderEndpoint> {
        ensure_provider(&self.repository, provider_id).await?;
        let input = sanitize_endpoint_update(input);
        validate_endpoint_update(&input)?;
        self.repository.update_endpoint(provider_id, endpoint_id, input).await
    }

    async fn delete_endpoint(&self, provider_id: &str, endpoint_id: &str) -> ProviderResult<()> {
        ensure_provider(&self.repository, provider_id).await?;
        self.repository.delete_endpoint(provider_id, endpoint_id).await
    }

    async fn list_endpoints(&self, provider_id: &str) -> ProviderResult<Vec<ProviderEndpoint>> {
        ensure_provider(&self.repository, provider_id).await?;
        self.repository.list_endpoints(provider_id).await
    }

    async fn create_api_key(&self, provider_id: &str, input: ProviderApiKeyCreate) -> ProviderResult<ProviderApiKey> {
        ensure_provider(&self.repository, provider_id).await?;
        let input = sanitize_api_key(input);
        validate_api_key(&input)?;
        ensure_api_formats_bound(&self.repository, provider_id, &input.api_formats).await?;
        ensure_allowed_models_bound(&self.repository, provider_id, &input.allowed_model_ids).await?;
        let encrypted = self.cipher.encrypt_provider_key(&input.api_key)?;
        self.repository.create_api_key(provider_id, input, encrypted).await
    }

    async fn list_api_keys(&self, provider_id: &str) -> ProviderResult<Vec<ProviderApiKey>> {
        ensure_provider(&self.repository, provider_id).await?;
        self.repository.list_api_keys(provider_id).await
    }

    async fn fetch_upstream_models(&self, provider_id: &str) -> ProviderResult<ProviderUpstreamModelsResponse> {
        ensure_provider(&self.repository, provider_id).await?;
        fetch_upstream_models(FetchUpstreamModels {
            repository: &self.repository,
            cipher: &self.cipher,
            fetcher: &self.fetcher,
            provider_id,
        })
        .await
    }

    async fn update_api_key(&self, provider_id: &str, key_id: &str, input: ProviderApiKeyUpdate) -> ProviderResult<ProviderApiKey> {
        ensure_provider(&self.repository, provider_id).await?;
        let input = sanitize_api_key_update(input);
        validate_api_key_update(&input)?;
        if input.is_active == Some(true)
            && let Some(key) = self.repository.quick_import_sync_key(provider_id, key_id).await?
            && has_hard_quick_import_status(&key.statuses)
        {
            return Err(ProviderError::InvalidInput(
                "quick import key has unresolved upstream sync anomalies; use quick import resolution".into(),
            ));
        }
        if let Some(api_formats) = &input.api_formats {
            ensure_api_formats_bound(&self.repository, provider_id, api_formats).await?;
        }
        if let Some(allowed_model_ids) = &input.allowed_model_ids {
            ensure_allowed_models_bound(&self.repository, provider_id, allowed_model_ids).await?;
        }
        let encrypted = input.api_key.as_deref().map(|api_key| self.cipher.encrypt_provider_key(api_key)).transpose()?;
        self.repository.update_api_key(provider_id, key_id, input, encrypted).await
    }

    async fn batch_update_api_key_priorities(&self, input: ProviderApiKeyPriorityBatchUpdate) -> ProviderResult<Vec<ProviderApiKey>> {
        validate_api_key_priority_batch(&input)?;
        self.repository.batch_update_api_key_priorities(input).await
    }

    async fn delete_api_key(&self, provider_id: &str, key_id: &str) -> ProviderResult<()> {
        ensure_provider(&self.repository, provider_id).await?;
        self.repository.delete_api_key(provider_id, key_id).await
    }

    async fn create_model_binding(&self, provider_id: &str, input: ProviderModelBindingCreate) -> ProviderResult<ProviderModelBinding> {
        let input = prepare_model_binding_create(&self.repository, &self.models, provider_id, input).await?;
        self.repository.create_model_binding(provider_id, input).await
    }

    async fn batch_update_model_bindings(&self, provider_id: &str, input: ProviderModelBindingBatchUpdate) -> ProviderResult<Vec<ProviderModelBinding>> {
        let input = prepare_model_binding_batch_update(&self.repository, &self.models, provider_id, input).await?;
        self.repository.batch_update_model_bindings(provider_id, input).await
    }

    async fn list_model_bindings(&self, provider_id: &str) -> ProviderResult<Vec<ProviderModelBinding>> {
        ensure_provider(&self.repository, provider_id).await?;
        self.repository.list_model_bindings(provider_id).await
    }

    async fn update_model_binding(&self, provider_id: &str, model_id: &str, input: ProviderModelBindingUpdate) -> ProviderResult<ProviderModelBinding> {
        ensure_provider(&self.repository, provider_id).await?;
        let input = sanitize_model_binding_update(input);
        validate_model_binding_update(&input)?;
        self.repository.update_model_binding(provider_id, model_id, input).await
    }

    async fn delete_model_binding(&self, provider_id: &str, model_id: &str) -> ProviderResult<()> {
        ensure_provider(&self.repository, provider_id).await?;
        self.repository.delete_model_binding(provider_id, model_id).await
    }

    async fn list_model_costs(&self, provider_id: &str) -> ProviderResult<ProviderModelCostListResponse> {
        ensure_provider(&self.repository, provider_id).await?;
        self.repository.list_model_costs(provider_id).await
    }

    async fn upsert_model_costs(&self, provider_id: &str, key_id: &str, input: ProviderModelCostBatchUpsert) -> ProviderResult<ProviderModelCostListResponse> {
        ensure_provider(&self.repository, provider_id).await?;
        let input = sanitize_model_cost_batch(input);
        validate_model_cost_batch(&input)?;
        ensure_model_cost_scope(&self.repository, provider_id, key_id, &input).await?;
        self.repository.upsert_model_costs(provider_id, key_id, input).await
    }

    async fn preview_quick_import(&self, input: ProviderQuickImportPreviewRequest) -> ProviderResult<ProviderQuickImportPreviewResponse> {
        preview_quick_import(
            QuickImportArgs {
                repository: &self.repository,
                models: &self.models,
                cipher: &self.cipher,
                importer: &self.importer,
            },
            input,
        )
        .await
    }

    async fn commit_quick_import(&self, input: ProviderQuickImportCommitRequest) -> ProviderResult<ProviderQuickImportCommitResponse> {
        commit_quick_import(
            QuickImportArgs {
                repository: &self.repository,
                models: &self.models,
                cipher: &self.cipher,
                importer: &self.importer,
            },
            input,
        )
        .await
    }

    async fn preview_quick_import_append(
        &self,
        provider_id: &str,
        input: ProviderQuickImportAppendPreviewRequest,
    ) -> ProviderResult<ProviderQuickImportPreviewResponse> {
        preview_quick_import_append(
            QuickImportArgs {
                repository: &self.repository,
                models: &self.models,
                cipher: &self.cipher,
                importer: &self.importer,
            },
            provider_id,
            input,
        )
        .await
    }

    async fn commit_quick_import_append(
        &self,
        provider_id: &str,
        input: ProviderQuickImportAppendCommitRequest,
    ) -> ProviderResult<ProviderQuickImportCommitResponse> {
        commit_quick_import_append(
            QuickImportArgs {
                repository: &self.repository,
                models: &self.models,
                cipher: &self.cipher,
                importer: &self.importer,
            },
            provider_id,
            input,
        )
        .await
    }

    async fn quick_import_resolution(&self, provider_id: &str, key_id: &str) -> ProviderResult<ProviderQuickImportResolutionResponse> {
        quick_import_resolution(
            QuickImportArgs {
                repository: &self.repository,
                models: &self.models,
                cipher: &self.cipher,
                importer: &self.importer,
            },
            provider_id,
            key_id,
        )
        .await
    }

    async fn accept_quick_import_current(&self, provider_id: &str, key_id: &str) -> ProviderResult<ProviderApiKey> {
        accept_quick_import_current(
            QuickImportArgs {
                repository: &self.repository,
                models: &self.models,
                cipher: &self.cipher,
                importer: &self.importer,
            },
            provider_id,
            key_id,
        )
        .await
    }

    async fn relink_quick_import_key(&self, provider_id: &str, key_id: &str, input: ProviderQuickImportRelinkRequest) -> ProviderResult<ProviderApiKey> {
        relink_quick_import_key(
            QuickImportArgs {
                repository: &self.repository,
                models: &self.models,
                cipher: &self.cipher,
                importer: &self.importer,
            },
            provider_id,
            key_id,
            input,
        )
        .await
    }

    async fn quick_import_model_associations(&self, provider_id: &str, key_id: &str) -> ProviderResult<ProviderQuickImportModelAssociationsResponse> {
        quick_import_model_associations(
            QuickImportArgs {
                repository: &self.repository,
                models: &self.models,
                cipher: &self.cipher,
                importer: &self.importer,
            },
            provider_id,
            key_id,
        )
        .await
    }

    async fn update_quick_import_model_associations(
        &self,
        provider_id: &str,
        key_id: &str,
        input: ProviderQuickImportModelAssociationsUpdate,
    ) -> ProviderResult<ProviderQuickImportModelAssociationsResponse> {
        update_quick_import_model_associations(
            QuickImportArgs {
                repository: &self.repository,
                models: &self.models,
                cipher: &self.cipher,
                importer: &self.importer,
            },
            provider_id,
            key_id,
            input,
        )
        .await
    }

    async fn quick_import_sync_settings(&self, provider_id: &str) -> ProviderResult<ProviderQuickImportSyncSettingsResponse> {
        quick_import_sync_settings(&self.repository, provider_id).await
    }

    async fn update_quick_import_sync_settings(
        &self,
        provider_id: &str,
        input: ProviderQuickImportSyncSettingsUpdate,
    ) -> ProviderResult<ProviderQuickImportSyncSettingsResponse> {
        update_quick_import_sync_settings(&self.repository, &self.cipher, provider_id, input).await
    }

    async fn run_quick_import_sync(&self, options: ProviderQuickImportSyncRunOptions) -> ProviderResult<ProviderQuickImportSyncRunReport> {
        run_quick_import_sync(
            SyncArgs {
                repository: &self.repository,
                models: &self.models,
                cipher: &self.cipher,
                importer: &self.importer,
            },
            options,
        )
        .await
    }

    async fn delete_model_cost(&self, provider_id: &str, key_id: &str, provider_model_id: &str) -> ProviderResult<()> {
        ensure_provider(&self.repository, provider_id).await?;
        ensure_model_cost_delete_scope(&self.repository, provider_id, key_id, provider_model_id).await?;
        self.repository.delete_model_cost(provider_id, key_id, provider_model_id).await
    }

    async fn list_request_records(&self, request: RequestRecordListRequest) -> ProviderResult<RequestRecordListResponse> {
        validate_request_record_list_request(&request)?;
        self.repository.list_request_records(request).await
    }

    async fn list_usage_records(&self, user_id: &str, request: RequestRecordListRequest) -> ProviderResult<UsageRecordListResponse> {
        if user_id.trim().is_empty() {
            return Err(ProviderError::InvalidInput("user_id cannot be blank".into()));
        }
        validate_request_record_list_request(&request)?;
        self.repository.list_usage_records(user_id, request).await
    }

    async fn list_active_request_records(&self, request: ActiveRequestRecordRequest) -> ProviderResult<ActiveRequestRecordResponse> {
        let request = sanitize_active_request_record_request(request);
        self.repository.list_active_request_records(request).await
    }

    async fn get_request_record(&self, request_id: &str) -> ProviderResult<RequestRecordDetail> {
        if request_id.trim().is_empty() {
            return Err(ProviderError::InvalidInput("request_id cannot be blank".into()));
        }
        self.repository.get_request_record(request_id).await
    }

    async fn list_provider_cooldowns(&self, request: ProviderCooldownListRequest) -> ProviderResult<ProviderCooldownListResponse> {
        let request = sanitize_provider_cooldown_request(request);
        validate_provider_cooldown_request(&request)?;
        self.repository.list_provider_cooldowns(request).await
    }

    async fn release_provider_cooldown(&self, provider_id: &str) -> ProviderResult<ProviderCooldown> {
        if provider_id.trim().is_empty() {
            return Err(ProviderError::InvalidInput("provider_id cannot be blank".into()));
        }
        self.repository.release_provider_cooldown(provider_id).await
    }
}
