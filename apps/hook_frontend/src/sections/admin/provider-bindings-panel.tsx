'use client';

import type { Theme } from '@mui/material/styles';
import type { Provider } from 'src/types/provider';
import type { GlobalModelResponse } from 'src/types/model';
import type { useProviderChildDialogs } from './provider-management-state';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import List from '@mui/material/List';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Drawer from '@mui/material/Drawer';
import Divider from '@mui/material/Divider';
import ListItem from '@mui/material/ListItem';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import ListItemText from '@mui/material/ListItemText';

import { useTranslate } from 'src/locales/use-locales';
import { useProviderModels, useProviderApiKeys, useProviderEndpoints } from 'src/actions/providers';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';

import { EnabledLabel } from './shared';
import { EmptyList } from './provider-bindings-shared';
import { ProviderApiKeysSection } from './provider-api-keys-section';
import { ProviderModelBindingsSection } from './provider-model-bindings-section';
import { formatApiFormat, defaultEndpointPath } from './provider-management-utils';

export function ProviderBindingsPanel({
  open,
  provider,
  models,
  onClose,
  dialogs,
}: {
  open: boolean;
  provider?: Provider;
  models: GlobalModelResponse[];
  onClose: () => void;
  dialogs: ReturnType<typeof useProviderChildDialogs>;
}) {
  const { t } = useTranslate('admin');
  const endpoints = useProviderEndpoints(provider?.id);
  const apiKeys = useProviderApiKeys(provider?.id);
  const providerModels = useProviderModels(provider?.id);

  return (
    <Drawer anchor="right" open={open} onClose={onClose} slotProps={drawerSlotProps}>
      <DrawerHeader title={provider?.name ?? t('providers.modelBindings')} onClose={onClose} />
      <Scrollbar>
        {provider ? (
          <ProviderBindingsContent
            providerId={provider.id}
            endpointItems={endpoints.items}
            endpointLoading={endpoints.isLoading}
            apiKeyItems={apiKeys.items}
            apiKeyLoading={apiKeys.isLoading}
            providerModelItems={providerModels.items}
            providerModelLoading={providerModels.isLoading}
            models={models}
            dialogs={dialogs}
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
  providerId: string;
  endpointItems: ReturnType<typeof useProviderEndpoints>['items'];
  endpointLoading: boolean;
  apiKeyItems: ReturnType<typeof useProviderApiKeys>['items'];
  apiKeyLoading: boolean;
  providerModelItems: ReturnType<typeof useProviderModels>['items'];
  providerModelLoading: boolean;
  models: GlobalModelResponse[];
  dialogs: ReturnType<typeof useProviderChildDialogs>;
};

function ProviderBindingsContent(props: ProviderBindingsContentProps) {
  return (
    <Stack spacing={2} sx={contentSx}>
      <EndpointSection
        items={props.endpointItems}
        loading={props.endpointLoading}
        onAdd={() => props.dialogs.setEndpointOpen(true)}
      />
      <ApiKeySection
        items={props.apiKeyItems}
        loading={props.apiKeyLoading}
        dialogs={props.dialogs}
      />
      <ProviderModelSection
        providerId={props.providerId}
        items={props.providerModelItems}
        loading={props.providerModelLoading}
        models={props.models}
        onAdd={() => props.dialogs.setModelOpen(true)}
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
    <PanelSection
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
    </PanelSection>
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

function ApiKeySection({
  items,
  loading,
  dialogs,
}: {
  items: ReturnType<typeof useProviderApiKeys>['items'];
  loading: boolean;
  dialogs: ReturnType<typeof useProviderChildDialogs>;
}) {
  return <ProviderApiKeysSection items={items} loading={loading} dialogs={dialogs} />;
}

function ProviderModelSection({
  providerId,
  items,
  loading,
  models,
  onAdd,
}: {
  providerId: string;
  items: ReturnType<typeof useProviderModels>['items'];
  loading: boolean;
  models: GlobalModelResponse[];
  onAdd: () => void;
}) {
  return <ProviderModelBindingsSection providerId={providerId} items={items} loading={loading} models={models} onAssociate={onAdd} />;
}

function PanelSection({
  title,
  actionLabel,
  children,
  onAdd,
}: {
  title: string;
  actionLabel: string;
  children: React.ReactNode;
  onAdd: () => void;
}) {
  return (
    <Box sx={{ border: (theme) => `1px solid ${theme.vars.palette.divider}`, borderRadius: 1 }}>
      <Stack
        direction="row"
        alignItems="center"
        justifyContent="space-between"
        sx={{ px: 2, py: 1.5 }}
      >
        <Typography variant="subtitle2">{title}</Typography>
        <Button
          color="inherit"
          variant="contained"
          startIcon={<Iconify icon="mingcute:add-line" />}
          onClick={onAdd}
        >
          {actionLabel}
        </Button>
      </Stack>
      <Divider />
      <Box sx={{ px: 2, py: 1 }}>{children}</Box>
    </Box>
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
