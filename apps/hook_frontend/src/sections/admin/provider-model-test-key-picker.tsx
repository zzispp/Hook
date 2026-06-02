'use client';

import type { ProviderApiKey } from 'src/types/provider';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { formatApiFormat } from './provider-management-utils';
import { endpointGridSx, endpointButtonSx } from './provider-model-test-styles';

export function ProviderModelTestKeyPicker({
  keys,
  selectedId,
  onSelect,
}: {
  keys: ProviderApiKey[];
  selectedId: string;
  onSelect: (id: string) => void;
}) {
  const { t } = useTranslate('admin');
  if (keys.length === 0) {
    return <Typography variant="body2" color="text.secondary">{t('providers.noTestableKeys')}</Typography>;
  }
  return (
    <Stack spacing={1}>
      <Typography variant="subtitle2">{t('providers.selectTestProviderKey')}</Typography>
      <Box sx={endpointGridSx}>
        {keys.map((key) => (
          <Button
            key={key.id}
            variant={selectedId === key.id ? 'soft' : 'outlined'}
            color={selectedId === key.id ? 'primary' : 'inherit'}
            onClick={() => onSelect(key.id)}
            sx={endpointButtonSx}
          >
            <Box sx={{ minWidth: 0, textAlign: 'left' }}>
              <Typography variant="subtitle2" noWrap>{key.name}</Typography>
              <Typography variant="caption" color="text.secondary" sx={{ wordBreak: 'break-word' }}>
                {keyMeta(key, t)}
              </Typography>
            </Box>
            <Chip size="small" label={selectedId === key.id ? t('providers.selected') : t('providers.activeAndAvailable')} />
          </Button>
        ))}
      </Box>
    </Stack>
  );
}

export function eligibleModelTestKeys(
  keys: ProviderApiKey[],
  globalModelId: string | undefined,
  apiFormat: string | undefined
) {
  if (!globalModelId || !apiFormat) return [];
  return keys
    .filter((key) => keyEligibleForTest(key, globalModelId, apiFormat))
    .sort((left, right) => compareKeysByFormatPriority(left, right, apiFormat));
}

export function firstEligibleModelTestKey(
  keys: ProviderApiKey[],
  globalModelId: string | undefined,
  apiFormat: string | undefined
) {
  return eligibleModelTestKeys(keys, globalModelId, apiFormat)[0];
}

function keyEligibleForTest(key: ProviderApiKey, globalModelId: string, apiFormat: string) {
  return (
    key.is_active &&
    key.api_formats.includes(apiFormat) &&
    (key.allowed_model_ids.length === 0 || key.allowed_model_ids.includes(globalModelId)) &&
    keyWithinTimeRange(key)
  );
}

function compareKeysByFormatPriority(left: ProviderApiKey, right: ProviderApiKey, apiFormat: string) {
  return keyFormatPriority(left, apiFormat) - keyFormatPriority(right, apiFormat) || left.name.localeCompare(right.name);
}

function keyFormatPriority(key: ProviderApiKey, apiFormat: string) {
  return key.global_priority_by_format[apiFormat] ?? key.internal_priority;
}

function keyWithinTimeRange(key: ProviderApiKey) {
  if (!key.time_range_enabled) return true;
  if (!key.time_range_start || !key.time_range_end) return false;
  const current = new Date();
  const currentMinute = current.getUTCHours() * 60 + current.getUTCMinutes();
  const start = minuteOfDay(key.time_range_start);
  const end = minuteOfDay(key.time_range_end);
  if (start === null || end === null) return false;
  if (start === end) return false;
  return start < end
    ? currentMinute >= start && currentMinute < end
    : currentMinute >= start || currentMinute < end;
}

function minuteOfDay(value: string) {
  const [hour, minute] = value.split(':').map(Number);
  if (!Number.isInteger(hour) || !Number.isInteger(minute)) return null;
  if (hour < 0 || hour > 23 || minute < 0 || minute > 59) return null;
  return hour * 60 + minute;
}

function keyMeta(key: ProviderApiKey, t: (key: string, options?: Record<string, unknown>) => string) {
  return [
    formatList(key.api_formats, t('providers.noSupportedFormats')),
    modelPermissionText(key.allowed_model_ids, t),
  ].join(' | ');
}

function formatList(values: string[], emptyText: string) {
  if (!values.length) return emptyText;
  return values.map(formatApiFormat).join(', ');
}

function modelPermissionText(values: string[], t: (key: string, options?: Record<string, unknown>) => string) {
  if (!values.length) return t('providers.allModels');
  return t('providers.selectedModelCount', { count: values.length });
}
