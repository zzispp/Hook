'use client';

import type { Theme } from '@mui/material/styles';
import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderKeyGroup } from 'src/types/provider-group';
import type { Provider, ProviderApiKey } from 'src/types/provider';
import type { useProviderChildDialogs } from './provider-management-state';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import List from '@mui/material/List';
import Stack from '@mui/material/Stack';
import Drawer from '@mui/material/Drawer';
import ListItem from '@mui/material/ListItem';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import ListItemText from '@mui/material/ListItemText';

import { useTranslate } from 'src/locales/use-locales';
import {
  useProviderModels,
  useProviderApiKeys,
  useProviderEndpoints,
  useProviderModelCosts,
} from 'src/actions/providers';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';

import { EnabledLabel } from './shared';
import { EmptyList } from './provider-bindings-shared';
import { ProviderPanelSection } from './provider-panel-section';
import { ProviderApiKeysSection } from './provider-api-keys-section';
import { ProviderModelCostsSection } from './provider-model-costs-section';
import { ProviderModelBindingsSection } from './provider-model-bindings-section';
import { ProviderModelMappingsSection } from './provider-model-mappings-section';
import { formatApiFormat, defaultEndpointPath } from './provider-management-utils';

export function ProviderBindingsPanel({
  open,
  provider,
  models,
  providerKeyGroups,
  onClose,
  dialogs,
  onAssociateKeyGroups,
  onAppendQuickImport,
  onResolveQuickImportKey,
  onManageQuickImportModels,
}: {
  open: boolean;
  provider?: Provider;
  models: GlobalModelResponse[];
  providerKeyGroups: ProviderKeyGroup[];
  onClose: () => void;
  dialogs: ReturnType<typeof useProviderChildDialogs>;
  onAssociateKeyGroups: (apiKey: ProviderApiKey) => void;
  onAppendQuickImport: (provider: Provider) => void;
  onResolveQuickImportKey: (provider: Provider, apiKey: ProviderApiKey) => void;
  onManageQuickImportModels: (provider: Provider, apiKey: ProviderApiKey) => void;
}) {
  const { t } = useTranslate('admin');
  const endpoints = useProviderEndpoints(provider?.id);
  const apiKeys = useProviderApiKeys(provider?.id);
  const providerModels = useProviderModels(provider?.id);
  const providerModelCosts = useProviderModelCosts(provider?.id);

  return (
    <Drawer anchor="right" open={open} onClose={onClose} slotProps={drawerSlotProps}>
      <DrawerHeader title={provider?.name ?? t('providers.modelBindings')} onClose={onClose} />
      <Scrollbar>
        {provider ? (
          <ProviderBindingsContent
            provider={provider}
            providerId={provider.id}
            endpointItems={endpoints.items}
            endpointLoading={endpoints.isLoading}
            apiKeyItems={apiKeys.items}
            apiKeyLoading={apiKeys.isLoading}
            providerModelItems={providerModels.items}
            providerModelLoading={providerModels.isLoading}
            providerModelCostItems={providerModelCosts.items}
            providerModelCostLoading={providerModelCosts.isLoading}
            models={models}
            providerKeyGroups={providerKeyGroups}
            dialogs={dialogs}
            onAssociateKeyGroups={onAssociateKeyGroups}
            onAppendQuickImport={onAppendQuickImport}
            onResolveQuickImportKey={onResolveQuickImportKey}
            onManageQuickImportModels={onManageQuickImportModels}
          />
        ) : null}
      </Scrollbar>
    </Drawer>
  );
}

function DrawerHeader({ title, onClose }: { title: string; onClose: () => void }) {
  return (
    <Box sx={headerSx}>
      <Typography variant="h6" noWrap sx={{ flexGrow: 1, minWidth: 0 }}>
        {title}
      </Typography>
      <IconButton onClick={onClose}>
        <Iconify icon="mingcute:close-line" />
      </IconButton>
    </Box>
  );
}

type ProviderBindingsContentProps = {
  provider: Provider;
  providerId: string;
  endpointItems: ReturnType<typeof useProviderEndpoints>['items'];
  endpointLoading: boolean;
  apiKeyItems: ReturnType<typeof useProviderApiKeys>['items'];
  apiKeyLoading: boolean;
  providerModelItems: ReturnType<typeof useProviderModels>['items'];
  providerModelLoading: boolean;
  providerModelCostItems: ReturnType<typeof useProviderModelCosts>['items'];
  providerModelCostLoading: boolean;
  models: GlobalModelResponse[];
  providerKeyGroups: ProviderKeyGroup[];
  dialogs: ReturnType<typeof useProviderChildDialogs>;
  onAssociateKeyGroups: (apiKey: ProviderApiKey) => void;
  onAppendQuickImport: (provider: Provider) => void;
  onResolveQuickImportKey: (provider: Provider, apiKey: ProviderApiKey) => void;
  onManageQuickImportModels: (provider: Provider, apiKey: ProviderApiKey) => void;
};

