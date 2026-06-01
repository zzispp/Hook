'use client';

import type { ProviderEndpoint, ProviderEndpointCreate } from 'src/types/provider';
import type {
  EditableBodyRule,
  EndpointEditState,
  EditableHeaderRule,
} from './provider-endpoint-rule-types';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import LoadingButton from '@mui/lab/LoadingButton';

import { useTranslate } from 'src/locales/use-locales';

import { isValidEndpointBaseUrl } from './provider-endpoint-validation';
import { ProviderEndpointRuleList } from './provider-endpoint-rule-list';
import { editableBodyRulesToApi, editableHeaderRulesToApi } from './provider-endpoint-rule-types';
import { OPENAI_COMPACT_API_FORMAT, defaultOpenAiCompactBodyRules } from './provider-endpoint-default-rules';
import { ProviderEndpointSearchSelect, ProviderEndpointMultiSearchSelect } from './provider-endpoint-select';
import {
  formatApiFormat,
  normalizeBaseUrl,
  API_FORMAT_OPTIONS,
  defaultEndpointPath,
} from './provider-management-utils';

export type AddEndpointForm = EndpointEditState & {
  apiFormat: string;
};

export type QuickAddEndpointForm = {
  apiFormats: string[];
  baseUrl: string;
};

export function ProviderEndpointQuickAddCard({
  form,
  adding,
  existingEndpoints,
  onFormChange,
  onApiFormatsChange,
  onAdd,
}: {
  form: QuickAddEndpointForm;
  adding: boolean;
  existingEndpoints: ProviderEndpoint[];
  onFormChange: (form: QuickAddEndpointForm) => void;
  onApiFormatsChange: (apiFormats: string[]) => void;
  onAdd: () => void;
}) {
  return (
    <QuickAddCard
      form={form}
      adding={adding}
      availableFormats={availableEndpointFormats(existingEndpoints)}
      onFormChange={onFormChange}
      onApiFormatsChange={onApiFormatsChange}
      onAdd={onAdd}
    />
  );
}

export function ProviderEndpointSingleAddCard({
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
  return (
    <SingleAddCard
      form={form}
      rulesOpen={rulesOpen}
      adding={adding}
      availableFormats={availableEndpointFormats(existingEndpoints)}
      onFormChange={onFormChange}
      onApiFormatChange={onApiFormatChange}
      onRulesOpenChange={onRulesOpenChange}
      onHeaderRulesChange={onHeaderRulesChange}
      onBodyRulesChange={onBodyRulesChange}
      onAdd={onAdd}
    />
  );
}

function SingleAddCard({
  form,
  rulesOpen,
  adding,
  availableFormats,
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
  availableFormats: string[];
  onFormChange: (form: AddEndpointForm) => void;
  onApiFormatChange: (apiFormat: string) => void;
  onRulesOpenChange: (open: boolean) => void;
  onHeaderRulesChange: (rules: EditableHeaderRule[]) => void;
  onBodyRulesChange: (rules: EditableBodyRule[]) => void;
  onAdd: () => void;
}) {
  const { t } = useTranslate('admin');
  const selectedPath = defaultEndpointPath(form.apiFormat);

  return (
    <Box sx={cardSx}>
      <Stack direction="row" spacing={1.5} alignItems="center" sx={headerSx}>
        <ProviderEndpointSearchSelect
          value={form.apiFormat}
          options={availableFormats.map((value) => ({ value, label: formatApiFormat(value) }))}
          placeholder={t('providers.selectFormat')}
          noOptionsText={t('common.noResults')}
          sx={{ minWidth: 220 }}
          onChange={onApiFormatChange}
        />
        <Box sx={{ flex: 1 }} />
        <LoadingButton
          size="small"
          variant="outlined"
          loading={adding}
          disabled={!form.apiFormat || !isValidEndpointBaseUrl(form.baseUrl)}
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
            placeholder={selectedPath || t('providers.defaultWhenBlank')}
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

function QuickAddCard({
  form,
  adding,
  availableFormats,
  onFormChange,
  onApiFormatsChange,
  onAdd,
}: {
  form: QuickAddEndpointForm;
  adding: boolean;
  availableFormats: string[];
  onFormChange: (form: QuickAddEndpointForm) => void;
  onApiFormatsChange: (apiFormats: string[]) => void;
  onAdd: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={cardSx}>
      <Stack direction="row" spacing={1.5} alignItems="center" sx={headerSx}>
        <Typography variant="caption" sx={labelSx}>{t('providers.quickAddEndpoints')}</Typography>
        <Box sx={{ flex: 1 }} />
        <Button size="small" variant="text" onClick={() => onApiFormatsChange(availableFormats)}>
          {t('common.selectAll')}
        </Button>
        <Button size="small" variant="text" onClick={() => onApiFormatsChange([])}>
          {t('common.clear')}
        </Button>
        <LoadingButton
          size="small"
          variant="outlined"
          loading={adding}
          disabled={!form.apiFormats.length || !isValidEndpointBaseUrl(form.baseUrl)}
          onClick={onAdd}
        >
          {t('common.add')}
        </LoadingButton>
      </Stack>
      <Divider />
      <Stack spacing={2} sx={{ p: 2 }}>
        <TextField
          fullWidth
          size="small"
          label={t('providers.baseUrl')}
          value={form.baseUrl}
          placeholder={t('providers.baseUrlPlaceholder')}
          helperText={t('providers.blankUsesEachFormatDefault')}
          onChange={(event) => onFormChange({ ...form, baseUrl: event.target.value })}
        />
        <ProviderEndpointMultiSearchSelect
          value={form.apiFormats}
          options={availableFormats.map((value) => ({ value, label: formatApiFormat(value) }))}
          placeholder={t('providers.selectFormats')}
          noOptionsText={t('common.noResults')}
          onChange={onApiFormatsChange}
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

export function emptyQuickAddEndpointForm(baseUrl = ''): QuickAddEndpointForm {
  return {
    apiFormats: [],
    baseUrl,
  };
}

export function quickAddEndpointPayloads(form: QuickAddEndpointForm): ProviderEndpointCreate[] {
  const baseUrl = normalizeBaseUrl(form.baseUrl);
  return form.apiFormats.map((apiFormat) => ({
    api_format: apiFormat,
    base_url: baseUrl,
    body_rules: quickAddBodyRules(apiFormat),
    is_active: true,
  }));
}

function quickAddBodyRules(apiFormat: string) {
  if (apiFormat !== OPENAI_COMPACT_API_FORMAT) return [];
  return editableBodyRulesToApi(defaultOpenAiCompactBodyRules());
}

const cardSx = { border: '1px dashed', borderColor: 'divider', borderRadius: 1, overflow: 'hidden' };
const headerSx = { px: 2, py: 1.25, bgcolor: 'action.hover' };
const labelSx = { color: 'text.secondary', fontWeight: 700, textTransform: 'uppercase', letterSpacing: 1 };

function availableEndpointFormats(existingEndpoints: ProviderEndpoint[]) {
  return API_FORMAT_OPTIONS.filter(
    (format) => !existingEndpoints.some((endpoint) => endpoint.api_format === format)
  );
}
