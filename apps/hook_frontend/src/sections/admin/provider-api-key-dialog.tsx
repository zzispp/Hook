'use client';

import type { ProviderEndpoint } from 'src/types/provider';
import type { useProviderChildDialogs } from './provider-management-state';

import { useMemo, useEffect } from 'react';

import Stack from '@mui/material/Stack';
import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import ListItemText from '@mui/material/ListItemText';

import { useTranslate } from 'src/locales/use-locales';
import { useProviderEndpoints } from 'src/actions/providers';

import { formatApiFormat } from './provider-management-utils';
import { SwitchRow, TextFieldRow, ManagementDialog } from './shared';

export function ProviderApiKeyDialog({
  dialogs,
  providerId,
}: {
  dialogs: ReturnType<typeof useProviderChildDialogs>;
  providerId?: string;
}) {
  const { t } = useTranslate('admin');
  const { apiKeyOpen, setApiKeyForm } = dialogs;
  const endpoints = useProviderEndpoints(apiKeyOpen ? providerId : null);
  const apiFormatOptions = useMemo(() => endpointApiFormats(endpoints.items), [endpoints.items]);

  useEffect(() => {
    if (!apiKeyOpen) return;
    setApiKeyForm((form) => formWithAvailableApiFormats(form, apiFormatOptions));
  }, [apiFormatOptions, apiKeyOpen, setApiKeyForm]);

  return (
    <ManagementDialog
      open={dialogs.apiKeyOpen}
      title={t('dialogs.createProviderKey')}
      submitting={dialogs.submitting}
      submitDisabled={!apiFormatOptions.length || !dialogs.apiKeyForm.api_formats.length}
      onClose={dialogs.closeApiKey}
      onSubmit={dialogs.submitApiKey}
    >
      <ApiKeyBasicFields dialogs={dialogs} />
      <ApiFormatSelect
        value={dialogs.apiKeyForm.api_formats}
        options={apiFormatOptions}
        onChange={(apiFormats) => dialogs.setApiKeyForm((form) => ({ ...form, api_formats: apiFormats }))}
      />
      <ApiKeyLimitFields dialogs={dialogs} />
      <ApiKeyTimeRangeFields dialogs={dialogs} />
      <ApiKeySwitches dialogs={dialogs} />
    </ManagementDialog>
  );
}

function ApiKeyBasicFields({ dialogs }: { dialogs: ReturnType<typeof useProviderChildDialogs> }) {
  const { t } = useTranslate('admin');

  return (
    <>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          required
          label={t('fields.keyName')}
          value={dialogs.apiKeyForm.name}
          onChange={(value) => dialogs.setApiKeyForm((form) => ({ ...form, name: value }))}
        />
        <TextFieldRow
          required
          type="password"
          label={t('fields.apiKey')}
          value={dialogs.apiKeyForm.api_key}
          onChange={(value) => dialogs.setApiKeyForm((form) => ({ ...form, api_key: value }))}
        />
      </Stack>
      <TextField
        fullWidth
        multiline
        minRows={2}
        label={t('providers.note')}
        value={dialogs.apiKeyForm.note}
        onChange={(event) => dialogs.setApiKeyForm((form) => ({ ...form, note: event.target.value }))}
      />
    </>
  );
}

function ApiKeyLimitFields({ dialogs }: { dialogs: ReturnType<typeof useProviderChildDialogs> }) {
  const { t } = useTranslate('admin');

  return (
    <>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          type="number"
          label={t('providers.priority')}
          value={dialogs.apiKeyForm.internal_priority}
          onChange={(value) => dialogs.setApiKeyForm((form) => ({ ...form, internal_priority: value }))}
        />
        <TextFieldRow
          type="number"
          label={t('providers.rpmLimit')}
          helperText={t('providers.adaptiveWhenBlank')}
          value={dialogs.apiKeyForm.rpm_limit}
          onChange={(value) => dialogs.setApiKeyForm((form) => ({ ...form, rpm_limit: value }))}
        />
      </Stack>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          type="number"
          label={t('providers.cacheTtl')}
          value={dialogs.apiKeyForm.cache_ttl_minutes}
          onChange={(value) => dialogs.setApiKeyForm((form) => ({ ...form, cache_ttl_minutes: value }))}
        />
        <TextFieldRow
          type="number"
          label={t('providers.probeInterval')}
          value={dialogs.apiKeyForm.max_probe_interval_minutes}
          onChange={(value) => dialogs.setApiKeyForm((form) => ({ ...form, max_probe_interval_minutes: value }))}
        />
      </Stack>
    </>
  );
}

