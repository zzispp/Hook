'use client';

import type { DisplayApiEndpoint } from './api-token-endpoints-utils';

import { toast } from 'sonner';
import { useCopyToClipboard } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Link from '@mui/material/Link';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

type Props = {
  endpoints: DisplayApiEndpoint[];
};

export function ApiTokenEndpointsPanel({ endpoints }: Props) {
  const { t } = useTranslate('admin');

  if (endpoints.length === 0) {
    return (
      <Box sx={{ px: 2, pb: 2 }}>
        <Alert severity="warning">{t('tokens.apiEndpoints.empty')}</Alert>
      </Box>
    );
  }

  return (
    <Stack spacing={1.5} sx={{ px: 2, pb: 2 }}>
      <Typography variant="subtitle2">{t('tokens.apiEndpoints.title')}</Typography>
      <Box sx={endpointGridSx}>
        {endpoints.map((endpoint) => (
          <EndpointRow key={endpoint.id} endpoint={endpoint} />
        ))}
      </Box>
    </Stack>
  );
}

function EndpointRow({ endpoint }: { endpoint: DisplayApiEndpoint }) {
  const { t } = useTranslate('admin');
  const { copy } = useCopyToClipboard();

  function copyEndpoint() {
    copy(endpoint.url);
    toast.success(t('tokens.apiEndpoints.copied'));
  }

  return (
    <Stack
      spacing={1}
      sx={{
        p: 1.5,
        minWidth: 0,
        border: '1px solid',
        borderColor: 'divider',
        borderRadius: 1,
      }}
    >
      <EndpointName endpoint={endpoint} />
      <Stack direction="row" spacing={0.5} alignItems="center" sx={{ minWidth: 0, flexGrow: 1 }}>
        <Tooltip title={t('tokens.apiEndpoints.copy')}>
          <Chip
            clickable
            color="primary"
            size="small"
            variant="soft"
            label={endpoint.url}
            onClick={copyEndpoint}
            sx={endpointUrlChipSx}
          />
        </Tooltip>
        <Tooltip title={t('tokens.apiEndpoints.copy')}>
          <IconButton size="small" onClick={copyEndpoint}>
            <Iconify icon="solar:copy-bold" width={16} />
          </IconButton>
        </Tooltip>
        <Tooltip title={t('tokens.apiEndpoints.speedTest')}>
          <IconButton
            component={Link}
            size="small"
            href={speedTestUrl(endpoint.url)}
            target="_blank"
            rel="noopener noreferrer"
          >
            <Iconify icon="eva:external-link-fill" width={16} />
          </IconButton>
        </Tooltip>
      </Stack>
    </Stack>
  );
}

function EndpointName({ endpoint }: { endpoint: DisplayApiEndpoint }) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction="row" spacing={1} alignItems="center" sx={{ minWidth: 0 }}>
      <Typography variant="subtitle2" noWrap>
        {endpoint.name}
      </Typography>
      {endpoint.isDefault ? (
        <Chip size="small" variant="soft" color="info" label={t('tokens.apiEndpoints.defaultBadge')} />
      ) : null}
      {endpoint.description ? (
        <Tooltip title={endpoint.description}>
          <Box component="span" sx={{ display: 'inline-flex', color: 'text.secondary' }}>
            <Iconify icon="solar:info-circle-bold" width={16} />
          </Box>
        </Tooltip>
      ) : null}
    </Stack>
  );
}

function speedTestUrl(url: string) {
  return `https://www.tcptest.cn/http/${encodeURIComponent(url)}`;
}

const endpointUrlChipSx = {
  minWidth: 0,
  maxWidth: 1,
  justifyContent: 'flex-start',
  fontFamily: 'monospace',
  '& .MuiChip-label': {
    minWidth: 0,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
  },
};

const endpointGridSx = {
  display: 'grid',
  gap: 1,
  gridTemplateColumns: {
    xs: 'minmax(0, 1fr)',
    md: 'repeat(2, minmax(0, 1fr))',
  },
};
