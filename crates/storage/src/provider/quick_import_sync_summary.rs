use std::collections::{BTreeMap, BTreeSet};

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use time::format_description::well_known::Rfc3339;
use types::provider::{
    Provider, ProviderOrigin, ProviderQuickImportSyncIssue, ProviderQuickImportSyncIssueScope, ProviderQuickImportSyncIssueSeverity,
    ProviderQuickImportSyncStatus, ProviderQuickImportSyncSummary,
};

use crate::{StorageError, StorageResult, json};

use super::{
    ProviderStore,
    record::{provider_api_keys, provider_quick_import_keys, provider_quick_import_sources},
};

pub async fn summaries_by_provider(store: &ProviderStore, providers: &[Provider]) -> StorageResult<BTreeMap<String, ProviderQuickImportSyncSummary>> {
    let provider_ids = quick_import_provider_ids(providers);
    if provider_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let sources = sources_by_provider(store, &provider_ids).await?;
    let keys = keys_by_provider(store, &provider_ids).await?;
    let key_names = key_names(store, &keys).await?;
    let mut issues = source_issues(&provider_ids, &sources)?;
    add_key_issues(&mut issues, &sources, keys, key_names)?;
    Ok(summaries(issues))
}

fn quick_import_provider_ids(providers: &[Provider]) -> Vec<String> {
    providers
        .iter()
        .filter(|provider| provider.provider_origin == ProviderOrigin::QuickImport)
        .map(|provider| provider.id.clone())
        .collect()
}

async fn sources_by_provider(store: &ProviderStore, provider_ids: &[String]) -> StorageResult<BTreeMap<String, provider_quick_import_sources::Model>> {
    let records = provider_quick_import_sources::Entity::find()
        .filter(provider_quick_import_sources::Column::ProviderId.is_in(provider_ids.to_owned()))
        .all(store.connection())
        .await?;
    Ok(records.into_iter().map(|record| (record.provider_id.clone(), record)).collect())
}

async fn keys_by_provider(store: &ProviderStore, provider_ids: &[String]) -> StorageResult<Vec<provider_quick_import_keys::Model>> {
    provider_quick_import_keys::Entity::find()
        .filter(provider_quick_import_keys::Column::ProviderId.is_in(provider_ids.to_owned()))
        .all(store.connection())
        .await
        .map_err(StorageError::from)
}

async fn key_names(store: &ProviderStore, keys: &[provider_quick_import_keys::Model]) -> StorageResult<BTreeMap<(String, String), String>> {
    let provider_ids = keys.iter().map(|key| key.provider_id.clone()).collect::<BTreeSet<_>>();
    let key_ids = keys.iter().map(|key| key.key_id.clone()).collect::<BTreeSet<_>>();
    if provider_ids.is_empty() || key_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let records = provider_api_keys::Entity::find()
        .filter(provider_api_keys::Column::ProviderId.is_in(provider_ids))
        .filter(provider_api_keys::Column::Id.is_in(key_ids))
        .all(store.connection())
        .await?;
    Ok(records.into_iter().map(|record| ((record.provider_id, record.id), record.name)).collect())
}

fn source_issues(
    provider_ids: &[String],
    sources: &BTreeMap<String, provider_quick_import_sources::Model>,
) -> StorageResult<BTreeMap<String, Vec<ProviderQuickImportSyncIssue>>> {
    let mut output = BTreeMap::new();
    for provider_id in provider_ids {
        let Some(source) = sources.get(provider_id) else {
            output.insert(
                provider_id.clone(),
                vec![source_issue(ProviderQuickImportSyncStatus::SourceNotConfigured, None, None)],
            );
            continue;
        };
        let mut issues = Vec::new();
        if !source.auto_sync_enabled {
            issues.push(source_issue(
                ProviderQuickImportSyncStatus::SyncDisabled,
                None,
                source.last_synced_at.map(format_timestamp).transpose()?,
            ));
        }
        if source_status(source)? == Some(ProviderQuickImportSyncStatus::SourceFetchFailed) {
            issues.push(source_issue(
                ProviderQuickImportSyncStatus::SourceFetchFailed,
                source.last_error.clone(),
                source.last_synced_at.map(format_timestamp).transpose()?,
            ));
        }
        if !issues.is_empty() {
            output.insert(provider_id.clone(), issues);
        }
    }
    Ok(output)
}

