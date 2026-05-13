'use client';

import type { Dispatch, SetStateAction } from 'react';
import type { SystemSettingsForm } from './system-settings-utils';

import Tab from '@mui/material/Tab';
import Chip from '@mui/material/Chip';
import Tabs from '@mui/material/Tabs';
import Stack from '@mui/material/Stack';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/locales/use-locales';

import { TextFieldRow } from './shared';

export type EmailTemplateType = 'registration' | 'password_reset';

type TemplatePatch = Partial<{
  subject: string;
  html: string;
}>;

type EmailTemplateEditorProps = {
  form: SystemSettingsForm;
  setForm: Dispatch<SetStateAction<SystemSettingsForm>>;
  templateType: EmailTemplateType;
  setTemplateType: (value: EmailTemplateType) => void;
};

const TEMPLATE_VARIABLES: Record<EmailTemplateType, string[]> = {
  registration: ['app_name', 'code', 'expire_minutes', 'email'],
  password_reset: ['app_name', 'reset_link', 'expire_minutes', 'email'],
};

export function EmailTemplateEditor({
  form,
  setForm,
  templateType,
  setTemplateType,
}: EmailTemplateEditorProps) {
  const { t } = useTranslate('admin');
  const values = templateValues(form, templateType);

  return (
    <Stack spacing={2}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} alignItems={{ md: 'center' }}>
        <Tabs value={templateType} onChange={(_, value) => setTemplateType(value)}>
          <Tab value="registration" label={t('systemSettings.email.templates.registration')} />
          <Tab value="password_reset" label={t('systemSettings.email.templates.passwordReset')} />
        </Tabs>
        <Stack direction="row" spacing={0.75} useFlexGap flexWrap="wrap">
          {TEMPLATE_VARIABLES[templateType].map((variable) => (
            <Chip key={variable} size="small" variant="soft" label={`{{${variable}}}`} />
          ))}
        </Stack>
      </Stack>
      <TextFieldRow
        label={t('systemSettings.fields.emailTemplateSubject')}
        value={values.subject}
        onChange={(value) => updateTemplate(setForm, templateType, { subject: value })}
      />
      <TextField
        fullWidth
        multiline
        minRows={12}
        label={t('systemSettings.fields.emailTemplateHtml')}
        value={values.html}
        slotProps={{ htmlInput: { spellCheck: false } }}
        onChange={(event) => updateTemplate(setForm, templateType, { html: event.target.value })}
      />
    </Stack>
  );
}

function templateValues(form: SystemSettingsForm, type: EmailTemplateType) {
  if (type === 'registration') {
    return {
      subject: form.email_template_registration_subject,
      html: form.email_template_registration_html,
    };
  }
  return {
    subject: form.email_template_password_reset_subject,
    html: form.email_template_password_reset_html,
  };
}

function updateTemplate(
  setForm: Dispatch<SetStateAction<SystemSettingsForm>>,
  type: EmailTemplateType,
  patch: TemplatePatch
) {
  setForm((current) => {
    if (type === 'registration') {
      return {
        ...current,
        email_template_registration_subject:
          patch.subject ?? current.email_template_registration_subject,
        email_template_registration_html: patch.html ?? current.email_template_registration_html,
      };
    }
    return {
      ...current,
      email_template_password_reset_subject:
        patch.subject ?? current.email_template_password_reset_subject,
      email_template_password_reset_html: patch.html ?? current.email_template_password_reset_html,
    };
  });
}
