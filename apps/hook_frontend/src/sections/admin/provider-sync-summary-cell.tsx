'use client';

import type { ChipProps } from '@mui/material/Chip';
import type { Provider } from 'src/types/provider';
import type { ProviderQuickImportSyncIssue } from 'src/types/provider-quick-import-summary';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Tooltip from '@mui/material/Tooltip';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { quickImportSyncStatusColor } from './provider-quick-import-status-utils';

const SUMMARY_PREVIEW_LIMIT = 2;

export function ProviderSyncSummaryCell({ provider }: { provider: Provider }) {
  const { t } = useTranslate('admin');
  const summary = provider.quick_import_sync_summary;

  if (provider.provider_origin !== 'quick_import') {
    return <Typography variant="body2">-</Typography>;
  }
  if (!summary || summary.issue_count === 0) {
    return (
      <Chip
        size="small"
        color="success"
        variant="soft"
        label={t('providers.quickImportSyncNormal')}
      />
    );
  }

  return (
    <Tooltip arrow title={<IssueTooltip issues={summary.issues} />}>
      <Stack spacing={0.5} sx={{ width: 180 }}>
        <Chip
          size="small"
          color={summarySeverityColor(summary.severity)}
          variant="soft"
          label={summaryLabel(summary.severity, summary.issue_count, t)}
          sx={{ width: 'fit-content' }}
        />
        {summary.issues.slice(0, SUMMARY_PREVIEW_LIMIT).map((issue) => (
          <Typography key={issueKey(issue)} noWrap variant="caption" color="text.secondary">
            {issuePreview(issue, t)}
          </Typography>
        ))}
      </Stack>
    </Tooltip>
  );
}

function IssueTooltip({ issues }: { issues: ProviderQuickImportSyncIssue[] }) {
  const { t } = useTranslate('admin');
  return (
    <Stack spacing={0.75} sx={{ maxWidth: 420 }}>
      {issues.map((issue) => (
        <Box key={issueKey(issue)}>
          <Stack direction="row" spacing={0.75} alignItems="center">
            <Typography variant="caption" sx={{ minWidth: 0, fontWeight: 600 }}>
              {issueTarget(issue, t)}
            </Typography>
            <Chip
              size="small"
              color={quickImportSyncStatusColor(issue.status)}
              variant="soft"
              label={t(`providers.quickImportSyncStatus.${issue.status}`)}
              sx={{ height: 20 }}
            />
          </Stack>
          {issue.message ? (
            <Typography variant="caption" sx={{ display: 'block', color: 'inherit' }}>
              {issue.message}
            </Typography>
          ) : null}
        </Box>
      ))}
    </Stack>
  );
}

function issuePreview(issue: ProviderQuickImportSyncIssue, t: (key: string) => string) {
  return `${issueTarget(issue, t)} · ${t(`providers.quickImportSyncStatus.${issue.status}`)}`;
}

function issueTarget(issue: ProviderQuickImportSyncIssue, t: (key: string) => string) {
  if (issue.scope === 'source') return t('providers.quickImportSyncSource');
  return issue.key_name || issue.key_id || t('providers.quickImportSyncUnknownKey');
}

function issueKey(issue: ProviderQuickImportSyncIssue) {
  return `${issue.scope}:${issue.key_id ?? 'source'}:${issue.status}`;
}

function summaryLabel(
  severity: ProviderQuickImportSyncIssue['severity'],
  count: number,
  t: (key: string, options?: Record<string, unknown>) => string
) {
  if (severity === 'error') return t('providers.quickImportSyncErrorCount', { count });
  if (severity === 'warning') return t('providers.quickImportSyncWarningCount', { count });
  return t('providers.quickImportSyncInfoCount', { count });
}

function summarySeverityColor(
  severity: ProviderQuickImportSyncIssue['severity']
): ChipProps['color'] {
  if (severity === 'error') return 'error';
  if (severity === 'warning') return 'warning';
  return 'info';
}
