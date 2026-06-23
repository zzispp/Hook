import type { ProviderQuickImportSyncStatus } from './provider-quick-import';

export type ProviderQuickImportSyncIssueScope = 'source' | 'key';
export type ProviderQuickImportSyncIssueSeverity = 'info' | 'warning' | 'error';

export type ProviderQuickImportSyncIssue = {
  scope: ProviderQuickImportSyncIssueScope;
  status: ProviderQuickImportSyncStatus;
  severity: ProviderQuickImportSyncIssueSeverity;
  key_id?: string | null;
  key_name?: string | null;
  message?: string | null;
  last_synced_at?: string | null;
};

export type ProviderQuickImportSyncSummary = {
  severity: ProviderQuickImportSyncIssueSeverity;
  issue_count: number;
  affected_key_count: number;
  last_synced_at?: string | null;
  issues: ProviderQuickImportSyncIssue[];
};
