use crate::application::{
    ProviderQuickImportTokenRefreshRunOptions, ProviderQuickImportTokenRefreshRunReport, ProviderRepository, ProviderResult, SecretCipher,
    UpstreamProviderImportSource,
};
use types::provider::ProviderQuickImportSourceConfig;

use super::{
    quick_import_shared::{refreshed_sub2api_token_patch, restore_source_config},
    quick_import_sync::SyncArgs,
};

pub async fn run_quick_import_token_refresh<R, M, C, I>(
    args: SyncArgs<'_, R, M, C, I>,
    options: ProviderQuickImportTokenRefreshRunOptions,
) -> ProviderResult<ProviderQuickImportTokenRefreshRunReport>
where
    R: ProviderRepository,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let sources = args.repository.list_sub2api_token_refresh_sources(options.limit).await?;
    let mut report = ProviderQuickImportTokenRefreshRunReport {
        scanned_count: sources.len() as u64,
        ..ProviderQuickImportTokenRefreshRunReport::default()
    };
    for source in sources {
        if let Err(error) = refresh_source(&args, source.clone(), &mut report, options.refresh_threshold_minutes).await {
            report.failed_count += 1;
            hook_tracing::warn_with_fields!(
                "sub2api quick import token refresh skipped source after failure",
                provider_id = source.provider_id,
                source_id = source.id,
                error = error
            );
        }
    }
    Ok(report)
}

async fn refresh_source<R, M, C, I>(
    args: &SyncArgs<'_, R, M, C, I>,
    source: crate::application::ProviderQuickImportSyncSource,
    report: &mut ProviderQuickImportTokenRefreshRunReport,
    refresh_threshold_minutes: i64,
) -> ProviderResult<()>
where
    R: ProviderRepository,
    C: SecretCipher,
    I: UpstreamProviderImportSource,
{
    let source_config = source_config(args.cipher, &source)?;
    let refreshed = args
        .importer
        .refreshed_source_config_with_threshold(&source_config, refresh_threshold_minutes)
        .await?;
    if let Some(refreshed) = refreshed {
        let changed = source_changed(&source_config, &refreshed);
        if !changed {
            report.skipped_count += 1;
            return Ok(());
        }
        args.repository
            .update_quick_import_sync_source(&source.provider_id, refreshed_sub2api_token_patch(args.cipher, &refreshed)?)
            .await?;
        report.refreshed_count += 1;
        return Ok(());
    }
    report.skipped_count += 1;
    Ok(())
}

fn source_config<C>(cipher: &C, source: &crate::application::ProviderQuickImportSyncSource) -> ProviderResult<ProviderQuickImportSourceConfig>
where
    C: SecretCipher,
{
    restore_source_config(cipher, source)
}

fn source_changed(current: &ProviderQuickImportSourceConfig, refreshed: &ProviderQuickImportSourceConfig) -> bool {
    current != refreshed
}
