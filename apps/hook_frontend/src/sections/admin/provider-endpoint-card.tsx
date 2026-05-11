'use client';

import type { EndpointEditState } from './provider-endpoint-rule-types';
import type { ProviderEndpoint, ProviderEndpointUpdate } from 'src/types/provider';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import LoadingButton from '@mui/lab/LoadingButton';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import { ProviderEndpointRuleList } from './provider-endpoint-rule-list';
import { formatApiFormat, defaultEndpointPath } from './provider-management-utils';
import {
  bodyRulesChanged,
  validateBodyRules,
  headerRulesChanged,
  bodyRulesToEditable,
  validateHeaderRules,
  headerRulesToEditable,
  editableBodyRulesToApi,
  editableHeaderRulesToApi,
} from './provider-endpoint-rule-types';

export function ProviderEndpointCard({
  endpoint,
  editState,
  expanded,
  saving,
  deleting,
  toggling,
  onEditStateChange,
  onExpandedChange,
  onSave,
  onDelete,
  onToggle,
}: {
  endpoint: ProviderEndpoint;
  editState: EndpointEditState;
  expanded: boolean;
  saving: boolean;
  deleting: boolean;
  toggling: boolean;
  onEditStateChange: (state: EndpointEditState) => void;
  onExpandedChange: (open: boolean) => void;
  onSave: (payload: ProviderEndpointUpdate) => void;
  onDelete: () => void;
  onToggle: () => void;
}) {
  const { t } = useTranslate('admin');
  const changed = endpointChanged(endpoint, editState);

  return (
    <Card variant="outlined" sx={{ borderRadius: 1 }}>
      <EndpointCardHeader endpoint={endpoint} deleting={deleting} toggling={toggling} onDelete={onDelete} onToggle={onToggle} />
      <Stack spacing={2} sx={{ p: 2 }}>
        <EndpointUrlFields endpoint={endpoint} editState={editState} onEditStateChange={onEditStateChange} />
        <ProviderEndpointRuleList
          open={expanded}
          headerRules={editState.headerRules}
          bodyRules={editState.bodyRules}
          onOpenChange={onExpandedChange}
          onHeaderRulesChange={(headerRules) => onEditStateChange({ ...editState, headerRules })}
          onBodyRulesChange={(bodyRules) => onEditStateChange({ ...editState, bodyRules })}
        />
        {changed && (
          <Stack direction="row" justifyContent="flex-end" spacing={1}>
            <Button size="small" color="inherit" onClick={() => onEditStateChange(editStateFromEndpoint(endpoint))}>
              {t('common.cancel')}
            </Button>
            <LoadingButton size="small" variant="contained" loading={saving} onClick={() => onSave(buildPatch(endpoint, editState))}>
              {t('common.save')}
            </LoadingButton>
          </Stack>
        )}
      </Stack>
    </Card>
  );
}

export function editStateFromEndpoint(endpoint: ProviderEndpoint): EndpointEditState {
  return {
    baseUrl: endpoint.base_url,
    customPath: endpoint.custom_path ?? '',
    headerRules: headerRulesToEditable(endpoint.header_rules),
    bodyRules: bodyRulesToEditable(endpoint.body_rules),
  };
}

export function endpointRuleCount(endpoint: ProviderEndpoint) {
  return (endpoint.header_rules?.length ?? 0) + (endpoint.body_rules?.length ?? 0);
}

export function validateEndpointEditState(state: EndpointEditState) {
  if (!state.baseUrl.trim()) return 'Base URL 不能为空';
  return validateHeaderRules(state.headerRules) || validateBodyRules(state.bodyRules);
}

function EndpointCardHeader({
  endpoint,
  deleting,
  toggling,
  onDelete,
  onToggle,
}: {
  endpoint: ProviderEndpoint;
  deleting: boolean;
  toggling: boolean;
  onDelete: () => void;
  onToggle: () => void;
}) {
  const { t } = useTranslate('admin');
  return (
    <Box>
      <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ px: 2, py: 1.25, bgcolor: 'action.hover' }}>
        <Typography variant="subtitle2">{formatApiFormat(endpoint.api_format)}</Typography>
        <Stack direction="row" spacing={0.5}>
          <IconButton size="small" title={endpoint.is_active ? t('actions.disable') : t('actions.enable')} disabled={toggling} onClick={onToggle}>
            <Iconify icon="ic:round-power-settings-new" width={16} />
          </IconButton>
          <IconButton size="small" title={t('common.delete')} disabled={deleting} color="error" onClick={onDelete}>
            <Iconify icon="solar:trash-bin-trash-bold" width={16} />
          </IconButton>
        </Stack>
      </Stack>
      <Divider />
    </Box>
  );
}

function EndpointUrlFields({
  endpoint,
  editState,
  onEditStateChange,
}: {
  endpoint: ProviderEndpoint;
  editState: EndpointEditState;
  onEditStateChange: (state: EndpointEditState) => void;
}) {
  const { t } = useTranslate('admin');
  const path = defaultEndpointPath(endpoint.api_format);
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5}>
      <TextField fullWidth size="small" label={t('providers.baseUrl')} value={editState.baseUrl} placeholder={t('providers.baseUrlPlaceholder')} onChange={(event) => onEditStateChange({ ...editState, baseUrl: event.target.value })} />
      <TextField
        fullWidth
        size="small"
        label={t('providers.customPath')}
        value={editState.customPath}
        placeholder={path || '留空使用默认'}
        helperText={path ? `${t('providers.defaultWhenBlank')} ${path}` : t('providers.defaultWhenBlank')}
        onChange={(event) => onEditStateChange({ ...editState, customPath: event.target.value })}
      />
    </Stack>
  );
}

function endpointChanged(endpoint: ProviderEndpoint, state: EndpointEditState) {
  return urlChanged(endpoint, state) || headerRulesChanged(endpoint.header_rules, state.headerRules) || bodyRulesChanged(endpoint.body_rules, state.bodyRules);
}

function urlChanged(endpoint: ProviderEndpoint, state: EndpointEditState) {
  return state.baseUrl !== endpoint.base_url || state.customPath !== (endpoint.custom_path ?? '');
}

function buildPatch(endpoint: ProviderEndpoint, state: EndpointEditState): ProviderEndpointUpdate {
  const patch: ProviderEndpointUpdate = {};
  if (state.baseUrl !== endpoint.base_url) patch.base_url = state.baseUrl;
  if (state.customPath !== (endpoint.custom_path ?? '')) patch.custom_path = state.customPath.trim() || null;
  if (headerRulesChanged(endpoint.header_rules, state.headerRules)) patch.header_rules = editableHeaderRulesToApi(state.headerRules);
  if (bodyRulesChanged(endpoint.body_rules, state.bodyRules)) patch.body_rules = editableBodyRulesToApi(state.bodyRules);
  return patch;
}
