'use client';

import type { UserGroup } from 'src/types/user-group';
import type { SystemSettingsForm } from './system-settings-utils';

import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';

import { labelWithAccountingCurrency } from 'src/utils/money-boundary';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow } from './shared';
import { SettingsSection } from './system-settings-section';
import { emailConfigComplete } from './system-settings-utils';

type Props = {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
  userGroups: UserGroup[];
};

export function SystemSettingsBaseSection({ form, setForm, userGroups }: Props) {
  const { t } = useTranslate('admin');
  const emailVerificationReady = form.email_config_enabled && emailConfigComplete(form);
  const emailVerificationDisabled =
    !emailVerificationReady && !form.registration_email_verification_enabled;
  const passwordResetDisabled = !emailVerificationReady && !form.password_reset_enabled;

  return (
    <SettingsSection title={t('systemSettings.sections.base')}>
      <Stack spacing={2}>
        <SwitchRow
          checked={form.allow_registration}
          label={t('systemSettings.fields.allowRegistration')}
          onChange={(checked) =>
            setForm((current) => ({ ...current, allow_registration: checked }))
          }
        />
        <DefaultUserGroupField form={form} setForm={setForm} userGroups={userGroups} />
        <SwitchRow
          checked={form.login_captcha_enabled}
          label={t('systemSettings.fields.loginCaptchaEnabled')}
          onChange={(checked) =>
            setForm((current) => ({ ...current, login_captcha_enabled: checked }))
          }
        />
        <SwitchRow
          checked={form.registration_captcha_enabled}
          label={t('systemSettings.fields.registrationCaptchaEnabled')}
          onChange={(checked) =>
            setForm((current) => ({ ...current, registration_captcha_enabled: checked }))
          }
        />
        <SwitchRow
          checked={form.support_ticket_captcha_enabled}
          label={t('systemSettings.fields.supportTicketCaptchaEnabled')}
          helperText={t('systemSettings.helper.supportTicketCaptchaEnabled')}
          onChange={(checked) =>
            setForm((current) => ({ ...current, support_ticket_captcha_enabled: checked }))
          }
        />
        <SwitchRow
          checked={form.registration_email_verification_enabled}
          disabled={emailVerificationDisabled}
          label={t('systemSettings.fields.registrationEmailVerificationEnabled')}
          helperText={
            emailVerificationReady
              ? undefined
              : t('systemSettings.helper.registrationEmailVerificationRequiresEmailConfig')
          }
          onChange={(checked) =>
            setForm((current) => ({
              ...current,
              registration_email_verification_enabled: checked,
            }))
          }
        />
        <SwitchRow
          checked={form.password_reset_enabled}
          disabled={passwordResetDisabled}
          label={t('systemSettings.fields.passwordResetEnabled')}
          helperText={
            emailVerificationReady
              ? undefined
              : t('systemSettings.helper.passwordResetRequiresEmailConfig')
          }
          onChange={(checked) =>
            setForm((current) => ({
              ...current,
              password_reset_enabled: checked,
            }))
          }
        />
        <TextFieldRow
          type="number"
          label={labelWithAccountingCurrency(t('systemSettings.fields.defaultUserGrant'))}
          value={form.default_user_grant}
          helperText={t('systemSettings.helper.defaultUserGrant')}
          onChange={(value) => setForm((current) => ({ ...current, default_user_grant: value }))}
        />
      </Stack>
    </SettingsSection>
  );
}

function DefaultUserGroupField({ form, setForm, userGroups }: Props) {
  const { t } = useTranslate('admin');

  return (
    <TextFieldRow
      required
      select
      label={t('systemSettings.fields.defaultUserGroupCode')}
      value={form.default_user_group_code}
      helperText={t('systemSettings.helper.defaultUserGroupCode')}
      onChange={(value) => setForm((current) => ({ ...current, default_user_group_code: value }))}
    >
      {userGroups.length === 0 ? (
        <MenuItem disabled value="">
          {t('userGroups.noActiveGroups')}
        </MenuItem>
      ) : null}
      {userGroups.map((group) => (
        <MenuItem key={group.code} value={group.code}>
          {group.name} ({group.code})
        </MenuItem>
      ))}
    </TextFieldRow>
  );
}
