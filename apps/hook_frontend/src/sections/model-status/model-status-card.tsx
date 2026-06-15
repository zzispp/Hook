import type { TFunction } from 'i18next';
import type { ModelStatusCheck } from 'src/types/model-status';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import { alpha } from '@mui/material/styles';
import Skeleton from '@mui/material/Skeleton';
import Typography from '@mui/material/Typography';

import { Iconify } from 'src/components/iconify';

import { StatusLabel } from './model-status-label';
import { latencyLabel, ModelStatusTimeline } from './model-status-timeline';

export function ModelStatusCard({ row, t }: { row: ModelStatusCheck; t: TFunction<'admin'> }) {
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
      <CornerGridLines position="left" />
      <CornerGridLines position="right" />
      <StatusHeader row={row} providerIcon={providerIcon} t={t} />
      <StatusMetrics row={row} t={t} />
      <AvailabilityLine row={row} t={t} />
      <Box sx={{ width: 1 }}>
        <ModelStatusTimeline points={row.timeline} t={t} />
      </Box>
    </Card>
  );
}

export function ModelStatusLoadingSkeleton() {
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

function CornerGridLines({ position }: { position: 'left' | 'right' }) {
  return (
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
        top: 8,
        [position]: 8,
        transition: (theme) => theme.transitions.create(['opacity']),
        '.MuiCard-root:hover &': { opacity: 0.15 },
      }}
    >
      <line x1="12" y1="0" x2="12" y2="24" />
      <line x1="0" y1="12" x2="24" y2="12" />
    </Box>
  );
}

function StatusHeader({
  row,
  providerIcon,
  t,
}: {
  row: ModelStatusCheck;
  providerIcon: string;
  t: TFunction<'admin'>;
}) {
  return (
    <Stack direction="row" spacing={2} alignItems="center">
      <ProviderIcon icon={providerIcon} />
      <Stack spacing={0.5} sx={{ flex: 1, minWidth: 0 }}>
        <Stack direction="row" spacing={1} alignItems="center" justifyContent="space-between">
          <Typography variant="subtitle1" noWrap sx={{ fontWeight: 'bold' }}>
            {row.name}
          </Typography>
          <StatusLabel status={row.last_status} t={t} />
        </Stack>
        <Stack
          direction="row"
          spacing={1}
          alignItems="center"
          sx={{ typography: 'caption', color: 'text.secondary' }}
        >
          <Box component="span" sx={apiFormatBadgeSx}>
            {row.api_format}
          </Box>
          <Typography variant="caption" noWrap>
            {row.model_name}
          </Typography>
        </Stack>
      </Stack>
    </Stack>
  );
}

function ProviderIcon({ icon }: { icon: string }) {
  return (
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
      <Iconify icon={icon as any} width={24} />
    </Box>
  );
}

function StatusMetrics({ row, t }: { row: ModelStatusCheck; t: TFunction<'admin'> }) {
  return (
    <Grid container spacing={2} sx={metricsGridSx}>
      <MetricItem
        icon="solar:bolt-bold"
        color="warning.main"
        label={t('modelStatusChecks.latency')}
        value={latencyLabel(row.last_latency_ms)}
      />
      <MetricItem
        icon="solar:clock-circle-bold"
        color="info.main"
        label={t('modelStatusChecks.interval')}
        value={formatInterval(row.interval_seconds)}
      />
    </Grid>
  );
}

function MetricItem({
  icon,
  color,
  label,
  value,
}: {
  icon: string;
  color: string;
  label: string;
  value: string;
}) {
  return (
    <Grid size={{ xs: 6 }}>
      <Stack direction="row" spacing={1} alignItems="center">
        <Iconify icon={icon as any} sx={{ color }} width={18} />
        <Stack spacing={0.25}>
          <Typography variant="caption" color="text.secondary">
            {label}
          </Typography>
          <Typography variant="subtitle2">{value}</Typography>
        </Stack>
      </Stack>
    </Grid>
  );
}

function AvailabilityLine({ row, t }: { row: ModelStatusCheck; t: TFunction<'admin'> }) {
  return (
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

function formatInterval(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.round(minutes / 60);
  return `${hours}h`;
}

const apiFormatBadgeSx = {
  px: 0.75,
  py: 0.25,
  borderRadius: 0.5,
  bgcolor: 'action.hover',
  border: (theme: any) => `1px solid ${theme.palette.divider}`,
};

const metricsGridSx = {
  py: 1.5,
  borderTop: (theme: any) => `1px dashed ${alpha(theme.palette.divider, 0.4)}`,
  borderBottom: (theme: any) => `1px dashed ${alpha(theme.palette.divider, 0.4)}`,
};
