'use client';

import type { GlobalModelResponse } from 'src/types/model';
import type { useProviderChildDialogs } from './provider-management-state';

import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Autocomplete from '@mui/material/Autocomplete';
import ListItemText from '@mui/material/ListItemText';

import { useTranslate } from 'src/locales/use-locales';
import { useProviderModels, useProviderEndpoints } from 'src/actions/providers';

import { SwitchRow, TextFieldRow, ManagementDialog } from './shared';
import { ApiKeyFormatFields, selectedValuesOutsideOptions } from './provider-api-key-format-fields';
import {
  selectedOptions,
  providerModelOptions,
  providerEndpointFormatOptions,
} from './provider-api-key-options';

export function ProviderApiKeyDialog({
  dialogs,
  models,
  providerId,
}: {
  dialogs: ReturnType<typeof useProviderChildDialogs>;
  models: GlobalModelResponse[];
  providerId?: string;
}) {
  const { t } = useTranslate('admin');
  const editing = !!dialogs.editingApiKey;
  const providerModels = useProviderModels(dialogs.apiKeyOpen ? providerId : null);
  const providerEndpoints = useProviderEndpoints(dialogs.apiKeyOpen ? providerId : null);
  const formatOptions = providerEndpointFormatOptions(providerEndpoints.items);
  const invalidFormats = selectedValuesOutsideOptions(
    dialogs.apiKeyForm.api_formats,
    formatOptions
  );
  const submitDisabled =
    formatOptions.length === 0 ||
    invalidFormats.length > 0 ||
    dialogs.apiKeyForm.api_formats.length === 0;

  return (
    <ManagementDialog
      open={dialogs.apiKeyOpen}
      title={editing ? t('dialogs.editProviderKey') : t('dialogs.createProviderKey')}
      submitting={dialogs.submitting}
      submitDisabled={submitDisabled}
      onClose={dialogs.closeApiKey}
      onSubmit={dialogs.submitApiKey}
    >
      <ApiKeyBasicFields dialogs={dialogs} />
      <ApiKeyFormatFields
        dialogs={dialogs}
        loading={providerEndpoints.isLoading}
        options={formatOptions}
        invalidFormats={invalidFormats}
      />
      <ApiKeyModelFields
        dialogs={dialogs}
        loading={providerModels.isLoading}
        models={models}
        providerModels={providerModels.items}
      />
      <ApiKeyLimitFields dialogs={dialogs} />
      <ApiKeyTimeRangeFields dialogs={dialogs} />
      <ApiKeySwitches dialogs={dialogs} />
    </ManagementDialog>
  );
}

function ApiKeyBasicFields({ dialogs }: { dialogs: ReturnType<typeof useProviderChildDialogs> }) {
  const { t } = useTranslate('admin');
  const editing = !!dialogs.editingApiKey;

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
          required={!editing}
          type="password"
          label={t('fields.apiKey')}
          helperText={editing ? t('providers.keyUpdateSecretHint') : undefined}
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
        onChange={(event) =>
          dialogs.setApiKeyForm((form) => ({ ...form, note: event.target.value }))
        }
      />
    </>
  );
}

function ApiKeyModelFields({
  dialogs,
  loading,
  models,
  providerModels,
}: {
  dialogs: ReturnType<typeof useProviderChildDialogs>;
  loading: boolean;
  models: GlobalModelResponse[];
  providerModels: ReturnType<typeof useProviderModels>['items'];
}) {
  const { t } = useTranslate('admin');
  const options = providerModelOptions(models, providerModels);
  const selected = selectedOptions(dialogs.apiKeyForm.allowed_model_ids, options);

  return (
    <Autocomplete
      multiple
      disableCloseOnSelect
      options={options}
      value={selected}
      getOptionLabel={(option) => option.label}
      isOptionEqualToValue={(option, current) => option.value === current.value}
      loading={loading}
      noOptionsText={loading ? t('common.loading') : t('providers.noBindableModels')}
      onChange={(_, values) =>
        dialogs.setApiKeyForm((form) => ({
          ...form,
          allowed_model_ids: values.map((value) => value.value),
        }))
      }
      renderOption={(props, option) => (
        <MenuItem {...props} key={option.value} value={option.value}>
          <ListItemText primary={option.label} secondary={option.description} />
        </MenuItem>
      )}
      renderInput={(params) => (
        <TextField
          {...params}
          label={t('providers.modelPermission')}
          helperText={t('providers.modelPermissionHelper')}
          placeholder={t('providers.searchOrAddProviderModel')}
        />
      )}
    />
  );
}

function ApiKeyLimitFields({ dialogs }: { dialogs: ReturnType<typeof useProviderChildDialogs> }) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <TextFieldRow
        type="number"
        label={t('providers.priority')}
        value={dialogs.apiKeyForm.internal_priority}
        onChange={(value) =>
          dialogs.setApiKeyForm((form) => ({ ...form, internal_priority: value }))
        }
      />
      <TextFieldRow
        type="number"
        label={t('providers.rpmLimit')}
        helperText={t('providers.adaptiveWhenBlank')}
        value={dialogs.apiKeyForm.rpm_limit}
        onChange={(value) => dialogs.setApiKeyForm((form) => ({ ...form, rpm_limit: value }))}
      />
    </Stack>
  );
}

function ApiKeyTimeRangeFields({
  dialogs,
}: {
  dialogs: ReturnType<typeof useProviderChildDialogs>;
}) {
  const { t } = useTranslate('admin');

  return (
    <>
      <SwitchRow
        checked={dialogs.apiKeyForm.time_range_enabled}
        label={t('providers.timeRangeEnabled')}
        onChange={(checked) =>
          dialogs.setApiKeyForm((form) => ({ ...form, time_range_enabled: checked }))
        }
      />
      {dialogs.apiKeyForm.time_range_enabled && (
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
          <TextFieldRow
            label={t('providers.timeRangeStart')}
            value={dialogs.apiKeyForm.time_range_start}
            onChange={(value) =>
              dialogs.setApiKeyForm((form) => ({ ...form, time_range_start: value }))
            }
          />
          <TextFieldRow
            label={t('providers.timeRangeEnd')}
            value={dialogs.apiKeyForm.time_range_end}
            onChange={(value) =>
              dialogs.setApiKeyForm((form) => ({ ...form, time_range_end: value }))
            }
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
