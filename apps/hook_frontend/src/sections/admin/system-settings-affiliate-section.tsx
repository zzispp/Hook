'use client';

import type { SystemSettingsForm } from './system-settings-utils';

import Stack from '@mui/material/Stack';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow } from './shared';
import { SettingsSection } from './system-settings-section';

type Props = {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
};

export function AffiliateSettingsSection({ form, setForm }: Props) {
  const { t } = useTranslate('admin');

  return (
    <SettingsSection title={t('systemSettings.sections.affiliate')}>
      <Stack spacing={2}>
        <SwitchRow
          checked={form.affiliate_enabled}
          label={t('systemSettings.fields.affiliateEnabled')}
          helperText={t('systemSettings.helper.affiliateEnabled')}
          onChange={(checked) => setForm((current) => ({ ...current, affiliate_enabled: checked }))}
        />
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.affiliateCommissionPercent')}
          value={form.affiliate_commission_percent}
          helperText={t('systemSettings.helper.affiliateCommissionPercent')}
          slotProps={{ htmlInput: { min: 0, max: 100, step: 0.01 } }}
          onChange={(value) =>
            setForm((current) => ({ ...current, affiliate_commission_percent: value }))
          }
        />
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.affiliateMinCommissionAmount')}
          value={form.affiliate_min_commission_amount}
          helperText={t('systemSettings.helper.affiliateMinCommissionAmount')}
          slotProps={{ htmlInput: { min: 0, step: 0.01 } }}
          onChange={(value) =>
            setForm((current) => ({ ...current, affiliate_min_commission_amount: value }))
          }
        />
      </Stack>
    </SettingsSection>
  );
}
