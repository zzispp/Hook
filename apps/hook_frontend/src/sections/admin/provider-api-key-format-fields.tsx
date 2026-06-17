'use client';

import type { useProviderChildDialogs } from './provider-management-state';
import type { providerEndpointFormatOptions } from './provider-api-key-options';

import Chip from '@mui/material/Chip';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Autocomplete from '@mui/material/Autocomplete';
import ListItemText from '@mui/material/ListItemText';

import { useTranslate } from 'src/locales/use-locales';

import { selectedOptions } from './provider-api-key-options';

export function ApiKeyFormatFields({
  dialogs,
  loading,
  options,
  invalidFormats,
}: {
  dialogs: ReturnType<typeof useProviderChildDialogs>;
  loading: boolean;
  options: ReturnType<typeof providerEndpointFormatOptions>;
  invalidFormats: string[];
}) {
  const { t } = useTranslate('admin');
  const selected = selectedOptions(dialogs.apiKeyForm.api_formats, options);
  const missingImageGeneration = missingImageGenerationFormat(dialogs.apiKeyForm.api_formats);
  const hasError = invalidFormats.length > 0 || missingImageGeneration;

  return (
    <Stack spacing={1}>
      <Autocomplete
        multiple
        disableCloseOnSelect
        options={options}
        value={selected}
        getOptionLabel={(option) => option.label}
        isOptionEqualToValue={(option, current) => option.value === current.value}
        loading={loading}
        noOptionsText={loading ? t('common.loading') : t('providers.noEndpointFormatsForKey')}
        onChange={(_, values) =>
          dialogs.setApiKeyForm((form) => ({
            ...form,
            api_formats: values.map((value) => value.value),
          }))
        }
        renderOption={(props, option) => (
          <MenuItem {...props} key={option.value} value={option.value}>
            <ListItemText primary={option.label} secondary={option.description} />
          </MenuItem>
        )}
        renderTags={(values, getTagProps) =>
          values.map((option, index) => (
            <Chip
              {...getTagProps({ index })}
              key={option.value}
              size="small"
              color={invalidFormats.includes(option.value) ? 'error' : 'default'}
              label={option.label}
            />
          ))
        }
        renderInput={(params) => (
          <TextField
            {...params}
            required
            error={hasError}
            label={t('providers.supportedFormats')}
            helperText={
              missingImageGeneration
                ? t('providers.imageEditRequiresImageGeneration')
                : t('providers.supportedFormatsHelper')
            }
            placeholder={t('providers.selectSupportedFormats')}
          />
        )}
      />
      {invalidFormats.length > 0 ? (
        <Alert severity="error">
          {t('providers.unboundKeyFormats', { formats: invalidFormats.join(', ') })}
        </Alert>
      ) : null}
      {missingImageGeneration ? (
        <Alert severity="error">
          {t('providers.imageEditRequiresImageGeneration')}
        </Alert>
      ) : null}
    </Stack>
  );
}

export function selectedValuesOutsideOptions(values: string[], options: { value: string }[]) {
  const allowed = new Set(options.map((option) => option.value));
  return values.filter((value) => !allowed.has(value));
}

export function missingImageGenerationFormat(values: string[]) {
  return values.includes('openai_image_edit') && !values.includes('openai_image');
}
