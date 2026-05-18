'use client';

import type { SystemSettingsForm } from './system-settings-utils';
import type { RequestRecordLevel } from 'src/types/system-setting';

import Stack from '@mui/material/Stack';
import Paper from '@mui/material/Paper';
import MenuItem from '@mui/material/MenuItem';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { TextFieldRow } from './shared';
import { SettingsSection } from './system-settings-section';

type RequestRecordSide = 'client' | 'provider';
type RequestRecordPanelFields = {
  level: 'client_request_record_level' | 'provider_request_record_level';
  maxRequestBodySizeKb: 'client_max_request_body_size_kb' | 'provider_max_request_body_size_kb';
  maxResponseBodySizeKb: 'client_max_response_body_size_kb' | 'provider_max_response_body_size_kb';
  sensitiveRequestHeaders: 'client_sensitive_request_headers' | 'provider_sensitive_request_headers';
};

const PANEL_FIELDS: Record<RequestRecordSide, RequestRecordPanelFields> = {
  client: {
    level: 'client_request_record_level',
    maxRequestBodySizeKb: 'client_max_request_body_size_kb',
    maxResponseBodySizeKb: 'client_max_response_body_size_kb',
    sensitiveRequestHeaders: 'client_sensitive_request_headers',
  },
  provider: {
    level: 'provider_request_record_level',
    maxRequestBodySizeKb: 'provider_max_request_body_size_kb',
    maxResponseBodySizeKb: 'provider_max_response_body_size_kb',
    sensitiveRequestHeaders: 'provider_sensitive_request_headers',
  },
};

export function RequestRecordSection({
  form,
  setForm,
}: {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
}) {
  const { t } = useTranslate('admin');

  return (
    <SettingsSection
      title={t('systemSettings.sections.requestRecord')}
      description={t('systemSettings.requestRecord.description')}
    >
      <Stack spacing={2}>
        <RequestRecordPanel side="client" form={form} setForm={setForm} />
        <RequestRecordPanel side="provider" form={form} setForm={setForm} />
      </Stack>
    </SettingsSection>
  );
}

function RequestRecordPanel({
  side,
  form,
  setForm,
}: {
  side: RequestRecordSide;
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
}) {
  const { t } = useTranslate('admin');
  const prefix = side === 'client' ? 'client' : 'provider';
  const fields = PANEL_FIELDS[side];

  return (
    <Paper variant="outlined" sx={{ p: 2, borderRadius: 1 }}>
      <Stack spacing={2}>
        <Stack spacing={0.5}>
          <Typography variant="subtitle2">
            {t(`systemSettings.requestRecord.panels.${prefix}.title`)}
          </Typography>
          <Typography variant="body2" color="text.secondary">
            {t(`systemSettings.requestRecord.panels.${prefix}.description`)}
          </Typography>
        </Stack>

        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
          <TextFieldRow
            select
            label={t('systemSettings.fields.requestRecordLevel')}
            value={form[fields.level]}
            helperText={t(`systemSettings.helper.${prefix}RequestRecordLevel`)}
            onChange={(value) =>
              setForm((current) => ({
                ...current,
                [fields.level]: value as RequestRecordLevel,
              }))
            }
          >
            <MenuItem value="basic">{t('systemSettings.requestRecord.levels.basic')}</MenuItem>
            <MenuItem value="headers">{t('systemSettings.requestRecord.levels.headers')}</MenuItem>
            <MenuItem value="full">{t('systemSettings.requestRecord.levels.full')}</MenuItem>
          </TextFieldRow>
          <TextFieldRow
            type="number"
            label={t('systemSettings.fields.maxRequestBodySizeKb')}
            value={form[fields.maxRequestBodySizeKb]}
            helperText={t(`systemSettings.helper.${prefix}MaxRequestBodySizeKb`)}
            onChange={(value) =>
              setForm((current) => ({
                ...current,
                [fields.maxRequestBodySizeKb]: value,
              }))
            }
          />
          <TextFieldRow
            type="number"
            label={t('systemSettings.fields.maxResponseBodySizeKb')}
            value={form[fields.maxResponseBodySizeKb]}
            helperText={t(`systemSettings.helper.${prefix}MaxResponseBodySizeKb`)}
            onChange={(value) =>
              setForm((current) => ({
                ...current,
                [fields.maxResponseBodySizeKb]: value,
              }))
            }
          />
        </Stack>

        <TextFieldRow
          label={t('systemSettings.fields.sensitiveRequestHeaders')}
          value={form[fields.sensitiveRequestHeaders]}
          helperText={t(`systemSettings.helper.${prefix}SensitiveRequestHeaders`)}
          onChange={(value) =>
            setForm((current) => ({
              ...current,
              [fields.sensitiveRequestHeaders]: value,
            }))
          }
        />
      </Stack>
    </Paper>
  );
}
