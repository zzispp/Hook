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
import {
  useProviderModels,
  useProviderApiKeys,
  useProviderEndpoints,
} from 'src/actions/providers';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';

import { EnabledLabel } from './shared';
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
  models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[];
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
  endpointItems: ReturnType<typeof useProviderEndpoints>['items'];
  endpointLoading: boolean;
  apiKeyItems: ReturnType<typeof useProviderApiKeys>['items'];
  apiKeyLoading: boolean;
  providerModelItems: ReturnType<typeof useProviderModels>['items'];
  providerModelLoading: boolean;
  models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[];
  dialogs: ReturnType<typeof useProviderChildDialogs>;
};

function ProviderBindingsContent(props: ProviderBindingsContentProps) {
  return (
    <Stack spacing={2} sx={contentSx}>
      <EndpointSection items={props.endpointItems} loading={props.endpointLoading} onAdd={() => props.dialogs.setEndpointOpen(true)} />
      <ApiKeySection items={props.apiKeyItems} loading={props.apiKeyLoading} onAdd={() => props.dialogs.setApiKeyOpen(true)} />
      <ProviderModelSection
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
    <PanelSection title={t('providers.endpoints')} actionLabel={t('actions.manageProviderEndpoints')} onAdd={onAdd}>
      <List dense disablePadding>
        {items.map((endpoint) => (
          <ListItem key={endpoint.id} disableGutters secondaryAction={<EnabledLabel enabled={endpoint.is_active} />}>
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

function endpointPathSummary(apiFormat: string, customPath: string | null | undefined, t: (key: string) => string) {
  const custom = customPath?.trim();
  if (custom) return `${t('providers.customPath')}: ${custom}`;

  const fallback = defaultEndpointPath(apiFormat);
  return fallback ? `${t('providers.defaultPath')}: ${fallback}` : t('providers.defaultWhenBlank');
}

function ApiKeySection({
  items,
  loading,
  onAdd,
}: {
  items: ReturnType<typeof useProviderApiKeys>['items'];
  loading: boolean;
  onAdd: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <PanelSection title={t('providers.keys')} actionLabel={t('actions.addProviderKey')} onAdd={onAdd}>
      <List dense disablePadding>
        {items.map((apiKey) => (
          <ListItem key={apiKey.id} disableGutters secondaryAction={<EnabledLabel enabled={apiKey.is_active} />}>
            <ListItemText
              primary={`${apiKey.name} · ${t('providers.priority')} ${apiKey.internal_priority}`}
              secondary={keySummary(apiKey.api_formats, apiKey.rpm_limit, t)}
            />
          </ListItem>
        ))}
        <EmptyList loading={loading} length={items.length} />
      </List>
    </PanelSection>
  );
}

function ProviderModelSection({
  items,
  loading,
  models,
  onAdd,
}: {
  items: ReturnType<typeof useProviderModels>['items'];
  loading: boolean;
  models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[];
  onAdd: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <PanelSection title={t('providers.modelBindings')} actionLabel={t('actions.addProviderModel')} onAdd={onAdd}>
      <List dense disablePadding>
        {items.map((binding) => (
          <ListItem key={binding.id} disableGutters>
            <ListItemText primary={modelLabel(binding.global_model_id, models)} />
          </ListItem>
        ))}
        <EmptyList loading={loading} length={items.length} />
      </List>
    </PanelSection>
  );
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
      <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ px: 2, py: 1.5 }}>
        <Typography variant="subtitle2">{title}</Typography>
        <Button color="inherit" variant="contained" startIcon={<Iconify icon="mingcute:add-line" />} onClick={onAdd}>
          {actionLabel}
        </Button>
      </Stack>
      <Divider />
      <Box sx={{ px: 2, py: 1 }}>{children}</Box>
    </Box>
  );
}

function EmptyList({ loading, length }: { loading: boolean; length: number }) {
  const { t } = useTranslate('admin');
  if (loading) {
    return (
      <Typography variant="body2" color="text.secondary" sx={{ py: 1 }}>
        {t('common.loading')}
      </Typography>
    );
  }
  if (length > 0) return null;
  return (
    <Typography variant="body2" color="text.secondary" sx={{ py: 1 }}>
      {t('common.noData')}
    </Typography>
  );
}

function keySummary(
  formats: string[] | null | undefined,
  rpmLimit: number | null | undefined,
  t: (key: string, options?: Record<string, unknown>) => string
) {
  const formatText = formats?.length ? formats.map(formatApiFormat).join(', ') : t('providers.allFormats');
  const rpmText = rpmLimit === null || rpmLimit === undefined ? t('providers.adaptive') : `${rpmLimit} RPM`;
  return `${formatText} · ${rpmText}`;
}

function modelLabel(id: string, models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[]) {
  const model = models.find((item) => item.id === id);
  return model?.display_name || model?.name || id;
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
