'use client';

import type { SystemSettingsForm } from './system-settings-utils';

import Stack from '@mui/material/Stack';

import { useTranslate } from 'src/locales/use-locales';

import { TextFieldRow } from './shared';
import { SettingsSection } from './system-settings-section';

type Props = {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
};

export function SystemSettingsTokenSection({ form, setForm }: Props) {
  const { t } = useTranslate('admin');

  return (
    <SettingsSection title={t('systemSettings.sections.tokens')}>
      <Stack spacing={2}>
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.tokenLimitPerUser')}
          value={form.token_limit_per_user}
          helperText={t('systemSettings.helper.tokenLimitPerUser')}
          onChange={(value) => setForm((current) => ({ ...current, token_limit_per_user: value }))}
        />
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.defaultRateLimitRpm')}
          value={form.default_rate_limit_rpm}
          helperText={t('systemSettings.helper.defaultRateLimitRpm')}
          onChange={(value) =>
            setForm((current) => ({ ...current, default_rate_limit_rpm: value }))
          }
        />
      </Stack>
    </SettingsSection>
  );
}
