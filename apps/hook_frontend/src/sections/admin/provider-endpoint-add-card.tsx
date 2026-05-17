'use client';

import type { ProviderEndpoint, ProviderEndpointCreate } from 'src/types/provider';
import type {
  EditableBodyRule,
  EndpointEditState,
  EditableHeaderRule,
} from './provider-endpoint-rule-types';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Divider from '@mui/material/Divider';
import TextField from '@mui/material/TextField';
import LoadingButton from '@mui/lab/LoadingButton';

import { useTranslate } from 'src/locales/use-locales';

import { ProviderEndpointRuleList } from './provider-endpoint-rule-list';
import { ProviderEndpointSearchSelect } from './provider-endpoint-select';
import { editableBodyRulesToApi, editableHeaderRulesToApi } from './provider-endpoint-rule-types';
import {
  formatApiFormat,
  normalizeBaseUrl,
  API_FORMAT_OPTIONS,
  defaultEndpointPath,
} from './provider-management-utils';

export type AddEndpointForm = EndpointEditState & {
  apiFormat: string;
};

export function ProviderEndpointAddCard({
  form,
  rulesOpen,
  adding,
  existingEndpoints,
  onFormChange,
  onApiFormatChange,
  onRulesOpenChange,
  onHeaderRulesChange,
  onBodyRulesChange,
  onAdd,
}: {
  form: AddEndpointForm;
  rulesOpen: boolean;
  adding: boolean;
  existingEndpoints: ProviderEndpoint[];
  onFormChange: (form: AddEndpointForm) => void;
  onApiFormatChange: (apiFormat: string) => void;
  onRulesOpenChange: (open: boolean) => void;
  onHeaderRulesChange: (rules: EditableHeaderRule[]) => void;
  onBodyRulesChange: (rules: EditableBodyRule[]) => void;
  onAdd: () => void;
}) {
  const { t } = useTranslate('admin');
  const availableFormats = API_FORMAT_OPTIONS.filter(
    (format) => !existingEndpoints.some((endpoint) => endpoint.api_format === format)
  );
  const selectedPath = defaultEndpointPath(form.apiFormat);

  return (
    <Box sx={cardSx}>
      <Stack direction="row" spacing={1.5} alignItems="center" sx={headerSx}>
        <ProviderEndpointSearchSelect
          value={form.apiFormat}
          options={availableFormats.map((value) => ({ value, label: formatApiFormat(value) }))}
          placeholder={t('providers.selectFormat')}
          sx={{ minWidth: 220 }}
          onChange={onApiFormatChange}
        />
        <Box sx={{ flex: 1 }} />
        <LoadingButton
          size="small"
          variant="outlined"
          loading={adding}
          disabled={!form.apiFormat || !normalizeBaseUrl(form.baseUrl)}
          onClick={onAdd}
        >
          {t('common.add')}
        </LoadingButton>
      </Stack>
      <Divider />
      <Stack spacing={2} sx={{ p: 2 }}>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5}>
          <TextField
            fullWidth
            size="small"
            label={t('providers.baseUrl')}
            value={form.baseUrl}
            placeholder={t('providers.baseUrlPlaceholder')}
            onChange={(event) => onFormChange({ ...form, baseUrl: event.target.value })}
          />
          <TextField
            fullWidth
            size="small"
            label={t('providers.customPath')}
            value={form.customPath}
            placeholder={selectedPath || '留空使用默认'}
            helperText={selectedPath ? `${t('providers.defaultWhenBlank')} ${selectedPath}` : t('providers.defaultWhenBlank')}
            onChange={(event) => onFormChange({ ...form, customPath: event.target.value })}
          />
        </Stack>
        <ProviderEndpointRuleList
          open={rulesOpen}
          headerRules={form.headerRules}
          bodyRules={form.bodyRules}
          onOpenChange={onRulesOpenChange}
          onHeaderRulesChange={onHeaderRulesChange}
          onBodyRulesChange={onBodyRulesChange}
        />
      </Stack>
    </Box>
  );
}

export function emptyAddEndpointForm(baseUrl = ''): AddEndpointForm {
  return {
    apiFormat: '',
    baseUrl,
    customPath: '',
    headerRules: [],
    bodyRules: [],
  };
}

export function addEndpointPayload(form: AddEndpointForm): ProviderEndpointCreate {
  return {
    api_format: form.apiFormat,
    base_url: normalizeBaseUrl(form.baseUrl),
    custom_path: form.customPath.trim() || null,
    header_rules: editableHeaderRulesToApi(form.headerRules),
    body_rules: editableBodyRulesToApi(form.bodyRules),
    is_active: true,
  };
}

const cardSx = { border: '1px dashed', borderColor: 'divider', borderRadius: 1, overflow: 'hidden' };
const headerSx = { px: 2, py: 1.25, bgcolor: 'action.hover' };
