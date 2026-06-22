'use client';

import type { Theme } from '@mui/material/styles';
import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderApiKey, ProviderKeyModelMappingsByKey } from 'src/types/provider';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';
import { useProviderKeyModelMappings } from 'src/actions/providers';

import { Iconify } from 'src/components/iconify';

import { EmptyList } from './provider-bindings-shared';

type Props = {
  providerId: string;
  apiKeys: ProviderApiKey[];
  models: GlobalModelResponse[];
  onManageKeyModels: (providerId: string, apiKey: ProviderApiKey) => void;
};

export function ProviderKeyModelMappingsSection({ providerId, apiKeys, models, onManageKeyModels }: Props) {
  const { t } = useTranslate('admin');
  const mappings = useProviderKeyModelMappings(providerId);

  return (
    <Box sx={panelSx}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" sx={headerSx}>
        <Box>
          <Typography variant="subtitle2">{t('providers.keyModelMappingsTitle')}</Typography>
          <Typography variant="caption" color="text.secondary">
            {t('providers.reasoningEffortHelper')}
          </Typography>
        </Box>
      </Stack>
      <Box>
        {mappings.items.map((item, index) => (
          <Box key={item.key_id}>
            {index > 0 ? <Divider /> : null}
            <KeySection
              item={item}
              apiKey={apiKeys.find((key) => key.id === item.key_id) ?? null}
              models={models}
              onManageKeyModels={onManageKeyModels}
            />
          </Box>
        ))}
        <EmptyList loading={mappings.isLoading} length={mappings.items.length} />
      </Box>
    </Box>
  );
}

function KeySection({
  item,
  apiKey,
  models,
  onManageKeyModels,
}: {
  item: ProviderKeyModelMappingsByKey;
  apiKey: ProviderApiKey | null;
  models: GlobalModelResponse[];
  onManageKeyModels: (providerId: string, apiKey: ProviderApiKey) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={1.5} sx={sectionSx}>
      <Stack direction="row" alignItems="flex-start" justifyContent="space-between" spacing={2}>
        <Box sx={{ minWidth: 0 }}>
          <Stack direction="row" spacing={1} alignItems="center" useFlexGap flexWrap="wrap">
            <Typography variant="subtitle2" noWrap>
              {item.key_name}
            </Typography>
            <Chip
              size="small"
              color={item.is_quick_import_key ? 'info' : 'default'}
              variant={item.is_quick_import_key ? 'soft' : 'outlined'}
              label={item.is_quick_import_key ? t('providers.quickImportLinkedToken') : t('providers.keyManagement')}
            />
          </Stack>
          <Typography variant="caption" color="text.secondary" sx={{ fontFamily: 'monospace' }}>
            {item.key_id}
          </Typography>
        </Box>
        <Button
          color="inherit"
          size="small"
          variant="outlined"
          startIcon={<Iconify icon="solar:settings-bold" width={14} />}
          disabled={!apiKey}
          onClick={() => apiKey && onManageKeyModels(item.provider_id, apiKey)}
        >
          {t('common.edit')}
        </Button>
      </Stack>
      {item.mappings.length > 0 ? (
        <Stack spacing={1}>
          {item.mappings.map((mapping) => (
            <MappingRow key={mapping.id} item={mapping} models={models} />
          ))}
        </Stack>
      ) : (
        <Typography variant="body2" color="text.secondary">
          {t('providers.quickImportNoAssociatedModels')}
        </Typography>
      )}
    </Stack>
  );
}

function MappingRow({
  item,
  models,
}: {
  item: ProviderKeyModelMappingsByKey['mappings'][number];
  models: GlobalModelResponse[];
}) {
  const { t } = useTranslate('admin');
  const model = models.find((entry) => entry.id === item.global_model_id);
  const title = model?.display_name || model?.name || item.global_model_id;
  const secondary = model?.display_name && model?.name ? model.name : model?.name || item.global_model_id;

  return (
    <Box sx={rowSx}>
      <Typography variant="subtitle2">{title}</Typography>
      <Typography variant="caption" sx={{ color: 'text.secondary' }}>
        {secondary}
      </Typography>
      <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap sx={{ mt: 1 }}>
        <Chip size="small" label={item.upstream_model_name} sx={{ fontFamily: 'monospace' }} />
        {item.reasoning_effort ? (
          <Chip size="small" label={`${t('providers.reasoningEffort')}: ${item.reasoning_effort}`} />
        ) : null}
      </Stack>
    </Box>
  );
}

const panelSx = {
  border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
  borderRadius: 2,
  overflow: 'hidden',
};

const headerSx = {
  px: 2,
  py: 1.5,
  borderBottom: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
};

const sectionSx = {
  px: 2,
  py: 1.5,
};

const rowSx = {
  border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
  borderRadius: 1.5,
  p: 1.5,
};
