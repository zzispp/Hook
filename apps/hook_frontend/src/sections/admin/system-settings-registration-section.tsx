'use client';

import type { UserGroup } from 'src/types/user-group';
import type { SystemSettingsForm } from './system-settings-utils';

import Stack from '@mui/material/Stack';
import Divider from '@mui/material/Divider';
import MenuItem from '@mui/material/MenuItem';
import Typography from '@mui/material/Typography';

import { labelWithAccountingCurrency } from 'src/utils/money-boundary';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow } from './shared';
import { SettingsSection } from './system-settings-section';
import { emailConfigComplete } from './system-settings-utils';
import { EmailTemplateEditor } from './system-settings-email-template-editor';

type Props = {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
  userGroups: UserGroup[];
};

export function SystemSettingsRegistrationSection({ form, setForm, userGroups }: Props) {
  const { t } = useTranslate('admin');

  return (
    <SettingsSection title={t('systemSettings.sections.registration')}>
      <Stack spacing={3}>
        <RegistrationControls form={form} setForm={setForm} userGroups={userGroups} />
        <Divider />
        <EmailRestrictionFields form={form} setForm={setForm} />
        <Divider />
        <EmailTemplateEditor
          form={form}
          setForm={setForm}
          templateType="registration"
          setTemplateType={() => undefined}
          availableTypes={['registration']}
        />
      </Stack>
    </SettingsSection>
  );
}

function RegistrationControls({ form, setForm, userGroups }: Props) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={2}>
      <RegistrationSwitches form={form} setForm={setForm} />
      <DefaultUserGroupField form={form} setForm={setForm} userGroups={userGroups} />
      <TextFieldRow
        type="number"
        label={labelWithAccountingCurrency(t('systemSettings.fields.defaultUserGrant'))}
        value={form.default_user_grant}
        helperText={t('systemSettings.helper.defaultUserGrant')}
        onChange={(value) => setForm((current) => ({ ...current, default_user_grant: value }))}
      />
    </Stack>
  );
}

function RegistrationSwitches({
  form,
  setForm,
}: Omit<Props, 'userGroups'>) {
  const { t } = useTranslate('admin');
  const emailVerificationReady = form.email_config_enabled && emailConfigComplete(form);
  const emailVerificationDisabled =
    !emailVerificationReady && !form.registration_email_verification_enabled;

  return (
    <Stack spacing={2}>
      <SwitchRow
        checked={form.allow_registration}
        label={t('systemSettings.fields.allowRegistration')}
        onChange={(checked) => setForm((current) => ({ ...current, allow_registration: checked }))}
      />
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
    </Stack>
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

function EmailRestrictionFields({
  form,
  setForm,
}: Omit<Props, 'userGroups'>) {
  const { t } = useTranslate('admin');
  const suffixEnabled = form.email_suffix_mode !== 'none';

  return (
    <Stack spacing={2}>
      <Typography variant="subtitle2">{t('systemSettings.email.suffixTitle')}</Typography>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          select
          label={t('systemSettings.fields.emailSuffixMode')}
          value={form.email_suffix_mode}
          onChange={(value) =>
            setForm((current) => ({
              ...current,
              email_suffix_mode: value as typeof current.email_suffix_mode,
            }))
          }
        >
          <MenuItem value="none">{t('systemSettings.email.suffixMode.none')}</MenuItem>
          <MenuItem value="whitelist">{t('systemSettings.email.suffixMode.whitelist')}</MenuItem>
          <MenuItem value="blacklist">{t('systemSettings.email.suffixMode.blacklist')}</MenuItem>
        </TextFieldRow>
        <TextFieldRow
          disabled={!suffixEnabled}
          label={t('systemSettings.fields.emailSuffixes')}
          value={form.email_suffixes}
          placeholder="example.com, company.com"
          helperText={t('systemSettings.email.emailSuffixesHelper')}
          onChange={(value) => setForm((current) => ({ ...current, email_suffixes: value }))}
        />
      </Stack>
    </Stack>
  );
}
