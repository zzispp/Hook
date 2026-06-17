'use client';

import type { ProviderQuickImportSyncEventDetailResponse } from 'src/types/provider-quick-import';

import Chip from '@mui/material/Chip';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableHead from '@mui/material/TableHead';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';
import TableContainer from '@mui/material/TableContainer';

import { useTranslate } from 'src/locales/use-locales';

export function QuickImportEventContent({
  event,
}: {
  event: ProviderQuickImportSyncEventDetailResponse;
}) {
  const { t } = useTranslate('admin');
  const payload = event.payload;

  return (
    <Stack spacing={3}>
      <QuickImportEventHeader event={event} />

      {payload ? (
        <QuickImportPayloadSections payload={payload} />
      ) : (
        <Alert severity="warning">{t('providers.quickImportEventNoSnapshot')}</Alert>
      )}
      <Typography color="text.secondary">{event.detail}</Typography>
    </Stack>
  );
}

function QuickImportPayloadSections({
  payload,
}: {
  payload: NonNullable<ProviderQuickImportSyncEventDetailResponse['payload']>;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={2}>
      <ContextRow
        label={t('providers.quickImportProviderSection')}
        value={payload.provider_name}
      />
      <ContextRow
        label={t('providers.quickImportBindLocalKey')}
        value={payload.local_key_name || '-'}
      />
      <ContextRow
        label={t('providers.quickImportEventUpstreamToken')}
        value={formatUpstreamToken(payload.upstream_token_name, payload.upstream_token_id)}
      />
      <ContextRow
        label={t('providers.quickImportEventAnomalySummary')}
        value={payload.anomaly_summary}
      />
      <ContextRow
        label={t('providers.quickImportEventActionSummary')}
        value={payload.action_summary}
      />
      {payload.missing_upstream_model_ids.length ? (
        <ModelTable
          title={t('providers.quickImportEventMissingModels')}
          head={[t('providers.quickImportEventMissingModelId')]}
          rows={payload.missing_upstream_model_ids.map((id) => [id])}
        />
      ) : null}
      {payload.upstream_models_snapshot.length ? (
        <ModelTable
          title={t('providers.quickImportEventUpstreamModelsSnapshot')}
          head={[
            t('providers.quickImportEventMissingModelId'),
            t('providers.quickImportEventSupportedEndpointTypes'),
          ]}
          rows={payload.upstream_models_snapshot.map((model) => [
            model.upstream_model_id,
            model.supported_endpoint_types.join(', '),
          ])}
        />
      ) : null}
    </Stack>
  );
}

function QuickImportEventHeader({
  event,
}: {
  event: ProviderQuickImportSyncEventDetailResponse;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={1}>
      <Typography variant="h5">{event.title}</Typography>
      <Chip
        size="small"
        color={event.snapshot_status === 'available' ? 'success' : 'warning'}
        label={
          event.snapshot_status === 'available'
            ? t('providers.quickImportSyncStatus.upstream_model_removed')
            : t('common.warning')
        }
        sx={{ alignSelf: 'flex-start' }}
      />
      <Typography variant="caption" color="text.disabled">
        {event.created_at}
      </Typography>
    </Stack>
  );
}

function ContextRow({ label, value }: { label: string; value: string }) {
  return (
    <Stack spacing={0.5}>
      <Typography variant="subtitle2">{label}</Typography>
      <Typography color="text.secondary">{value}</Typography>
    </Stack>
  );
}

function ModelTable({
  title,
  head,
  rows,
}: {
  title: string;
  head: string[];
  rows: string[][];
}) {
  return (
    <Stack spacing={1}>
      <Typography variant="subtitle2">{title}</Typography>
      <TableContainer sx={{ border: 1, borderColor: 'divider', borderRadius: 1.5 }}>
        <Table size="small">
          <TableHead>
            <TableRow>
              {head.map((cell) => (
                <TableCell key={cell}>{cell}</TableCell>
              ))}
            </TableRow>
          </TableHead>
          <TableBody>
            {rows.map((row, rowIndex) => (
              <TableRow key={`${title}-${rowIndex}`}>
                {row.map((cell, cellIndex) => (
                  <TableCell key={`${title}-${rowIndex}-${cellIndex}`}>{cell || '-'}</TableCell>
                ))}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    </Stack>
  );
}

function formatUpstreamToken(name?: string | null, id?: string | null) {
  if (!name && !id) {
    return '-';
  }

  if (!id) {
    return name || '-';
  }

  return `${name || '-'} (${id})`;
}
