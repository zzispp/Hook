use std::collections::BTreeMap;

use types::{
    model::GlobalModelResponse,
    provider::{
        ProviderApiKeyUpdate, ProviderModelCostBatchUpsert, ProviderQuickImportFetchFailureAction, ProviderQuickImportSourceConfig,
        ProviderQuickImportSyncStatus,
    },
};

use crate::application::{
    GlobalModelCatalog, ProviderError, ProviderQuickImportSyncKey, ProviderQuickImportSyncKeyPatch, ProviderQuickImportSyncRunOptions,
    ProviderQuickImportSyncRunReport, ProviderQuickImportSyncSource, ProviderRepository, ProviderResult, SecretCipher, UpstreamProviderImportSource,
    UpstreamSyncSnapshot,
};

use super::{
    quick_import_shared::restore_source_config,
    quick_import_sync_bindings::{BindingInfo, bindings_by_global},
    quick_import_sync_events::{key_events, source_failure_event, source_failure_key_event},
    quick_import_sync_globals::globals_by_id,
    quick_import_sync_outcome::key_outcome,
};

pub struct SyncArgs<'a, R, M, C, I> {
    pub repository: &'a R,
    pub models: &'a M,
    pub cipher: &'a C,
    pub importer: &'a I,
}

struct KeySyncContext<'a, R, M, C, I> {
    args: &'a SyncArgs<'a, R, M, C, I>,
    source: &'a ProviderQuickImportSyncSource,
    source_config: &'a ProviderQuickImportSourceConfig,
    snapshot: &'a UpstreamSyncSnapshot,
    globals: &'a BTreeMap<String, GlobalModelResponse>,
    bindings: &'a BTreeMap<String, BindingInfo>,
}

pub async fn run_quick_import_sync<R, M, C, I>(
    args: SyncArgs<'_, R, M, C, I>,
    options: ProviderQuickImportSyncRunOptions,
) -> ProviderResult<ProviderQuickImportSyncRunReport>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let sources = args.repository.list_quick_import_sync_sources(options.limit).await?;
    let mut report = ProviderQuickImportSyncRunReport {
        scanned_count: sources.len() as u64,
        ..ProviderQuickImportSyncRunReport::default()
    };
    for source in sources {
        let error_context = source_error_context(&source);
        sync_source(&args, source, &mut report)
            .await
            .map_err(|error| sync_source_error(error_context, error))?;
    }
    Ok(report)
}

fn source_error_context(source: &ProviderQuickImportSyncSource) -> String {
    format!(
        "provider \"{}\" (provider_id={}, source_id={})",
        source.provider_name, source.provider_id, source.id
    )
}

fn sync_source_error(context: String, error: ProviderError) -> ProviderError {
    let message = match error {
        ProviderError::Infrastructure(message) => message,
        other => other.to_string(),
    };
    ProviderError::Infrastructure(format!("provider quick import sync failed for {context}: {message}"))
}

async fn sync_source<R, M, C, I>(
    args: &SyncArgs<'_, R, M, C, I>,
    source: ProviderQuickImportSyncSource,
    report: &mut ProviderQuickImportSyncRunReport,
) -> ProviderResult<()>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let source_config = source_config(args.cipher, &source)?;
    let refreshed = match args.importer.refreshed_source_config(&source_config).await {
        Ok(refreshed) => refreshed.unwrap_or(source_config.clone()),
        Err(error) => return handle_source_failure(args.repository, source, error, report).await,
    };
    args.repository
        .update_quick_import_sync_source(
            &source.provider_id,
            super::quick_import_shared::refreshed_source_patch(args.cipher, &refreshed)?,
        )
        .await?;
    let snapshot = match args.importer.fetch_sync_snapshot(&refreshed).await {
        Ok(snapshot) => snapshot,
        Err(error) => return handle_source_failure(args.repository, source, error, report).await,
    };
    let keys = args.repository.list_quick_import_sync_keys(&source.id).await?;
    let globals = globals_by_id(args.models.list_global_models().await?);
    let bindings = bindings_by_global(args.repository.list_model_bindings(&source.provider_id).await?);
    let context = KeySyncContext {
        args,
        source: &source,
        source_config: &refreshed,
        snapshot: &snapshot,
        globals: &globals,
        bindings: &bindings,
    };
    for key in keys {
        sync_key(&context, key, report).await?;
    }
    args.repository
        .update_quick_import_sync_source_run(&source.id, Some(ProviderQuickImportSyncStatus::Ok), None, false)
        .await?;
    report.synced_count += 1;
    Ok(())
}