function ApiKeyTimeRangeFields({ dialogs }: { dialogs: ReturnType<typeof useProviderChildDialogs> }) {
  const { t } = useTranslate('admin');

  return (
    <>
      <SwitchRow
        checked={dialogs.apiKeyForm.time_range_enabled}
        label={t('providers.timeRangeEnabled')}
        onChange={(checked) => dialogs.setApiKeyForm((form) => ({ ...form, time_range_enabled: checked }))}
      />
      {dialogs.apiKeyForm.time_range_enabled && (
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
          <TextFieldRow
            label={t('providers.timeRangeStart')}
            value={dialogs.apiKeyForm.time_range_start}
            onChange={(value) => dialogs.setApiKeyForm((form) => ({ ...form, time_range_start: value }))}
          />
          <TextFieldRow
            label={t('providers.timeRangeEnd')}
            value={dialogs.apiKeyForm.time_range_end}
            onChange={(value) => dialogs.setApiKeyForm((form) => ({ ...form, time_range_end: value }))}
          />
        </Stack>
      )}
    </>
  );
}

function ApiKeySwitches({ dialogs }: { dialogs: ReturnType<typeof useProviderChildDialogs> }) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <SwitchRow
        checked={dialogs.apiKeyForm.is_active}
        label={t('common.enabled')}
        onChange={(checked) => dialogs.setApiKeyForm((form) => ({ ...form, is_active: checked }))}
      />
    </Stack>
  );
}

function ApiFormatSelect({
  value,
  options,
  onChange,
}: {
  value: string[];
  options: string[];
  onChange: (value: string[]) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TextField
      select
      fullWidth
      label={t('providers.supportedApiFormats')}
      value={value}
      SelectProps={{
        multiple: true,
        displayEmpty: true,
        renderValue: (selected) => apiFormatSelectLabel(selected as string[], options, t),
      }}
      onChange={(event) => onChange(selectedValues(event.target.value))}
    >
      {options.length ? (
        options.map((format) => (
          <MenuItem key={format} value={format}>
            <Checkbox checked={value.includes(format)} />
            <ListItemText primary={formatApiFormat(format)} />
          </MenuItem>
        ))
      ) : (
        <MenuItem disabled value="">
          <ListItemText primary={t('providers.noSupportedApiFormats')} />
        </MenuItem>
      )}
    </TextField>
  );
}

function selectedValues(value: string | string[]) {
  return Array.isArray(value) ? value : value.split(',').filter(Boolean);
}

function endpointApiFormats(endpoints: ProviderEndpoint[]) {
  return [...new Set(endpoints.map((endpoint) => endpoint.api_format).filter(Boolean))];
}

function formWithAvailableApiFormats(
  form: ReturnType<typeof useProviderChildDialogs>['apiKeyForm'],
  options: string[]
) {
  if (!options.length) {
    return form.api_formats.length ? { ...form, api_formats: [] } : form;
  }
  const selected = form.api_formats.filter((format) => options.includes(format));
  const apiFormats = selected.length ? selected : [options[0]];
  return sameValues(apiFormats, form.api_formats) ? form : { ...form, api_formats: apiFormats };
}

function sameValues(left: string[], right: string[]) {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

function apiFormatSelectLabel(selected: string[], options: string[], t: (key: string) => string) {
  if (!options.length) return t('providers.noSupportedApiFormats');
  if (!selected.length) return t('providers.selectFormat');
  return selected.map(formatApiFormat).join(', ');
}
