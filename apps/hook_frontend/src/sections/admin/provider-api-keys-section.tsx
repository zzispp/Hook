'use client';

import type { Theme } from '@mui/material/styles';
import type { ProviderKeyGroup } from 'src/types/provider-group';
import type { Provider, ProviderApiKey } from 'src/types/provider';
import type { useProviderChildDialogs } from './provider-management-state';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import { EmptyList } from './provider-bindings-shared';
import { ProviderApiKeyRow } from './provider-api-key-row';
import { providerKeyGroupNamesByKey } from './provider-groups-utils';

export function ProviderApiKeysSection({
  provider,
  items,
  loading,
  providerKeyGroups,
  dialogs,
  onAppendQuickImport,
  onResolveQuickImportKey,
  onManageQuickImportModels,
  onAssociateGroups,
}: {
  provider: Provider;
  items: ProviderApiKey[];
  loading: boolean;
  providerKeyGroups: ProviderKeyGroup[];
  dialogs: ReturnType<typeof useProviderChildDialogs>;
  onAppendQuickImport: (provider: Provider) => void;
  onResolveQuickImportKey: (apiKey: ProviderApiKey) => void;
  onManageQuickImportModels: (apiKey: ProviderApiKey) => void;
  onAssociateGroups: (apiKey: ProviderApiKey) => void;
}) {
  const { t } = useTranslate('admin');
  const groupNamesByKey = providerKeyGroupNamesByKey(providerKeyGroups);

  return (
    <Box sx={panelSx}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" sx={headerSx}>
        <Typography variant="subtitle2">{t('providers.keyManagement')}</Typography>
        <HeaderActions provider={provider} dialogs={dialogs} onAppendQuickImport={onAppendQuickImport} />
      </Stack>
      <Box sx={listSx}>
        {items.map((apiKey) => (
          <ProviderApiKeyRow
            key={apiKey.id}
            apiKey={apiKey}
            groupNames={groupNamesByKey.get(apiKey.id) ?? []}
            dialogs={dialogs}
            onResolveQuickImportKey={onResolveQuickImportKey}
            onManageQuickImportModels={onManageQuickImportModels}
            onAssociateGroups={onAssociateGroups}
          />
        ))}
        <EmptyList loading={loading} length={items.length} />
      </Box>
    </Box>
  );
}

function HeaderActions({
  provider,
  dialogs,
  onAppendQuickImport,
}: {
  provider: Provider;
  dialogs: ReturnType<typeof useProviderChildDialogs>;
  onAppendQuickImport: (provider: Provider) => void;
}) {
  const { t } = useTranslate('admin');
  return (
    <Stack direction="row" spacing={1}>
      {provider.provider_origin === 'quick_import' ? (
        <Button
          color="inherit"
          size="small"
          variant="outlined"
          startIcon={<Iconify icon="solar:import-bold" width={14} />}
          onClick={() => onAppendQuickImport(provider)}
        >
          {t('actions.quickImportAppendTokens')}
        </Button>
      ) : null}
      <Button
        color="inherit"
        size="small"
        variant="outlined"
        startIcon={<Iconify icon="mingcute:add-line" width={14} />}
        onClick={dialogs.openCreateApiKey}
      >
        {t('actions.addProviderKey')}
      </Button>
    </Stack>
  );
}

const panelSx = { border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`, borderRadius: 2, overflow: 'hidden' };
const headerSx = { p: 2, borderBottom: (theme: Theme) => `1px solid ${theme.vars.palette.divider}` };
const listSx = { '& > * + *': { borderTop: (theme: Theme) => `1px solid ${theme.vars.palette.divider}` } };