async fn sync_key<R, M, C, I>(
    context: &KeySyncContext<'_, R, M, C, I>,
    key: ProviderQuickImportSyncKey,
    report: &mut ProviderQuickImportSyncRunReport,
) -> ProviderResult<()>
where
    R: ProviderRepository,
    M: GlobalModelCatalog,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let outcome = key_outcome(
        context.args.importer,
        context.source,
        context.source_config,
        context.snapshot,
        context.globals,
        context.bindings,
        &key,
    )
    .await;
    if outcome.disable_key {
        disable_key(context.args.repository, &key).await?;
        report.disabled_key_count += 1;
    }
    let events = key_events(context.source, &key, &outcome);
    let updated_cost_count = outcome.costs.as_ref().map_or(0, Vec::len);
    if let Some(costs) = outcome.costs.clone() {
        context
            .args
            .repository
            .upsert_model_costs(&key.provider_id, &key.key_id, ProviderModelCostBatchUpsert { costs })
            .await?;
        report.updated_cost_count += updated_cost_count as u64;
    }
    context
        .args
        .repository
        .update_quick_import_sync_keys(&key.provider_id, vec![outcome.patch(key.key_id.clone())])
        .await?;
    context.args.repository.create_quick_import_sync_events(events).await
}

async fn handle_source_failure<R>(
    repository: &R,
    source: ProviderQuickImportSyncSource,
    error: ProviderError,
    report: &mut ProviderQuickImportSyncRunReport,
) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    let disable = source.sync_config.fetch_failure_action == ProviderQuickImportFetchFailureAction::DisableAfterFailures
        && source.consecutive_failures + 1 >= source.sync_config.fetch_failure_disable_threshold;
    let notify_source = source.last_status != Some(ProviderQuickImportSyncStatus::SourceFetchFailed);
    if notify_source {
        repository
            .create_quick_import_sync_events(vec![source_failure_event(&source, &error, disable)])
            .await?;
    }
    if disable {
        fail_source_keys(repository, &source, true, report).await?;
    } else {
        fail_source_keys(repository, &source, false, report).await?;
    }
    repository
        .update_quick_import_sync_source_run(
            &source.id,
            Some(ProviderQuickImportSyncStatus::SourceFetchFailed),
            Some(error.to_string()),
            true,
        )
        .await?;
    report.failed_count += 1;
    Ok(())
}

async fn fail_source_keys<R>(
    repository: &R,
    source: &ProviderQuickImportSyncSource,
    disable: bool,
    report: &mut ProviderQuickImportSyncRunReport,
) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    let mut patches = Vec::new();
    for key in repository.list_quick_import_sync_keys(&source.id).await? {
        if disable {
            disable_key(repository, &key).await?;
            report.disabled_key_count += 1;
        }
        patches.push(ProviderQuickImportSyncKeyPatch {
            key_id: key.key_id.clone(),
            statuses: vec![ProviderQuickImportSyncStatus::SourceFetchFailed],
            upstream_group_id: None,
            upstream_group: None,
            upstream_group_ratio: None,
            effective_cost_multiplier: None,
            last_error: None,
        });
        if disable && !key.statuses.contains(&ProviderQuickImportSyncStatus::SourceFetchFailed) {
            repository.create_quick_import_sync_events(vec![source_failure_key_event(source, &key)]).await?;
        }
    }
    repository.update_quick_import_sync_keys(&source.provider_id, patches).await?;
    Ok(())
}

async fn disable_key<R>(repository: &R, key: &ProviderQuickImportSyncKey) -> ProviderResult<()>
where
    R: ProviderRepository,
{
    repository
        .update_api_key(
            &key.provider_id,
            &key.key_id,
            ProviderApiKeyUpdate {
                is_active: Some(false),
                ..ProviderApiKeyUpdate::default()
            },
            None,
        )
        .await?;
    Ok(())
}

fn source_config<C>(cipher: &C, source: &ProviderQuickImportSyncSource) -> ProviderResult<ProviderQuickImportSourceConfig>
where
    C: SecretCipher,
{
    restore_source_config(cipher, source)
}

#[cfg(test)]
mod tests {
    use crate::application::{ProviderError, ProviderQuickImportSyncSource};

    use super::{source_error_context, sync_source_error};

    #[test]
    fn sync_source_error_includes_source_identity_and_original_error() {
        let source = sync_source_record();
        let error = sync_source_error(
            source_error_context(&source),
            ProviderError::Infrastructure("sub2api returned 401 Unauthorized: {\"code\":401,\"message\":\"invalid refresh token\"}".into()),
        );

        assert_eq!(
            error.to_string(),
            "infrastructure error: provider quick import sync failed for provider \"OpenAI\" (provider_id=provider-1, source_id=source-1): sub2api returned 401 Unauthorized: {\"code\":401,\"message\":\"invalid refresh token\"}"
        );
    }

    fn sync_source_record() -> ProviderQuickImportSyncSource {
        ProviderQuickImportSyncSource {
            id: "source-1".into(),
            provider_id: "provider-1".into(),
            provider_name: "OpenAI".into(),
            source_kind: types::provider::ProviderQuickImportSourceKind::Newapi,
            base_url: "https://newapi.example".into(),
            encrypted_system_access_token: "enc".into(),
            email: String::new(),
            encrypted_password: String::new(),
            encrypted_auth_token: String::new(),
            encrypted_refresh_token: String::new(),
            token_expires_at: None,
            user_id: "737".into(),
            recharge_multiplier: rust_decimal::Decimal::ONE,
            sync_config: types::provider::ProviderQuickImportSyncConfig::default(),
            last_status: None,
            last_error: None,
            last_synced_at: None,
            consecutive_failures: u32::default(),
        }
    }
}