fn add_key_issues(
    output: &mut BTreeMap<String, Vec<ProviderQuickImportSyncIssue>>,
    sources: &BTreeMap<String, provider_quick_import_sources::Model>,
    keys: Vec<provider_quick_import_keys::Model>,
    key_names: BTreeMap<(String, String), String>,
) -> StorageResult<()> {
    for key in keys {
        if source_hides_key_statuses(sources.get(&key.provider_id))? {
            continue;
        }
        let statuses = json::decode_required::<Vec<ProviderQuickImportSyncStatus>>(key.sync_statuses.clone())?;
        for status in statuses {
            if skip_key_status(status) {
                continue;
            }
            let issue = key_issue(status, &key, &key_names)?;
            output.entry(key.provider_id.clone()).or_default().push(issue);
        }
    }
    Ok(())
}

fn source_issue(status: ProviderQuickImportSyncStatus, message: Option<String>, last_synced_at: Option<String>) -> ProviderQuickImportSyncIssue {
    ProviderQuickImportSyncIssue {
        scope: ProviderQuickImportSyncIssueScope::Source,
        status,
        severity: severity(status),
        key_id: None,
        key_name: None,
        message,
        last_synced_at,
    }
}

fn key_issue(
    status: ProviderQuickImportSyncStatus,
    key: &provider_quick_import_keys::Model,
    key_names: &BTreeMap<(String, String), String>,
) -> StorageResult<ProviderQuickImportSyncIssue> {
    Ok(ProviderQuickImportSyncIssue {
        scope: ProviderQuickImportSyncIssueScope::Key,
        status,
        severity: severity(status),
        key_id: Some(key.key_id.clone()),
        key_name: key_names.get(&(key.provider_id.clone(), key.key_id.clone())).cloned(),
        message: key.last_sync_error.clone(),
        last_synced_at: key.last_synced_at.map(format_timestamp).transpose()?,
    })
}

fn skip_key_status(status: ProviderQuickImportSyncStatus) -> bool {
    status == ProviderQuickImportSyncStatus::Ok
}

fn source_hides_key_statuses(source: Option<&provider_quick_import_sources::Model>) -> StorageResult<bool> {
    let Some(source) = source else {
        return Ok(true);
    };
    Ok(!source.auto_sync_enabled || source_status(source)? == Some(ProviderQuickImportSyncStatus::SourceFetchFailed))
}

fn source_status(source: &provider_quick_import_sources::Model) -> StorageResult<Option<ProviderQuickImportSyncStatus>> {
    source
        .last_status
        .as_deref()
        .map(ProviderQuickImportSyncStatus::try_from)
        .transpose()
        .map_err(StorageError::Database)
}

fn summaries(issues: BTreeMap<String, Vec<ProviderQuickImportSyncIssue>>) -> BTreeMap<String, ProviderQuickImportSyncSummary> {
    issues
        .into_iter()
        .filter_map(|(provider_id, issues)| summary(issues).map(|summary| (provider_id, summary)))
        .collect()
}

fn summary(issues: Vec<ProviderQuickImportSyncIssue>) -> Option<ProviderQuickImportSyncSummary> {
    if issues.is_empty() {
        return None;
    }
    let affected_key_count = affected_key_count(&issues);
    let last_synced_at = latest_synced_at(&issues);
    Some(ProviderQuickImportSyncSummary {
        severity: max_severity(&issues),
        issue_count: issues.len() as u32,
        affected_key_count,
        last_synced_at,
        issues,
    })
}

