'use client';

import type { SystemSettingsForm } from './system-settings-utils';

import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import MenuItem from '@mui/material/MenuItem';
import FormControlLabel from '@mui/material/FormControlLabel';

import { useTranslate } from 'src/locales/use-locales';

import { TextFieldRow } from './shared';
import { SettingsSection } from './system-settings-section';

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
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
          <TextFieldRow
            select
            label={t('systemSettings.fields.requestRecordLevel')}
            value={form.request_record_level}
            helperText={t('systemSettings.helper.requestRecordLevel')}
            onChange={(value) =>
              setForm((current) => ({
                ...current,
                request_record_level: value as typeof current.request_record_level,
              }))
            }
          >
            <MenuItem value="basic">{t('systemSettings.requestRecord.levels.basic')}</MenuItem>
          </TextFieldRow>
          <TextFieldRow
            type="number"
            label={t('systemSettings.fields.maxRequestBodySizeKb')}
            value={form.max_request_body_size_kb}
            helperText={t('systemSettings.helper.maxRequestBodySizeKb')}
            onChange={(value) =>
              setForm((current) => ({ ...current, max_request_body_size_kb: value }))
            }
          />
          <TextFieldRow
            type="number"
            label={t('systemSettings.fields.maxResponseBodySizeKb')}
            value={form.max_response_body_size_kb}
            helperText={t('systemSettings.helper.maxResponseBodySizeKb')}
            onChange={(value) =>
              setForm((current) => ({ ...current, max_response_body_size_kb: value }))
            }
          />
        </Stack>

        <TextFieldRow
          label={t('systemSettings.fields.sensitiveRequestHeaders')}
          value={form.sensitive_request_headers}
          helperText={t('systemSettings.helper.sensitiveRequestHeaders')}
          onChange={(value) =>
            setForm((current) => ({ ...current, sensitive_request_headers: value }))
          }
        />

        <Stack direction={{ xs: 'column', md: 'row' }} spacing={1}>
          <RequestRecordSwitch
            checked={form.record_request_headers}
            label={t('systemSettings.fields.recordRequestHeaders')}
            onChange={(checked) =>
              setForm((current) => ({ ...current, record_request_headers: checked }))
            }
          />
          <RequestRecordSwitch
            checked={form.record_request_body}
            label={t('systemSettings.fields.recordRequestBody')}
            onChange={(checked) =>
              setForm((current) => ({ ...current, record_request_body: checked }))
            }
          />
          <RequestRecordSwitch
            checked={form.record_response_body}
            label={t('systemSettings.fields.recordResponseBody')}
            onChange={(checked) =>
              setForm((current) => ({ ...current, record_response_body: checked }))
            }
          />
        </Stack>
      </Stack>
    </SettingsSection>
  );
}

function RequestRecordSwitch({
  checked,
  label,
  onChange,
}: {
  checked: boolean;
  label: string;
  onChange: (checked: boolean) => void;
}) {
  return (
    <FormControlLabel
      control={<Switch checked={checked} onChange={(event) => onChange(event.target.checked)} />}
      label={label}
      sx={{ flex: 1, minWidth: 0 }}
    />
  );
}