function ProviderBindingsContent(props: ProviderBindingsContentProps) {
  return (
    <Stack spacing={2} sx={contentSx}>
      <EndpointSection
        items={props.endpointItems}
        loading={props.endpointLoading}
        onAdd={() => props.dialogs.setEndpointOpen(true)}
      />
      <ProviderApiKeysSection
        provider={props.provider}
        items={props.apiKeyItems}
        loading={props.apiKeyLoading}
        providerKeyGroups={props.providerKeyGroups}
        dialogs={props.dialogs}
        onAppendQuickImport={props.onAppendQuickImport}
        onResolveQuickImportKey={(apiKey) => props.onResolveQuickImportKey(props.provider, apiKey)}
        onManageQuickImportModels={(apiKey) => props.onManageQuickImportModels(props.provider, apiKey)}
        onAssociateGroups={props.onAssociateKeyGroups}
      />
      <ProviderModelSection
        providerId={props.providerId}
        endpoints={props.endpointItems}
        apiKeys={props.apiKeyItems}
        items={props.providerModelItems}
        loading={props.providerModelLoading}
        models={props.models}
        onAdd={() => props.dialogs.setModelOpen(true)}
      />
      <ProviderModelMappingsSection
        providerId={props.providerId}
        items={props.providerModelItems}
        loading={props.providerModelLoading}
        models={props.models}
      />
      <ProviderModelCostsSection
        providerId={props.providerId}
        apiKeys={props.apiKeyItems}
        bindings={props.providerModelItems}
        costs={props.providerModelCostItems}
        loading={props.providerModelCostLoading}
        models={props.models}
      />
    </Stack>
  );
}

function EndpointSection({
  items,
  loading,
  onAdd,
}: {
  items: ReturnType<typeof useProviderEndpoints>['items'];
  loading: boolean;
  onAdd: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <ProviderPanelSection
      title={t('providers.endpoints')}
      actionLabel={t('actions.manageProviderEndpoints')}
      onAdd={onAdd}
    >
      <List dense disablePadding>
        {items.map((endpoint) => (
          <ListItem
            key={endpoint.id}
            disableGutters
            secondaryAction={<EnabledLabel enabled={endpoint.is_active} />}
          >
            <ListItemText
              primary={`${formatApiFormat(endpoint.api_format)} · ${endpoint.base_url}`}
              secondary={endpointPathSummary(endpoint.api_format, endpoint.custom_path, t)}
            />
          </ListItem>
        ))}
        <EmptyList loading={loading} length={items.length} />
      </List>
    </ProviderPanelSection>
  );
}

function endpointPathSummary(
  apiFormat: string,
  customPath: string | null | undefined,
  t: (key: string) => string
) {
  const custom = customPath?.trim();
  if (custom) return `${t('providers.customPath')}: ${custom}`;

  const fallback = defaultEndpointPath(apiFormat);
  return fallback ? `${t('providers.defaultPath')}: ${fallback}` : t('providers.defaultWhenBlank');
}

function ProviderModelSection({
  providerId,
  endpoints,
  apiKeys,
  items,
  loading,
  models,
  onAdd,
}: {
  providerId: string;
  endpoints: ReturnType<typeof useProviderEndpoints>['items'];
  apiKeys: ReturnType<typeof useProviderApiKeys>['items'];
  items: ReturnType<typeof useProviderModels>['items'];
  loading: boolean;
  models: GlobalModelResponse[];
  onAdd: () => void;
}) {
  return (
    <ProviderModelBindingsSection
      providerId={providerId}
      endpoints={endpoints}
      apiKeys={apiKeys}
      items={items}
      loading={loading}
      models={models}
      onAssociate={onAdd}
    />
  );
}

const drawerSlotProps = {
  backdrop: { invisible: true },
  paper: {
    sx: [
      (theme: Theme) => ({
        ...theme.mixins.paperStyles(theme, {
          color: varAlpha(theme.vars.palette.background.defaultChannel, 0.95),
        }),
        width: { xs: 1, sm: 760 },
      }),
    ],
  },
};

const headerSx = {
  py: 2,
  pr: 1,
  pl: 2.5,
  gap: 1,
  display: 'flex',
  alignItems: 'center',
};

const contentSx = {
  px: 2.5,
  pb: 5,
};