fn affected_key_count(issues: &[ProviderQuickImportSyncIssue]) -> u32 {
    issues.iter().filter_map(|issue| issue.key_id.as_deref()).collect::<BTreeSet<_>>().len() as u32
}

fn latest_synced_at(issues: &[ProviderQuickImportSyncIssue]) -> Option<String> {
    issues.iter().filter_map(|issue| issue.last_synced_at.as_ref()).max().cloned()
}

fn max_severity(issues: &[ProviderQuickImportSyncIssue]) -> ProviderQuickImportSyncIssueSeverity {
    if issues.iter().any(|issue| issue.severity == ProviderQuickImportSyncIssueSeverity::Error) {
        return ProviderQuickImportSyncIssueSeverity::Error;
    }
    if issues.iter().any(|issue| issue.severity == ProviderQuickImportSyncIssueSeverity::Warning) {
        return ProviderQuickImportSyncIssueSeverity::Warning;
    }
    ProviderQuickImportSyncIssueSeverity::Info
}

fn severity(status: ProviderQuickImportSyncStatus) -> ProviderQuickImportSyncIssueSeverity {
    match status {
        ProviderQuickImportSyncStatus::SourceFetchFailed
        | ProviderQuickImportSyncStatus::UpstreamTokenDeleted
        | ProviderQuickImportSyncStatus::UpstreamTokenDisabled
        | ProviderQuickImportSyncStatus::UpstreamGroupRemoved
        | ProviderQuickImportSyncStatus::UpstreamGroupChanged
        | ProviderQuickImportSyncStatus::UpstreamModelRemoved
        | ProviderQuickImportSyncStatus::NoAssociatedModels => ProviderQuickImportSyncIssueSeverity::Error,
        ProviderQuickImportSyncStatus::UpstreamKeyUnavailable
        | ProviderQuickImportSyncStatus::CostUnavailable
        | ProviderQuickImportSyncStatus::CostPendingUpdate
        | ProviderQuickImportSyncStatus::ModelCandidateAvailable => ProviderQuickImportSyncIssueSeverity::Warning,
        ProviderQuickImportSyncStatus::SyncDisabled | ProviderQuickImportSyncStatus::SourceNotConfigured => ProviderQuickImportSyncIssueSeverity::Info,
        ProviderQuickImportSyncStatus::Ok => ProviderQuickImportSyncIssueSeverity::Info,
    }
}

fn format_timestamp(value: time::OffsetDateTime) -> StorageResult<String> {
    value
        .format(&Rfc3339)
        .map_err(|error| StorageError::Database(format!("quick import sync summary timestamp format failed: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(status: ProviderQuickImportSyncStatus) -> ProviderQuickImportSyncIssue {
        source_issue(status, None, None)
    }

    #[test]
    fn summary_uses_highest_severity() {
        let summary = summary(vec![
            issue(ProviderQuickImportSyncStatus::CostPendingUpdate),
            issue(ProviderQuickImportSyncStatus::UpstreamGroupRemoved),
        ])
        .expect("summary should exist");

        assert_eq!(summary.severity, ProviderQuickImportSyncIssueSeverity::Error);
        assert_eq!(summary.issue_count, 2);
    }

    #[test]
    fn ok_status_is_not_a_key_issue() {
        assert!(skip_key_status(ProviderQuickImportSyncStatus::Ok));
    }

    #[test]
    fn source_fetch_failed_key_status_is_not_skipped_without_source_context() {
        assert!(!skip_key_status(ProviderQuickImportSyncStatus::SourceFetchFailed));
    }

    #[test]
    fn planned_warning_statuses_are_warning() {
        for status in [
            ProviderQuickImportSyncStatus::UpstreamKeyUnavailable,
            ProviderQuickImportSyncStatus::CostUnavailable,
            ProviderQuickImportSyncStatus::CostPendingUpdate,
            ProviderQuickImportSyncStatus::ModelCandidateAvailable,
        ] {
            assert_eq!(severity(status), ProviderQuickImportSyncIssueSeverity::Warning);
        }
    }
}
