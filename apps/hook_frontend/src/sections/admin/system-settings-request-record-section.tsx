'use client';

import type { SystemSettingsForm } from './system-settings-utils';
import type { RequestRecordLevel } from 'src/types/system-setting';

import Stack from '@mui/material/Stack';
import Paper from '@mui/material/Paper';
import MenuItem from '@mui/material/MenuItem';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow } from './shared';
import { SettingsSection } from './system-settings-section';

type RequestRecordSide = 'client' | 'provider';
type RequestRecordLevelField = 'client_request_record_level' | 'provider_request_record_level';
type RequestRecordTextField =
  | 'client_max_request_body_size_kb'
  | 'provider_max_request_body_size_kb'
  | 'client_max_response_body_size_kb'
  | 'provider_max_response_body_size_kb'
  | 'client_sensitive_request_headers'
  | 'provider_sensitive_request_headers';
type RequestRecordPanelFields = {
  level: RequestRecordLevelField;
  requestHeaders: 'client_record_request_headers' | 'provider_record_request_headers';
  requestBody: 'client_record_request_body' | 'provider_record_request_body';
  responseHeaders: 'client_record_response_headers' | 'provider_record_response_headers';
  responseBody: 'client_record_response_body' | 'provider_record_response_body';
  maxRequestBodySizeKb: RequestRecordTextField;
  maxResponseBodySizeKb: RequestRecordTextField;
  sensitiveRequestHeaders: RequestRecordTextField;
};

const PANEL_FIELDS: Record<RequestRecordSide, RequestRecordPanelFields> = {
  client: {
    level: 'client_request_record_level',
    requestHeaders: 'client_record_request_headers',
    requestBody: 'client_record_request_body',
    responseHeaders: 'client_record_response_headers',
    responseBody: 'client_record_response_body',
    maxRequestBodySizeKb: 'client_max_request_body_size_kb',
    maxResponseBodySizeKb: 'client_max_response_body_size_kb',
    sensitiveRequestHeaders: 'client_sensitive_request_headers',
  },
  provider: {
    level: 'provider_request_record_level',
    requestHeaders: 'provider_record_request_headers',
    requestBody: 'provider_record_request_body',
    responseHeaders: 'provider_record_response_headers',
    responseBody: 'provider_record_response_body',
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

        <PayloadSwitches form={form} fields={fields} setForm={setForm} />

        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
          <RecordLevelField form={form} fields={fields} prefix={prefix} setForm={setForm} />
          <PayloadSizeField
            form={form}
            field={fields.maxRequestBodySizeKb}
            label={t('systemSettings.fields.maxRequestBodySizeKb')}
            helperText={t(`systemSettings.helper.${prefix}MaxRequestBodySizeKb`)}
            setForm={setForm}
          />
          <PayloadSizeField
            form={form}
            field={fields.maxResponseBodySizeKb}
            label={t('systemSettings.fields.maxResponseBodySizeKb')}
            helperText={t(`systemSettings.helper.${prefix}MaxResponseBodySizeKb`)}
            setForm={setForm}
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

function RecordLevelField({
  form,
  fields,
  prefix,
  setForm,
}: {
  form: SystemSettingsForm;
  fields: RequestRecordPanelFields;
  prefix: RequestRecordSide;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
}) {
  const { t } = useTranslate('admin');

  return (
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
  );
}

function PayloadSizeField({
  form,
  field,
  label,
  helperText,
  setForm,
}: {
  form: SystemSettingsForm;
  field: RequestRecordTextField;
  label: string;
  helperText: string;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
}) {
  return (
    <TextFieldRow
      type="number"
      label={label}
      value={form[field]}
      helperText={helperText}
      onChange={(value) =>
        setForm((current) => ({
          ...current,
          [field]: value,
        }))
      }
    />
  );
}

function PayloadSwitches({
  form,
  fields,
  setForm,
}: {
  form: SystemSettingsForm;
  fields: RequestRecordPanelFields;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
}) {
  const { t } = useTranslate('admin');
  const switches = [
    ['requestHeaders', fields.requestHeaders],
    ['requestBody', fields.requestBody],
    ['responseHeaders', fields.responseHeaders],
    ['responseBody', fields.responseBody],
  ] as const;

  return (
    <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1} sx={{ flexWrap: 'wrap' }}>
      {switches.map(([labelKey, field]) => (
        <SwitchRow
          key={field}
          checked={form[field]}
          label={t(`systemSettings.requestRecord.payloadSwitches.${labelKey}`)}
          onChange={(checked) =>
            setForm((current) => ({
              ...current,
              [field]: checked,
            }))
          }
        />
      ))}
    </Stack>
  );
}
