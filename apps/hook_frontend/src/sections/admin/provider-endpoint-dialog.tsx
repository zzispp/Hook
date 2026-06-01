'use client';

import type { EndpointManager } from './provider-endpoint-manager';
import type { Provider, ProviderEndpoint } from 'src/types/provider';
import type { useProviderChildDialogs } from './provider-management-state';

import Stack from '@mui/material/Stack';
import Dialog from '@mui/material/Dialog';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/locales/use-locales';
import { useProviderEndpoints } from 'src/actions/providers';

import { useEndpointManager } from './provider-endpoint-manager';
import { ProviderEndpointCard, editStateFromEndpoint } from './provider-endpoint-card';
import { ProviderEndpointQuickAddCard, ProviderEndpointSingleAddCard } from './provider-endpoint-add-card';

export function ProviderEndpointDialog({
  dialogs,
  provider,
}: {
  dialogs: ReturnType<typeof useProviderChildDialogs>;
  provider?: Provider;
}) {
  const { t } = useTranslate('admin');
  const endpointQuery = useProviderEndpoints(dialogs.endpointOpen ? provider?.id : null);
  const manager = useEndpointManager(provider, endpointQuery.items, endpointQuery.refresh);

  return (
    <Dialog fullWidth maxWidth="md" open={dialogs.endpointOpen} onClose={dialogs.closeEndpoint}>
      <DialogTitle>
        {t('providers.endpointManagement')}
        <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
          {t('providers.endpointManagementDescription', { name: provider?.name ?? '' })}
        </Typography>
      </DialogTitle>
      <DialogContent dividers sx={{ px: 3, py: 2 }}>
        <Stack spacing={2}>
          <ProviderEndpointQuickAddCard
            form={manager.quickAddForm}
            adding={manager.quickAdding}
            existingEndpoints={endpointQuery.items}
            onFormChange={manager.setQuickAddForm}
            onApiFormatsChange={manager.setQuickAddApiFormats}
            onAdd={() => void manager.quickAddEndpoints()}
          />
          <ConfiguredEndpoints loading={endpointQuery.isLoading} manager={manager} endpoints={endpointQuery.items} />
          <ProviderEndpointSingleAddCard
            form={manager.addForm}
            rulesOpen={manager.addRulesOpen}
            adding={manager.adding}
            existingEndpoints={endpointQuery.items}
            onFormChange={manager.setAddForm}
            onApiFormatChange={manager.setAddApiFormat}
            onRulesOpenChange={manager.setAddRulesOpen}
            onHeaderRulesChange={manager.setAddHeaderRules}
            onBodyRulesChange={manager.setAddBodyRules}
            onAdd={() => void manager.addEndpoint()}
          />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={dialogs.closeEndpoint}>{t('common.close')}</Button>
      </DialogActions>
    </Dialog>
  );
}

function ConfiguredEndpoints({
  loading,
  manager,
  endpoints,
}: {
  loading: boolean;
  manager: EndpointManager;
  endpoints: ProviderEndpoint[];
}) {
  const { t } = useTranslate('admin');
  if (loading) return <Typography variant="body2" color="text.secondary">{t('common.loading')}</Typography>;
  return (
    <Stack spacing={1.5}>
      <Typography variant="caption" sx={labelSx}>{t('providers.configuredEndpoints')}</Typography>
      {endpoints.map((endpoint) => (
        <ProviderEndpointCard
          key={endpoint.id}
          endpoint={endpoint}
          editState={manager.editStates[endpoint.id] ?? editStateFromEndpoint(endpoint)}
          expanded={manager.expanded[endpoint.id] ?? false}
          saving={manager.busy?.id === endpoint.id && manager.busy.action === 'save'}
          deleting={manager.busy?.id === endpoint.id && manager.busy.action === 'delete'}
          toggling={manager.busy?.id === endpoint.id && manager.busy.action === 'toggle'}
          onEditStateChange={(state) => manager.setEditState(endpoint.id, state)}
          onExpandedChange={(open) => manager.setExpanded(endpoint.id, open)}
          onSave={(payload) => void manager.saveEndpoint(endpoint, payload)}
          onDelete={() => void manager.deleteEndpoint(endpoint)}
          onToggle={() => void manager.toggleEndpoint(endpoint)}
        />
      ))}
      {!endpoints.length && <Typography variant="body2" color="text.secondary">{t('common.noData')}</Typography>}
    </Stack>
  );
}

const labelSx = { color: 'text.secondary', fontWeight: 700, textTransform: 'uppercase', letterSpacing: 1 };
