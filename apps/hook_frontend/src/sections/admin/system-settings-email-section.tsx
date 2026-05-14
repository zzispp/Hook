'use client';

import type { SystemSettingsForm } from './system-settings-utils';
import type { EmailTemplateType } from './system-settings-email-template-editor';

import { useState } from 'react';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import MenuItem from '@mui/material/MenuItem';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';
import { testSmtpConnection } from 'src/actions/system-settings';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import { SwitchRow, TextFieldRow } from './shared';
import { SettingsSection } from './system-settings-section';
import { EmailTemplateEditor } from './system-settings-email-template-editor';
import { smtpTestPayload, emailConfigComplete } from './system-settings-utils';

type SystemSettingsFormProps = {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
};

export function EmailSettingsSection({ form, setForm }: SystemSettingsFormProps) {
  const { t } = useTranslate('admin');
  const [templateType, setTemplateType] = useState<EmailTemplateType>('registration');
  const [testingSmtp, setTestingSmtp] = useState(false);

  const handleTestSmtp = async () => {
    setTestingSmtp(true);
    try {
      const result = await testSmtpConnection(smtpTestPayload(form));
      if (result.success) {
        toast.success(t('systemSettings.email.smtpTestSuccess'));
        return;
      }
      toast.error(result.message || t('systemSettings.email.smtpTestFailed'));
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : t('systemSettings.email.smtpTestFailed')
      );
    } finally {
      setTestingSmtp(false);
    }
  };

  return (
    <SettingsSection
      title={t('systemSettings.sections.email')}
      description={t('systemSettings.email.description')}
    >
      <Stack spacing={3}>
        <SmtpServerFields
          form={form}
          setForm={setForm}
          testingSmtp={testingSmtp}
          onTestSmtp={handleTestSmtp}
        />
        <Divider />
        <EmailRestrictionFields form={form} setForm={setForm} />
        <Divider />
        <EmailTemplateEditor
          form={form}
          setForm={setForm}
          templateType={templateType}
          setTemplateType={setTemplateType}
        />
      </Stack>
    </SettingsSection>
  );
}

function SmtpServerFields({
  form,
  setForm,
  testingSmtp,
  onTestSmtp,
}: SystemSettingsFormProps & {
  testingSmtp: boolean;
  onTestSmtp: () => void;
}) {
  const { t } = useTranslate('admin');
  const ticketEmailReady = form.email_config_enabled && emailConfigComplete(form);
  const ticketEmailDisabled =
    !ticketEmailReady && !form.support_ticket_email_notifications_enabled;
  const handleEmailConfigEnabledChange = (checked: boolean) => {
    setForm((current) => ({
      ...current,
      email_config_enabled: checked,
      registration_email_verification_enabled: checked
        ? current.registration_email_verification_enabled
        : false,
      support_ticket_email_notifications_enabled: checked
        ? current.support_ticket_email_notifications_enabled
        : false,
    }));
  };

  return (
    <Stack spacing={2}>
      <SwitchRow
        checked={form.email_config_enabled}
        label={t('systemSettings.fields.emailConfigEnabled')}
        helperText={t('systemSettings.email.emailConfigEnabledHelper')}
        onChange={handleEmailConfigEnabledChange}
      />
      <SwitchRow
        checked={form.support_ticket_email_notifications_enabled}
        disabled={ticketEmailDisabled}
        label={t('systemSettings.fields.supportTicketEmailNotificationsEnabled')}
        helperText={
          ticketEmailReady
            ? t('systemSettings.email.supportTicketEmailNotificationsHelper')
            : t('systemSettings.helper.supportTicketEmailNotificationsRequiresEmailConfig')
        }
        onChange={(checked) =>
          setForm((current) => ({
            ...current,
            support_ticket_email_notifications_enabled: checked,
          }))
        }
      />
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1.5} alignItems={{ sm: 'center' }}>
        <Typography variant="subtitle2" sx={{ flexGrow: 1 }}>
          {t('systemSettings.email.smtpTitle')}
        </Typography>
        <Button
          size="small"
          variant="outlined"
          loading={testingSmtp}
          startIcon={<Iconify icon="solar:play-circle-bold" />}
          onClick={onTestSmtp}
        >
          {t('systemSettings.email.testConnection')}
        </Button>
      </Stack>
      <SmtpConnectionFields form={form} setForm={setForm} />
      <SmtpAuthFields form={form} setForm={setForm} />
      <SmtpSenderFields form={form} setForm={setForm} />
    </Stack>
  );
}

function SmtpConnectionFields({ form, setForm }: SystemSettingsFormProps) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <TextFieldRow
        label={t('systemSettings.fields.smtpHost')}
        value={form.smtp_host}
        placeholder="smtp.example.com"
        onChange={(value) => setForm((current) => ({ ...current, smtp_host: value }))}
      />
      <TextFieldRow
        type="number"
        label={t('systemSettings.fields.smtpPort')}
        value={form.smtp_port}
        helperText={t('systemSettings.email.smtpPortHelper')}
        onChange={(value) => setForm((current) => ({ ...current, smtp_port: value }))}
      />
      <TextFieldRow
        select
        label={t('systemSettings.fields.smtpEncryption')}
        value={form.smtp_encryption}
        onChange={(value) =>
          setForm((current) => ({
            ...current,
            smtp_encryption: value as typeof current.smtp_encryption,
          }))
        }
      >
        <MenuItem value="tls">{t('systemSettings.email.encryption.tls')}</MenuItem>
        <MenuItem value="ssl">{t('systemSettings.email.encryption.ssl')}</MenuItem>
        <MenuItem value="none">{t('systemSettings.email.encryption.none')}</MenuItem>
      </TextFieldRow>
    </Stack>
  );
}

function SmtpAuthFields({ form, setForm }: SystemSettingsFormProps) {
  const { t } = useTranslate('admin');
  const passwordHelper = form.smtp_password_set
    ? t('systemSettings.email.smtpPasswordStored')
    : t('systemSettings.email.smtpPasswordEmpty');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <TextFieldRow
        label={t('systemSettings.fields.smtpUsername')}
        value={form.smtp_username}
        onChange={(value) => setForm((current) => ({ ...current, smtp_username: value }))}
      />
      <TextFieldRow
        type="password"
        label={t('systemSettings.fields.smtpPassword')}
        value={form.smtp_password}
        helperText={passwordHelper}
        slotProps={{ htmlInput: { autoComplete: 'new-password' } }}
        onChange={(value) => setForm((current) => ({ ...current, smtp_password: value }))}
      />
    </Stack>
  );
}

function SmtpSenderFields({ form, setForm }: SystemSettingsFormProps) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <TextFieldRow
        label={t('systemSettings.fields.smtpFromEmail')}
        value={form.smtp_from_email}
        placeholder="noreply@example.com"
        onChange={(value) => setForm((current) => ({ ...current, smtp_from_email: value }))}
      />
      <TextFieldRow
        label={t('systemSettings.fields.smtpFromName')}
        value={form.smtp_from_name}
        onChange={(value) => setForm((current) => ({ ...current, smtp_from_name: value }))}
      />
    </Stack>
  );
}

function EmailRestrictionFields({ form, setForm }: SystemSettingsFormProps) {
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
