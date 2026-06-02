'use client';

import type { ProviderEndpoint } from 'src/types/provider';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { formatApiFormat } from './provider-management-utils';
import { endpointGridSx, endpointButtonSx } from './provider-model-test-styles';

export function ProviderModelTestEndpointPicker({
  endpoints,
  selectedId,
  onSelect,
}: {
  endpoints: ProviderEndpoint[];
  selectedId: string;
  onSelect: (id: string) => void;
}) {
  const { t } = useTranslate('admin');
  if (endpoints.length === 0) {
    return <Typography variant="body2" color="text.secondary">{t('providers.noTestableEndpoints')}</Typography>;
  }
  return (
    <Stack spacing={1}>
      <Typography variant="subtitle2">{t('providers.selectTestEndpoint')}</Typography>
      <Box sx={endpointGridSx}>
        {endpoints.map((endpoint) => (
          <Button
            key={endpoint.id}
            variant={selectedId === endpoint.id ? 'soft' : 'outlined'}
            color={selectedId === endpoint.id ? 'primary' : 'inherit'}
            onClick={() => onSelect(endpoint.id)}
            sx={endpointButtonSx}
          >
            <Box sx={{ minWidth: 0, textAlign: 'left' }}>
              <Typography variant="subtitle2" noWrap>{formatApiFormat(endpoint.api_format)}</Typography>
              <Typography variant="caption" color="text.secondary" sx={{ wordBreak: 'break-all' }}>
                {endpoint.base_url}
              </Typography>
            </Box>
            <Chip size="small" label={selectedId === endpoint.id ? t('providers.selected') : t('common.enabled')} />
          </Button>
        ))}
      </Box>
    </Stack>
  );
}
