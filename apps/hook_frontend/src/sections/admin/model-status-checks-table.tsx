'use client';

import type { TFunction } from 'i18next';
import type { ModelStatusCheck } from 'src/types/model-status';
import type { ModelStatusCheckFormState } from './model-status-check-form';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import Tooltip from '@mui/material/Tooltip';
import { alpha } from '@mui/material/styles';
import Checkbox from '@mui/material/Checkbox';
import Skeleton from '@mui/material/Skeleton';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { updateModelStatusCheck } from 'src/actions/model-status';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import { StatusLabel } from '../model-status/model-status-label';
import { intervalLabel, modelStatusCheckFormFromRow } from './model-status-check-form';
import { latencyLabel, ModelStatusTimeline } from '../model-status/model-status-timeline';

type Props = {
  rows: ModelStatusCheck[];
  loading: boolean;
  selected: string[];
  t: TFunction<'admin'>;
  onEdit: (form: ModelStatusCheckFormState) => void;
  onDelete: (row: ModelStatusCheck) => void;
  onSelectRow: (id: string) => void;
  onSelectAllRows: (checked: boolean, ids: string[]) => void;
};

export function ModelStatusChecksTable(props: Props) {
  if (props.loading) {
    return <LoadingSkeleton t={props.t} />;
  }

  if (props.rows.length === 0) {
    return (
      <Box sx={{ py: 10, textAlign: 'center' }}>
        <Typography variant="h6" color="text.secondary">
          {props.t('modelStatusChecks.empty')}
        </Typography>
      </Box>
    );
  }

  return (
    <Grid container spacing={3}>
      {props.rows.map((row) => (
        <Grid key={row.id} size={{ xs: 12, md: 6 }}>
          <CheckCard row={row} {...props} />
        </Grid>
      ))}
    </Grid>
  );
}

function CheckCard({
  row,
  t,
  selected,
  onEdit,
  onDelete,
  onSelectRow,
}: Omit<Props, 'rows' | 'loading' | 'onSelectAllRows'> & { row: ModelStatusCheck }) {
  const checked = selected.includes(row.id);
  const providerIcon = getProviderIcon(row.api_format);

  return (
    <Card
      sx={{
        position: 'relative',
        p: 3,
        display: 'flex',
        flexDirection: 'column',
        gap: 2.5,
        overflow: 'hidden',
        borderRadius: 2,
        border: (theme) => `1px solid ${alpha(theme.palette.divider, 0.4)}`,
        bgcolor: 'background.paper',
        transition: (theme) => theme.transitions.create(['transform', 'box-shadow', 'border-color']),
        '&:hover': {
          transform: 'translateY(-4px)',
          boxShadow: (theme) => theme.vars?.customShadows?.z12 ?? theme.customShadows?.z12,
          borderColor: (theme) => alpha(theme.palette.primary.main, 0.4),
        },
      }}
    >
      {/* Decorative Grid Lines SVGs (only visible on hover) */}
      <Box
        component="svg"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1"
        sx={{
          position: 'absolute',
          height: 16,
          width: 16,
          color: 'text.disabled',
          opacity: 0,
          left: 8,
          top: 8,
          transition: (theme) => theme.transitions.create(['opacity']),
          '.MuiCard-root:hover &': { opacity: 0.15 },
        }}
      >
        <line x1="12" y1="0" x2="12" y2="24" />
        <line x1="0" y1="12" x2="24" y2="12" />
      </Box>

      <Box
        component="svg"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1"
        sx={{
          position: 'absolute',
          height: 16,
          width: 16,
          color: 'text.disabled',
          opacity: 0,
          right: 8,
          top: 8,
          transition: (theme) => theme.transitions.create(['opacity']),
          '.MuiCard-root:hover &': { opacity: 0.15 },
        }}
      >
        <line x1="12" y1="0" x2="12" y2="24" />
        <line x1="0" y1="12" x2="24" y2="12" />
      </Box>

      {/* Top Controls Overlay */}
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', zIndex: 2 }}>
        <Checkbox
          size="small"
          checked={checked}
          onClick={() => onSelectRow(row.id)}
          sx={{ p: 0.5 }}
        />

        <Stack direction="row" spacing={1} alignItems="center">
          <Switch
            size="small"
            checked={row.enabled}
            onChange={(event) => void toggleEnabled(row, event.target.checked, t)}
          />
          <Tooltip title={t('common.edit') || 'Edit'}>
            <IconButton size="small" onClick={() => onEdit(modelStatusCheckFormFromRow(row))}>
              <Iconify icon="solar:pen-bold" width={18} />
            </IconButton>
          </Tooltip>
          <Tooltip title={t('common.delete') || 'Delete'}>
            <IconButton size="small" color="error" onClick={() => onDelete(row)}>
              <Iconify icon="solar:trash-bin-trash-bold" width={18} />
            </IconButton>
          </Tooltip>
        </Stack>
      </Box>

      {/* Main Info Section: Logo & Name */}
      <Stack direction="row" spacing={2} alignItems="center" sx={{ mt: 0.5 }}>
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            width: 44,
            height: 44,
            borderRadius: 1.5,
            border: (theme) => `1px solid ${alpha(theme.palette.divider, 0.6)}`,
            bgcolor: (theme) => alpha(theme.palette.background.neutral, 0.4),
          }}
        >
          <Iconify icon={providerIcon as any} width={24} />
        </Box>
        <Stack spacing={0.5} sx={{ flex: 1, minWidth: 0 }}>
          <Stack direction="row" spacing={1} alignItems="center" justifyContent="space-between">
            <Typography variant="subtitle1" noWrap sx={{ fontWeight: 'bold' }}>
              {row.name}
            </Typography>
            <StatusLabel status={row.last_status} t={t} />
          </Stack>
          <Stack direction="row" spacing={1} alignItems="center" sx={{ typography: 'caption', color: 'text.secondary' }}>
            <Box component="span" sx={{ px: 0.75, py: 0.25, borderRadius: 0.5, bgcolor: 'action.hover', border: (theme) => `1px solid ${theme.palette.divider}` }}>
              {row.api_format}
            </Box>
            <Typography variant="caption" noWrap>
              {row.model_name}
            </Typography>
          </Stack>
        </Stack>
      </Stack>

      {/* 2-Column Stats Grid */}
      <Grid container spacing={2} sx={{ py: 1.5, borderTop: (theme) => `1px dashed ${alpha(theme.palette.divider, 0.4)}`, borderBottom: (theme) => `1px dashed ${alpha(theme.palette.divider, 0.4)}` }}>
        <Grid size={{ xs: 6 }}>
          <Stack direction="row" spacing={1} alignItems="center">
            <Iconify icon={"solar:bolt-bold" as any} sx={{ color: 'warning.main' }} width={18} />
            <Stack spacing={0.25}>
              <Typography variant="caption" color="text.secondary">
                {t('modelStatusChecks.latency')}
              </Typography>
              <Typography variant="subtitle2">
                {latencyLabel(row.last_latency_ms)}
              </Typography>
            </Stack>
          </Stack>
        </Grid>
        <Grid size={{ xs: 6 }}>
          <Stack direction="row" spacing={1} alignItems="center">
            <Iconify icon="solar:clock-circle-bold" sx={{ color: 'info.main' }} width={18} />
            <Stack spacing={0.25}>
              <Typography variant="caption" color="text.secondary">
                {t('modelStatusChecks.interval')}
              </Typography>
              <Typography variant="subtitle2">
                {intervalLabel(row.interval_seconds)}
              </Typography>
            </Stack>
          </Stack>
        </Grid>
      </Grid>

      {/* Uptime availability percentage and ratios */}
      <Stack direction="row" justifyContent="space-between" alignItems="center">
        <Typography variant="caption" color="text.secondary">
          {t('modelStatus.availability')}
        </Typography>
        <Stack direction="row" spacing={1} alignItems="center">
          <Typography variant="subtitle2" color="primary.main">
            {row.availability.availability_pct ? `${row.availability.availability_pct}%` : '-'}
          </Typography>
          <Typography variant="caption" color="text.secondary">
            ({row.availability.available_checks}/{row.availability.total_checks})
          </Typography>
        </Stack>
      </Stack>

      {/* History Timeline */}
      <Box sx={{ width: 1 }}>
        <ModelStatusTimeline points={row.timeline} t={t} />
      </Box>
    </Card>
  );
}

function LoadingSkeleton({ t }: { t: TFunction<'admin'> }) {
  return (
    <Grid container spacing={3}>
      {Array.from({ length: 4 }).map((_, idx) => (
        <Grid key={idx} size={{ xs: 12, md: 6 }}>
          <Card sx={{ p: 3, height: 220, display: 'flex', flexDirection: 'column', gap: 2 }}>
            <Stack direction="row" spacing={2} alignItems="center">
              <Skeleton variant="circular" width={40} height={40} />
              <Stack spacing={0.5} sx={{ flex: 1 }}>
                <Skeleton variant="text" width="60%" height={24} />
                <Skeleton variant="text" width="30%" height={16} />
              </Stack>
            </Stack>
            <Skeleton variant="rectangular" width="100%" height={40} sx={{ borderRadius: 1 }} />
            <Skeleton variant="rectangular" width="100%" height={28} sx={{ borderRadius: 1 }} />
          </Card>
        </Grid>
      ))}
    </Grid>
  );
}

function getProviderIcon(apiFormat: string): string {
  const format = apiFormat.toLowerCase();
  if (format.includes('openai')) return 'logos:openai-icon';
  if (format.includes('claude') || format.includes('anthropic')) return 'logos:anthropic-icon';
  if (format.includes('gemini') || format.includes('google')) return 'logos:google-gemini';
  if (format.includes('deepseek')) return 'simple-icons:deepseek';
  if (format.includes('groq')) return 'simple-icons:groq';
  if (format.includes('cohere')) return 'logos:cohere-icon';
  if (format.includes('mistral')) return 'logos:mistral-ai-icon';
  if (format.includes('huggingface')) return 'logos:huggingface';
  if (format.includes('azure')) return 'logos:microsoft-azure';
  if (format.includes('aws') || format.includes('bedrock')) return 'logos:aws';
  return 'solar:cpu-bolt-bold';
}

async function toggleEnabled(row: ModelStatusCheck, enabled: boolean, t: TFunction<'admin'>) {
  try {
    await updateModelStatusCheck(row.id, { enabled });
    toast.success(t('modelStatusChecks.messages.updated'));
  } catch (error) {
    toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
  }
}
